use std::{
    env,
    fs::{self, File},
    io::Write,
    sync::mpsc,
};

use threadpool::ThreadPool;

use crate::{playlist_scanner::PlaylistScanner, song_scanner::SongScanner};

#[test]
fn test_playlist_scan() {
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

    let mut pool = ThreadPool::new(1);

    let song_scanner = SongScanner::new(
        test_in_dir.clone(),
        &mut pool,
        test_out_dir.clone(),
        "".to_string(),
    );
    let playlist_scanner =
        PlaylistScanner::new(test_in_dir.clone(), test_out_dir.clone(), song_scanner);

    let mut input = File::create(test_in_dir.join("playlist.m3u")).unwrap();
    input.write_all(playlist_contents.as_bytes()).unwrap();

    let (tx_song, rx_song) = mpsc::channel();
    let (tx_playlist, rx_playlist) = mpsc::channel();
    playlist_scanner.start(tx_song, tx_playlist).unwrap();

    for (i, (_playlist, song)) in rx_song.into_iter().enumerate() {
        match i {
            0 => assert_eq!(song.unwrap().song.unwrap().title.unwrap(), "stream"),
            1 => assert_eq!(song.unwrap().song.unwrap().title.unwrap(), "320"),
            2 => assert_eq!(song.unwrap().song.unwrap().title.unwrap(), "stream.flac"),
            _ => {
                println!("{:?}", song);
                unreachable!()
            }
        }
    }

    // Check that rx_playlist has exactly one value.
    let _playlist_msg = rx_playlist.recv().unwrap();
    assert!(rx_playlist.try_recv().is_err());

    fs::remove_dir_all(test_in_dir).unwrap();
    fs::remove_dir_all(test_out_dir).unwrap();
}

#[test]
fn test_playlist_scan_with_extra_comments() {
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

    let mut pool = ThreadPool::new(1);

    let song_scanner = SongScanner::new(
        test_in_dir.clone(),
        &mut pool,
        test_out_dir.clone(),
        "".to_string(),
    );
    let playlist_scanner =
        PlaylistScanner::new(test_in_dir.clone(), test_out_dir.clone(), song_scanner);

    let mut input = File::create(test_in_dir.join("playlist.m3u")).unwrap();
    input.write_all(playlist_contents.as_bytes()).unwrap();

    let (tx_song, rx_song) = mpsc::channel();
    let (tx_playlist, rx_playlist) = mpsc::channel();
    playlist_scanner.start(tx_song, tx_playlist).unwrap();

    let mut titles = Vec::new();
    for (_playlist, song) in rx_song.into_iter() {
        titles.push(song.unwrap().song.unwrap().title.unwrap());
    }
    assert_eq!(titles, vec!["track1", "track2"]);

    // Check that rx_playlist has exactly one value.
    let _playlist_msg = rx_playlist.recv().unwrap();
    assert!(rx_playlist.try_recv().is_err());

    fs::remove_dir_all(test_in_dir).unwrap();
    fs::remove_dir_all(test_out_dir).unwrap();
}

#[test]
fn test_playlist_scan_single_entry() {
    let playlist_contents = r#"
#EXTM3U
#EXTINF:0,lonely_track
https://example.com/lonely_track"#;

    let test_out_dir = env::temp_dir().join("moosync-test-out-single");
    let test_in_dir = env::temp_dir().join("moosync-test-in-single");

    fs::create_dir_all(test_out_dir.clone()).unwrap();
    fs::create_dir_all(test_in_dir.clone()).unwrap();

    let mut pool = ThreadPool::new(1);

    let song_scanner = SongScanner::new(
        test_in_dir.clone(),
        &mut pool,
        test_out_dir.clone(),
        "".to_string(),
    );
    let playlist_scanner =
        PlaylistScanner::new(test_in_dir.clone(), test_out_dir.clone(), song_scanner);

    let mut input = File::create(test_in_dir.join("playlist.m3u")).unwrap();
    input.write_all(playlist_contents.as_bytes()).unwrap();

    let (tx_song, rx_song) = mpsc::channel();
    let (tx_playlist, rx_playlist) = mpsc::channel();
    playlist_scanner.start(tx_song, tx_playlist).unwrap();

    let songs: Vec<_> = rx_song
        .into_iter()
        .map(|(_playlist, song)| song.unwrap().song.unwrap().title.unwrap())
        .collect();
    assert_eq!(songs, vec!["lonely_track"]);

    // Check that rx_playlist has exactly one value.
    let _playlist_msg = rx_playlist.recv().unwrap();
    assert!(rx_playlist.try_recv().is_err());

    fs::remove_dir_all(test_in_dir).unwrap();
    fs::remove_dir_all(test_out_dir).unwrap();
}
