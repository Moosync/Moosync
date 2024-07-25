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

leptos_i18n::load_locales!();

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! { <App /> }
    })
}
