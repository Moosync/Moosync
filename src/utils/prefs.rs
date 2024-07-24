use leptos::{spawn_local, SignalSet};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use serde_wasm_bindgen::{from_value, to_value};
use types::errors::errors::Result;
use types::window::{DialogFilter, FileResponse};

use crate::console_log;

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

pub fn load_selective<T>(key: String, setter: impl SignalSet<Value = T> + 'static)
where
    T: DeserializeOwned,
{
    spawn_local(async move {
        let args = to_value(&KeyArgs { key: key.clone() }).unwrap();
        let res = invoke("load_selective", args).await;
        let parsed = serde_wasm_bindgen::from_value(res);
        if parsed.is_err() {
            console_log!("Failed to parse preference: {}", key);
            return;
        }
        setter.set(parsed.unwrap());
    });
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
        let file_resp: Vec<FileResponse> = from_value(res).unwrap();
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
        let file_resp: Vec<FileResponse> = from_value(res).unwrap();
        setter.set(file_resp.first().unwrap().path.clone());
    })
}
