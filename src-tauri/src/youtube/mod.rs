use database::cache::CacheHolder;
use macros::{generate_command_async, generate_command_async_cached};
use tauri::State;
use types::entities::SearchResult;
use youtube::{
    types::{ContinuationToken, PlaylistResponse},
    youtube::YoutubeScraper,
};

pub fn get_youtube_scraper_state() -> YoutubeScraper {
    YoutubeScraper::default()
}

// pub async fn search_yt_cache(youtube: YoutubeScraper, cache: CacheHolder, title: String, artists: Vec<String>) -> Result<SearchResult> {
//     let res = youtube.search_yt(title, artists).await?;

//     cache.set(format!("search_yt_{*}"), blob, expires)

//     Ok(res)
// }

generate_command_async_cached!(search_yt, YoutubeScraper, SearchResult, title: String, artists: Vec<String>);
generate_command_async_cached!(get_video_url, YoutubeScraper, String, id: String);
generate_command_async_cached!(get_playlist_content, YoutubeScraper, PlaylistResponse, id: String, continuation: Option<ContinuationToken>);
