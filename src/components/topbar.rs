use std::rc::Rc;

use crate::icons::{spotify_icon::SpotifyIcon, youtube_icon::YoutubeIcon};
use leptos::{
    component, create_effect, create_node_ref, create_rw_signal, create_write_slice, ev::Event,
    event_target_value, expect_context, view, window, CollectView, IntoView, RwSignal, SignalGet,
    SignalGetUntracked, SignalSet, SignalSetter, SignalUpdate,
};
use leptos_router::{use_navigate, NavigateOptions};
use leptos_use::on_click_outside;
use leptos_virtual_scroller::VirtualScroller;
use types::{
    entities::{
        GetEntityOptions, QueryableAlbum, QueryableArtist, QueryableGenre, QueryablePlaylist,
    },
    errors::Result,
    songs::{GetSongOptions, SearchableSong, Song},
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{MouseEvent, SubmitEvent};

use crate::{
    components::low_img::LowImg,
    icons::{
        next_icon::NextIcon, person_icon::PersonIcon, prev_icon::PrevIcon, search_icon::SearchIcon,
        settings_icon::SettingsIcon,
    },
    store::{
        modal_store::{ModalStore, Modals},
        player_store::PlayerStore,
        provider_store::ProviderStore,
    },
    utils::{common::get_low_img, db_utils::get_songs_by_option},
};

enum InputFocus {
    Focus,
    Blur,
}

#[derive(Clone)]
pub struct SearchResultItemData {
    pub cover: Option<String>,
    pub title: String,
    pub subtitle: String,
    pub on_click: Rc<Box<dyn Fn()>>,
    pub on_icon_click: Rc<Box<dyn Fn()>>,
}

#[tracing::instrument(level = "trace", skip(item))]
#[component]
pub fn SearchResultItem(item: SearchResultItemData) -> impl IntoView {
    view! {
        <div class="container-fluid single-result-container single-result">
            <div class="row justify-content-around">
                <LowImg
                    show_eq=|| false
                    eq_playing=|| false
                    cover_img=item.cover.unwrap_or_default()
                    play_now=move || {
                        item.on_icon_click.as_ref()();
                    }
                />

                <div
                    class="col text-container text-truncate my-auto"
                    on:click=move |_| {
                        item.on_click.as_ref()();
                    }
                >
                    <div class="song-title text-truncate">{item.title}</div>
                    <div class="song-subtitle text-truncate">{item.subtitle}</div>
                </div>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace", skip(class))]
#[component]
pub fn Settings(#[prop(optional)] class: &'static str) -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <div class=class>
            <SettingsIcon on:click=move |_| {
                tracing::debug!("Navigating to settings");
                navigate("/prefs/paths", Default::default())
            } />
        </div>
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn Accounts() -> impl IntoView {
    let show_accounts_popover = create_rw_signal(false);
    let provider_store = expect_context::<Rc<ProviderStore>>();

    let statuses = provider_store.get_all_statuses();

    let modal_store = expect_context::<RwSignal<ModalStore>>();
    let show_login_modal = move |key: String, name: String, account_id: String, logged_in: bool| {
        if logged_in {
            modal_store.update(|m| m.set_active_modal(Modals::SignoutModal(key, name, account_id)))
        } else {
            modal_store.update(|m| m.set_active_modal(Modals::LoginModal(key, name, account_id)))
        }
    };

    let target = create_node_ref();
    let _ = on_click_outside(target, move |_| {
        if show_accounts_popover.get_untracked() {
            show_accounts_popover.set(false);
        }
    });

    view! {
        <div node_ref=target>
            <PersonIcon on:click=move |_| show_accounts_popover.set(!show_accounts_popover.get()) />

            {move || {
                if show_accounts_popover.get() {
                    view! {
                        <div class="accounts-popover custom-popover">
                            <div class="buttons">
                                {move || {
                                    let binding = statuses.get();
                                    binding
                                        .into_iter()
                                        .map(|status| {
                                            let key = status.key.clone();
                                            let (title_out, title_in) = if status.logged_in {
                                                (
                                                    status.user_name.clone().unwrap_or_default(),
                                                    "Sign out".into(),
                                                )
                                            } else {
                                                ("Connect".into(), status.name.clone())
                                            };
                                            let title = create_rw_signal(title_out.clone());
                                            view! {
                                                <div
                                                    class="button-bg d-flex ripple w-100"
                                                    on:mouseover=move |_| {
                                                        title.set(title_in.clone());
                                                    }
                                                    on:mouseout=move |_| {
                                                        title.set(title_out.clone());
                                                    }
                                                    on:click=move |_| show_login_modal(
                                                        key.clone(),
                                                        status.name.clone(),
                                                        status.account_id.clone(),
                                                        status.logged_in,
                                                    )
                                                >

                                                    <div
                                                        class="d-flex w-100 h-100"
                                                        style=("background-color", status.bg_color.clone())
                                                    >
                                                        <div class="icon-wrapper d-flex my-auto">
                                                            {if status.key == "spotify" {
                                                                view! { <SpotifyIcon fill="#07C330".into() /> }
                                                            } else if status.key == "youtube" {
                                                                view! { <YoutubeIcon fill="#E62017".into() /> }
                                                            } else {
                                                                view! {}.into_view()
                                                            }}
                                                        </div>

                                                        <div class="title-wrapper flex-grow-1 my-auto text-truncate">
                                                            {title}
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>
                        </div>
                    }
                        .into_view()
                } else {
                    view! {}.into_view()
                }
            }}

        </div>
    }
}

async fn get_search_res(
    term: String,
    location: String,
    play_now: SignalSetter<Song>,
) -> Result<Vec<SearchResultItemData>> {
    match location.as_str() {
        "/main/artists" => {
            let res = crate::utils::invoke::get_entity_by_options(GetEntityOptions {
                artist: Some(QueryableArtist {
                    artist_name: Some(term.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?;

            let res = serde_wasm_bindgen::from_value::<Vec<QueryableArtist>>(res)?;
            Ok(res
                .into_iter()
                .map(|a| SearchResultItemData {
                    cover: a.artist_coverpath.clone(),
                    title: a.artist_name.clone().unwrap_or_default(),
                    subtitle: String::default(),
                    on_click: Rc::new(Box::new(move || {
                        use_navigate()(
                            format!(
                                "/main/artists/single?entity={}",
                                serde_json::to_string(&a).unwrap()
                            )
                            .as_str(),
                            NavigateOptions::default(),
                        );
                    })),
                    on_icon_click: Rc::new(Box::new(move || {})),
                })
                .collect())
        }
        "/main/albums" => {
            let res = crate::utils::invoke::get_entity_by_options(GetEntityOptions {
                album: Some(QueryableAlbum {
                    album_name: Some(term.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?;

            let res = serde_wasm_bindgen::from_value::<Vec<QueryableAlbum>>(res)?;
            Ok(res
                .into_iter()
                .map(|a| SearchResultItemData {
                    cover: a.album_coverpath_low.clone(),
                    title: a.album_name.clone().unwrap_or_default(),
                    subtitle: String::default(),
                    on_click: Rc::new(Box::new(move || {
                        use_navigate()(
                            format!(
                                "/main/albums/single?entity={}",
                                serde_json::to_string(&a).unwrap()
                            )
                            .as_str(),
                            NavigateOptions::default(),
                        );
                    })),
                    on_icon_click: Rc::new(Box::new(move || {})),
                })
                .collect())
        }
        "/main/genres" => {
            let res = crate::utils::invoke::get_entity_by_options(GetEntityOptions {
                genre: Some(QueryableGenre {
                    genre_name: Some(term.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?;

            let res = serde_wasm_bindgen::from_value::<Vec<QueryableGenre>>(res)?;
            Ok(res
                .into_iter()
                .map(|a| SearchResultItemData {
                    cover: None,
                    title: a.genre_name.clone().unwrap_or_default(),
                    subtitle: String::default(),
                    on_click: Rc::new(Box::new(move || {
                        use_navigate()(
                            format!(
                                "/main/genres/single?entity={}",
                                serde_json::to_string(&a).unwrap()
                            )
                            .as_str(),
                            NavigateOptions::default(),
                        );
                    })),
                    on_icon_click: Rc::new(Box::new(move || {})),
                })
                .collect())
        }
        "/main/playlists" => {
            let res = crate::utils::invoke::get_entity_by_options(GetEntityOptions {
                playlist: Some(QueryablePlaylist {
                    playlist_name: term.clone(),
                    playlist_path: Some(term.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?;

            let res = serde_wasm_bindgen::from_value::<Vec<QueryablePlaylist>>(res)?;

            Ok(res
                .into_iter()
                .map(|a| SearchResultItemData {
                    cover: a.playlist_coverpath.clone(),
                    title: a.playlist_name.clone(),
                    subtitle: String::default(),
                    on_click: Rc::new(Box::new(move || {
                        use_navigate()(
                            format!(
                                "/main/playlists/single?entity={}",
                                serde_json::to_string(&a).unwrap()
                            )
                            .as_str(),
                            NavigateOptions::default(),
                        );
                    })),
                    on_icon_click: Rc::new(Box::new(move || {})),
                })
                .collect())
        }
        _ => {
            let res = crate::utils::invoke::get_songs_by_options(GetSongOptions {
                song: Some(SearchableSong {
                    title: Some(term.clone()),
                    path: Some(term.clone()),
                    ..Default::default()
                }),
                album: Some(QueryableAlbum {
                    album_name: Some(term.clone()),
                    ..Default::default()
                }),
                artist: Some(QueryableArtist {
                    artist_name: Some(term.clone()),
                    ..Default::default()
                }),
                genre: Some(QueryableGenre {
                    genre_name: Some(term.clone()),
                    ..Default::default()
                }),
                playlist: Some(QueryablePlaylist {
                    playlist_name: term,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?;
            Ok(res
                .into_iter()
                .map(|s| SearchResultItemData {
                    cover: s.album.as_ref().and_then(|a| a.album_coverpath_low.clone()),
                    title: s.song.title.clone().unwrap_or_default(),
                    subtitle: s
                        .artists
                        .as_ref()
                        .map(|a| {
                            a.iter()
                                .filter_map(|a| a.artist_name.clone())
                                .collect::<Vec<_>>()
                                .join(",")
                        })
                        .unwrap_or_default(),
                    on_click: Rc::new(Box::new(move || {})),
                    on_icon_click: Rc::new(Box::new(move || {
                        play_now.set(s.clone());
                    })),
                })
                .collect())
        }
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn TopBar() -> impl IntoView {
    let show_searchbar = create_rw_signal(false);
    let input_value = create_rw_signal("".to_string());
    let results = create_rw_signal(vec![]);
    let handle_input_focus = move |focus: InputFocus| match focus {
        InputFocus::Focus => {
            show_searchbar.set(!results.get().is_empty());
        }
        InputFocus::Blur => show_searchbar.set(false),
    };

    create_effect(move |_| {
        let results = results.get();
        if !results.is_empty() {
            show_searchbar.set(true);
        }
    });

    let handle_page_change = move |ev: SubmitEvent| {
        ev.prevent_default();
        let text = input_value.get();
        let navigate = leptos_router::use_navigate();
        navigate(
            format!("/main/search?q={}", text).as_str(),
            Default::default(),
        );
    };

    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let play_now = create_write_slice(player_store, |p, val| p.play_now(val));

    let handle_text_change = move |ev: Event| {
        let text = event_target_value(&ev);
        input_value.set(text.clone());
        if text.is_empty() {
            return;
        }
        let value = format!("%{}%", text);
        let current_page = window().location().pathname().unwrap();
        tracing::debug!("current page {}", current_page);

        spawn_local(async move {
            let search_res = get_search_res(value.clone(), current_page, play_now).await;
            match search_res {
                Ok(res) => {
                    results.set(res);
                }
                Err(e) => tracing::error!("Failed to search {}: {:?}", value, e),
            }
        });

        //
    };

    view! {
        <div class="topbar-container align-items-center topbar is-open">
            <div class="container-fluid d-flex">
                <div class="row justify-content-start flex-grow-1">
                    <div class="col-auto my-auto">
                        // Prev next buttons
                        <div class="row justify-content-between">
                            <div class="col-6">
                                <PrevIcon on:click=move |_| {
                                    window().history().unwrap().back().unwrap();
                                } />
                            </div>
                            <div class="col-6">
                                <NextIcon on:click=move |_| {
                                    window().history().unwrap().forward().unwrap();
                                } />
                            </div>
                        </div>
                    </div>
                    // searchbar
                    <div class="col">
                        <div class="h-100 d-flex align-items-center search-container">
                            <div
                                class="w-100 searchbar-container"
                                class:full-border=move || !show_searchbar.get()
                                class:half-border=move || show_searchbar.get()
                            >
                                <div class="search-icon">
                                    <SearchIcon accent=true />
                                </div>
                                <form on:submit=handle_page_change>
                                    <input
                                        class="form-control searchbar"
                                        placeholder="Search..."

                                        on:blur=move |_| handle_input_focus(InputFocus::Blur)
                                        on:focus=move |_| handle_input_focus(InputFocus::Focus)
                                        on:input=handle_text_change
                                    />

                                </form>
                            </div>

                            <div
                                class="search-results d-flex"
                                class:search-invisible=move || !show_searchbar.get()
                            >
                                <VirtualScroller
                                    each=results
                                    item_height=95usize
                                    children=move |(_, item)| {
                                        view! { <SearchResultItem item=item.clone() /> }
                                    }
                                />
                                <div class="w-100"></div>
                            </div>
                        </div>
                    </div>

                    // Extra buttons
                    <div class="col-auto pr-5 ml-auto my-auto icons-bar d-flex">
                        <div class="row flex-grow-1">
                            <div class="col-auto d-flex">
                                <Accounts />
                                <Settings class="ml-2" />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
