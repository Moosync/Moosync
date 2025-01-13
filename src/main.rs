mod app;
mod components;
// mod expand;
mod icons;
mod modals;
mod pages;
mod players;
mod store;
mod utils;

use app::*;
use leptos::prelude::*;
use tracing_subscriber::{fmt, layer::SubscriberExt};
use utils::tracing_writer::MakeConsoleWriter;

leptos_i18n::load_locales!();

#[tracing::instrument(level = "trace", skip())]
fn main() {
    console_error_panic_hook::set_once();

    let filter = window().get("LOGGING_FILTER");
    let filter = if let Some(filter) = filter {
        filter.as_string().unwrap()
    } else {
        "error".into()
    };

    let filter = tracing_subscriber::filter::EnvFilter::try_new(filter).unwrap_or_default();
    let log_layer = fmt::layer()
        .pretty()
        .with_target(true)
        .without_time()
        .with_ansi(false)
        .with_writer(MakeConsoleWriter::default());

    let layer = fmt::layer()
        .pretty()
        .with_target(true)
        .with_ansi(true)
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
