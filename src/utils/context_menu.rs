use std::rc::Rc;

use leptos::{
    create_memo, expect_context, use_context, window, RwSignal, SignalGet, SignalUpdate, SignalWith,
};
use leptos_context_menu::{ContextMenuData, ContextMenuItemInner, ContextMenuItems};
use leptos_router::{use_navigate, use_query_map, NavigateOptions};
use types::{
    entities::{QueryableArtist, QueryablePlaylist},
    songs::Song,
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    modals::new_playlist_modal::PlaylistModalState,
    store::{
        modal_store::{ModalStore, Modals},
        player_store::PlayerStore,
    },
    utils::{entities::get_playlist_sort_cx_items, songs::get_songs_from_indices},
};

use super::{
    db_utils::{
        add_songs_to_library, add_to_playlist, create_playlist, export_playlist, remove_playlist,
        remove_songs_from_library,
    },
    invoke::load_theme,
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
    pub refresh_cb: Rc<Box<dyn Fn()>>,
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
        add_songs_to_library(self.current_or_list(), self.refresh_cb.clone());
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn remove_from_library(&self) {
        remove_songs_from_library(self.current_or_list(), self.refresh_cb.clone());
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
                        url_escape::encode_component(&serde_json::to_string(&album).unwrap())
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
                url_escape::encode_component(&serde_json::to_string(&artist).unwrap())
            )
            .as_str(),
            NavigateOptions::default(),
        );
    }

    pub fn remove_from_playlist(&self) {
        let params = use_query_map();
        let playlist = create_memo(move |_| {
            params.with(|params| {
                let entity = params.get("entity");
                if let Some(entity) = entity {
                    let playlist = serde_json::from_str::<QueryablePlaylist>(entity);
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

        let mut ret: ContextMenuItems<Self> = vec![
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
        ];

        let location = window().location().pathname().unwrap();
        if location.contains("playlists/single") {
            ret.insert(
                5,
                ContextMenuItemInner::new_with_handler(
                    "Remove from playlist".into(),
                    |_, cx| cx.remove_from_playlist(),
                    None,
                ),
            );
        }

        ret
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
    pub refresh_cb: Rc<Box<dyn Fn()>>,
}

impl ThemesContextMenu {
    #[tracing::instrument(level = "trace", skip(self))]
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
    #[tracing::instrument(level = "trace", skip(self))]
    fn get_menu_items(&self) -> ContextMenuItems<Self> {
        vec![
            ContextMenuItemInner::new_with_handler(
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
        ]
    }
}

pub struct PlaylistContextMenu {
    pub refresh_cb: Rc<Box<dyn Fn()>>,
}

impl PlaylistContextMenu {
    #[tracing::instrument(level = "trace", skip(self))]
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
    #[tracing::instrument(level = "trace", skip(self))]
    fn get_menu_items(&self) -> leptos_context_menu::ContextMenuItems<Self> {
        vec![
            ContextMenuItemInner::new_with_handler(
                "Import from Url".into(),
                |_, cx| cx.open_import_from_url_modal(),
                None,
            ),
            ContextMenuItemInner::new("Sort by".into(), Some(get_playlist_sort_cx_items())),
        ]
    }
}

pub struct PlaylistItemContextMenu {
    pub playlist: Option<QueryablePlaylist>,
    pub refresh_cb: Rc<Box<dyn Fn()>>,
}

impl PlaylistItemContextMenu {
    #[tracing::instrument(level = "trace", skip(self))]
    fn add_to_library(&self) {
        if let Some(playlist) = &self.playlist {
            create_playlist(playlist.clone(), None);
            self.refresh_cb.as_ref()();
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn remove_from_library(&self) {
        if let Some(playlist) = &self.playlist {
            remove_playlist(playlist.clone(), self.refresh_cb.clone());
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn export_playlist(&self) {
        if let Some(playlist) = &self.playlist {
            export_playlist(playlist.clone());
        }
    }
}

impl ContextMenuData<Self> for PlaylistItemContextMenu {
    #[tracing::instrument(level = "trace", skip(self))]
    fn get_menu_items(&self) -> leptos_context_menu::ContextMenuItems<Self> {
        if let Some(playlist) = &self.playlist {
            if let Some(library_item) = playlist.library_item {
                if library_item {
                    return vec![
                        ContextMenuItemInner::new_with_handler(
                            "Remove from library".into(),
                            |_, cx| cx.remove_from_library(),
                            None,
                        ),
                        ContextMenuItemInner::new_with_handler(
                            "Export playlist".into(),
                            |_, cx| cx.export_playlist(),
                            None,
                        ),
                    ];
                }
            }

            return vec![ContextMenuItemInner::new_with_handler(
                "Add to library".into(),
                |_, cx| cx.add_to_library(),
                None,
            )];
        }
        vec![]
    }
}
