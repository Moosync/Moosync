use std::path::PathBuf;

use diesel::{sqlite::Sqlite, Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tauri::{App, Manager};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn run_migrations(databse: &mut impl MigrationHarness<Sqlite>) {
    databse
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}
