use std::{fs, path::PathBuf, str::FromStr};

use crate::database::Database;
use types::{
    entities::{QueryableAlbum, QueryableArtist},
    songs::Song,
    traits::SearchByTerm,
};

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
