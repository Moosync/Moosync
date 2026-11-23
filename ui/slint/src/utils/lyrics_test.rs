use assertables::{assert_none, assert_some};
use rstest::rstest;
use songs_proto::moosync::types::{InnerSong, LyricLine, Lyrics, Song};
use tracing_test::traced_test;

use crate::{
    test_utils::{TestSlintSmContext, state_manager_fixture},
    utils::lyrics::fetch_song_lyrics,
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_song_lyrics_from_database(state_manager_fixture: TestSlintSmContext) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;

    let db = sm.get_database();
    let song = Song {
        song: Some(InnerSong {
            id: Some("lyrics_test_song_1".to_string()),
            title: Some("Song With Lyrics".to_string()),
            lyrics: Some(Lyrics {
                lines: vec![
                    LyricLine {
                        text: "First line".to_string(),
                        time_ms: 1000,
                    },
                    LyricLine {
                        text: "Second line".to_string(),
                        time_ms: 2000,
                    },
                ],
                is_synced: true,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    db.insert_songs(vec![song.clone()]).await.unwrap();

    let lyrics = fetch_song_lyrics(&sm, &song).await;

    assert_some!(&lyrics);
    let lyrics_val = lyrics.unwrap();
    assert!(lyrics_val.is_synced);
    assert_eq!(lyrics_val.lines.len(), 2);
    assert_eq!(lyrics_val.lines[0].text, "First line");
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_song_lyrics_none_when_empty(state_manager_fixture: TestSlintSmContext) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;

    let song = Song {
        song: Some(InnerSong {
            id: Some("lyrics_test_song_2".to_string()),
            title: Some("Song Without Lyrics".to_string()),
            lyrics: None,
            ..Default::default()
        }),
        ..Default::default()
    };

    let lyrics = fetch_song_lyrics(&sm, &song).await;

    assert_none!(lyrics);
}
