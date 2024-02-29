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
    id: String,
    url: String,
    artists: Vec<String>,
    title: String,
) -> Result<String> {
    lyrics
        .get_lyrics(librespot.inner(), id, url, artists, title)
        .await
}
