use database::cache::CacheHolder;
use librespot::LibrespotHolder;
use lyrics::LyricsFetcher;
use tauri::State;
use types::errors::errors::Result;

pub fn get_lyrics_state() -> LyricsFetcher {
    LyricsFetcher::new()
}

#[tauri::command()]
pub async fn get_lyrics(
    lyrics: State<'_, LyricsFetcher>,
    librespot: State<'_, LibrespotHolder>,
    cache: State<'_, CacheHolder>,
    id: String,
    url: String,
    artists: Vec<String>,
    title: String,
) -> Result<String> {
    let cache_string = format!("get_lyrics_{}_{}_{:?}_{}", id, url, artists, title);

    let cached = cache.get(&cache_string);
    if cached.is_ok() {
        return cached;
    }

    let res = lyrics
        .get_lyrics(librespot.inner(), id, url, artists, title)
        .await?;

    let _ = cache.set(cache_string.as_str(), &res, 7200);
    Ok(res)
}
