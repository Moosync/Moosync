use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

use hex::encode as hex_encode;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{Describe, Executor, Sqlite, SqlitePool};
use syn::{
    Expr, ExprLit, Lit, Macro,
    visit::{self, Visit},
};

#[derive(Serialize)]
struct OfflineQueryData<'a> {
    db_name: &'static str,
    query: &'a str,
    describe: Describe<Sqlite>,
    hash: String,
}

struct SqlxMacroVisitor {
    queries: BTreeSet<String>,
}

impl SqlxMacroVisitor {
    #[tracing::instrument(level = "debug", skip_all)]
    fn new() -> Self {
        Self {
            queries: BTreeSet::new(),
        }
    }
}

impl<'ast> Visit<'ast> for SqlxMacroVisitor {
    #[tracing::instrument(level = "debug", skip_all)]
    fn visit_macro(&mut self, mac: &'ast Macro) {
        let is_sqlx_query = mac.path.segments.last().is_some_and(|seg| {
            matches!(
                seg.ident.to_string().as_str(),
                "query" | "query_scalar" | "query_as"
            )
        });

        if is_sqlx_query && let Some(sql) = extract_first_string_literal(mac) {
            self.queries.insert(sql);
        }

        visit::visit_macro(self, mac);
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn extract_first_string_literal(mac: &Macro) -> Option<String> {
    for token in mac.tokens.clone() {
        if let proc_macro2::TokenTree::Literal(lit) = token
            && let Ok(Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            })) = syn::parse_str::<Expr>(&lit.to_string())
        {
            return Some(lit_str.value());
        }
    }
    None
}

#[tracing::instrument(level = "debug", skip_all)]
fn hash_query(query: &str) -> String { hex_encode(Sha256::digest(query.as_bytes())) }

#[tracing::instrument(level = "debug", skip_all)]
async fn apply_migrations(pool: &SqlitePool, migrations_dir: &Path) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = fs::read_dir(migrations_dir)
        .map_err(|e| format!("Failed to read migrations dir {migrations_dir:?}: {e}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
        .collect();
    entries.sort();

    for path in entries {
        let sql = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read migration {path:?}: {e}"))?;
        sqlx::raw_sql(&sql)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to execute migration {path:?}: {e}"))?;
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
fn collect_queries(src_dir: &Path) -> Result<BTreeSet<String>, String> {
    let mut visitor = SqlxMacroVisitor::new();
    let entries =
        fs::read_dir(src_dir).map_err(|e| format!("Failed to read src dir {src_dir:?}: {e}"))?;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read {path:?}: {e}"))?;
        let parsed =
            syn::parse_file(&content).map_err(|e| format!("Failed to parse {path:?}: {e}"))?;
        visitor.visit_file(&parsed);
    }
    Ok(visitor.queries)
}

#[tracing::instrument(level = "debug", skip_all)]
async fn write_offline_cache(
    pool: &SqlitePool,
    queries: &BTreeSet<String>,
    cache_dir: &Path,
) -> Result<(), String> {
    if cache_dir.exists() {
        for entry in fs::read_dir(cache_dir)
            .map_err(|e| format!("Failed to read cache dir {cache_dir:?}: {e}"))?
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("query-") && n.ends_with(".json"))
            {
                let _ = fs::remove_file(path);
            }
        }
    } else {
        fs::create_dir_all(cache_dir)
            .map_err(|e| format!("Failed to create cache dir {cache_dir:?}: {e}"))?;
    }

    for query in queries {
        let describe = pool
            .describe(query)
            .await
            .map_err(|e| format!("SQLx describe failed for query:\n{query}\nError: {e}"))?;
        let hash = hash_query(query);
        let data = OfflineQueryData {
            db_name: "SQLite",
            query,
            describe,
            hash: hash.clone(),
        };
        let mut json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize query {hash}: {e}"))?;
        json.push('\n');
        let out_path = cache_dir.join(format!("query-{hash}.json"));
        fs::write(&out_path, json).map_err(|e| format!("Failed to write {out_path:?}: {e}"))?;
    }
    Ok(())
}

#[tokio::main]
#[tracing::instrument(level = "debug", skip_all)]
async fn main() -> Result<(), String> {
    let workspace_root = env::var("BUILD_WORKSPACE_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().expect("Failed to get current dir"));

    let db_dir = workspace_root.join("core/database");
    let migrations_dir = db_dir.join("migrations");
    let src_dir = db_dir.join("src");
    let cache_dir = db_dir.join(".sqlx");

    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .map_err(|e| format!("Failed to open in-memory SQLite: {e}"))?;

    apply_migrations(&pool, &migrations_dir).await?;
    let queries = collect_queries(&src_dir)?;
    write_offline_cache(&pool, &queries, &cache_dir).await?;

    println!(
        "Generated {} sqlx offline query files in {}",
        queries.len(),
        cache_dir.display()
    );
    Ok(())
}
