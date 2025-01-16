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

// Moosync
// Copyright (C) 2025 Moosync
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
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use leptos::prelude::*;
use leptos_context_menu::ContextMenuItemInner;
use types::songs::Song;

use crate::{
    store::ui_store::{SongSortBy, SongSortByColumns, UiStore},
    utils::invoke::get_lyrics,
};

#[tracing::instrument(level = "trace", skip(song_list, song_indices))]
pub fn get_songs_from_indices<T, Y>(song_list: &T, song_indices: Y) -> Vec<Song>
where
    T: Get<Value = Vec<Song>>,
    Y: Get<Value = Vec<usize>>,
{
    let song_list = song_list.get();
    let song_indices = song_indices.get();
    song_indices
        .iter()
        .map(|index| song_list.get(*index).unwrap().clone())
        .collect()
}

#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_album() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::Album,
            asc: !ui_store.get_song_sort_by().asc,
        })
    });
}
#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_artist() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::Artist,
            asc: !ui_store.get_song_sort_by().asc,
        })
    });
}
#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_date() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::Date,
            asc: !ui_store.get_song_sort_by().asc,
        })
    });
}
#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_genre() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::Genre,
            asc: !ui_store.get_song_sort_by().asc,
        })
    });
}
#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_playcount() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::PlayCount,
            asc: !ui_store.get_song_sort_by().asc,
        })
    });
}
#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_title() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::Title,
            asc: !ui_store.get_song_sort_by().asc,
        })
    });
}

#[tracing::instrument(level = "trace", skip())]
pub fn get_sort_cx_items<T>() -> Vec<ContextMenuItemInner<T>>
where
    T: Send + Sync,
{
    vec![
        ContextMenuItemInner::new_with_handler(
            "Album (Track No.)".into(),
            |_, _| sort_by_album(),
            None,
        ),
        ContextMenuItemInner::new_with_handler("Artist".into(), |_, _| sort_by_artist(), None),
        ContextMenuItemInner::new_with_handler("Date Added".into(), |_, _| sort_by_date(), None),
        ContextMenuItemInner::new_with_handler("Genre".into(), |_, _| sort_by_genre(), None),
        ContextMenuItemInner::new_with_handler(
            "Play count".into(),
            |_, _| sort_by_playcount(),
            None,
        ),
        ContextMenuItemInner::new_with_handler("Title".into(), |_, _| sort_by_title(), None),
    ]
}

pub async fn fetch_lyrics(song: Option<Song>) -> Option<String> {
    tracing::debug!("Fetching lyrics");
    if let Some(song) = song {
        let lyrics = song.song.lyrics.clone();
        if lyrics.is_none() {
            let res = get_lyrics(
                song.song._id.clone().unwrap_or_default(),
                song.song.playback_url.clone().unwrap_or_default(),
                song.artists
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .map(|a| a.artist_name.clone().unwrap_or_default())
                    .collect::<Vec<String>>(),
                song.song.title.clone().unwrap_or_default(),
            )
            .await;
            if let Ok(lyrics) = res {
                return Some(lyrics);
            } else {
                tracing::error!("Failed to fetch lyrics: {:?}", res.unwrap_err());
            }
        }
        return lyrics;
    }

    return None;
}
