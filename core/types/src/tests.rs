// use super::common::SearchByTerm;
// use songs_proto::moosync::types::{Artist, InnerSong, Song, SongType};
// use std::str::FromStr;

// #[test]
// fn test_song_type_display() {
//     assert_eq!(SongType::Local.to_string(), "LOCAL");
//     assert_eq!(SongType::Url.to_string(), "URL");
//     assert_eq!(SongType::Spotify.to_string(), "SPOTIFY");
//     assert_eq!(SongType::Dash.to_string(), "DASH");
//     assert_eq!(SongType::Hls.to_string(), "HLS");
// }

// #[test]
// fn test_song_type_from_str() {
//     assert_eq!(SongType::from_str("LOCAL").unwrap(), SongType::Local);
//     assert_eq!(SongType::from_str("URL").unwrap(), SongType::Url);
//     assert_eq!(SongType::from_str("SPOTIFY").unwrap(), SongType::Spotify);
//     assert_eq!(SongType::from_str("DASH").unwrap(), SongType::Dash);
//     assert_eq!(SongType::from_str("HLS").unwrap(), SongType::Hls);
//     assert!(SongType::from_str("INVALID").is_err());
// }

// #[test]
// fn test_inner_song_search_by_term() {
//     let term = Some("test search".to_string());
//     let song = InnerSong::search_by_term(term.clone());
//     assert_eq!(song.title, term);
//     assert_eq!(song.path, term);
// }

// #[test]
// fn test_song_debug_format() {
//     let song = Song {
//         song: InnerSong {
//             _id: Some("123".to_string()),
//             title: Some("My Song".to_string()),
//             ..Default::default()
//         },
//         artists: Some(vec![Artist {
//             artist_name: Some("The Artist".to_string()),
//             ..Default::default()
//         }]),
//         ..Default::default()
//     };

//     let debug_str = format!("{:?}", song);
//     assert_eq!(debug_str, "The Artist - My Song (123)");
// }

// #[test]
// fn test_song_debug_format_no_info() {
//     let song = Song::default();
//     let debug_str = format!("{:?}", song);
//     assert_eq!(debug_str, "No Artist - No Title (No ID)");
// }
