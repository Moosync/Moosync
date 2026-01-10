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

// Test the sanitize_title method
#[test]
fn test_sanitize_title() {
    let lyrics_fetcher = LyricsFetcher::new();

    // Test removing content in parentheses and brackets
    let result = lyrics_fetcher.sanitize_title("Title (Official Music Video)");
    assert_eq!(result, "title ", "Should remove content in parentheses");

    let result = lyrics_fetcher.sanitize_title("Title [Official Music Video]");
    assert_eq!(result, "title ", "Should remove content in brackets");

    // Test removing emojis
    let result = lyrics_fetcher.sanitize_title("Title 🔥");
    assert_eq!(result, "title ", "Should remove emojis");

    // Test removing content after //
    let result = lyrics_fetcher.sanitize_title("Title //artist");
    assert_eq!(result, "title ", "Should remove content after //");

    // Test combined cases
    let result = lyrics_fetcher.sanitize_title("Title (feat. Artist) [Official Video] 🎵");
    assert_eq!(result, "title   ", "Should handle multiple patterns");
}

// Test the get_url method
#[test]
fn test_get_url() {
    let lyrics_fetcher = LyricsFetcher::new();

    // Test with single artist
    let artists = vec!["Artist Name".to_string()];
    let title = "Song Title";
    let base = "https://example.com/search?q=";
    let result = lyrics_fetcher.get_url(base, &artists, title, true);
    assert_eq!(
        result, "https://example.com/search?q=Artist Name - song title lyrics",
        "Should format URL correctly with lyrics suffix"
    );

    // Test with multiple artists
    let artists = vec!["Artist1".to_string(), "Artist2".to_string()];
    let title = "Collaboration";
    let base = "https://example.com/search?q=";
    let result = lyrics_fetcher.get_url(base, &artists, title, false);
    assert_eq!(
        result, "https://example.com/search?q=Artist1, Artist2 - collaboration",
        "Should format URL correctly with multiple artists and no lyrics suffix"
    );

    // Test with special characters
    let artists = vec!["Artist & Co.".to_string()];
    let title = "Song: Subtitle";
    let base = "https://example.com/search?q=";
    let result = lyrics_fetcher.get_url(base, &artists, title, true);
    assert_eq!(
        result, "https://example.com/search?q=Artist & Co. - song: subtitle lyrics",
        "Should handle special characters correctly"
    );
}

// Test for error handling
#[test]
fn test_lyrics_error_handling() {
    let lyrics_fetcher = LyricsFetcher::new();

    // Test with empty artist list
    let artists: Vec<String> = vec![];
    let title = "Song";
    let base = "https://example.com/search?q=";
    let result = lyrics_fetcher.get_url(base, &artists, title, true);

    // Empty artists should result in " - Title lyrics" (lowercase)
    assert_eq!(
        result, "https://example.com/search?q= - song lyrics",
        "Should handle empty artist list gracefully"
    );

    // Test with empty title
    let artists = vec!["Artist".to_string()];
    let title = "";
    let result = lyrics_fetcher.get_url(base, &artists, title, true);

    // Empty title should result in "Artist - lyrics"
    assert_eq!(
        result, "https://example.com/search?q=Artist -  lyrics",
        "Should handle empty title gracefully"
    );
}
