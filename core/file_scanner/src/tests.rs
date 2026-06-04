use std::{
    env,
    fs::{self, File},
    io::Write,
    sync::{Arc, Mutex},
};

use songs_proto::moosync::types::{Playlist, Song};

use crate::{ScanProgress, ScannerHolder};

#[tokio::test]
async fn test_playlist_scan() {
    let playlist_contents = r#"
#EXTM3U
#EXTINF:0,stream
#EXTVLCOPT:network-caching=1000
https://cast.animu.com.br:9079/stream
#EXTINF:0,320
#EXTVLCOPT:network-caching=1000
https://radio.stereoanime.net/listen/stereoanime/320
#EXTINF:0,stream.flac
#EXTVLCOPT:network-caching=1000
https://chiru.no/stream.flac"#;

    let test_out_dir = env::temp_dir().join("moosync-test-out");
    let test_in_dir = env::temp_dir().join("moosync-test-in");

    fs::create_dir_all(test_out_dir.clone()).unwrap();
    fs::create_dir_all(test_in_dir.clone()).unwrap();

    let mut input = File::create(test_in_dir.join("playlist.m3u")).unwrap();
    input.write_all(playlist_contents.as_bytes()).unwrap();

    let song_count = Arc::new(Mutex::new(0));
    let song_count_clone = song_count.clone();
    let playlist_count = Arc::new(Mutex::new(0));
    let playlist_count_clone = playlist_count.clone();

    let mut scanner = ScannerHolder::new();
    scanner.set_scan_dir(test_in_dir.clone());
    scanner.set_thumbnail_dir(test_out_dir.clone());
    scanner.set_artist_split("".to_string());

    let song_count_clone_inner = song_count_clone.clone();
    scanner.set_on_song(move |_playlist_id: Option<String>, songs: Vec<Song>| {
        let song_count_clone_inner = song_count_clone_inner.clone();
        async move {
            let mut count = song_count_clone_inner.lock().unwrap();
            for song in songs {
                match *count {
                    0 => assert_eq!(song.song.unwrap().title.unwrap(), "stream"),
                    1 => assert_eq!(song.song.unwrap().title.unwrap(), "320"),
                    2 => assert_eq!(song.song.unwrap().title.unwrap(), "stream.flac"),
                    _ => unreachable!(),
                }
                *count += 1;
            }
        }
    });

    scanner.set_on_playlist(move |playlists: Vec<Playlist>| {
        let playlist_count_clone = playlist_count_clone.clone();
        async move {
            let mut count = playlist_count_clone.lock().unwrap();
            for _playlist in playlists {
                *count += 1;
            }
        }
    });

    let mut progress_rx = scanner.add_subscriber();

    scanner.start_scan().await.unwrap();

    // Verify progress channel
    let mut progress_events = Vec::new();
    while let Ok(evt) = progress_rx.try_recv() {
        progress_events.push(evt);
    }
    assert!(!progress_events.is_empty());
    assert_eq!(*progress_events.last().unwrap(), ScanProgress::STOPPED);

    // Verify assertions inside callbacks were called expected times
    for _ in 0..100 {
        if *song_count.lock().unwrap() == 3 && *playlist_count.lock().unwrap() == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    assert_eq!(*song_count.lock().unwrap(), 3);
    assert_eq!(*playlist_count.lock().unwrap(), 1);

    fs::remove_dir_all(test_in_dir).unwrap();
    fs::remove_dir_all(test_out_dir).unwrap();
}

#[tokio::test]
async fn test_playlist_scan_with_extra_comments() {
    let playlist_contents = r#"
#EXTM3U
# This is an extra comment line
#EXTINF:0,track1
https://example.com/track1
# Another comment
#EXTINF:0,track2
https://example.com/track2"#;

    let test_out_dir = env::temp_dir().join("moosync-test-out-comments");
    let test_in_dir = env::temp_dir().join("moosync-test-in-comments");

    fs::create_dir_all(test_out_dir.clone()).unwrap();
    fs::create_dir_all(test_in_dir.clone()).unwrap();

    let mut input = File::create(test_in_dir.join("playlist.m3u")).unwrap();
    input.write_all(playlist_contents.as_bytes()).unwrap();

    let song_count = Arc::new(Mutex::new(0));
    let song_count_clone = song_count.clone();
    let playlist_count = Arc::new(Mutex::new(0));
    let playlist_count_clone = playlist_count.clone();

    let mut scanner = ScannerHolder::new();
    scanner.set_scan_dir(test_in_dir.clone());
    scanner.set_thumbnail_dir(test_out_dir.clone());
    scanner.set_artist_split("".to_string());

    let song_count_clone_inner = song_count_clone.clone();
    scanner.set_on_song(move |_playlist_id: Option<String>, songs: Vec<Song>| {
        let song_count_clone_inner = song_count_clone_inner.clone();
        async move {
            let mut count = song_count_clone_inner.lock().unwrap();
            for song in songs {
                match *count {
                    0 => assert_eq!(song.song.unwrap().title.unwrap(), "track1"),
                    1 => assert_eq!(song.song.unwrap().title.unwrap(), "track2"),
                    _ => unreachable!(),
                }
                *count += 1;
            }
        }
    });

    scanner.set_on_playlist(move |playlists: Vec<Playlist>| {
        let playlist_count_clone = playlist_count_clone.clone();
        async move {
            let mut count = playlist_count_clone.lock().unwrap();
            for _playlist in playlists {
                *count += 1;
            }
        }
    });

    let mut progress_rx = scanner.add_subscriber();

    scanner.start_scan().await.unwrap();

    // Verify progress channel
    let mut progress_events = Vec::new();
    while let Ok(evt) = progress_rx.try_recv() {
        progress_events.push(evt);
    }
    assert!(!progress_events.is_empty());
    assert_eq!(*progress_events.last().unwrap(), ScanProgress::STOPPED);

    // Verify assertions inside callbacks were called expected times
    for _ in 0..100 {
        if *song_count.lock().unwrap() == 2 && *playlist_count.lock().unwrap() == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    assert_eq!(*song_count.lock().unwrap(), 2);
    assert_eq!(*playlist_count.lock().unwrap(), 1);

    fs::remove_dir_all(test_in_dir).unwrap();
    fs::remove_dir_all(test_out_dir).unwrap();
}

#[tokio::test]
async fn test_playlist_scan_single_entry() {
    let playlist_contents = r#"
#EXTM3U
#EXTINF:0,lonely_track
https://example.com/lonely_track"#;

    let test_out_dir = env::temp_dir().join("moosync-test-out-single");
    let test_in_dir = env::temp_dir().join("moosync-test-in-single");

    fs::create_dir_all(test_out_dir.clone()).unwrap();
    fs::create_dir_all(test_in_dir.clone()).unwrap();

    let mut input = File::create(test_in_dir.join("playlist.m3u")).unwrap();
    input.write_all(playlist_contents.as_bytes()).unwrap();

    let song_count = Arc::new(Mutex::new(0));
    let song_count_clone = song_count.clone();
    let playlist_count = Arc::new(Mutex::new(0));
    let playlist_count_clone = playlist_count.clone();

    let mut scanner = ScannerHolder::new();
    scanner.set_scan_dir(test_in_dir.clone());
    scanner.set_thumbnail_dir(test_out_dir.clone());
    scanner.set_artist_split("".to_string());

    let song_count_clone_inner = song_count_clone.clone();
    scanner.set_on_song(move |_playlist_id: Option<String>, songs: Vec<Song>| {
        let song_count_clone_inner = song_count_clone_inner.clone();
        async move {
            let mut count = song_count_clone_inner.lock().unwrap();
            for song in songs {
                match *count {
                    0 => assert_eq!(song.song.unwrap().title.unwrap(), "lonely_track"),
                    _ => unreachable!(),
                }
                *count += 1;
            }
        }
    });

    scanner.set_on_playlist(move |playlists: Vec<Playlist>| {
        let playlist_count_clone = playlist_count_clone.clone();
        async move {
            let mut count = playlist_count_clone.lock().unwrap();
            for _playlist in playlists {
                *count += 1;
            }
        }
    });

    let mut progress_rx = scanner.add_subscriber();

    scanner.start_scan().await.unwrap();

    // Verify progress channel
    let mut progress_events = Vec::new();
    while let Ok(evt) = progress_rx.try_recv() {
        progress_events.push(evt);
    }
    assert!(!progress_events.is_empty());
    assert_eq!(*progress_events.last().unwrap(), ScanProgress::STOPPED);

    // Verify assertions inside callbacks were called expected times
    for _ in 0..100 {
        if *song_count.lock().unwrap() == 1 && *playlist_count.lock().unwrap() == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    assert_eq!(*song_count.lock().unwrap(), 1);
    assert_eq!(*playlist_count.lock().unwrap(), 1);

    fs::remove_dir_all(test_in_dir).unwrap();
    fs::remove_dir_all(test_out_dir).unwrap();
}
