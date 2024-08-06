use std::rc::Rc;

use js_sys::Function;
use leptos::{spawn_local, SignalSet};
use serde::Deserialize;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use serde_wasm_bindgen::{from_value, to_value};
use types::errors::errors::Result;
use types::window::{DialogFilter, FileResponse};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::BeforeUnloadEvent;

use crate::console_log;
use crate::utils::common::listen_event;

use super::common::{invoke, listen};

#[derive(Serialize)]
struct KeyArgs {
    key: String,
}

#[derive(Serialize)]
struct SetKeyArgs<T: Serialize> {
    key: String,
    value: T,
}

pub fn load_selective<T>(key: String, setter: impl SignalSet<Value = T> + 'static)
where
    T: DeserializeOwned,
{
    spawn_local(async move {
        let res = load_selective_async(key.clone()).await;
        if res.is_err() {
            console_log!("Failed to load preference: {}", key);
            return;
        }
        setter.set(res.unwrap());
    });
}

pub async fn load_selective_async<T>(key: String) -> Result<T>
where
    T: DeserializeOwned,
{
    let args = to_value(&KeyArgs { key: key.clone() }).unwrap();
    let res = invoke("load_selective", args).await?;
    let parsed = serde_wasm_bindgen::from_value(res);

    Ok(parsed?)
}

pub fn save_selective_number(key: String, value: String) {
    let val = value.parse::<f64>().unwrap();
    save_selective(key, val)
}

pub fn save_selective<T>(key: String, value: T)
where
    T: Serialize + 'static,
{
    spawn_local(async move {
        let args = to_value(&SetKeyArgs {
            key: key.clone(),
            value,
        })
        .unwrap();
        invoke("save_selective", args).await;
    });
}

#[cfg(feature = "mock")]
pub fn load_selective_mock(key: &'static str) -> Result<Value> {
    let ret = match key {
        "spotify" => serde_json::from_str("{\"client_id\": \"e2a60dbeffd34cc7b1bd76a84ad6c1b2\", \"client_secret\": \"8922002dadae481ca783a4752de96970\"}").unwrap(),
        _ => Value::Null
    };

    Ok(ret)
}

#[cfg(not(feature = "mock"))]
pub async fn set_secure_async(key: String, value: Value) -> Result<()> {
    let args = to_value(&SetKeyArgs { key, value }).unwrap();
    invoke("set_secure", args).await;
    Ok(())
}

#[cfg(feature = "mock")]
pub async fn set_secure_async(key: &'static str, value: Value) -> Result<()> {
    let local_sotrage = leptos::web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap();
    local_sotrage
        .set(key, serde_json::to_string(&value).unwrap().as_str())
        .unwrap();
    Ok(())
}

pub fn open_file_browser(
    directory: bool,
    multiple: bool,
    filters: Vec<DialogFilter>,
    setter: impl SignalSet<Value = Vec<String>> + 'static,
) {
    #[derive(Serialize)]
    struct FileBrowserArgs {
        directory: bool,
        multiple: bool,
        filters: Vec<DialogFilter>,
    }
    spawn_local(async move {
        let args = to_value(&FileBrowserArgs {
            directory,
            multiple,
            filters,
        })
        .unwrap();

        let res = invoke("open_file_browser", args).await;
        if res.is_err() {
            console_log!("Failed to open file browser");
            return;
        }
        let file_resp: Vec<FileResponse> = from_value(res.unwrap()).unwrap();
        setter.set(file_resp.iter().map(|f| f.path.clone()).collect());
    })
}

pub fn open_file_browser_single(
    directory: bool,
    filters: Vec<DialogFilter>,
    setter: impl SignalSet<Value = String> + 'static,
) {
    #[derive(Serialize)]
    struct FileBrowserArgs {
        directory: bool,
        multiple: bool,
        filters: Vec<DialogFilter>,
    }
    spawn_local(async move {
        let args = to_value(&FileBrowserArgs {
            directory,
            multiple: false,
            filters,
        })
        .unwrap();

        let res = invoke("open_file_browser", args).await;
        if res.is_err() {
            console_log!("Failed to open file browser");
            return;
        }
        let file_resp: Vec<FileResponse> = from_value(res.unwrap()).unwrap();
        setter.set(file_resp.first().unwrap().path.clone());
    })
}

pub fn watch_preferences(cb: fn((String, JsValue))) -> js_sys::Function {
    listen_event("preference-changed", move |data: JsValue| {
        let res = js_sys::Reflect::get(&data, &JsValue::from_str("payload")).unwrap();
        if res.is_array() {
            let key = js_sys::Reflect::get(&res, &JsValue::from_f64(0f64))
                .unwrap()
                .as_string()
                .unwrap();
            let value = js_sys::Reflect::get(&res, &JsValue::from_f64(1f64)).unwrap();
            cb((key, value));
            return;
        }
        console_log!("Received invalid preference change: {:?}", data);
    })
}
