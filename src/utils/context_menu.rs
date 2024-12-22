use leptos::{use_context, RwSignal, SignalGet, SignalUpdate};
use leptos_context_menu::{ContextMenuData, ContextMenuItemInner, ContextMenuItems};
use leptos_router::{use_navigate, NavigateOptions};
use serde::Serialize;
use types::{
    entities::{QueryableArtist, QueryablePlaylist},
    songs::Song,
};
use wasm_bindgen_futures::spawn_local;

use crate::{store::player_store::PlayerStore, utils::songs::get_songs_from_indices};

use super::{
    common::invoke,
    db_utils::{add_songs_to_library, add_to_playlist, remove_songs_from_library},
    songs::get_sort_cx_items,
};

#[derive(Clone)]
pub struct SongItemContextMenu<T>
where
    T: SignalGet<Value = Vec<Song>>,
{
    pub current_song: Option<Song>,
    pub song_list: T,
    pub selected_songs: RwSignal<Vec<usize>>,
    pub playlists: RwSignal<Vec<QueryablePlaylist>>,
}

impl<T> SongItemContextMenu<T>
where
    T: SignalGet<Value = Vec<Song>>,
{
    #[tracing::instrument(level = "trace", skip(self))]
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

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn play_now(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.play_now_multiple(self.current_or_list()));
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn add_to_queue(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.add_to_queue(self.current_or_list()));
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn play_next(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.play_next_multiple(self.current_or_list()));
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn clear_queue_and_play(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| {
            store.clear_queue();
            store.play_now_multiple(self.current_or_list())
        });
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn add_to_library(&self) {
        add_songs_to_library(self.current_or_list());
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn remove_from_library(&self) {
        remove_songs_from_library(self.current_or_list());
    }

    #[tracing::instrument(level = "trace", skip(self, id))]
    pub fn add_to_playlist(&self, id: String) {
        add_to_playlist(id, self.current_or_list());
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn goto_album(&self) {
        let navigate = use_navigate();
        if let Some(song) = &self.current_song {
            if let Some(album) = &song.album {
                navigate(
                    format!(
                        "/main/albums/single?entity={}",
                        serde_json::to_string(&album).unwrap()
                    )
                    .as_str(),
                    NavigateOptions::default(),
                );
            }
        }
    }

    #[tracing::instrument(level = "trace", skip(self, artist))]
    pub fn goto_artist(&self, artist: QueryableArtist) {
        let navigate = use_navigate();
        navigate(
            format!(
                "/main/artists/single?entity={}",
                serde_json::to_string(&artist).unwrap()
            )
            .as_str(),
            NavigateOptions::default(),
        );
    }
}

impl<T> ContextMenuData<Self> for SongItemContextMenu<T>
where
    T: SignalGet<Value = Vec<Song>>,
{
    #[tracing::instrument(level = "trace", skip(self))]
    fn get_menu_items(&self) -> ContextMenuItems<Self> {
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
                "Remove from library".into(),
                |_, cx| cx.remove_from_library(),
                None,
            )
        } else {
            ContextMenuItemInner::<Self>::new_with_handler(
                "Add to library".into(),
                |_, cx| cx.add_to_library(),
                None,
            )
        };

        vec![
            ContextMenuItemInner::new_with_handler("Play now".into(), |_, cx| cx.play_now(), None),
            ContextMenuItemInner::new_with_handler(
                "Play next".into(),
                |_, cx| cx.play_next(),
                None,
            ),
            ContextMenuItemInner::new_with_handler(
                "Clear queue and play".into(),
                |_, cx| cx.clear_queue_and_play(),
                None,
            ),
            ContextMenuItemInner::new_with_handler(
                "Add to queue".into(),
                |_, cx| cx.add_to_queue(),
                None,
            ),
            ContextMenuItemInner::new("Add to playlist".into(), Some(playlist_items)),
            library_menu_item,
            ContextMenuItemInner::new_with_handler(
                "Goto album".into(),
                |_, cx| cx.goto_album(),
                None,
            ),
            ContextMenuItemInner::new("Goto artists".into(), Some(artist_items)),
        ]
    }
}

pub struct SortContextMenu {}

impl ContextMenuData<Self> for SortContextMenu {
    #[tracing::instrument(level = "trace", skip(self))]
    fn get_menu_items(&self) -> ContextMenuItems<Self> {
        get_sort_cx_items()
    }
}

pub struct ThemesContextMenu {
    pub id: Option<String>,
}

impl ThemesContextMenu {
    #[tracing::instrument(level = "trace", skip(self))]
    fn export_theme(&self) {
        let id = self.id.clone();
        if let Some(id) = id {
            #[derive(Serialize)]
            struct ExportThemeArgs {
                id: String,
            }
            spawn_local(async move {
                let res = invoke(
                    "export_theme",
                    serde_wasm_bindgen::to_value(&ExportThemeArgs { id }).unwrap(),
                )
                .await;
                if let Err(err) = res {
                    tracing::error!("Error exporting theme {:?}", err);
                }
            });
        }
    }
}

impl ContextMenuData<Self> for ThemesContextMenu {
    #[tracing::instrument(level = "trace", skip(self))]
    fn get_menu_items(&self) -> ContextMenuItems<Self> {
        vec![ContextMenuItemInner::new_with_handler(
            "Export theme".into(),
            |_, cx| cx.export_theme(),
            None,
        )]
    }
}
