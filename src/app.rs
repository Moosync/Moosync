use std::rc::Rc;

use leptos::{component, create_rw_signal, provide_context, view, IntoView};
use leptos_router::{Outlet, Redirect, Route, Router, Routes};

use crate::{
    components::{musicbar::MusicBar, sidebar::Sidebar, topbar::TopBar},
    modals::modal_manager::ModalManager,
    pages::{
        albums::{AllAlbums, SingleAlbum},
        artists::{AllArtists, SingleArtist},
        genres::{AllGenres, SingleGenre},
        playlists::{AllPlaylists, SinglePlaylist},
        search::Search,
        settings::Settings,
        songs::AllSongs,
    },
    store::{modal_store::ModalStore, player_store::PlayerStore, provider_store::ProviderStore},
};

#[component]
pub fn RedirectAll() -> impl IntoView {
    // TODO: Change to all songs
    view! { <Redirect path="/prefs" /> }
}

#[component]
pub fn MainApp() -> impl IntoView {
    view! {
        <div>
            <Sidebar />
            <MusicBar />
            <TopBar />
            <ModalManager />
            <div class="main-container">
                <Outlet />
            </div>
        </div>
    }
}

#[component]
pub fn PrefApp() -> impl IntoView {
    view! {
        <div>
            <ModalManager />
            <div class="prefs-container">
                <Outlet />
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(create_rw_signal(PlayerStore::new()));
    provide_context(Rc::new(ProviderStore::new()));
    provide_context(create_rw_signal(ModalStore::default()));

    view! {
        <Router>
            <main id="app">
                <div class="appContainer">
                    <Routes>
                        <Route path="/main" view=MainApp>
                            <Route path="" view=AllSongs />
                            <Route path="playlists" view=AllPlaylists />
                            <Route path="playlists/:id" view=SinglePlaylist />
                            <Route path="artists" view=AllArtists />
                            <Route path="artists/:id" view=SingleArtist />
                            <Route path="albums" view=AllAlbums />
                            <Route path="albums/:id" view=SingleAlbum />
                            <Route path="genres" view=AllGenres />
                            <Route path="genres/:id" view=SingleGenre />
                            <Route path="search" view=Search />
                        </Route>
                        <Route path="/prefs" view=PrefApp>
                            <Route path="" view=Settings></Route>
                        </Route>
                        <Route path="*" view=RedirectAll />
                    </Routes>
                </div>
            </main>
        </Router>
    }
}
