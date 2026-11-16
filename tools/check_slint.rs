use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use futures::executor::block_on;
use i_slint_compiler::{
    CompilerConfiguration, DefaultTranslationContext, EmbedResourcesKind,
    diagnostics::BuildDiagnostics, generator::OutputFormat, load_root_file,
    source_path::SourcePath,
};

const DEFAULT_ENTRY_FILES: &[&str] = &["ui/slint/src/app.slint", "ui/slint/src/app_mobile.slint"];

#[tracing::instrument(level = "info", skip_all)]
fn main() -> ExitCode {
    if let Ok(workspace_dir) = env::var("BUILD_WORKSPACE_DIRECTORY")
        && let Err(error) = env::set_current_dir(&workspace_dir)
    {
        eprintln!("Failed to change directory to {workspace_dir}: {error}");
        return ExitCode::FAILURE;
    }

    let files = collect_target_files();
    if files.is_empty() {
        return ExitCode::SUCCESS;
    }

    let mut has_errors = false;
    for path in files {
        if !check_file(&path) {
            has_errors = true;
        }
    }

    if has_errors {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn collect_target_files() -> Vec<PathBuf> {
    let args: Vec<String> = env::args().skip(1).collect();
    if !args.is_empty() && !args.iter().any(|arg| arg.ends_with(".slint")) {
        return Vec::new();
    }

    DEFAULT_ENTRY_FILES
        .iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect()
}

#[tracing::instrument(level = "debug", skip_all)]
fn build_compiler_config() -> CompilerConfiguration {
    let mut compiler_config = CompilerConfiguration::new(OutputFormat::Interpreter);
    compiler_config.default_translation_context = DefaultTranslationContext::None;
    compiler_config.embed_resources = EmbedResourcesKind::Nothing;
    compiler_config
}

#[tracing::instrument(level = "debug", skip_all)]
fn check_file(path: &Path) -> bool {
    let source_code = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("Failed to read {}: {error}", path.display());
            return false;
        }
    };

    let diagnostics = BuildDiagnostics::default();
    let compiler_config = build_compiler_config();
    let source_path = SourcePath::new(path);

    let (diagnostics, _) = block_on(load_root_file(
        &source_path,
        source_code,
        diagnostics,
        compiler_config,
    ));

    let has_errors = diagnostics.has_errors();
    if !diagnostics.is_empty() {
        diagnostics.print();
    }

    !has_errors
}
