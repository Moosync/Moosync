use serde::Serialize;
use serde_wasm_bindgen::to_value;

use super::common::invoke;

#[derive(Serialize)]
struct OpenArgs {
    url: String,
}

#[cfg(not(feature = "mock"))]
pub async fn open_external(url: String) {
    let args = to_value(&OpenArgs { url }).unwrap();
    invoke("open_external", args).await;
}

#[cfg(feature = "mock")]
pub async fn open_external(url: String) {
    let window = leptos::web_sys::window().expect("Window object does not exist");
    window
        .open_with_url_and_target(url.as_str(), "_blank")
        .unwrap();
}
