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

use std::{fs, path::PathBuf, str::FromStr};

use crate::database::Database;
use types::{
    common::SearchByTerm,
    entities::{QueryableAlbum, QueryableArtist},
    songs::Song,
};

#[tracing::instrument(level = "trace", skip())]
fn cleanup() {
    fs::remove_file(PathBuf::from_str("test.db").unwrap()).unwrap();
    fs::remove_file(PathBuf::from_str("test.db-shm").unwrap()).unwrap();
    fs::remove_file(PathBuf::from_str("test.db-wal").unwrap()).unwrap();
}

#[test]
fn test_insert() {
    let db = Database::new(PathBuf::from_str("test.db").unwrap());

    let res = db
        .insert_songs(vec![Song {
            song: Default::default(),
            album: Some(QueryableAlbum::search_by_term(Some(
                "Test album".to_string(),
            ))),
            artists: Some(vec![QueryableArtist::search_by_term(Some(
                "Test artist".to_string(),
            ))]),
            genre: Some(vec![]),
        }])
        .unwrap();

    cleanup();

    assert_eq!(res.len(), 1);
    assert_eq!(res[0].genre.clone().unwrap().len(), 0);
    assert!(res[0].song._id.is_some());
    assert!(res[0].album.clone().unwrap().album_id.is_some());
    assert!(res[0].artists.clone().unwrap()[0].artist_id.is_some());
}
