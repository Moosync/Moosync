use leptos::{use_context, ReadSignal, RwSignal, SignalGet, SignalUpdate};
use leptos_context_menu::{ContextMenuData, ContextMenuItemInner, ContextMenuItems};
use leptos_router::{use_navigate, NavigateOptions};
use types::{entities::QueryablePlaylist, songs::Song};

use crate::{console_log, store::player_store::PlayerStore, utils::songs::get_songs_from_indices};

use super::{
    db_utils::{add_songs_to_library, add_to_playlist, remove_songs_from_library},
    songs::get_sort_cx_items,
};

#[derive(Clone)]
pub struct SongItemContextMenu {
    pub current_song: Option<Song>,
    pub song_list: ReadSignal<Vec<Song>>,
    pub selected_songs: RwSignal<Vec<usize>>,
    pub playlists: RwSignal<Vec<QueryablePlaylist>>,
}

impl SongItemContextMenu {
    fn current_or_list(&self) -> Vec<Song> {
        let selected_songs = self.selected_songs.get();
        let ret = if selected_songs.is_empty() {
            if let Some(song) = self.current_song.as_ref() {
                vec![song.clone()]
            } else {
                vec![]
            }
        } else {
            get_songs_from_indices(self.song_list, self.selected_songs)
        };

        console_log!("Got songs {:?}", ret);
        ret
    }

    pub fn play_now(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.play_now_multiple(self.current_or_list()));
    }

    pub fn add_to_queue(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.add_to_queue(self.current_or_list()));
    }

    pub fn play_next(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.play_next_multiple(self.current_or_list()));
    }

    pub fn clear_queue_and_play(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| {
            store.clear_queue();
            store.play_now_multiple(self.current_or_list())
        });
    }

    pub fn add_to_library(&self) {
        add_songs_to_library(self.current_or_list());
    }

    pub fn remove_from_library(&self) {
        remove_songs_from_library(self.current_or_list());
    }

    pub fn add_to_playlist(&self, id: String) {
        add_to_playlist(id, self.current_or_list());
    }

    pub fn goto_album(&self) {
        let navigate = use_navigate();
        if let Some(song) = &self.current_song {
            if let Some(album) = &song.album {
                if let Some(id) = album.album_id.clone() {
                    navigate(
                        format!("/main/albums/single?id={}", id).as_str(),
                        NavigateOptions::default(),
                    );
                }
            }
        }
    }

    pub fn goto_artist(&self, id: String) {
        let navigate = use_navigate();
        navigate(
            format!("/main/artists/single?id={}", id).as_str(),
            NavigateOptions::default(),
        );
    }
}

impl ContextMenuData<Self> for SongItemContextMenu {
    fn get_menu_items(&self) -> ContextMenuItems<Self> {
        let mut artist_items = vec![];
        if let Some(song) = &self.current_song {
            if let Some(artists) = &song.artists {
                for artist in artists.clone() {
                    let artist_name = artist.artist_name.clone().unwrap_or_default();
                    let artist_id = artist.artist_id.clone().unwrap_or_default();
                    artist_items.push(ContextMenuItemInner::<Self>::new_with_handler(
                        artist_name,
                        move |_, cx| cx.goto_artist(artist_id.clone()),
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
    fn get_menu_items(&self) -> ContextMenuItems<Self> {
        get_sort_cx_items()
    }
}
