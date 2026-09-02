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

use assertables::{assert_none, assert_some, assert_some_eq_x};
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
fn test_lyrics_scanner_scan_lrc_file_parsing(temp_dir_fixture: TempDir) {
    let audio_path = temp_dir_fixture.path().join("sample_song.mp3");
    let lrc_path = temp_dir_fixture.path().join("sample_song.lrc");
    let lrc_content = "\
[ti:Sample Title]
[ar:Sample Artist]
[00:05.12]First line of lyrics
[00:10.45]Second line of lyrics
[00:15.99]Third line of lyrics
";
    fs::write(&lrc_path, lrc_content).unwrap();

    let lyrics = LyricsScanner::scan_lrc(audio_path);

    assert_some!(&lyrics);
    let lyrics_text = lyrics.unwrap();
    assert!(lyrics_text.contains("First line of lyrics"));
    assert!(lyrics_text.contains("Second line of lyrics"));
    assert!(lyrics_text.contains("Third line of lyrics"));
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
fn test_lyrics_scanner_extract_lyrics_from_tag_or_lrc(temp_dir_fixture: TempDir) {
    let mut tag = Tag::new(lofty::tag::TagType::Id3v2);
    tag.insert_text(
        lofty::prelude::ItemKey::Lyrics,
        "Embedded ID3 Lyrics Text".to_string(),
    );
    let dummy_path = temp_dir_fixture.path().join("dummy.mp3");

    let extracted = LyricsScanner::extract_lyrics(&tag, &dummy_path);

    assert_some_eq_x!(extracted.as_deref(), "Embedded ID3 Lyrics Text");
}
