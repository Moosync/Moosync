use js_sys::wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_namespace = ["window", "__MOOSYNC__"])]
extern "C" {
    pub type YTPlayer;

    #[wasm_bindgen(constructor)]
    pub fn new(element: &str) -> YTPlayer;

    #[wasm_bindgen(method)]
    pub fn load(this: &YTPlayer, val: &str, autoplay: bool);

    #[wasm_bindgen(method)]
    pub fn play(this: &YTPlayer);

    #[wasm_bindgen(method)]
    pub fn pause(this: &YTPlayer);

    #[wasm_bindgen(method)]
    pub fn stop(this: &YTPlayer);

    #[wasm_bindgen(method)]
    pub fn seek(this: &YTPlayer, time: f64);

    #[wasm_bindgen(method)]
    pub fn getVolume(this: &YTPlayer) -> f64;

    #[wasm_bindgen(method)]
    pub fn setVolume(this: &YTPlayer, volume: f64);

    #[wasm_bindgen(method)]
    pub fn on(this: &YTPlayer, event: &str, callback: &JsValue);

    #[wasm_bindgen(method)]
    pub fn once(this: &YTPlayer, event: &str, callback: &JsValue);

    #[wasm_bindgen(method)]
    pub fn removeAllListeners(this: &YTPlayer);
}
