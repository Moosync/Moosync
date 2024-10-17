mod app;
mod components;
mod icons;
mod modals;
mod pages;
mod players;
mod store;
mod utils;

use app::*;
use leptos::*;
use tracing_subscriber::{fmt, layer::SubscriberExt};
use utils::tracing_writer::MakeConsoleWriter;

leptos_i18n::load_locales!();

#[tracing::instrument(level = "trace", skip())]
fn main() {
    console_error_panic_hook::set_once();
    let filter = tracing_subscriber::filter::LevelFilter::DEBUG;
    let log_layer = fmt::layer()
        .pretty()
        .with_ansi(true)
        .with_target(true)
        .without_time()
        .with_writer(MakeConsoleWriter::default());

    let layer = fmt::layer()
        .pretty()
        .with_target(true)
        .with_ansi(false)
        .without_time()
        .with_writer(MakeConsoleWriter::new_log_file());

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(log_layer)
        .with(filter);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    mount_to_body(|| {
        view! { <App /> }
    })
}
