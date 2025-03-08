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

use leptos::{prelude::*, task::spawn_local};
use leptos_context_menu::{
    BottomSheet, ContextMenu, ContextMenuData, ContextMenuItemInner, ContextMenuItems, Menu,
};
use leptos_router::{
    hooks::{use_navigate, use_query_map},
    NavigateOptions,
};
use types::{
    entities::{QueryableArtist, QueryablePlaylist},
    songs::Song,
    ui::extensions::ExtensionProviderScope,
};

use crate::{
    i18n::use_i18n,
    modals::new_playlist_modal::PlaylistModalState,
    store::{
        modal_store::{ModalStore, Modals},
        player_store::PlayerStore,
        provider_store::ProviderStore,
        ui_store::UiStore,
    },
    utils::{entities::get_playlist_sort_cx_items, songs::get_songs_from_indices},
};

use super::{
    db_utils::{
        add_songs_to_library, add_to_playlist, create_playlist, export_playlist, remove_playlist,
        remove_songs_from_library,
    },
    invoke::{
        get_playlist_context_menu, get_song_context_menu, load_theme, trigger_context_menu_action,
    },
    songs::get_sort_cx_items,
};

#[derive(Clone)]
pub struct SongItemContextMenu<T>
where
    T: Get<Value = Vec<Song>> + Send + Sync + 'static,
{
    pub current_song: Option<Song>,
    pub song_list: T,
    pub selected_songs: RwSignal<Vec<usize>>,
    pub playlists: RwSignal<Vec<QueryablePlaylist>>,
    pub refresh_cb: Arc<Box<dyn Fn() + Send + Sync>>,
}

impl<T> SongItemContextMenu<T>
where
    T: Get<Value = Vec<Song>> + Send + Sync + 'static,
{
    #[tracing::instrument(level = "debug", skip(self))]
    fn current_or_list(&self) -> Vec<Song> {
        let selected_songs = self.selected_songs.get();
        let ret = if selected_songs.is_empty() {
            if let Some(song) = self.current_song.as_ref() {
                vec![song.clone()]
            } else {
                vec![]
            }
        } else {
            get_songs_from_indices(&self.song_list, self.selected_songs)
        };

        tracing::debug!("Got songs {:?}", ret);
        ret
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn play_now(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.play_now_multiple(self.current_or_list()));
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn add_to_queue(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.add_to_queue(self.current_or_list()));
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn play_next(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.play_next_multiple(self.current_or_list()));
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn clear_queue_and_play(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| {
            store.clear_queue();
            store.play_now_multiple(self.current_or_list())
        });
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn add_to_library(&self) {
        add_songs_to_library(self.current_or_list(), self.refresh_cb.clone());
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_from_library(&self) {
        remove_songs_from_library(self.current_or_list(), self.refresh_cb.clone());
    }

    #[tracing::instrument(level = "debug", skip(self, id))]
    pub fn add_to_playlist(&self, id: String) {
        add_to_playlist(id, self.current_or_list());
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn goto_album(&self) {
        let navigate = use_navigate();
        if let Some(song) = &self.current_song {
            if let Some(album) = &song.album {
                navigate(
                    format!(
                        "/main/albums/single?entity={}",
                        url_escape::encode_component(&serde_json::to_string(&album).unwrap())
                    )
                    .as_str(),
                    NavigateOptions::default(),
                );
            }
        }
    }

    #[tracing::instrument(level = "debug", skip(self, artist))]
    pub fn goto_artist(&self, artist: QueryableArtist) {
        let navigate = use_navigate();
        navigate(
            format!(
                "/main/artists/single?entity={}",
                url_escape::encode_component(&serde_json::to_string(&artist).unwrap())
            )
            .as_str(),
            NavigateOptions::default(),
        );
    }

    pub fn remove_from_playlist(&self) {
        let params = use_query_map();
        let playlist = Memo::new(move |_| {
            params.with(|params| {
                let entity = params.get("entity");
                if let Some(entity) = entity {
                    let playlist = serde_json::from_str::<QueryablePlaylist>(&entity);
                    if let Ok(playlist) = playlist {
                        return Some(playlist);
                    }
                }
                None
            })
        });

        let playlist = playlist.get();
        if let Some(playlist_id) = playlist.and_then(|p| p.playlist_id) {
            let selected_songs = self
                .current_or_list()
                .into_iter()
                .filter_map(|s| s.song._id)
                .collect();
            let refresh_cb = self.refresh_cb.clone();
            spawn_local(async move {
                let res =
                    crate::utils::invoke::remove_from_playlist(playlist_id, selected_songs).await;
                if let Err(e) = res {
                    tracing::error!("Error removing songs from playlist: {:?}", e);
                } else {
                    refresh_cb.as_ref()();
                }
            });
        }
    }
}

impl<T> ContextMenuData<Self> for SongItemContextMenu<T>
where
    T: Get<Value = Vec<Song>> + Send + Sync + 'static,
{
    #[tracing::instrument(level = "debug", skip(self))]
    fn get_menu_items(&self) -> ReadSignal<ContextMenuItems<Self>> {
        let i18n = use_i18n();

        let mut artist_items = vec![];
        if let Some(song) = &self.current_song {
            if let Some(artists) = &song.artists {
                for artist in artists.clone() {
                    let artist_name = artist.artist_name.clone().unwrap_or_default();
                    artist_items.push(ContextMenuItemInner::<Self>::new_with_handler(
                        artist_name,
                        move |_, cx| cx.goto_artist(artist.clone()),
                        None,
                    ))
                }
            }
        }

        let mut playlist_items = vec![];
        for playlist in self.playlists.get().iter() {
            let playlist_name = playlist.playlist_name.clone();
            let playlist_id = playlist.playlist_id.clone().unwrap_or_default();
            playlist_items.push(ContextMenuItemInner::<Self>::new_with_handler(
                playlist_name,
                move |_, cx| cx.add_to_playlist(playlist_id.clone()),
                None,
            ))
        }

        let library_menu_item = if self
            .current_song
            .clone()
            .map(|s| s.song.library_item.unwrap_or_default())
            .unwrap_or_default()
        {
            ContextMenuItemInner::<Self>::new_with_handler(
                i18n.get_keys()
                    .contextMenu()
                    .song()
                    .remove()
                    .build_string()
                    .into(),
                |_, cx| cx.remove_from_library(),
                None,
            )
        } else {
            ContextMenuItemInner::<Self>::new_with_handler(
                i18n.get_keys()
                    .contextMenu()
                    .song()
                    .add()
                    .build_string()
                    .into(),
                |_, cx| cx.add_to_library(),
                None,
            )
        };

        let ret: RwSignal<ContextMenuItems<Self>> = RwSignal::new(vec![
            ContextMenuItemInner::new_with_handler("Play now".into(), |_, cx| cx.play_now(), None),
            ContextMenuItemInner::new_with_handler(
                i18n.get_keys()
                    .contextMenu()
                    .song()
                    .playNext()
                    .build_string()
                    .into(),
                |_, cx| cx.play_next(),
                None,
            ),
            ContextMenuItemInner::new_with_handler(
                i18n.get_keys()
                    .contextMenu()
                    .song()
                    .clearAndPlay()
                    .build_string()
                    .into(),
                |_, cx| cx.clear_queue_and_play(),
                None,
            ),
            ContextMenuItemInner::new_with_handler(
                i18n.get_keys()
                    .contextMenu()
                    .song()
                    .addToQueue()
                    .build_string()
                    .into(),
                |_, cx| cx.add_to_queue(),
                None,
            ),
            ContextMenuItemInner::new(
                i18n.get_keys()
                    .contextMenu()
                    .playlist()
                    .add()
                    .build_string()
                    .into(),
                Some(playlist_items),
            ),
            library_menu_item,
            ContextMenuItemInner::new_with_handler(
                i18n.get_keys()
                    .contextMenu()
                    .song()
                    .gotoAlbum()
                    .build_string()
                    .into(),
                |_, cx| cx.goto_album(),
                None,
            ),
            ContextMenuItemInner::new(
                i18n.get_keys()
                    .contextMenu()
                    .song()
                    .gotoArtists()
                    .build_string()
                    .into(),
                Some(artist_items),
            ),
        ]);

        let location = window().location().pathname().unwrap();
        if location.contains("playlists/single") {
            ret.update(|ret| {
                ret.insert(
                    5,
                    ContextMenuItemInner::new_with_handler(
                        i18n.get_keys()
                            .contextMenu()
                            .song()
                            .removeFromPlaylist()
                            .build_string()
                            .into(),
                        |_, cx| cx.remove_from_playlist(),
                        None,
                    ),
                );
            });
        }

        let provider_store = expect_context::<Arc<ProviderStore>>();
        let song_list = self.current_or_list();
        spawn_local(async move {
            let valid_providers =
                provider_store.get_provider_keys(ExtensionProviderScope::SongContextMenu);
            for key in valid_providers {
                let song_list = song_list.clone();
                let ctx_menu = get_song_context_menu(key.clone(), song_list).await;
                match ctx_menu {
                    Ok(ctx_menu) => {
                        let key = key.clone();
                        ret.update(move |menu| {
                            menu.extend(ctx_menu.into_iter().map(move |v| {
                                let key = key.clone();
                                ContextMenuItemInner::new_with_handler(
                                    v.name,
                                    move |_, _| {
                                        let key = key.clone();
                                        let action = v.action_id.clone();
                                        spawn_local(async move {
                                            trigger_context_menu_action(key, action).await.unwrap()
                                        });
                                    },
                                    None,
                                )
                            }));
                        });
                    }
                    Err(e) => tracing::error!("Failed to get context menu from {}: {:?}", key, e),
                }
            }
        });

        ret.read_only()
    }
}

pub struct SortContextMenu {}

impl ContextMenuData<Self> for SortContextMenu {
    #[tracing::instrument(level = "debug", skip(self))]
    fn get_menu_items(&self) -> ReadSignal<ContextMenuItems<Self>> {
        RwSignal::new(get_sort_cx_items()).read_only()
    }
}

pub struct ThemesContextMenu {
    pub id: Option<String>,
    pub refresh_cb: Arc<Box<dyn Fn() + Send + Sync>>,
}

impl ThemesContextMenu {
    #[tracing::instrument(level = "debug", skip(self))]
    fn export_theme(&self) {
        let id = self.id.clone();
        if let Some(id) = id {
            spawn_local(async move {
                let res = crate::utils::invoke::export_theme(id).await;
                if let Err(err) = res {
                    tracing::error!("Error exporting theme {:?}", err);
                }
            });
        }
    }

    fn remove_theme(&self) {
        let id = self.id.clone();
        if let Some(id) = id {
            let refresh_cb = self.refresh_cb.clone();
            spawn_local(async move {
                let res = crate::utils::invoke::remove_theme(id).await;
                if let Err(err) = res {
                    tracing::error!("Error removing theme {:?}", err);
                }
                refresh_cb.as_ref()();
            });
        }
    }

    fn edit_theme(&self) {
        let modal_store: RwSignal<ModalStore> = expect_context();
        let id = self.id.clone();
        let refresh_cb = self.refresh_cb.clone();
        if let Some(id) = id {
            spawn_local(async move {
                let theme = load_theme(id).await;
                if let Ok(theme) = theme {
                    modal_store.update(|m| {
                        m.set_active_modal(Modals::ThemeModal(
                            types::ui::themes::ThemeModalState::NewTheme(theme),
                        ));
                        m.on_modal_close(move || refresh_cb.as_ref()());
                    });
                }
            });
        }
    }
}

impl ContextMenuData<Self> for ThemesContextMenu {
    #[tracing::instrument(level = "debug", skip(self))]
    fn get_menu_items(&self) -> ReadSignal<ContextMenuItems<Self>> {
        RwSignal::new(vec![
            ContextMenuItemInner::<Self>::new_with_handler(
                "Edit theme".into(),
                |_, cx| cx.edit_theme(),
                None,
            ),
            ContextMenuItemInner::new_with_handler(
                "Export theme".into(),
                |_, cx| cx.export_theme(),
                None,
            ),
            ContextMenuItemInner::new_with_handler(
                "Remove theme".into(),
                |_, cx| cx.remove_theme(),
                None,
            ),
        ])
        .read_only()
    }
}

pub struct PlaylistContextMenu {
    pub refresh_cb: Arc<Box<dyn Fn() + Send + Sync>>,
}

impl PlaylistContextMenu {
    #[tracing::instrument(level = "debug", skip(self))]
    fn open_import_from_url_modal(&self) {
        let modal_store: RwSignal<ModalStore> = expect_context();
        modal_store.update(|modal_store| {
            modal_store.set_active_modal(Modals::NewPlaylistModal(PlaylistModalState::None, None));
            let cb = self.refresh_cb.clone();
            modal_store.on_modal_close(move || {
                cb.as_ref()();
            });
        });
    }
}

impl ContextMenuData<Self> for PlaylistContextMenu {
    #[tracing::instrument(level = "debug", skip(self))]
    fn get_menu_items(&self) -> ReadSignal<ContextMenuItems<Self>> {
        let i18n = use_i18n();
        RwSignal::new(vec![
            ContextMenuItemInner::<Self>::new_with_handler(
                i18n.get_keys()
                    .contextMenu()
                    .playlist()
                    .addFromURL()
                    .build_string()
                    .into(),
                |_, cx| cx.open_import_from_url_modal(),
                None,
            ),
            ContextMenuItemInner::new(
                i18n.get_keys()
                    .contextMenu()
                    .sort_by()
                    .build_string()
                    .into(),
                Some(get_playlist_sort_cx_items()),
            ),
        ])
        .read_only()
    }
}

pub struct PlaylistItemContextMenu {
    pub playlist: Option<QueryablePlaylist>,
    pub refresh_cb: Arc<Box<dyn Fn() + Send + Sync>>,
}

impl PlaylistItemContextMenu {
    #[tracing::instrument(level = "debug", skip(self))]
    fn add_to_library(&self) {
        if let Some(playlist) = &self.playlist {
            create_playlist(playlist.clone(), None);
            self.refresh_cb.as_ref()();
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn remove_from_library(&self) {
        if let Some(playlist) = &self.playlist {
            remove_playlist(playlist.clone(), self.refresh_cb.clone());
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn export_playlist(&self) {
        if let Some(playlist) = &self.playlist {
            export_playlist(playlist.clone());
        }
    }
}

impl ContextMenuData<Self> for PlaylistItemContextMenu {
    #[tracing::instrument(level = "debug", skip(self))]
    fn get_menu_items(&self) -> ReadSignal<ContextMenuItems<Self>> {
        let i18n = use_i18n();
        let mut ret = RwSignal::new(vec![]);
        if let Some(playlist) = &self.playlist {
            if let Some(library_item) = playlist.library_item {
                if library_item {
                    ret = RwSignal::new(vec![
                        ContextMenuItemInner::<Self>::new_with_handler(
                            i18n.get_keys()
                                .contextMenu()
                                .playlist()
                                .remove()
                                .build_string()
                                .into(),
                            |_, cx| cx.remove_from_library(),
                            None,
                        ),
                        ContextMenuItemInner::new_with_handler(
                            i18n.get_keys()
                                .contextMenu()
                                .playlist()
                                .export()
                                .build_string()
                                .into(),
                            |_, cx| cx.export_playlist(),
                            None,
                        ),
                    ]);
                }
            } else {
                ret = RwSignal::new(vec![ContextMenuItemInner::<Self>::new_with_handler(
                    i18n.get_keys()
                        .contextMenu()
                        .playlist()
                        .save()
                        .build_string()
                        .into(),
                    |_, cx| cx.add_to_library(),
                    None,
                )]);
            }

            let provider_store = expect_context::<Arc<ProviderStore>>();
            let playlist = playlist.clone();
            spawn_local(async move {
                let vaild_providers =
                    provider_store.get_provider_keys(ExtensionProviderScope::PlaylistContextMenu);
                for key in vaild_providers {
                    let playlist = playlist.clone();
                    let ctx_menu = get_playlist_context_menu(key.clone(), playlist).await;
                    match ctx_menu {
                        Ok(ctx_menu) => {
                            let key = key.clone();
                            ret.update(move |menu| {
                                menu.extend(ctx_menu.into_iter().map(move |v| {
                                    let key = key.clone();
                                    ContextMenuItemInner::new_with_handler(
                                        v.name,
                                        move |_, _| {
                                            let key = key.clone();
                                            let action = v.action_id.clone();
                                            spawn_local(async move {
                                                trigger_context_menu_action(key, action)
                                                    .await
                                                    .unwrap()
                                            });
                                        },
                                        None,
                                    )
                                }));
                            });
                        }
                        Err(e) => {
                            tracing::error!("Failed to get context menu from {}: {:?}", key, e)
                        }
                    }
                }
            });

            return ret.read_only();
        }
        RwSignal::new(vec![]).read_only()
    }
}

pub struct SongsContextMenu {
    song_update_request: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
}

impl SongsContextMenu {
    #[tracing::instrument(level = "debug", skip(song_update_request))]
    pub fn new(song_update_request: Option<Box<dyn Fn() + Send + Sync>>) -> Self {
        Self {
            song_update_request: song_update_request.map(Arc::new),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn add_song_from_url(&self) {
        let modal_store: RwSignal<ModalStore> = expect_context();
        modal_store.update(|modal_store| {
            modal_store.set_active_modal(Modals::SongFromUrlModal);
            let cb = self.song_update_request.clone();
            modal_store.on_modal_close(move || {
                if cb.is_some() {
                    let cb = cb.clone().unwrap();
                    cb();
                }
            });
        });
    }
}

impl ContextMenuData<Self> for SongsContextMenu {
    #[tracing::instrument(level = "debug", skip(self))]
    fn get_menu_items(&self) -> ReadSignal<ContextMenuItems<Self>> {
        RwSignal::new(vec![
            ContextMenuItemInner::new("Sort by".into(), Some(get_sort_cx_items())),
            ContextMenuItemInner::<Self>::new_with_handler(
                "Add from Url".into(),
                |_, cx| cx.add_song_from_url(),
                None,
            ),
        ])
        .read_only()
    }
}

pub fn create_context_menu<T>(data: T) -> Arc<Box<dyn Menu<T> + Send + Sync>>
where
    T: ContextMenuData<T> + 'static + Send + Sync,
{
    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();
    if is_mobile {
        Arc::new(Box::new(BottomSheet::new(data)))
    } else {
        Arc::new(Box::new(ContextMenu::new(data)))
    }
}
