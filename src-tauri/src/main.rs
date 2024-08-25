#![feature(proc_macro_hygiene)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tracing::instrument(level = "trace", skip())]
fn main() {
    app_lib::run();
}
