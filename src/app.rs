use leptos::{component, create_rw_signal, provide_context, view, IntoView};

use crate::{
    components::{musicbar::MusicBar, sidebar::Sidebar},
    store::player_store::PlayerStore,
};

#[component]
pub fn App() -> impl IntoView {
    provide_context(create_rw_signal(PlayerStore::new()));

    view! {
        <main id="app">
            <div class="appContainer">
                <Sidebar/>
                <MusicBar/>
            </div>
        </main>
    }
}
