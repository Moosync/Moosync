use database::cache::CacheHolder;
use macros::generate_command_async_cached;
use tauri::State;
use types::entities::SearchResult;
use youtube::{
    types::{ContinuationToken, PlaylistResponse},
    youtube::YoutubeScraper,
};

pub fn get_youtube_scraper_state() -> YoutubeScraper {
    YoutubeScraper::default()
}
