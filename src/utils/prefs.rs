use std::rc::Rc;

use leptos::{spawn_local, SignalSet};
use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use types::errors::Result;
use types::themes::ThemeDetails;
use types::window::{DialogFilter, FileResponse};
use wasm_bindgen::JsValue;

use crate::utils::common::listen_event;

use super::common::invoke;

#[derive(Serialize)]
struct KeyArgs {
    key: String,
}

#[derive(Serialize)]
struct SetKeyArgs<T: Serialize> {
    key: String,
    value: T,
}

#[tracing::instrument(level = "trace", skip(key, setter))]
pub fn load_selective<T>(key: String, setter: impl SignalSet<Value = T> + 'static)
where
    T: DeserializeOwned,
{
    spawn_local(async move {
        let res = load_selective_async(key.clone()).await;
        if let Err(e) = res {
            tracing::error!("Failed to load preference: {}: {:?}", key, e);
            return;
        }
        setter.set(res.unwrap());
    });
}

#[tracing::instrument(level = "trace", skip(key))]
pub async fn load_selective_async<T>(key: String) -> Result<T>
where
    T: DeserializeOwned,
{
    let args = to_value(&KeyArgs { key: key.clone() }).unwrap();
    let res = invoke("load_selective", args).await?;
    let parsed = serde_wasm_bindgen::from_value(res);

    Ok(parsed?)
}

#[tracing::instrument(level = "trace", skip(key, value))]
pub fn save_selective_number(key: String, value: String) {
    let val = value.parse::<f64>().unwrap();
    tracing::debug!("Parsed {} as f64", value);
    save_selective(key, val)
}

#[tracing::instrument(level = "trace", skip(key, value))]
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
        let _ = invoke("save_selective", args).await;
    });
}

#[tracing::instrument(level = "trace", skip(key))]
#[cfg(feature = "mock")]
pub fn load_selective_mock(key: &'static str) -> Result<Value> {
    let ret = match key {
        "spotify" => serde_json::from_str("{\"client_id\": \"e2a60dbeffd34cc7b1bd76a84ad6c1b2\", \"client_secret\": \"8922002dadae481ca783a4752de96970\"}").unwrap(),
        _ => Value::Null
    };

    Ok(ret)
}

#[tracing::instrument(level = "trace", skip(directory, multiple, filters, setter))]
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
            tracing::error!("Failed to open file browser");
            return;
        }
        let file_resp: Vec<FileResponse> = from_value(res.unwrap()).unwrap();
        tracing::debug!("Got file response {:?}", file_resp);
        setter.set(file_resp.iter().map(|f| f.path.clone()).collect());
    })
}

#[tracing::instrument(level = "trace", skip(directory, filters, setter))]
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
            tracing::error!("Failed to open file browser");
            return;
        }
        let file_resp: Vec<FileResponse> = from_value(res.unwrap()).unwrap();
        setter.set(file_resp.first().unwrap().path.clone());
    })
}

#[tracing::instrument(level = "trace", skip(cb))]
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
        tracing::error!("Received invalid preference change: {:?}", data);
    })
}

#[tracing::instrument(level = "trace", skip(path, cb))]
pub fn import_theme<T>(path: String, cb: T)
where
    T: Fn() + 'static,
{
    let cb = Rc::new(Box::new(cb));
    #[derive(Serialize)]
    struct ImportThemeArgs {
        path: String,
    }
    spawn_local(async move {
        let args = to_value(&ImportThemeArgs { path }).unwrap();

        let res = invoke("import_theme", args).await;
        if res.is_err() {
            tracing::error!("Failed to import theme");
        }

        let cb = cb.clone();
        cb();
    })
}

#[tracing::instrument(level = "trace", skip(theme, cb))]
pub fn save_theme<T>(theme: ThemeDetails, cb: T)
where
    T: Fn() + 'static,
{
    let cb = Rc::new(Box::new(cb));
    #[derive(Serialize)]
    struct SaveThemeArgs {
        theme: ThemeDetails,
    }
    spawn_local(async move {
        let args = to_value(&SaveThemeArgs { theme }).unwrap();

        let res = invoke("save_theme", args).await;
        if res.is_err() {
            tracing::error!("Failed to save theme");
        }

        let cb = cb.clone();
        cb();
    });
}
