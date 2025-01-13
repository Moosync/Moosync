use std::rc::Rc;

use leptos::prelude::Owner;
use leptos::{prelude::Set, task::spawn_local};
use serde::{de::DeserializeOwned, Serialize};
use types::themes::ThemeDetails;
use types::window::DialogFilter;
use wasm_bindgen::JsValue;

use crate::utils::common::listen_event;

#[tracing::instrument(level = "trace", skip(key, setter))]
pub fn load_selective<T>(key: String, setter: impl Set<Value = T> + 'static)
where
    T: DeserializeOwned,
{
    spawn_local(async move {
        let res = super::invoke::load_selective(key.clone()).await;
        if let Err(e) = res {
            tracing::error!("Failed to load preference: {}: {:?}", key, e);
            return;
        }
        setter.set(serde_wasm_bindgen::from_value(res.unwrap()).unwrap());
    });
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
        let res = super::invoke::save_selective(key.clone(), Some(value)).await;
        if let Err(e) = res {
            tracing::error!("Error saving selective {}: {:?}", key, e);
        }
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
    setter: impl Set<Value = Vec<String>> + 'static,
) {
    spawn_local(async move {
        let res = super::invoke::open_file_browser(directory, multiple, filters).await;
        if res.is_err() {
            tracing::error!("Failed to open file browser");
            return;
        }
        tracing::debug!("Got file response {:?}", res);
        setter.set(res.unwrap().iter().map(|f| f.path.clone()).collect());
    })
}

#[tracing::instrument(level = "trace", skip(directory, filters, setter))]
pub fn open_file_browser_single(
    directory: bool,
    filters: Vec<DialogFilter>,
    setter: impl Set<Value = String> + 'static,
) {
    spawn_local(async move {
        let file_resp = super::invoke::open_file_browser(directory, false, filters).await;
        if file_resp.is_err() {
            tracing::error!("Failed to open file browser");
            return;
        }
        setter.set(file_resp.unwrap().first().unwrap().path.clone());
    })
}

#[tracing::instrument(level = "trace", skip(cb))]
pub fn watch_preferences(cb: fn((String, JsValue))) -> js_sys::Function {
    let owner = Owner::new();
    listen_event("preference-changed", move |data: JsValue| {
        let res = js_sys::Reflect::get(&data, &JsValue::from_str("payload")).unwrap();
        if res.is_array() {
            let key = js_sys::Reflect::get(&res, &JsValue::from_f64(0f64))
                .unwrap()
                .as_string()
                .unwrap();
            let value = js_sys::Reflect::get(&res, &JsValue::from_f64(1f64)).unwrap();
            owner.with(|| {
                cb((key, value));
            });
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
    spawn_local(async move {
        let res = super::invoke::import_theme(path).await;
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
    spawn_local(async move {
        let res = super::invoke::save_theme(theme).await;
        if res.is_err() {
            tracing::error!("Failed to save theme");
        }

        let cb = cb.clone();
        cb();
    });
}
