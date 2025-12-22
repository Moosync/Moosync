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
