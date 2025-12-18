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

use std::{hash::Hash, sync::Arc};

use crate::utils::error::MoosyncError;
use crate::{
    components::provider_icon::ProviderIcon,
    icons::{
        fav_playlist_icon::FavPlaylistIcon, play_hover_icon::PlayHoverIcon,
        song_default_icon::SongDefaultIcon,
    },
    store::ui_store::UiStore,
    utils::common::convert_file_src,
};
use leptos::{IntoView, component, prelude::*, view};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use leptos_virtual_scroller::VirtualGridScroller;
use serde::Serialize;

type CardContextMenu<T> = Option<Arc<Box<dyn Fn(leptos::ev::MouseEvent, T)>>>;

#[derive(Default, Clone)]
pub struct SimplifiedCardItem<T>
where
    T: Serialize + Send + Sync,
{
    pub title: String,
    pub cover: Option<String>,
    pub id: T,
    pub icon: Option<String>,
    pub context_menu: CardContextMenu<T>,
}

#[tracing::instrument(level = "debug", skip(item, songs_view, on_click))]
#[component()]
pub fn CardItem<T>(
    #[prop()] item: SimplifiedCardItem<T>,
    #[prop(optional, default = false)] songs_view: bool,
    #[prop(optional)] on_click: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
    #[prop(optional)] is_mobile: bool,
) -> impl IntoView
where
    T: Serialize + Send + Sync,
{
    let show_default_icon = RwSignal::new(item.cover.is_none());

    let item_width = if is_mobile {
        (document().body().unwrap().client_width() / 2) - 30
    } else {
        200
    } as usize;

    view! {
        <div
            class="card mb-2 card-grow"
            style=move || {
                if !is_mobile {
                    "width: 200px;".to_string()
                } else {
                    format!("width: {item_width}px;")
                }
            }
        >
            <div class="card-img-top">
                <div class="embed-responsive embed-responsive-1by1">
                    <div class="embed-responsive-item img-container">

                        <div class="card_overlay">
                            {move || {
                                let on_click = on_click.clone();
                                if songs_view {
                                    view! {
                                        <div
                                            class="play-button-song-list card-overlay-background d-flex justify-content-center w-100 h-100"
                                            on:click=move |_| {
                                                let on_click = on_click.clone();
                                                if let Some(cb) = on_click {
                                                    cb();
                                                }
                                            }
                                        >
                                            <PlayHoverIcon />
                                        </div>
                                    }
                                        .into_any()
                                } else {
                                    ().into_any()
                                }
                            }}
                        </div>

                        <div class="provider-icon-overlay me-auto justify-content-center d-flex align-items-center">
                            {if let Some(icon) = item.icon.clone() {
                                view! { <ProviderIcon extension=icon /> }.into_any()
                            } else {
                                ().into_any()
                            }}
                        </div>
                        {move || {
                            if show_default_icon.get() {
                                view! {
                                    <SongDefaultIcon class="rounded-corners img-fluid w-100 h-100"
                                        .into() />
                                }
                                    .into_any()
                            } else {
                                if let Some(cover) = item.cover.clone() {
                                    if cover == "favorites" {
                                        return view! {
                                            <FavPlaylistIcon class="rounded-corners img-fluid w-100 h-100" />
                                        }
                                            .into_any();
                                    }
                                }
                                view! {
                                    <img
                                        src=item.cover.clone().map(convert_file_src)
                                        class="rounded-corners img-fluid w-100 h-100"
                                        on:error=move |e| {
                                            tracing::error!(
                                                "Error loading cover image {:?}", MoosyncError::from(e.error())
                                            );
                                            show_default_icon.set(true);
                                        }
                                    />
                                }
                                    .into_any()
                            }
                        }}

                    </div>
                </div>
            </div>
            <div class="card-body">
                <p class="card-title text-truncate">{item.title}</p>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "debug", skip(items, card_item, songs_view, on_click, key))]
#[component()]
pub fn CardView<T, S, C, K, KN>(
    #[prop()] items: S,
    #[prop()] key: KN,
    #[prop()] card_item: C,
    #[prop(optional, default = false)] songs_view: bool,
    #[prop(optional)] on_click: Option<Box<dyn Fn(T) + Send + Sync>>,
    #[prop(optional, default = "")] redirect_root: &'static str,
) -> impl IntoView
where
    T: 'static + Clone + Serialize + Send + Sync,
    C: Fn((usize, &T)) -> SimplifiedCardItem<T> + 'static + Send + Sync + Clone,
    KN: (Fn(&T) -> K) + 'static + Send + Sync + Clone,
    K: Eq + Hash + 'static,
    S: With<Value = Vec<T>> + Copy + 'static + Send + Sync,
{
    let on_click = on_click.map(Arc::new);

    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();

    let item_width = if is_mobile {
        (document().body().unwrap().client_width() / 2) - 15
    } else {
        220
    } as usize;
    let item_height = if is_mobile {
        (document().body().unwrap().client_width() / 2) + 55
    } else {
        275
    } as usize;

    let owner = Owner::new();
    view! {
        <VirtualGridScroller
            each=items
            key=move |(_, item)| key(item)
            item_width=item_width
            item_height=item_height
            children=move |data| {
                let data1 = data.1.clone();
                let data2 = data.1.clone();
                let card_item_data = card_item(data);
                let card_item_data1 = card_item_data.clone();
                let on_click = on_click.clone();
                let owner = owner.clone();
                if songs_view {
                    let owner = owner.clone();
                    let owner_cl = owner.clone();
                    view! {
                        <CardItem
                            on:contextmenu=move |ev| {
                                ev.prevent_default();
                                if let Some(cb) = &card_item_data1.context_menu {
                                    owner.with(|| cb(ev, data1.clone()));
                                }
                            }
                            item=card_item_data
                            songs_view=songs_view
                            is_mobile=is_mobile
                            on_click=Arc::new(
                                Box::new(move || {
                                    if let Some(cb) = on_click.clone() {
                                        owner_cl.with(|| cb(data2.clone()));
                                    }
                                }),
                            )
                        />
                    }
                        .into_any()
                } else {
                    let id = card_item_data.id.clone();
                    let owner = owner.clone();
                    let owner_cl = owner.clone();
                    view! {
                        <div on:click=move |_| {
                            owner
                                .with(|| {
                                    use_navigate()(
                                        format!(
                                            "{}/single?entity={}",
                                            redirect_root,
                                            url_escape::encode_component(
                                                &serde_json::to_string(&id).unwrap(),
                                            ),
                                        )
                                            .as_str(),
                                        NavigateOptions::default(),
                                    );
                                });
                        }>
                            <CardItem
                                on:contextmenu=move |ev| {
                                    ev.prevent_default();
                                    if let Some(cb) = &card_item_data1.context_menu {
                                        owner_cl.with(|| cb(ev, data1.clone()));
                                    }
                                }
                                item=card_item_data
                                songs_view=songs_view
                                is_mobile=is_mobile
                            />
                        </div>
                    }
                        .into_any()
                }
            }
        />
    }
}
