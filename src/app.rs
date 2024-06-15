use std::rc::Rc;

use leptos::{component, create_rw_signal, provide_context, view, IntoView};
use leptos_router::{Route, Router, Routes};

use crate::{
    components::{musicbar::MusicBar, sidebar::Sidebar, topbar::TopBar}, modals::modal_manager::ModalManager, pages::{albums::{AllAlbums, SingleAlbum}, artists::{AllArtists, SingleArtist}, genres::{AllGenres, SingleGenre}, playlists::{AllPlaylists, SinglePlaylist}, songs::AllSongs}, store::{modal_store::ModalStore, player_store::PlayerStore, provider_store::ProviderStore}
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
                            <Route path="/playlists" view=AllPlaylists/>
                            <Route path="/playlists/:id" view=SinglePlaylist/>
                            <Route path="/artists" view=AllArtists/>
                            <Route path="/artists/:id" view=SingleArtist/>
                            <Route path="/albums" view=AllAlbums/>
                            <Route path="/albums/:id" view=SingleAlbum/>
                            <Route path="/genres" view=AllGenres/>
                            <Route path="/genres/:id" view=SingleGenre/>
                        </Routes>
                    </div>
                </div>
            </main>
        </Router>
    }
}
