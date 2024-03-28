use leptos::*;
use serde::{Deserialize, Serialize};
use types::songs::QueryableSong;

use crate::components::{musicbar::MusicBar, sidebar::Sidebar};

#[component]
pub fn App() -> impl IntoView {
    let song = QueryableSong::empty();
    view! {
        <main id="app">
            <div class="appContainer">
                <Sidebar/>
                <MusicBar/>
            </div>
        </main>
    }
}
