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

use std::fs;

use assertables::{assert_none, assert_some};
use lofty::tag::Tag;
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::context::desktop::lyrics_scanner::LyricsScanner;

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir_fixture() -> TempDir {
    TempDir::new("moosync_lrc_test").expect("failed to create temp dir")
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_scanner_scan_lrc_synced_parsing(temp_dir_fixture: TempDir) {
    let audio_path = temp_dir_fixture.path().join("sample_song.mp3");
    let lrc_path = temp_dir_fixture.path().join("sample_song.lrc");
    let lrc_content = "\
[ti:Sample Title]
[ar:Sample Artist]
[offset:500]
[00:05.12]First line of lyrics
[00:10.45]Second line of lyrics
[00:15.99]Third line of lyrics
";
    fs::write(&lrc_path, lrc_content).unwrap();

    let lyrics = LyricsScanner::scan_lrc(audio_path);

    assert_some!(&lyrics);
    let lyrics_val = lyrics.unwrap();
    assert!(lyrics_val.is_synced);
    assert_eq!(lyrics_val.lines.len(), 3);
    assert_eq!(lyrics_val.lines[0].text, "First line of lyrics");
    assert_eq!(lyrics_val.lines[0].time_ms, 5620);
    assert_eq!(lyrics_val.lines[1].text, "Second line of lyrics");
    assert_eq!(lyrics_val.lines[1].time_ms, 10950);
    assert_eq!(lyrics_val.lines[2].text, "Third line of lyrics");
    assert_eq!(lyrics_val.lines[2].time_ms, 16490);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_scanner_scan_lrc_multiple_timestamps_on_single_line(temp_dir_fixture: TempDir) {
    let audio_path = temp_dir_fixture.path().join("multi_ts.mp3");
    let lrc_path = temp_dir_fixture.path().join("multi_ts.lrc");
    let lrc_content = "[00:01.00][00:05.00]Chorus line\n";
    fs::write(&lrc_path, lrc_content).unwrap();

    let lyrics = LyricsScanner::scan_lrc(audio_path);

    assert_some!(&lyrics);
    let lyrics_val = lyrics.unwrap();
    assert!(lyrics_val.is_synced);
    assert_eq!(lyrics_val.lines.len(), 2);
    assert_eq!(lyrics_val.lines[0].text, "Chorus line");
    assert_eq!(lyrics_val.lines[0].time_ms, 1000);
    assert_eq!(lyrics_val.lines[1].text, "Chorus line");
    assert_eq!(lyrics_val.lines[1].time_ms, 5000);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_scanner_scan_plain_unsynced_text(temp_dir_fixture: TempDir) {
    let audio_path = temp_dir_fixture.path().join("plain.mp3");
    let lrc_path = temp_dir_fixture.path().join("plain.lrc");
    let lrc_content = "Just plain line 1\nJust plain line 2\n";
    fs::write(&lrc_path, lrc_content).unwrap();

    let lyrics = LyricsScanner::scan_lrc(audio_path);

    assert_some!(&lyrics);
    let lyrics_val = lyrics.unwrap();
    assert!(!lyrics_val.is_synced);
    assert_eq!(lyrics_val.lines.len(), 2);
    assert_eq!(lyrics_val.lines[0].text, "Just plain line 1");
    assert_eq!(lyrics_val.lines[0].time_ms, 0);
    assert_eq!(lyrics_val.lines[1].text, "Just plain line 2");
    assert_eq!(lyrics_val.lines[1].time_ms, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_scanner_extract_lyrics_from_id3_tag(temp_dir_fixture: TempDir) {
    let mut tag = Tag::new(lofty::tag::TagType::Id3v2);
    tag.insert_text(
        lofty::prelude::ItemKey::Lyrics,
        "[00:02.00]Tag Lyric Line 1\n[00:04.50]Tag Lyric Line 2".to_string(),
    );
    let dummy_path = temp_dir_fixture.path().join("dummy.mp3");

    let extracted = LyricsScanner::extract_lyrics(&tag, &dummy_path);

    assert_some!(&extracted);
    let lyrics_val = extracted.unwrap();
    assert!(lyrics_val.is_synced);
    assert_eq!(lyrics_val.lines.len(), 2);
    assert_eq!(lyrics_val.lines[0].text, "Tag Lyric Line 1");
    assert_eq!(lyrics_val.lines[0].time_ms, 2000);
    assert_eq!(lyrics_val.lines[1].text, "Tag Lyric Line 2");
    assert_eq!(lyrics_val.lines[1].time_ms, 4500);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_scanner_scan_lrc_non_existent_returns_none(temp_dir_fixture: TempDir) {
    let non_existent = temp_dir_fixture.path().join("missing_song_123.mp3");

    let lyrics = LyricsScanner::scan_lrc(non_existent);

    assert_none!(lyrics);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_scanner_scan_lrc_empty_file_returns_none(temp_dir_fixture: TempDir) {
    let audio_path = temp_dir_fixture.path().join("empty.mp3");
    let lrc_path = temp_dir_fixture.path().join("empty.lrc");
    fs::write(&lrc_path, "   \n\n  ").unwrap();

    let lyrics = LyricsScanner::scan_lrc(audio_path);

    assert_none!(lyrics);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_scanner_scan_lrc_only_headers_returns_none(temp_dir_fixture: TempDir) {
    let audio_path = temp_dir_fixture.path().join("headers_only.mp3");
    let lrc_path = temp_dir_fixture.path().join("headers_only.lrc");
    fs::write(&lrc_path, "[ti:Title]\n[ar:Artist]\n[al:Album]\n").unwrap();

    let lyrics = LyricsScanner::scan_lrc(audio_path);

    assert_none!(lyrics);
}
