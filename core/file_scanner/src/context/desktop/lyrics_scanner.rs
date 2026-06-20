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

use std::{fs, path::PathBuf};

use lazy_static::lazy_static;
use lofty::tag::Tag;
use regex::Regex;

pub struct LyricsScanner;

impl LyricsScanner {
    #[tracing::instrument(level = "debug", skip(path))]
    pub fn scan_lrc(mut path: PathBuf) -> Option<String> {
        path.set_extension("lrc");
        if path.exists() {
            lazy_static! {
                static ref LRC_REGEX: Regex = Regex::new(r"\[\d{2}:\d{2}.\d{2}\]").unwrap();
            }
            let data = fs::read(path).ok()?;
            let mut parsed_lyrics = "".to_string();
            let parsed = String::from_utf8_lossy(&data).to_string();
            for line in parsed.split('\n') {
                if LRC_REGEX.is_match(line) {
                    parsed_lyrics.push_str(&LRC_REGEX.replace_all(line, ""));
                    parsed_lyrics.push('\n');
                }
            }
            return Some(parsed_lyrics);
        }
        None
    }

    pub fn extract_lyrics(metadata: &Tag, path: &PathBuf) -> Option<String> {
        metadata
            .get_string(&lofty::prelude::ItemKey::Lyrics)
            .map(|s| s.to_string())
            .or_else(|| Self::scan_lrc(path.clone()))
    }
}
