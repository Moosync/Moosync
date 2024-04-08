use leptos::{spawn_local, SignalSet};
use serde::Serialize;
use serde_json::Value;
use serde_wasm_bindgen::{from_value, to_value};
use types::errors::errors::Result;

use super::common::invoke;

#[derive(Serialize)]
struct KeyArgs {
    key: &'static str,
}

#[derive(Serialize)]
struct SetKeyArgs {
    key: &'static str,
    value: Value
}

pub fn load_selective(key: &'static str, setter: impl SignalSet<Value = Value> + 'static) {
    spawn_local(async move {
        let args = to_value(&KeyArgs { key }).unwrap();
        let res = invoke("load_selective", args).await;
        setter.set(from_value(res).unwrap());
    });
}

pub async fn load_selective_async(key: &'static str) -> Result<Value> {
    if cfg!(feature = "mock") {
        #[cfg(feature = "mock")]
        return load_selective_mock(key);
    }

    let args = to_value(&KeyArgs { key })?;
    let res = invoke("load_selective", args).await;
    Ok(from_value(res)?)
}

#[cfg(not(feature = "mock"))]
pub async fn get_secure_async(key: &'static str) -> Result<Value> {
        let args = to_value(&KeyArgs { key }).unwrap();
        let res = invoke("get_secure", args).await;
        Ok(from_value(res)?)
}

#[cfg(feature = "mock")]
pub async fn get_secure_async(key: &'static str) -> Result<Value> {
    let local_sotrage = leptos::web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let res = local_sotrage.get(key).unwrap();
    if let Some(res) = res {
        return Ok(serde_json::from_str(&res)?);
    }
    Ok(Value::Null)
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
pub async fn set_secure_async(key: &'static str, value: Value) -> Result<()> {
    let args = to_value(&SetKeyArgs {key, value}).unwrap();
    invoke("set_secure", args).await;
    Ok(())
} 

#[cfg(feature = "mock")]
pub async fn set_secure_async(key: &'static str, value: Value) -> Result<()> {
    let local_sotrage = leptos::web_sys::window().unwrap().local_storage().unwrap().unwrap();
    local_sotrage.set(key, serde_json::to_string(&value).unwrap().as_str()).unwrap();
    Ok(())
} 
