// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

mod app;
mod components;
// mod expand;
mod icons;
mod modals;
mod pages;
mod players;
mod store;
mod utils;
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

use crate::i18n::I18nContextProvider;
use app::*;
use leptos::prelude::*;
use tracing_subscriber::{fmt, layer::SubscriberExt};
use utils::tracing_writer::MakeConsoleWriter;

#[tracing::instrument(level = "debug", skip())]
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
        view! {
            <I18nContextProvider>
                <App />
            </I18nContextProvider>
        }
    })
}
