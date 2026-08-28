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

use extensions_proto::moosync::types::{ExtensionDetail, FetchedExtensionManifest};
use songs_proto::moosync::types::{Album, Artist, Genre, Playlist, Song};

use crate::{
    AlbumModel, PlaylistModel,
    utils::{
        default_empty_icon, default_entity_cover, default_folder_icon, default_song_cover,
        load_icon, to_artist_model, to_extension_item, to_fetched_extension_item, to_genre_model,
        to_song_model,
    },
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_default_song_cover_smoke() { let _img = default_song_cover(); }

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_default_entity_cover_smoke() { let _img = default_entity_cover(); }

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_default_empty_icon_smoke() { let _img = default_empty_icon(); }

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_default_folder_icon_smoke() { let _img = default_folder_icon(); }

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_load_icon_smoke() { let _img = load_icon(""); }

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_song_model_smoke() {
    let song = Song::default();
    let _model = to_song_model(&song, None);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_album_model_smoke() {
    let album = Album::default();
    let model: AlbumModel = album.into();
    let _album: Album = model.into();
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_artist_model_smoke() {
    let artist = Artist::default();
    let _model = to_artist_model(&artist, None);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_playlist_model_smoke() {
    let playlist = Playlist::default();
    let model: PlaylistModel = playlist.into();
    let _playlist: Playlist = model.into();
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_genre_model_smoke() {
    let genre = Genre::default();
    let _model = to_genre_model(&genre);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_extension_item_smoke() {
    let detail = ExtensionDetail::default();
    let _item = to_extension_item(&detail);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_fetched_extension_item_smoke() {
    let manifest = FetchedExtensionManifest::default();
    let _item = to_fetched_extension_item(&manifest);
}
