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

use crate::components::provider_icon::ProviderIcon;
use crate::i18n::use_i18n;
use crate::store::ui_store::UiStore;
use leptos::task::spawn_local;
use leptos::{
    component, ev::Event, prelude::*, reactive::wrappers::write::SignalSetter, view, IntoView,
};
use leptos_i18n::t_string;
use leptos_router::{hooks::use_navigate, NavigateOptions};
use leptos_use::on_click_outside;
use leptos_virtual_scroller::VirtualScroller;
use types::ui::extensions::ExtensionProviderScope;
use types::{
    entities::{
        GetEntityOptions, QueryableAlbum, QueryableArtist, QueryableGenre, QueryablePlaylist,
    },
    errors::Result,
    songs::{GetSongOptions, SearchableSong, Song},
};
use web_sys::SubmitEvent;

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
};

enum InputFocus {
    Focus,
    Blur,
}

#[derive(Clone)]
pub struct SearchResultItemData {
    pub key: Option<String>,
    pub cover: Option<String>,
    pub title: String,
    pub subtitle: String,
    pub on_click: Arc<Box<dyn Fn() + Send + Sync>>,
    pub on_icon_click: Arc<Box<dyn Fn() + Send + Sync>>,
}

#[tracing::instrument(level = "debug", skip(item))]
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

#[tracing::instrument(level = "debug", skip(class))]
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

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn Accounts() -> impl IntoView {
    let show_accounts_popover = RwSignal::new(false);
    let provider_store = expect_context::<Arc<ProviderStore>>();

    let statuses = provider_store.get_all_statuses();

    let modal_store = expect_context::<RwSignal<ModalStore>>();
    let show_login_modal = move |key: String, name: String, account_id: String, logged_in: bool| {
        if logged_in {
            modal_store.update(|m| m.set_active_modal(Modals::SignoutModal(key, name, account_id)))
        } else {
            modal_store.update(|m| m.set_active_modal(Modals::LoginModal(key, name, account_id)))
        }
    };

    let target = NodeRef::new();
    let _ = on_click_outside(target, move |_| {
        if show_accounts_popover.get_untracked() {
            show_accounts_popover.set(false);
        }
    });

    let i18n = use_i18n();
    view! {
        <div node_ref=target>
            <PersonIcon on:click=move |_| show_accounts_popover.set(!show_accounts_popover.get()) />

            {move || {
                if show_accounts_popover.get() {
                    view! {
                        <div class="accounts-popover custom-popover">
                            <div class="buttons">
                                {move || {
                                    let binding = statuses
                                        .get()
                                        .into_iter()
                                        .filter(|s| {
                                            s.scopes.contains(&ExtensionProviderScope::Accounts)
                                        });
                                    binding
                                        .map(|status| {
                                            let key = status.key.clone();
                                            let (title_out, title_in) = if status.logged_in {
                                                (
                                                    status.user_name.clone().unwrap_or_default(),
                                                    t_string!(i18n, accounts.sign_out).into(),
                                                )
                                            } else {
                                                (
                                                    t_string!(i18n, accounts.connect).into(),
                                                    status.name.clone(),
                                                )
                                            };
                                            let title = RwSignal::new(title_out.clone());
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
                                                            <ProviderIcon extension=key.clone() accounts=true />
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
                        .into_any()
                } else {
                    ().into_any()
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
                    key: a.artist_id.clone(),
                    cover: a.artist_coverpath.clone(),
                    title: a.artist_name.clone().unwrap_or_default(),
                    subtitle: String::default(),
                    on_click: Arc::new(Box::new(move || {
                        use_navigate()(
                            format!(
                                "/main/artists/single?entity={}",
                                url_escape::encode_component(&serde_json::to_string(&a).unwrap())
                            )
                            .as_str(),
                            NavigateOptions::default(),
                        );
                    })),
                    on_icon_click: Arc::new(Box::new(move || {})),
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
                    key: a.album_id.clone(),
                    cover: a.album_coverpath_low.clone(),
                    title: a.album_name.clone().unwrap_or_default(),
                    subtitle: String::default(),
                    on_click: Arc::new(Box::new(move || {
                        use_navigate()(
                            format!(
                                "/main/albums/single?entity={}",
                                url_escape::encode_component(&serde_json::to_string(&a).unwrap())
                            )
                            .as_str(),
                            NavigateOptions::default(),
                        );
                    })),
                    on_icon_click: Arc::new(Box::new(move || {})),
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
                    key: a.genre_id.clone(),
                    cover: None,
                    title: a.genre_name.clone().unwrap_or_default(),
                    subtitle: String::default(),
                    on_click: Arc::new(Box::new(move || {
                        use_navigate()(
                            format!(
                                "/main/genres/single?entity={}",
                                url_escape::encode_component(&serde_json::to_string(&a).unwrap())
                            )
                            .as_str(),
                            NavigateOptions::default(),
                        );
                    })),
                    on_icon_click: Arc::new(Box::new(move || {})),
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
                    key: a.playlist_id.clone(),
                    cover: a.playlist_coverpath.clone(),
                    title: a.playlist_name.clone(),
                    subtitle: String::default(),
                    on_click: Arc::new(Box::new(move || {
                        use_navigate()(
                            format!(
                                "/main/playlists/single?entity={}",
                                url_escape::encode_component(&serde_json::to_string(&a).unwrap())
                            )
                            .as_str(),
                            NavigateOptions::default(),
                        );
                    })),
                    on_icon_click: Arc::new(Box::new(move || {})),
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
                    key: s.song._id.clone(),
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
                    on_click: Arc::new(Box::new(move || {})),
                    on_icon_click: Arc::new(Box::new(move || {
                        play_now.set(s.clone());
                    })),
                })
                .collect())
        }
    }
}

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn TopBar() -> impl IntoView {
    let show_searchbar = RwSignal::new(false);
    let input_value = RwSignal::new("".to_string());
    let results = RwSignal::new(vec![]);
    let handle_input_focus = move |focus: InputFocus| match focus {
        InputFocus::Focus => {
            show_searchbar.set(!results.get().is_empty());
        }
        InputFocus::Blur => show_searchbar.set(false),
    };

    Effect::new(move || {
        let results = results.get();
        show_searchbar.set(!results.is_empty());
    });

    let navigate = leptos_router::hooks::use_navigate();
    let handle_page_change = move |ev: SubmitEvent| {
        ev.prevent_default();
        let text = input_value.get();
        navigate(
            format!("/main/search?q={text}").as_str(),
            Default::default(),
        );
    };

    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let play_now = create_write_slice(player_store, |p, val| p.play_now(val));

    let handle_text_change = move |ev: Event| {
        let text = event_target_value(&ev);
        input_value.set(text.clone());
        if text.is_empty() {
            results.update(|r| r.clear());
            return;
        }
        let value = format!("%{text}%");
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
    };

    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();

    let i18n = use_i18n();
    view! {
        <div
            class="topbar-container align-items-center topbar is-open"
            class:topbar-mobile=is_mobile
        >
            <div class="container-fluid d-flex">
                <div class="row justify-content-start flex-grow-1">
                    <div class="col-auto my-auto nav-icons">
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
                                        placeholder=move || {
                                            t_string!(i18n, topbar.searchPlaceholder)
                                        }

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
                                    key=|(_, r)| r.key.clone()
                                    item_height=95usize
                                    children=move |(_, item)| {
                                        view! { <SearchResultItem item=item.clone() /> }
                                    }
                                    header=Some(())
                                />
                            // <div class="w-100"></div>
                            </div>
                        </div>
                    </div>

                    // Extra buttons
                    <div
                        class="col-auto pr-5 ml-auto my-auto icons-bar d-flex"
                        class:icons-invisible=move || is_mobile && show_searchbar.get()
                    >
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
