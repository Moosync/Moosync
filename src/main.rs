mod app;
mod components;
mod icons;
mod modals;
mod pages;
mod players;
mod store;
mod utils;

use app::*;
use ev::keydown;
use i18n::use_i18n;
use leptos::*;
use leptos_use::use_event_listener;
use tracing_subscriber::{fmt, layer::SubscriberExt};
use utils::{
    invoke::{save_selective, toggle_dev_tools},
    tracing_writer::MakeConsoleWriter,
};

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

    let _ = use_event_listener(document().body(), keydown, |ev| {
        if ev.shift_key() && ev.ctrl_key() && ev.key_code() == 75 {
            spawn_local(async move {
                let _ = toggle_dev_tools().await;
            });
        }
    });

    mount_to_body(|| {
        view! { <App /> }
    })
}
