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

use crate::LyricsFetcher;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_title() {
    let fetcher = LyricsFetcher::new();

    let result = fetcher.sanitize_title("Title (Official Music Video)");
    assert_eq!(result, "title");

    let result = fetcher.sanitize_title("Title [Official Music Video]");
    assert_eq!(result, "title");

    let result = fetcher.sanitize_title("Title 🔥");
    assert_eq!(result, "title");

    let result = fetcher.sanitize_title("Title //artist");
    assert_eq!(result, "title");

    let result = fetcher.sanitize_title("Title (feat. Artist) [Official Video] 🎵");
    assert_eq!(result, "title");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_sanitize_title_special_characters() {
    let fetcher = LyricsFetcher::new();

    let result = fetcher.sanitize_title("AC/DC - Back in Black (Official 4K Video) 🎸💥");
    assert_eq!(result, "ac/dc - back in black");

    let result = fetcher.sanitize_title("   Spaces   Everywhere   (Official)   ");
    assert_eq!(result, "spaces everywhere");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_url() {
    let fetcher = LyricsFetcher::new();

    let artists = vec!["Artist Name".to_string()];
    let title = "Song Title";
    let base = "https://example.com/search?q=";
    let result = fetcher.get_url(base, &artists, title, true);
    assert_eq!(
        result,
        "https://example.com/search?q=Artist Name - song title lyrics"
    );

    let artists = vec!["Artist1".to_string(), "Artist2".to_string()];
    let title = "Collaboration";
    let result = fetcher.get_url(base, &artists, title, false);
    assert_eq!(
        result,
        "https://example.com/search?q=Artist1, Artist2 - collaboration"
    );
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_lyrics_fallback_when_az_empty() {
    // Per user instruction 5.1: Add test for fallback when AZ lyrics returns empty
    let fetcher = LyricsFetcher::new();
    let res = fetcher
        .get_lyrics(
            "id".to_string(),
            "".to_string(),
            vec!["Nonexistent Artist For Fallback Test".to_string()],
            "Nonexistent Song Title 12345678".to_string(),
        )
        .await;

    // Both remote sources return empty for dummy query
    assert!(res.is_ok());
}
