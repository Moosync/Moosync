use std::rc::Rc;

use leptos::{
    component, create_rw_signal, leptos_dom::Transparent, provide_context, view, CollectView,
    IntoView, View,
};
use leptos_router::{Outlet, Redirect, Route, Router, Routes};
use types::ui::preferences::PreferenceUIFile;

use crate::{
    components::{
        musicbar::MusicBar,
        sidebar::{Sidebar, Tab},
        topbar::TopBar,
    },
    console_log,
    icons::{
        albums_icon::{AlbumsIcon, AlbumsIconProps},
        allsongs_icon::{AllSongsIcon, AllSongsIconProps},
        artists_icon::{ArtistsIcon, ArtistsIconProps},
        explore_icon::{ExploreIcon, ExploreIconProps},
        genres_icon::{GenresIcon, GenresIconProps},
        playlists_icon::{PlaylistsIcon, PlaylistsIconProps},
        queue_icon::{QueueIcon, QueueIconProps},
    },
    modals::modal_manager::ModalManager,
    pages::{
        albums::{AllAlbums, SingleAlbum},
        artists::{AllArtists, SingleArtist},
        genres::{AllGenres, SingleGenre},
        playlists::{AllPlaylists, SinglePlaylist},
        search::Search,
        settings::SettingsPage,
        songs::AllSongs,
    },
    store::{modal_store::ModalStore, player_store::PlayerStore, provider_store::ProviderStore},
};

#[component]
pub fn RedirectAll() -> impl IntoView {
    // TODO: Change to all songs
    view! { <Redirect path="/main" /> }
}

#[component]
pub fn RedirectPrefs() -> impl IntoView {
    // TODO: Change to all songs
    view! { <Redirect path="/prefs/paths" /> }
}

#[component]
fn CommonApp() -> impl IntoView {
    view! {
        <div>
            <MusicBar />
            <ModalManager />
            <Outlet />
        </div>
    }
}

#[component]
pub fn MainApp() -> impl IntoView {
    let tabs = vec![
        Tab::new("Queue", "Queue", ""),
        Tab::new("All Songs", "All Songs", "/main"),
        Tab::new("Playlists", "Playlists", "/main/playlists"),
        Tab::new("Artists", "Artists", "/main/artists"),
        Tab::new("Albums", "Albums", "/main/albums"),
        Tab::new("Genres", "Genres", "/main/genres"),
        Tab::new("Explore", "Explore", "/main/explore"),
    ];
    view! {
        <div>
            <TopBar />
            <Sidebar tabs=tabs />
            <div class="main-container">
                <Outlet />
            </div>
        </div>
    }
}

#[component]
pub fn PrefApp(#[prop()] prefs: PreferenceUIFile) -> impl IntoView {
    let mut tabs = vec![];
    for page in prefs.page {
        tabs.push(Tab::new(
            page.title.as_str(),
            page.icon.as_str(),
            format!("/prefs/{}", page.path).as_str(),
        ))
    }
    view! {
        <div>
            <Sidebar tabs=tabs />
            <div class="main-container">
                <Outlet />
            </div>
        </div>
    }
}

#[component(transparent)]
fn SettingRoutes() -> impl IntoView {
    let prefs: PreferenceUIFile = serde_yaml::from_str(include_str!("prefs.yaml")).unwrap();
    let prefs_clone = prefs.clone();
    view! {
        <Route path="/prefs" view=move || view! { <PrefApp prefs=prefs_clone.clone() /> }>
            <Route
                path=":page"
                view=move || {
                    view! { <SettingsPage prefs=prefs.clone() /> }
                }
            />
            <Route path="" view=RedirectPrefs />
        </Route>
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
                        <Route path="/" view=CommonApp>
                            <Route path="main" view=MainApp>
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
                            <SettingRoutes />
                        </Route>
                        <Route path="*" view=RedirectAll />
                    </Routes>
                </div>
            </main>
        </Router>
    }
}
