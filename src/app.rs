use std::rc::Rc;

use leptos::{component, create_rw_signal, provide_context, view, IntoView};
use leptos_router::{Route, Router, Routes};

use crate::{
    components::{musicbar::MusicBar, sidebar::Sidebar, topbar::TopBar}, modals::modal_manager::ModalManager, pages::songs::AllSongs, store::{modal_store::ModalStore, player_store::PlayerStore, provider_store::ProviderStore}
};

#[component]
pub fn App() -> impl IntoView {
    provide_context(create_rw_signal(PlayerStore::new()));
    provide_context(Rc::new(ProviderStore::new()));
    provide_context(create_rw_signal(ModalStore::default()));

    view! {
        <Router>
            <main id="app">
                <div class="appContainer">
                    <Sidebar/>
                    <MusicBar/>
                    <TopBar/>
                    <ModalManager />
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
