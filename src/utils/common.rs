use chrono::{Duration, NaiveTime, Timelike};
use types::songs::Song;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn convertFileSrc(path: &str, protocol: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__MOOSYNC__"])]
    async fn getBlobUrl(src: &str) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::utils::common::log(&format_args!($($t)*).to_string()))
}

pub fn format_duration(secs: f64) -> String {
    if secs < 0.0 {
        return "Live".into();
    }

    let duration = Duration::seconds(secs as i64);
    let formatted_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap() + duration;
    if formatted_time.hour() == 0 {
        formatted_time.format("%M:%S").to_string()
    } else {
        formatted_time.format("%H:%M:%S").to_string()
    }
}

pub fn convert_file_src(path: String) -> String {
    if path.is_empty() {
        return path;
    }

    if !path.starts_with("http:") && !path.starts_with("https:") {
        let res = convertFileSrc(path.as_str(), "asset");
        return res.as_string().unwrap();
    }
    path
}

pub async fn get_blob_url(src: String) -> String {
    let res = getBlobUrl(src.as_str()).await;
    console_log!("Got blob url {}", res.as_string().unwrap());
    res.as_string().unwrap()
}

pub fn get_low_img(song: &Song) -> String {
    if let Some(cover) = &song.song.song_cover_path_low {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = song
        .album
        .as_ref()
        .and_then(|a| a.album_coverpath_low.clone())
    {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = &song.song.song_cover_path_high {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = song
        .album
        .as_ref()
        .and_then(|a| a.album_coverpath_high.clone())
    {
        return convert_file_src(cover.to_string());
    }

    String::new()
}

pub fn get_high_img(song: &Song) -> String {
    if let Some(cover) = &song.song.song_cover_path_high {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = song
        .album
        .as_ref()
        .and_then(|a| a.album_coverpath_high.clone())
    {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = &song.song.song_cover_path_low {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = song
        .album
        .as_ref()
        .and_then(|a| a.album_coverpath_low.clone())
    {
        return convert_file_src(cover.to_string());
    }

    String::new()
}

macro_rules! fetch_infinite {
    ($provider:expr, $fetch_content:ident, $update_signal:expr, $($arg:expr),*) => {
        let provider = $provider.clone();
        spawn_local(async move {
            let mut offset = 0;
            let provider = provider.lock().unwrap();
            loop {
                let res = provider.$fetch_content($($arg,)* 50, offset).await;
                if res.is_err() {
                    break;
                }

                let mut res = res.unwrap();
                let len = res.len() as u32;

                if len == 0 {
                    break;
                }

                offset += len;

                $update_signal.update(|signal| {
                    signal.append(&mut res);
                });
            }
        });
    };
}

pub(crate) use fetch_infinite;
