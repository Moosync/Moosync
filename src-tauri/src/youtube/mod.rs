use macros::generate_command_async;
use tauri::State;
use types::entities::SearchResult;
use youtube::{
    types::{ContinuationToken, PlaylistResponse},
    youtube::YoutubeScraper,
};

pub fn get_youtube_scraper_state() -> YoutubeScraper {
    YoutubeScraper::default()
}

generate_command_async!(search_yt, YoutubeScraper, SearchResult, title: String, artists: Vec<String>);
generate_command_async!(get_video_url, YoutubeScraper, String, id: String);
generate_command_async!(get_playlist_content, YoutubeScraper, PlaylistResponse, id: String, continuation: Option<ContinuationToken>);
