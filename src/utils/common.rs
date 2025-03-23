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

use chrono::{Duration, NaiveTime, Timelike};
use types::songs::Song;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    pub fn listen(event: &str, cb: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub fn addPluginListener(plugin: &str, event: &str, cb: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    pub fn emit(event: &str, value: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn convertFileSrc(path: &str, protocol: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__MOOSYNC__"])]
    async fn getBlobUrl(src: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__MOOSYNC__"])]
    fn createYTPlayer(element: &str) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn warn(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn info(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn debug(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn trace(s: &str);
}

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::utils::common::log(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! console_warn {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::utils::common::warn(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! console_error {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::utils::common::error(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! console_info {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::utils::common::info(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! console_debug {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::utils::common::debug(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! console_trace {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::utils::common::trace(&format_args!($($t)*).to_string()))
}

#[tracing::instrument(level = "debug", skip(secs))]
pub fn format_duration(secs: f64, with_suffix: bool) -> String {
    if secs < 0.0 {
        return "Live".into();
    }

    let duration = Duration::seconds(secs as i64);
    let formatted_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap() + duration;

    let (time, suffix) = if formatted_time.hour() == 0 {
        (formatted_time.format("%M:%S").to_string(), "Mins")
    } else {
        (formatted_time.format("%H:%M:%S").to_string(), "Hours")
    };

    if with_suffix {
        format!("{} {}", time, suffix)
    } else {
        time
    }
}

#[tracing::instrument(level = "debug", skip(path))]
pub fn convert_file_src(path: String) -> String {
    if path.is_empty() || path.starts_with("asset:") || path == "favorites" {
        return path;
    }

    if !path.starts_with("http:") && !path.starts_with("https:") {
        let res = convertFileSrc(path.as_str(), "asset");
        return res.as_string().unwrap();
    }
    path
}

#[tracing::instrument(level = "debug", skip(src))]
pub async fn get_blob_url(src: String) -> String {
    let res = getBlobUrl(src.as_str()).await;
    tracing::debug!("Got blob url {}", res.as_string().unwrap());
    res.as_string().unwrap()
}

#[tracing::instrument(level = "debug", skip(song))]
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

#[tracing::instrument(level = "debug", skip(song))]
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
    ($provider:expr, $fetch_content:ident, $update_signal:expr, $next_page_signal:ident, $is_loading:ident, $($arg:expr),*) => {
            'fetch: {
                use types::providers::generic::Pagination;
                use types::common::Unique;
                use leptos::prelude::*;
                use std::sync::Arc;

                let next_page_map = $next_page_signal.get_untracked();
                let mut pagination_lock = next_page_map.get(&$provider).cloned();

                if pagination_lock.is_none() {
                    let new_page = Arc::new(futures::lock::Mutex::new(Pagination::new_limit(50, 0)));
                    let new_page_clone = new_page.clone();
                    let provider = $provider.clone();
                    $next_page_signal.update_untracked(move |signal| {
                        signal.insert(provider, new_page_clone);

                    });
                    pagination_lock = Some(new_page);
                }

                let pagination_lock = pagination_lock.unwrap();
                let mut pagination = pagination_lock.lock().await;
                if !pagination.is_valid {
                    break 'fetch Ok(true);
                }

                let key = $provider.clone();
                $is_loading.update(move |map| {
                    map.insert(key, true);
                });

                let res = crate::utils::invoke::$fetch_content($provider.clone(), $($arg,)* pagination.clone(), false).await;
                if res.is_err() {
                    tracing::error!("Error fetching content {:?}", res);
                    let key = $provider.clone();
                    $is_loading.update(move |map| {
                        map.remove(&key);
                    });
                    break 'fetch Err(res.unwrap_err());
                }

                if let Ok(res) = res {
                    let (mut res, next_page) = res;
                    let len = res.len() as u32;

                    if len == 0 {
                        tracing::debug!("got 0 len content");
                        pagination.invalidate();
                        let key = $provider.clone();
                        $is_loading.update(move |map| {
                            map.remove(&key);
                        });
                        break 'fetch Ok(false);
                    }


                    *pagination = next_page;
                    tracing::debug!("Updating pagination {:?}", pagination);
                        //     let next_page_mut = signal.get_mut(&$provider).unwrap();
                        //     let next_page_mut = next_page.lock().await;
                        //     *next_page_mut = next_page;
                        // signal.insert($provider.clone(), next_page);

                    $update_signal.update(|signal| {
                        signal.append(&mut res);
                        signal.unique();
                        tracing::debug!("new playlists {:?}", signal);
                    });

                    let key = $provider.clone();
                    $is_loading.update(move |map| {
                        map.remove(&key);
                    });

                    break 'fetch Ok(true);
                }

                let key = $provider.clone();
                $is_loading.update(move |map| {
                    map.remove(&key);
                });
                Ok(false)
            }
    };
}

#[tracing::instrument(level = "debug", skip(event, cb))]
pub fn listen_event<F>(event: &str, cb: F) -> js_sys::Function
where
    F: Fn(JsValue) + 'static,
{
    let closure = Closure::wrap(Box::new(move |data: JsValue| {
        cb(data);
    }) as Box<dyn Fn(JsValue)>);
    let res = listen(event, closure.into_js_value());

    let event = event.to_string();
    let data = Box::new(move || {
        let event = event.clone();
        let unlisten = wasm_bindgen_futures::JsFuture::from(res.clone());
        spawn_local(async move {
            let unlisten = unlisten.await.unwrap();
            if unlisten.is_function() {
                let func = js_sys::Function::from(unlisten);
                tracing::debug!("Cleaning up listener for {}", event.clone());
                func.call0(&JsValue::NULL).unwrap();
                tracing::debug!("Cleaned up listener for {}", event.clone());
            }
        });
    }) as Box<dyn FnMut()>;

    let unlisten = Closure::wrap(data);

    js_sys::Function::from(unlisten.into_js_value())
}

#[tracing::instrument(level = "debug", skip(event, cb))]
pub fn listen_plugin_event<F>(plugin: &str, event: &str, cb: F) -> js_sys::Function
where
    F: Fn(JsValue) + 'static,
{
    let closure = Closure::wrap(Box::new(move |data: JsValue| {
        cb(data);
    }) as Box<dyn Fn(JsValue)>);
    let res = addPluginListener(plugin, event, closure.into_js_value());

    let event = event.to_string();
    let data = Box::new(move || {
        let event = event.clone();
        let unlisten = wasm_bindgen_futures::JsFuture::from(res.clone());
        spawn_local(async move {
            let resolved = unlisten.await.unwrap();
            let unlisten =
                js_sys::Reflect::get(&resolved, &JsValue::from_str("unregister")).unwrap();
            if unlisten.is_function() {
                let func = js_sys::Function::from(unlisten);
                tracing::debug!("Cleaning up listener for {}", event.clone());
                func.call0(&JsValue::NULL).unwrap();
                tracing::debug!("Cleaned up listener for {}", event.clone());
            }
        });
    }) as Box<dyn FnMut()>;

    let unlisten = Closure::wrap(data);

    js_sys::Function::from(unlisten.into_js_value())
}

pub fn get_locale(key: &str) -> Locale {
    match key {
        "af_ZA" => Locale::af_ZA,
        "ar_SA" => Locale::ar_SA,
        "ca_ES" => Locale::ca_ES,
        "cs_CZ" => Locale::cs_CZ,
        "da_DK" => Locale::da_DK,
        "de_DE" => Locale::de_DE,
        "el_GR" => Locale::el_GR,
        "es_ES" => Locale::es_ES,
        "fi_FI" => Locale::fi_FI,
        "fr_FR" => Locale::fr_FR,
        "he_IL" => Locale::he_IL,
        "hi_IN" => Locale::hi_IN,
        "hu_HU" => Locale::hu_HU,
        "it_IT" => Locale::it_IT,
        "ja_JP" => Locale::ja_JP,
        "ko_KR" => Locale::ko_KR,
        "nl_NL" => Locale::nl_NL,
        "no_NO" => Locale::no_NO,
        "pl_PL" => Locale::pl_PL,
        "pt_BR" => Locale::pt_BR,
        "pt_PT" => Locale::pt_PT,
        "ro_RO" => Locale::ro_RO,
        "ru_RU" => Locale::ru_RU,
        "sr_SP" => Locale::sr_SP,
        "sv_SE" => Locale::sv_SE,
        "tr_TR" => Locale::tr_TR,
        "uk_UA" => Locale::uk_UA,
        "vi_VN" => Locale::vi_VN,
        "zh_CN" => Locale::zh_CN,
        "zh_TW" => Locale::zh_TW,
        _ => Locale::en_US,
    }
}

pub(crate) use fetch_infinite;
use wasm_bindgen_futures::spawn_local;

use crate::i18n::Locale;
