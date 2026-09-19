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

use std::{
    fs,
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;
use lofty::tag::Tag;
use regex::Regex;
use songs_proto::moosync::types::{LyricLine, Lyrics};

lazy_static! {
    static ref TIMESTAMP_REGEX: Regex =
        Regex::new(r"\[(\d{1,2}):(\d{2})(?:\.(\d{1,3}))?\]").unwrap();
    static ref OFFSET_REGEX: Regex = Regex::new(r"\[offset:\s*([+-]?\d+)\s*\]").unwrap();
}

pub struct LyricsScanner;

impl LyricsScanner {
    #[tracing::instrument(level = "debug", skip_all)]
    fn parse_timestamp_ms(minutes_str: &str, seconds_str: &str, frac_str: Option<&str>) -> i64 {
        let minutes: i64 = minutes_str.parse().unwrap_or(0);
        let seconds: i64 = seconds_str.parse().unwrap_or(0);
        let frac_ms: i64 = match frac_str {
            Some(f) if f.len() == 1 => f.parse::<i64>().unwrap_or(0) * 100,
            Some(f) if f.len() == 2 => f.parse::<i64>().unwrap_or(0) * 10,
            Some(f) if f.len() >= 3 => f[..3].parse::<i64>().unwrap_or(0),
            _ => 0,
        };
        (minutes * 60 + seconds) * 1000 + frac_ms
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn parse_lyrics_content(content: &str) -> Option<Lyrics> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut offset_ms: i64 = 0;
        let mut synced_lines: Vec<LyricLine> = Vec::new();
        let mut raw_lines: Vec<String> = Vec::new();
        let mut has_synced_tag = false;

        for line in content.lines() {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                continue;
            }

            if let Some(captures) = OFFSET_REGEX.captures(line_trimmed) {
                if let Some(m) = captures.get(1) {
                    offset_ms = m.as_str().parse::<i64>().unwrap_or(0);
                }
                continue;
            }

            let timestamps: Vec<i64> = TIMESTAMP_REGEX
                .captures_iter(line_trimmed)
                .map(|cap| {
                    let min = cap.get(1).map_or("0", |m| m.as_str());
                    let sec = cap.get(2).map_or("0", |m| m.as_str());
                    let frac = cap.get(3).map(|m| m.as_str());
                    Self::parse_timestamp_ms(min, sec, frac)
                })
                .collect();

            if !timestamps.is_empty() {
                has_synced_tag = true;
                let text = TIMESTAMP_REGEX
                    .replace_all(line_trimmed, "")
                    .trim()
                    .to_string();
                for ts in timestamps {
                    synced_lines.push(LyricLine {
                        text: text.clone(),
                        time_ms: (ts + offset_ms).max(0),
                    });
                }
            } else if !line_trimmed.starts_with('[') || !line_trimmed.ends_with(']') {
                raw_lines.push(line_trimmed.to_string());
            }
        }

        if has_synced_tag {
            if synced_lines.is_empty() {
                return None;
            }
            synced_lines.sort_by_key(|l| l.time_ms);
            return Some(Lyrics {
                lines: synced_lines,
                is_synced: true,
            });
        }

        if raw_lines.is_empty() {
            return None;
        }

        let lines = raw_lines
            .into_iter()
            .map(|text| LyricLine { text, time_ms: 0 })
            .collect();
        Some(Lyrics {
            lines,
            is_synced: false,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn scan_lrc(mut path: PathBuf) -> Option<Lyrics> {
        path.set_extension("lrc");
        if !path.exists() {
            return None;
        }
        let lyrics = fs::read_to_string(path).ok()?;
        Self::parse_lyrics_content(&lyrics)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn extract_lyrics(metadata: &Tag, path: &Path) -> Option<Lyrics> {
        metadata
            .get_string(&lofty::prelude::ItemKey::Lyrics)
            .and_then(Self::parse_lyrics_content)
            .or_else(|| Self::scan_lrc(path.to_path_buf()))
    }
}
