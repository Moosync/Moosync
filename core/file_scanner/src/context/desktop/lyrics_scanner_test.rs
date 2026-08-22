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

use std::{env::temp_dir, fs};

use lofty::tag::Tag;
use uuid::Uuid;

use crate::context::desktop::lyrics_scanner::LyricsScanner;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_scanner_scan_lrc_file_parsing() {
    let test_dir = temp_dir().join(format!("moosync_lrc_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&test_dir).unwrap();

    let audio_path = test_dir.join("sample_song.mp3");
    let lrc_path = test_dir.join("sample_song.lrc");

    let lrc_content = "\
[ti:Sample Title]
[ar:Sample Artist]
[00:05.12]First line of lyrics
[00:10.45]Second line of lyrics
[00:15.99]Third line of lyrics
";
    fs::write(&lrc_path, lrc_content).unwrap();

    let lyrics = LyricsScanner::scan_lrc(audio_path.clone());
    assert!(lyrics.is_some());
    let lyrics_text = lyrics.unwrap();
    assert!(lyrics_text.contains("First line of lyrics"));
    assert!(lyrics_text.contains("Second line of lyrics"));
    assert!(lyrics_text.contains("Third line of lyrics"));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_scanner_scan_lrc_non_existent_returns_none() {
    let non_existent = temp_dir().join("missing_song_123.mp3");
    let lyrics = LyricsScanner::scan_lrc(non_existent);
    assert!(lyrics.is_none());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_scanner_extract_lyrics_from_tag_or_lrc() {
    let mut tag = Tag::new(lofty::tag::TagType::Id3v2);
    tag.insert_text(
        lofty::prelude::ItemKey::Lyrics,
        "Embedded ID3 Lyrics Text".to_string(),
    );

    let dummy_path = temp_dir().join("dummy.mp3");
    let extracted = LyricsScanner::extract_lyrics(&tag, &dummy_path);
    assert_eq!(extracted, Some("Embedded ID3 Lyrics Text".to_string()));
}
