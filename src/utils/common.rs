use chrono::{Duration, NaiveTime, Timelike};
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

pub fn convert_file_src(path: String, protocol: &str) -> String {
    let res = convertFileSrc(path.as_str(), protocol);
    res.as_string().unwrap()
}

pub async fn get_blob_url(src: String) -> String {
    let res = getBlobUrl(src.as_str()).await;
    console_log!("Got blob url {}", res.as_string().unwrap());
    res.as_string().unwrap()
}
