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

use std::{env::temp_dir, fs, path::PathBuf};

use songs_proto::moosync::types::{
    Album, Artist, Genre, GetEntityOptions, GetSongOptions, InnerSong, Playlist, SearchableSong,
    Song, SongType, entity_result::Result as EntityResultVariant,
};
use uuid::Uuid;

use crate::{cache::CacheHolder, database::Database};

// Helper function to create a unique test DB path
#[tracing::instrument(level = "debug", skip_all)]
fn get_test_db_path() -> PathBuf {
    let file_name = format!("moosync_test_{}.db", Uuid::new_v4());
    temp_dir().join(file_name)
}

// Helper function to clean up DB files
#[tracing::instrument(level = "debug", skip_all)]
fn cleanup(db_path: &PathBuf) {
    let base_path = db_path.to_string_lossy().to_string();

    // Ignore errors as files might not exist
    let _ = fs::remove_file(db_path);
    let _ = fs::remove_file(format!("{}-shm", base_path));
    let _ = fs::remove_file(format!("{}-wal", base_path));
}

// Test utility function to create a test song
#[tracing::instrument(level = "debug", skip_all)]
fn create_test_song(title: &str, path: &str) -> Song {
    Song {
        song: Some(InnerSong {
            id: None,
            title: Some(title.to_string()),
            path: Some(path.to_string()),
            song_cover_path_high: None,
            song_cover_path_low: None,
            date_added: None,
            size: Some(0.0),
            bitrate: Some(0.0),
            codec: None,
            duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                seconds: 300,
                nanos: 0,
            }),
            sample_rate: Some(0.0),
            lyrics: None,
            r#type: SongType::Local.into(),
            ..Default::default()
        }),
        album: Some(Album {
            album_id: None,
            album_name: Some("Test Album".to_string()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_id: None,
            artist_name: Some("Test Artist".to_string()),
            ..Default::default()
        }],
        genre: vec![Genre {
            genre_id: None,
            genre_name: Some("Test Genre".to_string()),
            ..Default::default()
        }],
    }
}

// Test song insertion
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_insert_song() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    let test_song = create_test_song("Test Song", "/path/to/test.mp3");
    let result = db.insert_songs(vec![test_song]).unwrap();

    assert_eq!(result.len(), 1);
    assert!(result[0].song.clone().unwrap().id.is_some());
    assert_eq!(
        result[0].song.clone().unwrap().title.as_ref().unwrap(),
        "Test Song"
    );

    // Test album was created
    let album = result[0].album.clone().unwrap();
    assert!(album.album_id.is_some());
    assert_eq!(album.album_name, Some("Test Album".to_string()));

    // Test artist was created
    let artists = result[0].artists.clone();
    assert_eq!(artists.len(), 1);
    assert!(artists[0].artist_id.is_some());
    assert_eq!(artists[0].artist_name, Some("Test Artist".to_string()));

    // Test genre was created
    let genres = result[0].genre.clone();
    assert_eq!(genres.len(), 1);
    assert!(genres[0].genre_id.is_some());
    assert_eq!(genres[0].genre_name, Some("Test Genre".to_string()));

    cleanup(&db_path);
}

// Test fetching songs by options
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_songs_by_options() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Insert test songs
    db.insert_songs(vec![
        create_test_song("Song 1", "/path/to/song1.mp3"),
        create_test_song("Song 2", "/path/to/song2.mp3"),
        create_test_song("Different", "/path/to/different.mp3"),
    ])
    .unwrap();

    // Test fetching by partial title match
    let options = GetSongOptions {
        song: Some(SearchableSong {
            title: Some("%Song%".to_string()),
            ..Default::default()
        }),
        inclusive: Some(true),
        ..Default::default()
    };

    let songs = db.get_songs_by_options(options).unwrap();
    assert_eq!(songs.len(), 2);
    assert!(
        songs
            .iter()
            .any(|s| s.song.clone().unwrap().title.as_ref().unwrap() == "Song 1")
    );
    assert!(
        songs
            .iter()
            .any(|s| s.song.clone().unwrap().title.as_ref().unwrap() == "Song 2")
    );

    // Test fetching by exact path
    let options = GetSongOptions {
        song: Some(SearchableSong {
            path: Some("/path/to/different.mp3".to_string()),
            ..Default::default()
        }),
        inclusive: Some(false),
        ..Default::default()
    };

    let songs = db.get_songs_by_options(options).unwrap();
    assert_eq!(songs.len(), 1);
    assert_eq!(
        songs[0].song.clone().unwrap().title.as_ref().unwrap(),
        "Different"
    );

    cleanup(&db_path);
}

// Test updating a song
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_update_song() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Insert a test song
    let songs = db
        .insert_songs(vec![create_test_song(
            "Original Title",
            "/path/to/original.mp3",
        )])
        .unwrap();
    let song_id = songs[0].song.clone().unwrap().id.clone().unwrap();

    // Update the song
    let updatable_song = InnerSong {
        id: Some(song_id.clone()),
        title: Some("Updated Title".to_string()),
        ..Default::default()
    };

    db.update_song(&updatable_song).unwrap();

    // Fetch the updated song
    let options = GetSongOptions {
        song: Some(SearchableSong {
            id: Some(song_id),
            ..Default::default()
        }),
        inclusive: Some(false),
        ..Default::default()
    };

    let updated_songs = db.get_songs_by_options(options).unwrap();
    assert_eq!(updated_songs.len(), 1);
    assert_eq!(
        updated_songs[0]
            .song
            .clone()
            .unwrap()
            .title
            .as_ref()
            .unwrap(),
        "Updated Title"
    );

    cleanup(&db_path);
}

// Test removing songs
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_remove_songs() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Insert test songs
    let songs = db
        .insert_songs(vec![
            create_test_song("Song to Keep", "/path/to/keep.mp3"),
            create_test_song("Song to Remove", "/path/to/remove.mp3"),
        ])
        .unwrap();

    // Get IDs
    let keep_id = songs[0].song.clone().unwrap().id.clone().unwrap();
    let remove_id = songs[1].song.clone().unwrap().id.clone().unwrap();

    // Make sure we have 2 songs before removing
    let initial_songs = db
        .get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong {
                r#type: Some(SongType::Local.into()), /* Filter by song type to ensure we get all
                                                       * test songs */
                ..Default::default()
            }),
            inclusive: Some(true),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(initial_songs.len(), 2);

    // Add analytics data to both songs
    db.increment_play_count(&keep_id).unwrap();
    db.increment_play_count(&remove_id).unwrap();
    db.increment_play_time(&keep_id, 60.0).unwrap();
    db.increment_play_time(&remove_id, 120.0).unwrap();

    // Remove one song
    db.remove_songs(&vec![&remove_id]).unwrap();

    // Verify only one song remains
    let all_songs = db
        .get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong {
                r#type: Some(SongType::Local.into()), /* Filter by song type to ensure we get all
                                                       * remaining songs */
                ..Default::default()
            }),
            inclusive: Some(true),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(all_songs.len(), 1);
    assert_eq!(
        all_songs[0].song.clone().unwrap().id.as_ref().unwrap(),
        &keep_id
    );

    // Verify analytics data for the removed song is also gone
    let analytics = db.get_top_listened_songs().unwrap();
    let removed_song_analytics = analytics
        .songs
        .iter()
        .find(|(song)| &song.song_id == &remove_id);
    assert!(
        removed_song_analytics.is_none(),
        "Analytics for removed song should be deleted"
    );

    // Verify analytics data for the kept song is still there
    let kept_song_analytics = analytics
        .songs
        .iter()
        .find(|(song)| song.song_id == keep_id);
    assert!(
        kept_song_analytics.is_some(),
        "Analytics for kept song should still exist"
    );

    // Try to fetch the removed song specifically
    let removed_options = GetSongOptions {
        song: Some(SearchableSong {
            id: Some(remove_id),
            ..Default::default()
        }),
        inclusive: Some(false),
        ..Default::default()
    };

    let removed_songs = db.get_songs_by_options(removed_options).unwrap();
    assert_eq!(removed_songs.len(), 0);

    cleanup(&db_path);
}

// Test playlist CRUD operations
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_operations() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Create a playlist
    let playlist = Playlist {
        playlist_id: None,
        playlist_name: "Test Playlist".to_string(),
        playlist_coverpath: None,
        playlist_path: None,
        playlist_desc: None,
        extension: None,
        icon: None,
        library_item: None,
        ..Default::default()
    };

    let playlist_id = db.create_playlist(playlist).unwrap();

    // Insert songs
    let songs = db
        .insert_songs(vec![
            create_test_song("Playlist Song 1", "/path/to/playlist1.mp3"),
            create_test_song("Playlist Song 2", "/path/to/playlist2.mp3"),
        ])
        .unwrap();

    // Add songs to playlist
    db.add_to_playlist(&playlist_id, &songs).unwrap();

    // Get playlist songs
    let playlist_options = Playlist {
        playlist_id: Some(playlist_id.clone()),
        ..Default::default()
    };

    let result = db
        .get_entity_by_options(GetEntityOptions {
            playlist: Some(playlist_options.clone()),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();

    // The result is returned as a Playlists variant
    let Some(EntityResultVariant::Playlists(playlists_list)) = result.result else {
        panic!("Expected Playlists variant");
    };
    assert_eq!(playlists_list.playlists.len(), 1);
    let playlist = &playlists_list.playlists[0];

    // Verify we can access the playlist's properties
    assert!(playlist.playlist_name.contains("Test Playlist"));

    // Remove one song from playlist
    let song_id_to_remove = songs[0].song.clone().unwrap().id.clone().unwrap();
    db.remove_from_playlist(&playlist_id, &vec![song_id_to_remove])
        .unwrap();

    // Delete the playlist
    db.remove_playlist(&playlist_id).unwrap();

    // Verify playlist is gone
    let all_playlists = db
        .get_entity_by_options(GetEntityOptions {
            playlist: Some(Playlist::default()),
            inclusive: Some(true),
            ..Default::default()
        })
        .unwrap();

    let Some(EntityResultVariant::Playlists(playlists_list)) = all_playlists.result else {
        panic!("Expected Playlists variant");
    };
    assert_eq!(playlists_list.playlists.len(), 0);

    cleanup(&db_path);
}

// Test album operations
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_operations() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Insert songs with the same album
    db.insert_songs(vec![
        create_test_song("Album Song 1", "/path/to/album1.mp3"),
        create_test_song("Album Song 2", "/path/to/album2.mp3"),
    ])
    .unwrap();

    // Get the album
    let album_options = Album {
        album_name: Some("Test Album".to_string()),
        ..Default::default()
    };

    let result = db
        .get_entity_by_options(GetEntityOptions {
            album: Some(album_options.clone()),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();

    // The result is returned as an Albums variant
    let Some(EntityResultVariant::Albums(albums_list)) = result.result else {
        panic!("Expected Albums variant");
    };
    assert_eq!(albums_list.albums.len(), 1);
    let album = &albums_list.albums[0];

    // Verify we can access the album's properties
    assert!(album.album_name.as_deref().unwrap().contains("Test Album"));

    // Test updating album
    let mut album_to_update = Album {
        album_name: Some("Test Album".to_string()),
        year: Some("2023".to_string()),
        ..Default::default()
    };

    // First get the album ID
    let albums = db
        .get_entity_by_options(GetEntityOptions {
            album: Some(album_options),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();

    let Some(EntityResultVariant::Albums(albums_list)) = albums.result else {
        panic!("Expected Albums variant");
    };
    let album_id = albums_list.albums[0].album_id.clone().unwrap();

    album_to_update.album_id = Some(album_id.clone());
    db.update_album(album_to_update).unwrap();

    // Verify update
    let updated_album = db
        .get_entity_by_options(GetEntityOptions {
            album: Some(Album {
                album_id: Some(album_id),
                ..Default::default()
            }),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();

    let Some(EntityResultVariant::Albums(updated_albums_list)) = updated_album.result else {
        panic!("Expected Albums variant");
    };
    let year = updated_albums_list.albums[0].year.as_deref().unwrap();

    assert_eq!(year, "2023");

    cleanup(&db_path);
}

// Test artist operations
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_artist_operations() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Insert songs with the same artist
    db.insert_songs(vec![
        create_test_song("Artist Song 1", "/path/to/artist1.mp3"),
        create_test_song("Artist Song 2", "/path/to/artist2.mp3"),
    ])
    .unwrap();

    // Get the artist
    let artist_options = Artist {
        artist_name: Some("Test Artist".to_string()),
        ..Default::default()
    };

    let result = db
        .get_entity_by_options(GetEntityOptions {
            artist: Some(artist_options.clone()),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();

    // The result is returned as an Artists variant
    let Some(EntityResultVariant::Artists(artists_list)) = result.result else {
        panic!("Expected Artists variant");
    };
    assert!(
        !artists_list.artists.is_empty(),
        "Should return at least one artist"
    );
    let artist = &artists_list.artists[0];

    // Verify we can access the artist's properties
    assert!(
        artist
            .artist_name
            .as_deref()
            .unwrap()
            .contains("Test Artist")
    );

    // Test updating artist
    let mut artist_to_update = Artist {
        artist_name: Some("Test Artist".to_string()),
        artist_coverpath: Some("https://example.com/cover.jpg".to_string()),
        ..Default::default()
    };

    // First get the artist ID
    let artists = db
        .get_entity_by_options(GetEntityOptions {
            artist: Some(artist_options),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();

    let Some(EntityResultVariant::Artists(artists_list)) = artists.result else {
        panic!("Expected Artists variant");
    };
    assert!(
        !artists_list.artists.is_empty(),
        "Artists array should not be empty"
    );

    let artist_id = artists_list.artists[0].artist_id.clone().unwrap();
    assert!(!artist_id.is_empty(), "artistId should not be empty");

    artist_to_update.artist_id = Some(artist_id.clone());
    db.update_artist(artist_to_update).unwrap();

    // Verify update
    let updated_artist = db
        .get_entity_by_options(GetEntityOptions {
            artist: Some(Artist {
                artist_id: Some(artist_id.clone()),
                ..Default::default()
            }),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();

    let Some(EntityResultVariant::Artists(updated_artists_list)) = updated_artist.result else {
        panic!("Expected Artists variant");
    };
    assert!(
        !updated_artists_list.artists.is_empty(),
        "Updated artists array should not be empty"
    );

    // Check if the update was successful by verifying the artist_id matches
    let retrieved_id = updated_artists_list.artists[0]
        .artist_id
        .as_deref()
        .expect("artistId should exist");

    assert_eq!(
        retrieved_id, artist_id,
        "Retrieved artist ID should match the one we set"
    );

    cleanup(&db_path);
}

// Test searching
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Insert variety of entities
    db.insert_songs(vec![
        create_test_song("Searchable Song", "/path/to/searchable.mp3"),
        create_test_song("Another Track", "/path/to/track.mp3"),
    ])
    .unwrap();

    // Create a searchable playlist
    let playlist = Playlist {
        playlist_id: None,
        playlist_name: "Searchable Playlist".to_string(),
        playlist_coverpath: None,
        playlist_path: None,
        ..Default::default()
    };

    db.create_playlist(playlist).unwrap();

    // Search for "Search"
    let search_results = db.search_all("Search").unwrap();

    assert!(!search_results.songs.is_empty());
    assert!(!search_results.playlists.is_empty());

    assert!(search_results.songs.iter().any(|s| {
        s.song
            .clone()
            .unwrap()
            .title
            .as_ref()
            .unwrap()
            .contains("Searchable")
    }));

    assert!(
        search_results
            .playlists
            .iter()
            .any(|p| p.playlist_name.contains("Searchable"))
    );

    cleanup(&db_path);
}

// Test analytics operations
#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_analytics() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Insert a song
    let songs = db
        .insert_songs(vec![create_test_song(
            "Analytics Test",
            "/path/to/analytics.mp3",
        )])
        .unwrap();

    let song_id = songs[0].song.clone().unwrap().id.unwrap();

    // Increment play count multiple times
    for _ in 0..5 {
        db.increment_play_count(&song_id).unwrap();
    }

    // Add some play time
    db.increment_play_time(&song_id, 120.0).unwrap();
    db.increment_play_time(&song_id, 180.0).unwrap();

    // Get top listened songs
    let analytics = db.get_top_listened_songs().unwrap();

    // Verify analytics contains songs
    assert!(
        !analytics.songs.is_empty(),
        "Analytics should contain songs"
    );

    // Find our song in the analytics data
    let song_analytics = analytics
        .songs
        .iter()
        .find(|(song)| song.song_id.as_str() == song_id.as_str());

    // Verify our song was found and has the expected play time
    assert!(song_analytics.is_some(), "Song should be in analytics data");
    if let Some(listen_time) = song_analytics {
        assert!(listen_time.time > 0.0, "Play time should be recorded");
    }

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_cache_holder() {
    let db_path = get_test_db_path();
    let cache_holder = CacheHolder::new(db_path.clone());

    // Success Path
    let key = "test_url";
    let data = serde_json::json!({ "foo": "bar" });
    cache_holder.set(key, &data, 5).unwrap();

    let retrieved: serde_json::Value = cache_holder.get(key).unwrap();
    assert_eq!(retrieved, data);

    // Conflict (Upsert)
    let new_data = serde_json::json!({ "foo": "baz" });
    cache_holder.set(key, &new_data, 5).unwrap();

    let retrieved_new: serde_json::Value = cache_holder.get(key).unwrap();
    assert_eq!(retrieved_new, new_data);

    // Expiration
    let exp_key = "expired_url";
    cache_holder.set(exp_key, &data, 0).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    let res: Result<serde_json::Value, _> = cache_holder.get(exp_key);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Cache expired"));

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_db_creation_and_playlist() {
    let base_dir = temp_dir().join(format!("moosync_test_dir_{}", Uuid::new_v4()));
    assert!(!base_dir.exists());

    // Database::new() creates directory
    let db = Database::new(base_dir.clone());
    assert!(base_dir.exists());

    // Create playlist without ID
    let pl_no_id = Playlist {
        playlist_name: "No ID Playlist".to_string(),
        ..Default::default()
    };
    let id_1 = db.create_playlist(pl_no_id).unwrap();
    assert!(!id_1.is_empty());

    // Create playlist without Name (should default to "New playlist")
    let pl_no_name = Playlist {
        playlist_id: Some(Uuid::new_v4().to_string()),
        playlist_name: "".to_string(),
        ..Default::default()
    };
    let id_2 = db.create_playlist(pl_no_name).unwrap();

    // Fetch and verify name
    let entity_res = db
        .get_entity_by_options(GetEntityOptions {
            playlist: Some(Playlist {
                playlist_id: Some(id_2),
                ..Default::default()
            }),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();
    let Some(EntityResultVariant::Playlists(playlists_list)) = entity_res.result else {
        panic!("Expected Playlists variant");
    };
    assert_eq!(playlists_list.playlists.len(), 1);
    assert_eq!(&playlists_list.playlists[0].playlist_name, "New playlist");

    // With Path (Duplicate check)
    let unique_path = "/path/to/unique_playlist".to_string();
    let pl_path_1 = Playlist {
        playlist_name: "Playlist Path 1".to_string(),
        playlist_path: Some(unique_path.clone()),
        ..Default::default()
    };
    let pl_path_id_1 = db.create_playlist(pl_path_1).unwrap();

    let pl_path_2 = Playlist {
        playlist_name: "Playlist Path 2 (Duplicate Path)".to_string(),
        playlist_path: Some(unique_path),
        ..Default::default()
    };
    let pl_path_id_2 = db.create_playlist(pl_path_2).unwrap();
    assert_eq!(pl_path_id_1, pl_path_id_2);

    cleanup(&base_dir.join("songs.db"));
    let _ = fs::remove_dir_all(base_dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_is_song_in_playlist() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    let playlist_id = db
        .create_playlist(Playlist {
            playlist_name: "PL".to_string(),
            ..Default::default()
        })
        .unwrap();

    let song = create_test_song("Song", "/path/song.mp3");
    let inserted = db.insert_songs(vec![song]).unwrap();
    let song_id = inserted[0].song.clone().unwrap().id.unwrap();

    // False when not in playlist
    assert!(!db.is_song_in_playlist(&playlist_id, &song_id).unwrap());

    // True when in playlist
    db.add_to_playlist(&playlist_id, &inserted).unwrap();
    assert!(db.is_song_in_playlist(&playlist_id, &song_id).unwrap());

    // False for non-existent IDs
    assert!(!db.is_song_in_playlist("non_existent_pl", &song_id).unwrap());
    assert!(
        !db.is_song_in_playlist(&playlist_id, "non_existent_song")
            .unwrap()
    );

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_insert_songs_edge_cases() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Song is None (should be skipped)
    let song_none = Song {
        song: None,
        ..Default::default()
    };
    let res = db.insert_songs(vec![song_none]).unwrap();
    assert_eq!(res.len(), 1);

    // Create an entity first
    let song_1 = Song {
        song: Some(InnerSong {
            title: Some("Song 1".to_string()),
            path: Some("/path/1".to_string()),
            ..Default::default()
        }),
        album: Some(Album {
            album_name: Some("Common Album".to_string()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_name: Some("Common Artist".to_string()),
            ..Default::default()
        }],
        genre: vec![Genre {
            genre_name: Some("Common Genre".to_string()),
            ..Default::default()
        }],
    };

    let inserted_1 = db.insert_songs(vec![song_1]).unwrap();
    let album_id = inserted_1[0]
        .album
        .as_ref()
        .unwrap()
        .album_id
        .clone()
        .unwrap();
    let artist_id = inserted_1[0].artists[0].artist_id.clone().unwrap();
    let genre_id = inserted_1[0].genre[0].genre_id.clone().unwrap();

    // Insert another song sharing the same album, artist, genre names
    let song_2 = Song {
        song: Some(InnerSong {
            title: Some("Song 2".to_string()),
            path: Some("/path/2".to_string()),
            ..Default::default()
        }),
        album: Some(Album {
            album_name: Some("Common Album".to_string()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_name: Some("Common Artist".to_string()),
            ..Default::default()
        }],
        genre: vec![Genre {
            genre_name: Some("Common Genre".to_string()),
            ..Default::default()
        }],
    };

    let inserted_2 = db.insert_songs(vec![song_2]).unwrap();
    let album_id_2 = inserted_2[0]
        .album
        .as_ref()
        .unwrap()
        .album_id
        .clone()
        .unwrap();
    let artist_id_2 = inserted_2[0].artists[0].artist_id.clone().unwrap();
    let genre_id_2 = inserted_2[0].genre[0].genre_id.clone().unwrap();

    assert_eq!(album_id, album_id_2);
    assert_eq!(artist_id, artist_id_2);
    assert_eq!(genre_id, genre_id_2);

    // Test on_conflict behavior for same path
    let song_update = Song {
        song: Some(InnerSong {
            title: Some("Updated Song 2".to_string()),
            path: Some("/path/2".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let _ = db.insert_songs(vec![song_update]).unwrap();

    let fetched = db
        .get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong {
                path: Some("/path/2".to_string()),
                ..Default::default()
            }),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();

    assert_eq!(fetched.len(), 1);
    assert_eq!(
        fetched[0].song.as_ref().unwrap().title.as_ref().unwrap(),
        "Updated Song 2"
    );

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_files_not_in_db() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Empty input
    let empty_res = db.files_not_in_db(vec![]).unwrap();
    assert!(empty_res.is_empty());

    // Insert some files
    db.insert_songs(vec![
        create_test_song("Song 1", "/path/to/song1.mp3"),
        create_test_song("Song 2", "/path/to/song2.mp3"),
    ])
    .unwrap();

    // Matching path and size
    let files = vec![(PathBuf::from("/path/to/song1.mp3"), 0.0)];
    let res = db.files_not_in_db(files).unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].0.to_str().unwrap(), "/path/to/song1.mp3");

    // Path matches but size differs -> should NOT return it
    let files_mismatch = vec![(PathBuf::from("/path/to/song1.mp3"), 100.0)];
    let res_mismatch = db.files_not_in_db(files_mismatch).unwrap();
    assert!(res_mismatch.is_empty());

    // Chunking logic check (> 998 files)
    let mut big_list = Vec::new();
    for i in 0..1005 {
        big_list.push((PathBuf::from(format!("/path/to/song_{}.mp3", i)), 0.0));
    }
    let res_big = db.files_not_in_db(big_list.clone()).unwrap();
    assert!(res_big.is_empty());

    // Insert one of the big list files and check
    db.insert_songs(vec![create_test_song("Song 999", "/path/to/song_999.mp3")])
        .unwrap();
    let res_big_with_match = db.files_not_in_db(big_list).unwrap();
    assert_eq!(res_big_with_match.len(), 1);
    assert_eq!(
        res_big_with_match[0].0.to_str().unwrap(),
        "/path/to/song_999.mp3"
    );

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_export_playlist() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Non-existent playlist
    let err_res = db.export_playlist("non_existent");
    assert!(err_res.is_err());
    assert!(
        err_res
            .unwrap_err()
            .to_string()
            .contains("Playlist not found")
    );

    // Create a playlist
    let playlist = Playlist {
        playlist_name: "Export PL".to_string(),
        ..Default::default()
    };
    let playlist_id = db.create_playlist(playlist).unwrap();

    // Local song
    let local_song = Song {
        song: Some(InnerSong {
            title: Some("Local Song".to_string()),
            path: Some("/local/track.mp3".to_string()),
            duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                seconds: 180,
                nanos: 0,
            }),
            song_cover_path_high: Some("/cover/high.jpg".to_string()),
            r#type: SongType::Local.into(),
            ..Default::default()
        }),
        album: Some(Album {
            album_name: Some("Local Album".to_string()),
            ..Default::default()
        }),
        genre: vec![Genre {
            genre_name: Some("Pop".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    // Remote song
    let remote_song = Song {
        song: Some(InnerSong {
            title: Some("Remote Song".to_string()),
            url: Some("https://example.com/stream.mp3".to_string()),
            duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                seconds: 240,
                nanos: 0,
            }),
            r#type: SongType::Url.into(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let inserted = db.insert_songs(vec![local_song, remote_song]).unwrap();
    db.add_to_playlist(&playlist_id, &inserted).unwrap();

    let export = db.export_playlist(&playlist_id).unwrap();

    // Verify format
    assert!(export.starts_with("#EXTM3U"));
    assert!(export.contains("#PLAYLIST:Export PL"));

    // Check local song entry
    assert!(export.contains("#EXTINF:180,Local Song"));
    assert!(export.contains("#EXTALB:Local Album"));
    assert!(export.contains("#EXTGENRE:Pop"));
    assert!(export.contains("#EXTIMG:/cover/high.jpg"));
    assert!(export.contains("file:///local/track.mp3"));

    // Check remote song entry
    assert!(export.contains("#EXTINF:240,Remote Song"));
    assert!(export.contains("https://example.com/stream.mp3"));

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_ops_edge_cases() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    let playlist_id = db
        .create_playlist(Playlist {
            playlist_name: "Ops PL".to_string(),
            ..Default::default()
        })
        .unwrap();

    // add_to_playlist with None song field
    let empty_song = Song {
        song: None,
        ..Default::default()
    };
    db.add_to_playlist(&playlist_id, &vec![empty_song]).unwrap();

    // add_to_playlist duplicate songs
    let song = create_test_song("Song", "/path.mp3");
    let inserted = db.insert_songs(vec![song]).unwrap();
    let song_id = inserted[0].song.clone().unwrap().id.unwrap();

    db.add_to_playlist(&playlist_id, &inserted).unwrap();
    db.add_to_playlist(&playlist_id, &inserted).unwrap();

    assert!(db.is_song_in_playlist(&playlist_id, &song_id).unwrap());

    // remove_from_playlist for song not in playlist
    db.remove_from_playlist(&playlist_id, &vec!["non_existent_song"])
        .unwrap();

    // remove_playlist
    db.remove_playlist(&playlist_id).unwrap();
    assert!(!db.is_song_in_playlist(&playlist_id, &song_id).unwrap());

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_updates() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // update_song without ID (no-op)
    let song_no_id = InnerSong {
        title: Some("No ID".to_string()),
        ..Default::default()
    };
    db.update_song(&song_no_id).unwrap();

    // Insert song to update
    let song = Song {
        song: Some(InnerSong {
            title: Some("Initial Song".to_string()),
            path: Some("/song.mp3".to_string()),
            ..Default::default()
        }),
        album: Some(Album {
            album_name: Some("Initial Album".to_string()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_name: Some("Initial Artist".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let inserted = db.insert_songs(vec![song]).unwrap();
    let song_id = inserted[0].song.as_ref().unwrap().id.clone().unwrap();
    let album_id = inserted[0]
        .album
        .as_ref()
        .unwrap()
        .album_id
        .clone()
        .unwrap();
    let artist_id = inserted[0].artists[0].artist_id.clone().unwrap();

    // update_playlist
    let playlist_id = db
        .create_playlist(Playlist {
            playlist_name: "Initial PL".to_string(),
            ..Default::default()
        })
        .unwrap();

    db.update_playlist(Playlist {
        playlist_id: Some(playlist_id.clone()),
        playlist_name: "Updated PL".to_string(),
        ..Default::default()
    })
    .unwrap();

    let entity_res = db
        .get_entity_by_options(GetEntityOptions {
            playlist: Some(Playlist {
                playlist_id: Some(playlist_id),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
    let Some(EntityResultVariant::Playlists(playlists_list)) = entity_res.result else {
        panic!("Expected Playlists variant");
    };
    assert_eq!(&playlists_list.playlists[0].playlist_name, "Updated PL");

    // update_songs
    let mut updated_song = inserted[0].clone();
    updated_song.song.as_mut().unwrap().title = Some("Updated Title".to_string());
    updated_song.album.as_mut().unwrap().album_name = Some("Updated Album".to_string());
    updated_song.artists[0].artist_name = Some("Updated Artist".to_string());

    db.update_songs(vec![updated_song]).unwrap();

    let fetched_songs = db
        .get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong {
                id: Some(song_id.clone()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        fetched_songs[0]
            .song
            .as_ref()
            .unwrap()
            .title
            .as_ref()
            .unwrap(),
        "Updated Title"
    );

    let fetched_albums = db
        .get_entity_by_options(GetEntityOptions {
            album: Some(Album {
                album_id: Some(album_id),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
    let Some(EntityResultVariant::Albums(albums_list)) = fetched_albums.result else {
        panic!("Expected Albums variant");
    };
    assert_eq!(
        albums_list.albums[0].album_name.as_deref().unwrap(),
        "Updated Album"
    );

    let fetched_artists = db
        .get_entity_by_options(GetEntityOptions {
            artist: Some(Artist {
                artist_id: Some(artist_id),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
    let Some(EntityResultVariant::Artists(artists_list)) = fetched_artists.result else {
        panic!("Expected Artists variant");
    };
    assert_eq!(
        artists_list.artists[0].artist_name.as_deref().unwrap(),
        "Updated Artist"
    );

    // update_lyrics
    db.update_lyrics(song_id.clone(), "New Lyrics".to_string())
        .unwrap();
    let fetched_songs2 = db
        .get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong {
                id: Some(song_id),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        fetched_songs2[0]
            .song
            .as_ref()
            .unwrap()
            .lyrics
            .as_ref()
            .unwrap(),
        "New Lyrics"
    );

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_entity_by_options() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Null case
    let null_res = db
        .get_entity_by_options(GetEntityOptions::default())
        .unwrap();
    assert!(null_res.result.is_none());

    // Insert data
    let song = Song {
        song: Some(InnerSong {
            title: Some("Song".to_string()),
            path: Some("/path.mp3".to_string()),
            ..Default::default()
        }),
        album: Some(Album {
            album_name: Some("UniqueAlbumName".to_string()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_name: Some("UniqueArtistName".to_string()),
            ..Default::default()
        }],
        genre: vec![Genre {
            genre_name: Some("UniqueGenreName".to_string()),
            ..Default::default()
        }],
    };
    db.insert_songs(vec![song]).unwrap();

    // Query Album (exact vs partial)
    let alb_opt_exact = Album {
        album_name: Some("UniqueAlbumName".to_string()),
        ..Default::default()
    };
    let alb_res_exact = db
        .get_entity_by_options(GetEntityOptions {
            album: Some(alb_opt_exact),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();
    let Some(EntityResultVariant::Albums(alb_exact)) = alb_res_exact.result else {
        panic!("Expected Albums variant");
    };
    assert_eq!(alb_exact.albums.len(), 1);

    let alb_opt_partial = Album {
        album_name: Some("%Unique%".to_string()),
        ..Default::default()
    };
    let alb_res_partial = db
        .get_entity_by_options(GetEntityOptions {
            album: Some(alb_opt_partial),
            inclusive: Some(true),
            ..Default::default()
        })
        .unwrap();
    let Some(EntityResultVariant::Albums(alb_partial)) = alb_res_partial.result else {
        panic!("Expected Albums variant");
    };
    assert_eq!(alb_partial.albums.len(), 1);

    // Query Artist
    let art_opt = Artist {
        artist_name: Some("UniqueArtistName".to_string()),
        ..Default::default()
    };
    let art_res = db
        .get_entity_by_options(GetEntityOptions {
            artist: Some(art_opt),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();
    let Some(EntityResultVariant::Artists(art_list)) = art_res.result else {
        panic!("Expected Artists variant");
    };
    assert_eq!(art_list.artists.len(), 1);

    // Query Genre
    let gen_opt = Genre {
        genre_name: Some("UniqueGenreName".to_string()),
        ..Default::default()
    };
    let gen_res = db
        .get_entity_by_options(GetEntityOptions {
            genre: Some(gen_opt),
            inclusive: Some(false),
            ..Default::default()
        })
        .unwrap();
    let Some(EntityResultVariant::Genres(gen_list)) = gen_res.result else {
        panic!("Expected Genres variant");
    };
    assert_eq!(gen_list.genres.len(), 1);

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_analytics_edge_cases() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    let song = create_test_song("Song", "/path.mp3");
    let inserted = db.insert_songs(vec![song]).unwrap();
    let song_id = inserted[0].song.as_ref().unwrap().id.clone().unwrap();

    // Increment count when it does not exist
    db.increment_play_count(&song_id).unwrap();

    // Increment time when it exists
    db.increment_play_time(&song_id, 50.0).unwrap();

    // Fetch and check
    let analytics1 = db.get_top_listened_songs().unwrap();
    assert_eq!(analytics1.songs.len(), 1);
    assert_eq!(analytics1.songs[0].song_id, song_id);
    assert_eq!(analytics1.songs[0].time, 50.0);

    // Increment time when it does not exist for another song
    let song2 = create_test_song("Song 2", "/path2.mp3");
    let inserted2 = db.insert_songs(vec![song2]).unwrap();
    let song_id2 = inserted2[0].song.as_ref().unwrap().id.clone().unwrap();

    db.increment_play_time(&song_id2, 120.0).unwrap();

    let analytics2 = db.get_top_listened_songs().unwrap();
    assert_eq!(analytics2.songs.len(), 2);
    assert_eq!(analytics2.songs[0].time, 120.0);
    assert_eq!(analytics2.total_listen_time, 170.0);

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_songs_by_entities() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Insert songs with album, artist, genre
    let song = Song {
        song: Some(InnerSong {
            title: Some("Song Entity".to_string()),
            path: Some("/entity/track.mp3".to_string()),
            ..Default::default()
        }),
        album: Some(Album {
            album_name: Some("Target Album".to_string()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_name: Some("Target Artist".to_string()),
            ..Default::default()
        }],
        genre: vec![Genre {
            genre_name: Some("Target Genre".to_string()),
            ..Default::default()
        }],
    };

    let inserted = db.insert_songs(vec![song]).unwrap();
    let song_id = inserted[0].song.as_ref().unwrap().id.clone().unwrap();

    // Test get_songs_by_options with Album
    let songs_by_album = db
        .get_songs_by_options(GetSongOptions {
            album: Some(Album {
                album_name: Some("Target Album".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(songs_by_album.len(), 1);
    assert_eq!(
        songs_by_album[0]
            .song
            .as_ref()
            .unwrap()
            .id
            .as_ref()
            .unwrap(),
        &song_id
    );

    // Test get_songs_by_options with Artist
    let songs_by_artist = db
        .get_songs_by_options(GetSongOptions {
            artist: Some(Artist {
                artist_name: Some("Target Artist".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(songs_by_artist.len(), 1);
    assert_eq!(
        songs_by_artist[0]
            .song
            .as_ref()
            .unwrap()
            .id
            .as_ref()
            .unwrap(),
        &song_id
    );

    // Test get_songs_by_options with Genre
    let songs_by_genre = db
        .get_songs_by_options(GetSongOptions {
            genre: Some(Genre {
                genre_name: Some("Target Genre".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(songs_by_genre.len(), 1);
    assert_eq!(
        songs_by_genre[0]
            .song
            .as_ref()
            .unwrap()
            .id
            .as_ref()
            .unwrap(),
        &song_id
    );

    // Test add_to_playlist_bridge directly
    let playlist_id = db
        .create_playlist(Playlist {
            playlist_name: "Bridge PL".to_string(),
            ..Default::default()
        })
        .unwrap();

    db.add_to_playlist_bridge(playlist_id.clone(), song_id.clone())
        .unwrap();
    assert!(db.is_song_in_playlist(&playlist_id, &song_id).unwrap());

    cleanup(&db_path);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_remove_songs_outside_directories() {
    let db_path = get_test_db_path();
    let db = Database::new(db_path.clone());

    // Insert 1 song inside the directory we will scan
    let song_inside = create_test_song("Inside", "/music/folders/pop/song1.mp3");
    // Insert 1 song outside the directory we will scan
    let song_outside = create_test_song("Outside", "/downloads/song2.mp3");
    // Insert 1 song with no path (e.g. YouTube stream)
    let mut song_no_path = create_test_song("Stream", "");
    if let Some(ref mut inner) = song_no_path.song {
        inner.path = None; // Explicitly set to None
        inner.playback_url = Some("https://youtube.com/watch?v=123".to_string());
    }

    db.insert_songs(vec![song_inside, song_outside, song_no_path])
        .unwrap();

    // Verify all 3 songs are initially inserted
    let query_options = GetSongOptions {
        song: Some(SearchableSong::default()),
        ..Default::default()
    };
    let songs = db.get_songs_by_options(query_options.clone()).unwrap();
    assert_eq!(songs.len(), 3);

    // Call remove_songs_outside_directories with /music/folders as scan_dir
    let scan_dirs = vec![PathBuf::from("/music/folders")];
    db.remove_songs_outside_directories(&scan_dirs).unwrap();

    // Verify:
    // - "Inside" is preserved
    // - "Stream" is preserved (because it has no path)
    // - "Outside" is removed
    let remaining = db.get_songs_by_options(query_options).unwrap();
    assert_eq!(remaining.len(), 2);

    let titles: Vec<String> = remaining
        .iter()
        .map(|s| s.song.as_ref().unwrap().title.clone().unwrap())
        .collect();

    assert!(titles.contains(&"Inside".to_string()));
    assert!(titles.contains(&"Stream".to_string()));
    assert!(!titles.contains(&"Outside".to_string()));

    cleanup(&db_path);
}
