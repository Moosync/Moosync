use database::types::entities::SearchResult;
use macros::generate_command_async;
use tauri::State;
use youtube::youtube::YoutubeScraper;

pub fn get_youtube_scraper_state() -> YoutubeScraper {
    YoutubeScraper::default()
}

generate_command_async!(search_yt, YoutubeScraper, SearchResult, title: String, artists: Vec<String>);
generate_command_async!(get_video_url, YoutubeScraper, String, id: String);
