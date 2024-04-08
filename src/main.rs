mod app;
mod components;
mod icons;
mod pages;
mod players;
mod providers;
mod store;
mod utils;
mod modals;

use app::*;
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! { <App/> }
    })
}
