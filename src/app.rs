// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;

use crate::i18n::t;
use crate::{
    components::prefs::static_components::SettingRoutes,
    i18n::{use_i18n, Locale},
    pages::explore::Explore,
    players::librespot::LibrespotPlayer,
    store::ui_store::UiStore,
    utils::{
        common::{emit, get_locale, listen_event},
        invoke::{get_css, load_selective, load_theme, toggle_dev_tools},
        prefs::watch_preferences,
    },
};
use leptos::{
    component,
    ev::{contextmenu, keydown},
    prelude::*,
    view, IntoView,
};
use leptos_context_menu::provide_context_menu_state;
use leptos_i18n::provide_i18n_context;
use leptos_router::{
    components::{Outlet, ParentRoute, Redirect, Route, Router, Routes},
    path,
};
use leptos_use::use_event_listener;
use serde::Serialize;
use types::{
    preferences::CheckboxPreference, ui::extensions::ExtensionUIRequest,
    ui::player_details::PlayerState,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;

use crate::{
    components::{
        musicbar::MusicBar,
        sidebar::{Sidebar, Tab},
        topbar::TopBar,
    },
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

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn RedirectAll() -> impl IntoView {
    tracing::info!("Current location {:?}", window().location().href());
    view! { <Redirect path="/main/allsongs" /> }
}

#[tracing::instrument(level = "trace", skip())]
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

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn MainApp() -> impl IntoView {
    let i18n = use_i18n();
    let tabs = vec![
        Tab::new(
            i18n.get_keys().sidebar().tabs().queue().build_string(),
            "Queue",
            "queue",
        ),
        Tab::new(
            i18n.get_keys().sidebar().tabs().allSongs().build_string(),
            "Songs",
            "/main/allsongs",
        ),
        Tab::new(
            i18n.get_keys().sidebar().tabs().playlists().build_string(),
            "Playlists",
            "/main/playlists",
        ),
        Tab::new(
            i18n.get_keys().sidebar().tabs().artists().build_string(),
            "Artists",
            "/main/artists",
        ),
        Tab::new(
            i18n.get_keys().sidebar().tabs().albums().build_string(),
            "Albums",
            "/main/albums",
        ),
        Tab::new(
            i18n.get_keys().sidebar().tabs().genre().build_string(),
            "Genres",
            "/main/genres",
        ),
        Tab::new(
            i18n.get_keys().sidebar().tabs().explore().build_string(),
            "Explore",
            "/main/explore",
        ),
    ];
    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();
    let sidebar_open = create_read_slice(ui_store, |u| u.get_sidebar_open());

    Effect::new(move || {
        let sidebar_open = sidebar_open.get();
        let document = window().document().unwrap();
        let style_element = document
            .get_element_by_id("dynamic-styles")
            .or_else(|| {
                let style = document.create_element("style").unwrap();
                style.set_id("dynamic-styles");
                document.head().unwrap().append_child(&style).unwrap();
                Some(style)
            })
            .unwrap();
        if !sidebar_open {
            style_element.set_inner_html(
                r#"
            .main-container {
                padding-left: 70px !important;
            }
            "#,
            );
        } else {
            style_element.set_inner_html("");
        }
    });

    let class = if is_mobile {
        "main-container main-container-mobile"
    } else {
        "main-container"
    };

    view! {
        <div>
            <TopBar />
            <Sidebar tabs=tabs />
            <div class=class>
                <Outlet />
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace", skip(id))]
fn handle_theme(id: String) {
    let update_css = move |custom_css: String| {
        let mut custom_style_sheet = document().get_element_by_id("custom-css");
        if custom_style_sheet.is_none() {
            let el = document().create_element("style").unwrap();
            el.set_id("custom-css");

            if let Some(head) = document().head() {
                head.append_child(&el).unwrap();
            }

            custom_style_sheet = Some(el);
        }

        let custom_style_sheet = custom_style_sheet.unwrap();
        custom_style_sheet.set_inner_html(custom_css.as_str());
    };

    listen_event("theme-updated", move |data| {
        let payload = js_sys::Reflect::get(&data, &JsValue::from_str("payload")).unwrap();
        if let Some(custom_css) = payload.as_string() {
            update_css(custom_css);
        }
    });

    spawn_local(async move {
        let theme = load_theme(id).await.unwrap();

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
            .set_property("--textPrimary", &theme.theme.text_primary)
            .unwrap();
        style
            .set_property("--textSecondary", &theme.theme.text_secondary)
            .unwrap();
        style
            .set_property("--textInverse", &theme.theme.text_inverse)
            .unwrap();
        style.set_property("--accent", &theme.theme.accent).unwrap();
        style
            .set_property("--divider", &theme.theme.divider)
            .unwrap();

        if theme.theme.custom_css.is_some() {
            spawn_local(async move {
                if let Ok(custom_css) = get_css(theme.id).await {
                    update_css(custom_css);
                }
            });
        }
    });
}

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn App() -> impl IntoView {
    let _ = use_event_listener(document().body(), keydown, |ev| {
        if ev.shift_key() && ev.ctrl_key() && ev.key_code() == 75 {
            spawn_local(async move {
                let _ = toggle_dev_tools().await;
            });
        }
    });

    provide_context_menu_state();
    provide_context(RwSignal::new(UiStore::new()));
    provide_context(RwSignal::new(ModalStore::default()));

    {
        let ui_store = expect_context::<RwSignal<UiStore>>();
        let is_mobile = window()
            .get("is_mobile")
            .and_then(|v| v.as_bool())
            .unwrap_or_default();
        ui_store.update(|u| u.set_is_mobile(is_mobile));
        let is_mobile_player = window()
            .get("is_mobile_player")
            .and_then(|v| v.as_bool())
            .unwrap_or_default();
        ui_store.update(|u| u.set_is_mobile_player(is_mobile_player));

        if is_mobile {
            let body = document().body().unwrap();
            body.set_class_name("body-mobile");
        }
    }

    provide_context(PlayerStore::new());
    provide_context(Arc::new(ProviderStore::new()));

    provide_i18n_context::<Locale>();

    spawn_local(async move {
        let id = load_selective("themes.active_theme".into()).await.unwrap();
        handle_theme(serde_wasm_bindgen::from_value(id).unwrap());
    });

    let _ = use_event_listener(document().body(), contextmenu, |ev| {
        ev.prevent_default();
    });

    let owner = Owner::new();
    let ui_requests_unlisten = listen_event("ui-requests", move |data| {
        owner.with(|| {
            let payload = js_sys::Reflect::get(&data, &JsValue::from_str("payload")).unwrap();
            let payload: ExtensionUIRequest = serde_wasm_bindgen::from_value(payload).unwrap();

            #[tracing::instrument(level = "trace", skip(payload, data))]
            fn send_reply<T>(payload: ExtensionUIRequest, data: T)
            where
                T: Serialize + Clone,
            {
                let value = serde_wasm_bindgen::to_value(&data).unwrap();
                spawn_local(async move {
                    let res = emit(format!("ui-reply-{}", payload.channel).as_str(), value);
                    wasm_bindgen_futures::JsFuture::from(res).await.unwrap();
                });
            }

            match payload.type_.as_str() {
                "getCurrentSong" => {
                    let data = create_read_slice(expect_context::<RwSignal<PlayerStore>>(), |p| {
                        p.get_current_song()
                    })
                    .get_untracked();
                    send_reply(payload, data);
                }
                "getVolume" => {
                    let data = create_read_slice(expect_context::<RwSignal<PlayerStore>>(), |p| {
                        p.get_volume()
                    })
                    .get_untracked();
                    send_reply(payload, data);
                }
                "getTime" => {
                    let data = create_read_slice(expect_context::<RwSignal<PlayerStore>>(), |p| {
                        p.get_time()
                    })
                    .get_untracked();
                    send_reply(payload, data);
                }
                "getQueue" => {
                    let data = create_read_slice(expect_context::<RwSignal<PlayerStore>>(), |p| {
                        p.get_queue()
                    })
                    .get_untracked();
                    send_reply(payload, data);
                }
                "getPlayerState" => {
                    let data = create_read_slice(expect_context::<RwSignal<PlayerStore>>(), |p| {
                        p.get_player_state()
                    })
                    .get_untracked();
                    send_reply(payload, data);
                }
                _ => {}
            };
        });
    });

    let watch_prefs_unlisten = watch_preferences(|(key, value)| {
        tracing::debug!("Preferences changed: {} = {:?}", key, value);
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
        } else if key == "prefs.themes.active_theme" {
            let value = value.as_string().unwrap();
            handle_theme(value);
        } else if key == "prefs.i18n_language" {
            let i18n = use_i18n();
            let value = serde_wasm_bindgen::from_value::<Vec<CheckboxPreference>>(value).unwrap();
            if let Some(enabled) = value.into_iter().find(|v| v.enabled) {
                tracing::debug!("Setting locale to {:?}", enabled.key);
                i18n.set_locale(get_locale(&enabled.key))
            }
        }
    });

    let unlisten_mpris = listen_event("media_button_press", move |data: JsValue| {
        let payload = js_sys::Reflect::get(&data, &JsValue::from_str("payload")).unwrap();
        let (key, value): (i32, Option<f64>) = serde_wasm_bindgen::from_value(payload).unwrap();
        let player_store: RwSignal<PlayerStore> = expect_context();

        match key {
            0 => player_store.update(|p| p.set_state(PlayerState::Playing)),
            1 => player_store.update(|p| p.set_state(PlayerState::Paused)),
            2 => player_store.update(|p| p.set_state(PlayerState::Stopped)),
            6 => player_store.update(|p| p.next_song()),
            7 => player_store.update(|p| p.prev_song()),
            12 => player_store.update(|p| p.force_seek(value.unwrap_or_default())),
            13 => player_store.update(|p| match p.get_player_state() {
                PlayerState::Playing => p.set_state(PlayerState::Paused),
                _ => p.set_state(PlayerState::Playing),
            }),
            15 => player_store.update(|p| p.set_volume(value.unwrap_or_default())),

            _ => {}
        }
    });

    let window = window();
    if let Err(e) = window.add_event_listener_with_callback("beforeunload", &watch_prefs_unlisten) {
        tracing::error!("Failed to set unmount hook: {:?}", e);
    }

    if let Err(e) = window.add_event_listener_with_callback("beforeunload", &ui_requests_unlisten) {
        tracing::error!("Failed to set unmount hook: {:?}", e);
    }

    if let Err(e) = window.add_event_listener_with_callback("beforeunload", &unlisten_mpris) {
        tracing::error!("Failed to set unmount hook: {:?}", e);
    }

    view! {
        <Router>
            <main id="app">
                <div class="appContainer">
                    <Routes transition=true fallback=|| "Not found.">
                        <ParentRoute path=path!("/") view=CommonApp>
                            <ParentRoute path=path!("main") view=MainApp>
                                <Route path=path!("allsongs") view=AllSongs />
                                <Route path=path!("playlists") view=AllPlaylists />
                                <Route path=path!("playlists/single") view=SinglePlaylist />
                                <Route path=path!("artists") view=AllArtists />
                                <Route path=path!("artists/single") view=SingleArtist />
                                <Route path=path!("albums") view=AllAlbums />
                                <Route path=path!("albums/single") view=SingleAlbum />
                                <Route path=path!("genres") view=AllGenres />
                                <Route path=path!("genres/single") view=SingleGenre />
                                <Route path=path!("search") view=Search />
                                <Route path=path!("explore") view=Explore />
                            </ParentRoute>
                            <SettingRoutes />
                        </ParentRoute>
                        <Route path=path!("*") view=RedirectAll />
                    </Routes>
                </div>
            </main>
        </Router>
    }
}
