use std::rc::Rc;

use crate::{
    components::prefs::static_components::SettingRoutes,
    console_log,
    players::librespot::LibrespotPlayer,
    utils::{
        common::invoke,
        prefs::{load_selective_async, watch_preferences},
    },
};
use leptos::{
    component, create_rw_signal, document, expect_context, provide_context, view, window, IntoView,
    RwSignal, SignalUpdate,
};
use leptos_i18n::provide_i18n_context;
use leptos_router::{Outlet, Redirect, Route, Router, Routes};
use serde::Serialize;
use types::{preferences::CheckboxPreference, themes::ThemeDetails};
use wasm_bindgen::{convert::IntoWasmAbi, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;

use crate::{
    components::{
        musicbar::MusicBar,
        sidebar::{Sidebar, Tab},
        topbar::TopBar,
    },
    i18n::Locale,
    modals::modal_manager::ModalManager,
    pages::{
        albums::{AllAlbums, SingleAlbum},
        artists::{AllArtists, SingleArtist},
        genres::{AllGenres, SingleGenre},
        playlists::{AllPlaylists, SinglePlaylist},
        search::Search,
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

fn handle_theme(id: String) {
    #[derive(Serialize)]
    struct LoadThemeArgs {
        id: String,
    }
    spawn_local(async move {
        let theme = invoke(
            "load_theme",
            serde_wasm_bindgen::to_value(&LoadThemeArgs { id }).unwrap(),
        )
        .await
        .unwrap();
        let theme: ThemeDetails = serde_wasm_bindgen::from_value(theme).unwrap();

        let document_element = document()
            .document_element()
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();

        let style = document_element.style();
        style.set_css_text("");

        style
            .set_property("--primary", &theme.theme.primary)
            .unwrap();
        style
            .set_property("--secondary", &theme.theme.secondary)
            .unwrap();
        style
            .set_property("--tertiary", &theme.theme.tertiary)
            .unwrap();
        style
            .set_property("--textPrimary", &theme.theme.textPrimary)
            .unwrap();
        style
            .set_property("--textSecondary", &theme.theme.textSecondary)
            .unwrap();
        style
            .set_property("--textInverse", &theme.theme.textInverse)
            .unwrap();
        style.set_property("--accent", &theme.theme.accent).unwrap();
        style
            .set_property("--divider", &theme.theme.divider)
            .unwrap();

        if let Some(custom_css) = theme.theme.customCSS {
            let mut custom_style_sheet = document().get_element_by_id("custom-css");
            if custom_style_sheet.is_none() {
                let el = document().create_element("style").unwrap();
                el.set_id("css-element");
                custom_style_sheet = Some(el);
            }

            let custom_style_sheet = custom_style_sheet.unwrap();
            custom_style_sheet.set_inner_html(custom_css.as_str());
        }
    });
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(create_rw_signal(PlayerStore::new()));
    provide_context(Rc::new(ProviderStore::new()));
    provide_context(create_rw_signal(ModalStore::default()));

    provide_i18n_context::<Locale>();

    spawn_local(async move {
        let id = load_selective_async("themes.active_theme".into())
            .await
            .unwrap();
        handle_theme(id);
    });

    let unlisten = watch_preferences(|(key, value)| {
        console_log!("Preferences changed: {} = {:?}", key, value);
        if key == "prefs.volume_persist_mode" {
            let player_store = expect_context::<RwSignal<PlayerStore>>();
            player_store.update(|store| {
                store.update_volume_mode(serde_wasm_bindgen::from_value(value).unwrap())
            });
        } else if key == "prefs.spotify.enable" {
            let enabled: Vec<CheckboxPreference> = serde_wasm_bindgen::from_value(value).unwrap();
            for pref in enabled {
                if pref.key == "enable" {
                    LibrespotPlayer::set_enabled(pref.enabled)
                }
            }
        } else if key == "prefs.spotify.username" {
            let value = value.as_string().unwrap();
            LibrespotPlayer::set_has_username(!value.is_empty())
        } else if key == "prefs.spotify.password" {
            let value = value.as_string().unwrap();
            LibrespotPlayer::set_has_password(!value.is_empty())
        } else if key == "prefs.themes.active_theme" {
            let value = value.as_string().unwrap();
            handle_theme(value);
        }
    });

    let window = window();
    if let Err(e) = window.add_event_listener_with_callback("beforeunload", &unlisten) {
        console_log!("Failed to set unmount hook: {:?}", e);
    }

    view! {
        <Router>
            <main id="app">
                <div class="appContainer">
                    <Routes>
                        <Route path="/" view=CommonApp>
                            <Route path="main" view=MainApp>
                                <Route path="" view=AllSongs />
                                <Route path="playlists" view=AllPlaylists />
                                <Route path="playlists/single" view=SinglePlaylist />
                                <Route path="artists" view=AllArtists />
                                <Route path="artists/single" view=SingleArtist />
                                <Route path="albums" view=AllAlbums />
                                <Route path="albums/single" view=SingleAlbum />
                                <Route path="genres" view=AllGenres />
                                <Route path="genres/single" view=SingleGenre />
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
