use diesel::sqlite::Sqlite;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
pub const CACHE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations_cache");

#[tracing::instrument(level = "trace", skip(databse))]
pub fn run_migrations(databse: &mut impl MigrationHarness<Sqlite>) {
    databse
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}

#[tracing::instrument(level = "trace", skip(databse))]
pub fn run_migration_cache(databse: &mut impl MigrationHarness<Sqlite>) {
    databse
        .run_pending_migrations(CACHE_MIGRATIONS)
        .expect("Failed to run migrations");
}
