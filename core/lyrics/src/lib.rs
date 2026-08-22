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

use regex::Regex;
use serde_json::Value;

pub mod error;
use crate::error::LyricsError;

#[cfg(test)]
mod lib_test;
#[cfg(test)]
mod lib_test_smoke;

#[derive(Debug, Clone)]
pub struct LyricsFetcher {
    az_base_url: Option<String>,
    genius_base_url: Option<String>,
}

impl Default for LyricsFetcher {
    #[tracing::instrument(level = "debug", skip_all)]
    fn default() -> Self { Self::new() }
}

#[plugin_macro::generate]
impl LyricsFetcher {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> LyricsFetcher {
        LyricsFetcher {
            az_base_url: None,
            genius_base_url: None,
        }
    }

    #[cfg(test)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_urls(az_base_url: String, genius_base_url: String) -> LyricsFetcher {
        LyricsFetcher {
            az_base_url: Some(az_base_url),
            genius_base_url: Some(genius_base_url),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize_title(&self, title: &str) -> String {
        let re1 = Regex::new(r"\((.*?)\)|\[(.*?)\]").unwrap();
        let re2 =
            Regex::new(r"[\u{1F300}-\u{1F3FF}\u{1F400}-\u{1F64F}\u{1F680}-\u{1F6FF}\u{274C}]")
                .unwrap();
        let re3 = Regex::new(r"//([^/]+)").unwrap();

        let result = re1.replace_all(title, "").to_string();

        let result = re2.replace_all(&result, "").to_string();

        let result = re3.replace_all(&result, "").to_string();

        let cleaned = result
            .to_lowercase()
            .replace("official", "")
            .replace("music", "")
            .replace("video", "");

        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_url(&self, base: &str, artists: &[String], title: &str, append_lyrics: bool) -> String {
        let mut parsed_title = self.sanitize_title(title);

        for a in artists {
            parsed_title = parsed_title.replace(a.as_str(), "");
        }

        if append_lyrics {
            parsed_title.push_str(" lyrics");
        }

        format!("{}{} - {}", base, artists.join(", ").as_str(), parsed_title)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_genius_lyrics(
        &self,
        artists: &[String],
        title: &str,
    ) -> Result<String, LyricsError> {
        let base = self
            .genius_base_url
            .as_deref()
            .unwrap_or("https://genius.com/api/search/song?q=");
        let url = self.get_url(base, artists, title, false);

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows; U; Windows NT 5.1; it; rv:1.8.1.11) Gecko/20071127 Firefox/2.0.0.11")
            .build()?;

        let resp = client.get(url).send().await?.text().await?;
        let json: Value = serde_json::from_str(resp.as_str())?;

        // tracing::info!("{}", resp);

        let url = json
            .get("response")
            .and_then(|resp| resp.get("sections"))
            .and_then(|sections| sections.get(0))
            .and_then(|sec| sec.get("hits"))
            .and_then(|hits| hits.get(0))
            .and_then(|hit| hit.get("result"))
            .and_then(|res| res.get("url"))
            .and_then(|url| url.as_str());

        let Some(url) = url else {
            return Ok(String::new());
        };

        let lyrics_resp = client.get(url).send().await?.text().await?;

        let Some(split) = lyrics_resp.split("window.__PRELOADED_STATE__ = ").nth(1) else {
            return Ok(String::new());
        };

        let Some(split) = split.split("');").next() else {
            return Ok(String::new());
        };

        let parsed = split.replace("JSON.parse(", "");
        let Some(split) = parsed.split("\"lyricsData").nth(1) else {
            return Ok(String::new());
        };

        let Some(split) = split.split("html\\\"").nth(1) else {
            return Ok(String::new());
        };

        let Some(split) = split.split("\",").next() else {
            return Ok(String::new());
        };

        let res = split
            .trim_start_matches([':', '\\', '"', ' '])
            .replace("<br>", "\n")
            .replace("\\\\n", "")
            .replace('\\', "");

        // Remove HTML tags using regex
        let re = Regex::new(r#"<([^>]+)>"#).unwrap();
        let data = re.replace_all(&res, "").to_string();

        Ok(data)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_az_lyrics(&self, artists: &[String], title: &str) -> Result<String, LyricsError> {
        let base = self
            .az_base_url
            .as_deref()
            .unwrap_or("https://search.azlyrics.com/suggest.php?q=");
        let url = self.get_url(base, artists, title, false);

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows; U; Windows NT 5.1; it; rv:1.8.1.11) Gecko/20071127 Firefox/2.0.0.11")
            .build()?;

        let request = client.get(url).send().await?;
        let lyrics_resp = request.text().await?;

        let suggestions: Value = serde_json::from_str(&lyrics_resp)?;
        if let Some(suggestions) = suggestions.get("songs")
            && let Some(suggestion) = suggestions.as_array()
            && let Some(suggestion) = suggestion.first()
            && let Some(suggestion) = suggestion.as_object()
            && let Some(suggestion) = suggestion.get("url")
            && let Some(song_url) = suggestion.as_str()
        {
            let lyrics_resp = client.get(song_url).send().await?.text().await?;

            let lyrics = lyrics_resp
                .split("<div class=\"ringtone\">")
                .nth(1)
                .unwrap_or("")
                .split("<div class=\"noprint\"")
                .next()
                .unwrap_or("");

            let index = lyrics.find("-->").unwrap_or(0) + 3;
            let lyrics = &lyrics.get(index..);
            if let Some(lyrics) = lyrics {
                let lyrics = lyrics
                    .split("</div>")
                    .next()
                    .unwrap_or("")
                    .replace("<br>", "\n");
                return Ok(lyrics.to_string());
            }
        }

        // let lyrics_resp = reqwest::get(url).await?.text().await?;

        Ok(String::new())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_lyrics(
        &self,
        _id: String,
        _url: String,
        artists: Vec<String>,
        title: String,
    ) -> Result<String, LyricsError> {
        let az_res = self.get_az_lyrics(&artists, &title).await;
        if let Ok(lyrics) = az_res
            && !lyrics.is_empty()
        {
            return Ok(lyrics);
        }

        self.get_genius_lyrics(&artists, &title).await
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn main() {}

impl types::plugin::Plugin for LyricsFetcher {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(
        _context: &types::plugin::PluginContext,
    ) -> types::plugin::Arc<types::plugin::RwLock<Self>> {
        types::plugin::Arc::new(types::plugin::RwLock::new(LyricsFetcher::new()))
    }
}
