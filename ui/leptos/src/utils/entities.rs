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

use itertools::Itertools;
use leptos::prelude::*;
use leptos_context_menu::ContextMenuItemInner;
use songs_proto::moosync::types::Artist;

use crate::store::ui_store::{PlaylistSortBy, PlaylistSortByColumns, UiStore};

#[tracing::instrument(level = "debug", skip())]
fn sort_by_name() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_playlist_sort_by(PlaylistSortBy {
            sort_by: PlaylistSortByColumns::Title,
            asc: true,
        })
    });
}

#[tracing::instrument(level = "debug", skip())]
fn sort_by_provider() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_playlist_sort_by(PlaylistSortBy {
            sort_by: PlaylistSortByColumns::Provider,
            asc: true,
        })
    });
}

#[tracing::instrument(level = "debug", skip())]
pub fn get_playlist_sort_cx_items<T>() -> Vec<ContextMenuItemInner<T>>
where
    T: Send + Sync,
{
    vec![
        ContextMenuItemInner::new_with_handler("Title".into(), |_, _| sort_by_name(), None),
        ContextMenuItemInner::new_with_handler("Provider".into(), |_, _| sort_by_provider(), None),
    ]
}

#[tracing::instrument(level = "debug", skip(artists))]
pub fn get_artist_string(artists: Vec<Artist>) -> String {
    artists
        .iter()
        .filter_map(|a| a.artist_name.clone())
        .join(", ")
}
