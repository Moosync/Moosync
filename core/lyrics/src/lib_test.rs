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

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

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
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/suggest.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"songs": []})))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/search/song"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": {
                "sections": [{
                    "hits": [{
                        "result": {
                            "url": format!("{}/genius_song", mock_server.uri())
                        }
                    }]
                }]
            }
        })))
        .mount(&mock_server)
        .await;

    let html = r#"window.__PRELOADED_STATE__ = JSON.parse('{"lyricsData": {"body": {"html\"": "<p>Fallback Genius Lyrics Content</p>", "other": 1}}}');"#;
    Mock::given(method("GET"))
        .and(path("/genius_song"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(html, "text/html"))
        .mount(&mock_server)
        .await;

    let fetcher = LyricsFetcher::new_with_urls(
        format!("{}/suggest.php?q=", mock_server.uri()),
        format!("{}/api/search/song?q=", mock_server.uri()),
    );

    let res = fetcher
        .get_lyrics(
            "id".to_string(),
            "".to_string(),
            vec!["Artist Name".to_string()],
            "Song Title".to_string(),
        )
        .await;

    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "Fallback Genius Lyrics Content");
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_lyrics_az_success() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/suggest.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "songs": [{"url": format!("{}/az_song", mock_server.uri())}]
        })))
        .mount(&mock_server)
        .await;

    let html = r#"<div class="ringtone"></div><!-- -->Direct AZ Lyrics Content</div><div class="noprint">"#;
    Mock::given(method("GET"))
        .and(path("/az_song"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(html, "text/html"))
        .mount(&mock_server)
        .await;

    let fetcher = LyricsFetcher::new_with_urls(
        format!("{}/suggest.php?q=", mock_server.uri()),
        format!("{}/api/search/song?q=", mock_server.uri()),
    );

    let res = fetcher
        .get_lyrics(
            "id".to_string(),
            "".to_string(),
            vec!["Artist Name".to_string()],
            "Song Title".to_string(),
        )
        .await;

    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "Direct AZ Lyrics Content");
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_lyrics_all_sources_empty() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/suggest.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"songs": []})))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/search/song"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": {"sections": [{"hits": []}]}
        })))
        .mount(&mock_server)
        .await;

    let fetcher = LyricsFetcher::new_with_urls(
        format!("{}/suggest.php?q=", mock_server.uri()),
        format!("{}/api/search/song?q=", mock_server.uri()),
    );

    let res = fetcher
        .get_lyrics(
            "id".to_string(),
            "".to_string(),
            vec!["Artist Name".to_string()],
            "Song Title".to_string(),
        )
        .await;

    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "");
}
