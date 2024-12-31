use std::rc::Rc;

use crate::{
    components::{
        better_animated_outlet::AnimatedOutletSimultaneous, prefs::static_components::SettingRoutes,
    },
    pages::explore::Explore,
    players::librespot::LibrespotPlayer,
    store::ui_store::UiStore,
    utils::{
        common::{emit, listen_event},
        invoke::{get_css, load_selective, load_theme},
        prefs::watch_preferences,
    },
};
use leptos::{
    component, create_effect, create_read_slice, create_rw_signal, document, ev::contextmenu,
    expect_context, provide_context, view, window, IntoView, RwSignal, SignalGet,
    SignalGetUntracked, SignalUpdate,
};
use leptos_context_menu::provide_context_menu_state;
use leptos_i18n::provide_i18n_context;
use leptos_router::{Outlet, Redirect, Route, Router, Routes};
use leptos_use::use_event_listener;
use serde::Serialize;
use types::{
    extensions::ExtensionUIRequest, preferences::CheckboxPreference,
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
    let tabs = vec![
        Tab::new("Queue", "Queue", "queue"),
        Tab::new("All Songs", "All Songs", "/main/allsongs"),
        Tab::new("Playlists", "Playlists", "/main/playlists"),
        Tab::new("Artists", "Artists", "/main/artists"),
        Tab::new("Albums", "Albums", "/main/albums"),
        Tab::new("Genres", "Genres", "/main/genres"),
        Tab::new("Explore", "Explore", "/main/explore"),
    ];
    let ui_store = expect_context::<RwSignal<UiStore>>();
    let sidebar_open = create_read_slice(ui_store, |u| u.get_sidebar_open());

    create_effect(move |_| {
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

    view! {
        <div>
            <TopBar />
            <Sidebar tabs=tabs />
            <AnimatedOutletSimultaneous class="main-container" outro="route-out" intro="route-in" />
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
    // window_event_listener(leptos::ev::contextmenu, move |ev| {
    //     ev.prevent_default();
    // });

    provide_context_menu_state();
    provide_context(PlayerStore::new());
    provide_context(Rc::new(ProviderStore::new()));
    provide_context(create_rw_signal(ModalStore::default()));
    provide_context(create_rw_signal(UiStore::new()));

    provide_i18n_context::<Locale>();

    spawn_local(async move {
        let id = load_selective("themes.active_theme".into()).await.unwrap();
        handle_theme(serde_wasm_bindgen::from_value(id).unwrap());
    });

    let _ = use_event_listener(document().body(), contextmenu, |ev| {
        ev.prevent_default();
    });

    let ui_requests_unlisten = listen_event("ui-requests", move |data| {
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
                let data =
                    create_read_slice(expect_context::<RwSignal<PlayerStore>>(), |p| p.get_time())
                        .get_untracked();
                send_reply(payload, data);
            }
            "getQueue" => {
                let data =
                    create_read_slice(expect_context::<RwSignal<PlayerStore>>(), |p| p.get_queue())
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
                    <Routes>
                        <Route path="/" view=CommonApp>
                            <Route path="main" view=MainApp>
                                <Route path="allsongs" view=AllSongs />
                                <Route path="playlists" view=AllPlaylists />
                                <Route path="playlists/single" view=SinglePlaylist />
                                <Route path="artists" view=AllArtists />
                                <Route path="artists/single" view=SingleArtist />
                                <Route path="albums" view=AllAlbums />
                                <Route path="albums/single" view=SingleAlbum />
                                <Route path="genres" view=AllGenres />
                                <Route path="genres/single" view=SingleGenre />
                                <Route path="search" view=Search />
                                <Route path="explore" view=Explore />
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
