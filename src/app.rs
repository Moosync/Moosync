use leptos::{component, create_rw_signal, provide_context, view, IntoView};
use leptos_router::{Route, Router, Routes};

use crate::{
    components::{musicbar::MusicBar, sidebar::Sidebar},
    pages::songs::AllSongs,
    store::player_store::PlayerStore,
};

#[component]
pub fn App() -> impl IntoView {
    provide_context(create_rw_signal(PlayerStore::new()));

    view! {
        <Router>
            <main id="app">
                <div class="appContainer">
                    <Sidebar/>
                    <MusicBar/>
                    <div class="main-container">
                        <Routes>
                            <Route path="/" view=AllSongs/>
                        </Routes>
                    </div>
                </div>
            </main>
        </Router>
    }
}
