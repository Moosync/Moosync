// // Moosync
// // Copyright (C) 2024, 2025  Moosync <support@moosync.app>
// //
// // This program is free software: you can redistribute it and/or modify
// // it under the terms of the GNU General Public License as published by
// // the Free Software Foundation, either version 3 of the License, or
// // (at your option) any later version.
// //
// // This program is distributed in the hope that it will be useful,
// // but WITHOUT ANY WARRANTY; without even the implied warranty of
// // MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// // GNU General Public License for more details.
// //
// // You should have received a copy of the GNU General Public License
// // along with this program.  If not, see <http://www.gnu.org/licenses/>.

// use std::{env::temp_dir, fs, path::PathBuf};

// use crate::database::Database;
// use songs_proto::moosync::types::{Album, Artist, Genre, GetEntityOptions, Playlist};
// use types::songs::{GetSongOptions, InnerSong, SearchableSong, Song, SongType};
// use uuid::Uuid;

// // Helper function to create a unique test DB path
// fn get_test_db_path() -> PathBuf {
//     let file_name = format!("moosync_test_{}.db", Uuid::new_v4());
//     temp_dir().join(file_name)
// }

// // Helper function to clean up DB files
// fn cleanup(db_path: &PathBuf) {
//     let base_path = db_path.to_string_lossy().to_string();

//     // Ignore errors as files might not exist
//     let _ = fs::remove_file(db_path);
//     let _ = fs::remove_file(format!("{}-shm", base_path));
//     let _ = fs::remove_file(format!("{}-wal", base_path));
// }

// // Test utility function to create a test song
// fn create_test_song(title: &str, path: &str) -> Song {
//     Song {
//         song: InnerSong {
//             _id: None,
//             title: Some(title.to_string()),
//             path: Some(path.to_string()),
//             song_cover_path_high: None,
//             song_cover_path_low: None,
//             date_added: None,
//             size: Some(0.0),
//             bitrate: Some(0.0),
//             codec: None,
//             duration: Some(300.0),
//             sample_rate: Some(0.0),
//             lyrics: None,
//             type_: SongType::Local,
//             ..Default::default()
//         },
//         album: Some(Album {
//             album_id: None,
//             album_name: Some("Test Album".to_string()),
//             ..Default::default()
//         }),
//         artists: Some(vec![Artist {
//             artist_id: None,
//             artist_name: Some("Test Artist".to_string()),
//             ..Default::default()
//         }]),
//         genre: Some(vec![Genre {
//             genre_id: None,
//             genre_name: Some("Test Genre".to_string()),
//             ..Default::default()
//         }]),
//     }
// }

// // Test song insertion
// #[test]
// fn test_insert_song() {
//     let db_path = get_test_db_path();
//     let db = Database::new(db_path.clone());

//     let test_song = create_test_song("Test Song", "/path/to/test.mp3");
//     let result = db.insert_songs(vec![test_song]).unwrap();

//     assert_eq!(result.len(), 1);
//     assert!(result[0].song.id.is_some());
//     assert_eq!(result[0].song.title.as_ref().unwrap(), "Test Song");

//     // Test album was created
//     let album = result[0].album.clone().unwrap();
//     assert!(album.album_id.is_some());
//     assert_eq!(album.album_name, Some("Test Album".to_string()));

//     // Test artist was created
//     let artists = result[0].artists.clone().unwrap();
//     assert_eq!(artists.len(), 1);
//     assert!(artists[0].artist_id.is_some());
//     assert_eq!(artists[0].artist_name, Some("Test Artist".to_string()));

//     // Test genre was created
//     let genres = result[0].genre.clone().unwrap();
//     assert_eq!(genres.len(), 1);
//     assert!(genres[0].genre_id.is_some());
//     assert_eq!(genres[0].genre_name, Some("Test Genre".to_string()));

//     cleanup(&db_path);
// }

// // Test fetching songs by options
// #[test]
// fn test_get_songs_by_options() {
//     let db_path = get_test_db_path();
//     let db = Database::new(db_path.clone());

//     // Insert test songs
//     db.insert_songs(vec![
//         create_test_song("Song 1", "/path/to/song1.mp3"),
//         create_test_song("Song 2", "/path/to/song2.mp3"),
//         create_test_song("Different", "/path/to/different.mp3"),
//     ])
//     .unwrap();

//     // Test fetching by partial title match
//     let options = GetSongOptions {
//         song: Some(SearchableSong {
//             title: Some("%Song%".to_string()),
//             ..Default::default()
//         }),
//         inclusive: Some(true),
//         ..Default::default()
//     };

//     let songs = db.get_songs_by_options(options).unwrap();
//     assert_eq!(songs.len(), 2);
//     assert!(songs
//         .iter()
//         .any(|s| s.song.title.as_ref().unwrap() == "Song 1"));
//     assert!(songs
//         .iter()
//         .any(|s| s.song.title.as_ref().unwrap() == "Song 2"));

//     // Test fetching by exact path
//     let options = GetSongOptions {
//         song: Some(SearchableSong {
//             path: Some("/path/to/different.mp3".to_string()),
//             ..Default::default()
//         }),
//         inclusive: Some(false),
//         ..Default::default()
//     };

//     let songs = db.get_songs_by_options(options).unwrap();
//     assert_eq!(songs.len(), 1);
//     assert_eq!(songs[0].song.title.as_ref().unwrap(), "Different");

//     cleanup(&db_path);
// }

// // Test updating a song
// #[test]
// fn test_update_song() {
//     let db_path = get_test_db_path();
//     let db = Database::new(db_path.clone());

//     // Insert a test song
//     let songs = db
//         .insert_songs(vec![create_test_song(
//             "Original Title",
//             "/path/to/original.mp3",
//         )])
//         .unwrap();
//     let song_id = songs[0].song.id.clone().unwrap();

//     // Update the song
//     let updatable_song = InnerSong {
//         _id: Some(song_id.clone()),
//         title: Some("Updated Title".to_string()),
//         ..Default::default()
//     };

//     db.update_song(updatable_song).unwrap();

//     // Fetch the updated song
//     let options = GetSongOptions {
//         song: Some(SearchableSong {
//             _id: Some(song_id),
//             ..Default::default()
//         }),
//         inclusive: Some(false),
//         ..Default::default()
//     };

//     let updated_songs = db.get_songs_by_options(options).unwrap();
//     assert_eq!(updated_songs.len(), 1);
//     assert_eq!(
//         updated_songs[0].song.title.as_ref().unwrap(),
//         "Updated Title"
//     );

//     cleanup(&db_path);
// }

// // Test removing songs
// #[test]
// fn test_remove_songs() {
//     let db_path = get_test_db_path();
//     let db = Database::new(db_path.clone());

//     // Insert test songs
//     let songs = db
//         .insert_songs(vec![
//             create_test_song("Song to Keep", "/path/to/keep.mp3"),
//             create_test_song("Song to Remove", "/path/to/remove.mp3"),
//         ])
//         .unwrap();

//     // Get IDs
//     let keep_id = songs[0].song.id.clone().unwrap();
//     let remove_id = songs[1].song.id.clone().unwrap();

//     // Make sure we have 2 songs before removing
//     let initial_songs = db
//         .get_songs_by_options(GetSongOptions {
//             song: Some(SearchableSong {
//                 type_: Some(SongType::Local), // Filter by song type to ensure we get all test songs
//                 ..Default::default()
//             }),
//             inclusive: Some(true),
//             ..Default::default()
//         })
//         .unwrap();
//     assert_eq!(initial_songs.len(), 2);

//     // Add analytics data to both songs
//     db.increment_play_count(keep_id.clone()).unwrap();
//     db.increment_play_count(remove_id.clone()).unwrap();
//     db.increment_play_time(keep_id.clone(), 60.0).unwrap();
//     db.increment_play_time(remove_id.clone(), 120.0).unwrap();

//     // Remove one song
//     db.remove_songs(vec![remove_id.clone()]).unwrap();

//     // Verify only one song remains
//     let all_songs = db
//         .get_songs_by_options(GetSongOptions {
//             song: Some(SearchableSong {
//                 type_: Some(SongType::Local), // Filter by song type to ensure we get all remaining songs
//                 ..Default::default()
//             }),
//             inclusive: Some(true),
//             ..Default::default()
//         })
//         .unwrap();
//     assert_eq!(all_songs.len(), 1);
//     assert_eq!(all_songs[0].song.id.as_ref().unwrap(), &keep_id);

//     // Verify analytics data for the removed song is also gone
//     let analytics = db.get_top_listened_songs().unwrap();
//     let removed_song_analytics = analytics.songs.iter().find(|(id, _)| id == &remove_id);
//     assert!(
//         removed_song_analytics.is_none(),
//         "Analytics for removed song should be deleted"
//     );

//     // Verify analytics data for the kept song is still there
//     let kept_song_analytics = analytics.songs.iter().find(|(id, _)| id == &keep_id);
//     assert!(
//         kept_song_analytics.is_some(),
//         "Analytics for kept song should still exist"
//     );

//     // Try to fetch the removed song specifically
//     let removed_options = GetSongOptions {
//         song: Some(SearchableSong {
//             _id: Some(remove_id),
//             ..Default::default()
//         }),
//         inclusive: Some(false),
//         ..Default::default()
//     };

//     let removed_songs = db.get_songs_by_options(removed_options).unwrap();
//     assert_eq!(removed_songs.len(), 0);

//     cleanup(&db_path);
// }

// // Test playlist CRUD operations
// #[test]
// fn test_playlist_operations() {
//     let db_path = get_test_db_path();
//     let db = Database::new(db_path.clone());

//     // Create a playlist
//     let playlist = Playlist {
//         playlist_id: None,
//         playlist_name: "Test Playlist".to_string(),
//         playlist_coverpath: None,
//         playlist_path: None,
//         playlist_desc: None,
//         extension: None,
//         icon: None,
//         library_item: None,
//         ..Default::default()
//     };

//     let playlist_id = db.create_playlist(playlist).unwrap();

//     // Insert songs
//     let songs = db
//         .insert_songs(vec![
//             create_test_song("Playlist Song 1", "/path/to/playlist1.mp3"),
//             create_test_song("Playlist Song 2", "/path/to/playlist2.mp3"),
//         ])
//         .unwrap();

//     // Add songs to playlist
//     db.add_to_playlist(playlist_id.clone(), songs.clone())
//         .unwrap();

//     // Get playlist songs
//     let playlist_options = Playlist {
//         playlist_id: Some(playlist_id.clone()),
//         ..Default::default()
//     };

//     let result = db
//         .get_entity_by_options(GetEntityOptions {
//             playlist: Some(playlist_options.clone()),
//             inclusive: Some(false),
//             ..Default::default()
//         })
//         .unwrap();

//     // The result is returned as an array of playlists
//     let playlists = result.as_array().unwrap();
//     assert_eq!(playlists.len(), 1);
//     let playlist = &playlists[0];

//     // Verify we can access the playlist's properties
//     assert!(playlist["playlist_name"]
//         .as_str()
//         .unwrap()
//         .contains("Test Playlist"));

//     // Remove one song from playlist
//     let song_id_to_remove = songs[0].song.id.clone().unwrap();
//     db.remove_from_playlist(playlist_id.clone(), vec![song_id_to_remove])
//         .unwrap();

//     // Delete the playlist
//     db.remove_playlist(playlist_id).unwrap();

//     // Verify playlist is gone
//     let all_playlists = db
//         .get_entity_by_options(GetEntityOptions {
//             playlist: Some(Playlist::default()),
//             inclusive: Some(true),
//             ..Default::default()
//         })
//         .unwrap();

//     let playlists = all_playlists.as_array().unwrap();
//     assert_eq!(playlists.len(), 0);

//     cleanup(&db_path);
// }

// // Test album operations
// #[test]
// fn test_album_operations() {
//     let db_path = get_test_db_path();
//     let db = Database::new(db_path.clone());

//     // Insert songs with the same album
//     db.insert_songs(vec![
//         create_test_song("Album Song 1", "/path/to/album1.mp3"),
//         create_test_song("Album Song 2", "/path/to/album2.mp3"),
//     ])
//     .unwrap();

//     // Get the album
//     let album_options = Album {
//         album_name: Some("Test Album".to_string()),
//         ..Default::default()
//     };

//     let result = db
//         .get_entity_by_options(GetEntityOptions {
//             album: Some(album_options.clone()),
//             inclusive: Some(false),
//             ..Default::default()
//         })
//         .unwrap();

//     // The result is returned as an array of albums
//     let albums = result.as_array().unwrap();
//     assert_eq!(albums.len(), 1);
//     let album = &albums[0];

//     // Verify we can access the album's properties
//     assert!(album["album_name"].as_str().unwrap().contains("Test Album"));

//     // Test updating album
//     let mut album_to_update = Album {
//         album_name: Some("Test Album".to_string()),
//         year: Some("2023".to_string()),
//         ..Default::default()
//     };

//     // First get the album ID
//     let albums = db
//         .get_entity_by_options(GetEntityOptions {
//             album: Some(album_options),
//             inclusive: Some(false),
//             ..Default::default()
//         })
//         .unwrap();

//     let album_id = albums.as_array().unwrap()[0]
//         .as_object()
//         .unwrap()
//         .get("album_id")
//         .unwrap()
//         .as_str()
//         .unwrap()
//         .to_string();

//     album_to_update.album_id = Some(album_id.clone());
//     db.update_album(album_to_update).unwrap();

//     // Verify update
//     let updated_album = db
//         .get_entity_by_options(GetEntityOptions {
//             album: Some(Album {
//                 album_id: Some(album_id),
//                 ..Default::default()
//             }),
//             inclusive: Some(false),
//             ..Default::default()
//         })
//         .unwrap();

//     let year = updated_album.as_array().unwrap()[0]
//         .as_object()
//         .unwrap()
//         .get("year")
//         .unwrap()
//         .as_str()
//         .unwrap();

//     assert_eq!(year, "2023");

//     cleanup(&db_path);
// }

// // Test artist operations
// #[test]
// fn test_artist_operations() {
//     let db_path = get_test_db_path();
//     let db = Database::new(db_path.clone());

//     // Insert songs with the same artist
//     db.insert_songs(vec![
//         create_test_song("Artist Song 1", "/path/to/artist1.mp3"),
//         create_test_song("Artist Song 2", "/path/to/artist2.mp3"),
//     ])
//     .unwrap();

//     // Get the artist
//     let artist_options = Artist {
//         artist_name: Some("Test Artist".to_string()),
//         ..Default::default()
//     };

//     let result = db
//         .get_entity_by_options(GetEntityOptions {
//             artist: Some(artist_options.clone()),
//             inclusive: Some(false),
//             ..Default::default()
//         })
//         .unwrap();

//     // The result is returned as an array of artists
//     let artists = result.as_array().unwrap();
//     assert!(!artists.is_empty(), "Should return at least one artist");
//     let artist = &artists[0];

//     // Verify we can access the artist's properties
//     assert!(artist["artist_name"]
//         .as_str()
//         .unwrap()
//         .contains("Test Artist"));

//     // Test updating artist
//     let mut artist_to_update = Artist {
//         artist_name: Some("Test Artist".to_string()),
//         artist_coverpath: Some("https://example.com/cover.jpg".to_string()),
//         ..Default::default()
//     };

//     // First get the artist ID
//     let artists = db
//         .get_entity_by_options(GetEntityOptions {
//             artist: Some(artist_options),
//             inclusive: Some(false),
//             ..Default::default()
//         })
//         .unwrap();

//     // Debug check for array content
//     let artists_array = artists.as_array().unwrap();
//     assert!(
//         !artists_array.is_empty(),
//         "Artists array should not be empty"
//     );

//     // Safely access the artist_id
//     let artist_obj = &artists_array[0];
//     assert!(artist_obj.is_object(), "Artist should be an object");

//     let artist_id_value = artist_obj
//         .get("artist_id")
//         .expect("artist_id field should exist");
//     assert!(artist_id_value.is_string(), "artist_id should be a string");

//     let artist_id = artist_id_value.as_str().unwrap().to_string();
//     assert!(!artist_id.is_empty(), "artist_id should not be empty");

//     artist_to_update.artist_id = Some(artist_id.clone());
//     db.update_artist(artist_to_update).unwrap();

//     // Verify update
//     let updated_artist = db
//         .get_entity_by_options(GetEntityOptions {
//             artist: Some(Artist {
//                 artist_id: Some(artist_id.clone()),
//                 ..Default::default()
//             }),
//             inclusive: Some(false),
//             ..Default::default()
//         })
//         .unwrap();

//     let artists_array = updated_artist.as_array().unwrap();
//     assert!(
//         !artists_array.is_empty(),
//         "Updated artists array should not be empty"
//     );

//     let artist_obj = &artists_array[0];

//     // Check if the update was successful by verifying the artist_id matches
//     let retrieved_id = artist_obj
//         .get("artist_id")
//         .expect("artist_id field should exist")
//         .as_str()
//         .expect("artist_id should be a string");

//     assert_eq!(
//         retrieved_id, artist_id,
//         "Retrieved artist ID should match the one we set"
//     );

//     cleanup(&db_path);
// }

// // Test searching
// #[test]
// fn test_search() {
//     let db_path = get_test_db_path();
//     let db = Database::new(db_path.clone());

//     // Insert variety of entities
//     db.insert_songs(vec![
//         create_test_song("Searchable Song", "/path/to/searchable.mp3"),
//         create_test_song("Another Track", "/path/to/track.mp3"),
//     ])
//     .unwrap();

//     // Create a searchable playlist
//     let playlist = Playlist {
//         playlist_id: None,
//         playlist_name: "Searchable Playlist".to_string(),
//         playlist_coverpath: None,
//         playlist_path: None,
//         ..Default::default()
//     };

//     db.create_playlist(playlist).unwrap();

//     // Search for "Search"
//     let search_results = db.search_all("Search".to_string()).unwrap();

//     assert!(!search_results.songs.is_empty());
//     assert!(!search_results.playlists.is_empty());

//     assert!(search_results.songs.iter().any(|s| s
//         .song
//         .title
//         .as_ref()
//         .unwrap()
//         .contains("Searchable")));

//     assert!(search_results
//         .playlists
//         .iter()
//         .any(|p| p.playlist_name.contains("Searchable")));

//     cleanup(&db_path);
// }

// // Test analytics operations
// #[test]
// fn test_analytics() {
//     let db_path = get_test_db_path();
//     let db = Database::new(db_path.clone());

//     // Insert a song
//     let songs = db
//         .insert_songs(vec![create_test_song(
//             "Analytics Test",
//             "/path/to/analytics.mp3",
//         )])
//         .unwrap();

//     let song_id = songs[0].song.id.clone().unwrap();

//     // Increment play count multiple times
//     for _ in 0..5 {
//         db.increment_play_count(song_id.clone()).unwrap();
//     }

//     // Add some play time
//     db.increment_play_time(song_id.clone(), 120.0).unwrap();
//     db.increment_play_time(song_id.clone(), 180.0).unwrap();

//     // Get top listened songs
//     let analytics = db.get_top_listened_songs().unwrap();

//     // Verify analytics contains songs
//     assert!(
//         !analytics.songs.is_empty(),
//         "Analytics should contain songs"
//     );

//     // Find our song in the analytics data
//     let song_analytics = analytics
//         .songs
//         .iter()
//         .find(|(song, _)| song.as_str() == song_id.as_str());

//     // Verify our song was found and has the expected play time
//     assert!(song_analytics.is_some(), "Song should be in analytics data");
//     if let Some(listen_time) = song_analytics {
//         assert!(listen_time.time > 0.0, "Play time should be recorded");
//     }

//     cleanup(&db_path);
// }
