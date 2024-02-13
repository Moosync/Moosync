use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    sync::Mutex,
};

use chacha20poly1305::{
    aead::{generic_array::GenericArray, Aead, OsRng},
    AeadCore, ChaCha20Poly1305, ChaChaPoly1305, Key, KeyInit, KeySizeUser,
};
use json_dotpath::DotPaths;
use keyring::Entry;
use preferences::{Preferences, PreferencesMap};
use rand::thread_rng;
use rand::Rng;
use ring::aead::{BoundKey, SealingKey, UnboundKey, AES_256_GCM};
use serde::Serialize;
use serde_json::Value;
use tauri::{App, Error, Manager, State};
use whoami;

use crate::generate_command;

pub struct PreferenceConfig {
    pub config_file: Mutex<PathBuf>,
    pub secret: Mutex<Key>,
}

impl PreferenceConfig {
    pub fn load_selective(&self, key: String) -> Result<Value, Error> {
        let config_file_path = self.config_file.lock().unwrap();
        let mut config_file = File::open(config_file_path.as_os_str())?;
        let mut prefs = String::new();
        config_file.read_to_string(&mut prefs)?;

        let value: Value = serde_json::from_str(&prefs).unwrap();
        let val: Option<Value> = value
            .dot_get(format!("prefs.{}", key).as_str())
            .map_err(|e| Error::AssetNotFound(e.to_string()))?;
        if val.is_none() {
            println!("No value found for {}", key);
            return Ok(Value::Null);
        }

        Ok(val.unwrap())
    }

    pub fn save_selective(&self, key: String, value: Value) -> Result<(), Error> {
        let config_file_path = self.config_file.lock().unwrap();
        let mut config_file = File::open(config_file_path.as_os_str())?;
        let mut prefs = String::new();
        config_file.read_to_string(&mut prefs)?;

        let mut prefs: Value = serde_json::from_str(&prefs)?;
        prefs
            .dot_set(format!("prefs.{}", key).as_str(), value)
            .unwrap();

        let mut config_file = File::create(config_file_path.as_os_str())?;
        config_file.write_all(serde_json::to_string(&prefs)?.as_bytes())?;
        Ok(())
    }

    pub fn get_secure(&self, key: String) -> Result<Value, Error> {
        let data = self.load_selective(key)?;
        if data.is_null() {
            return Ok(data);
        }
        let data = data.as_str().unwrap().to_string();
        let mut split = data.split(":");
        let nonce = split.next().unwrap();
        let nonce = GenericArray::clone_from_slice(&hex::decode(nonce).unwrap()[0..12]);
        let ciphertext = hex::decode(split.next().unwrap()).unwrap();

        let secret = self.secret.lock().unwrap();
        let cipher = ChaCha20Poly1305::new(&secret);
        let plaintext =
            String::from_utf8(cipher.decrypt(&nonce, ciphertext.as_slice()).unwrap()).unwrap();

        Ok(Value::String(plaintext))
    }

    pub fn set_secure(&self, key: String, value: Value) -> Result<(), Error> {
        let secret = self.secret.lock().unwrap();
        let cipher = ChaCha20Poly1305::new(&secret);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let encrypted = cipher
            .encrypt(&nonce, (value.as_str()).unwrap().as_bytes())
            .unwrap();

        let parsed = format!("{}:{}", hex::encode(nonce), hex::encode(encrypted));

        self.save_selective(key, parsed.into())?;

        Ok(())
    }
}

pub fn get_preference_state(app: &mut App) -> Result<PreferenceConfig, Error> {
    let data_dir = app.path().app_config_dir()?;
    let config_file = data_dir.join("config.json");
    println!("{:?}", data_dir);

    if !data_dir.exists() {
        fs::create_dir_all(data_dir)?;
    }

    if !config_file.exists() {
        let mut file = File::create(config_file.clone())?;
        file.write(b"{}")?;
    }

    // TODO: Error handling
    let entry = Entry::new("moosync", whoami::username().as_str()).unwrap();
    let secret = if let Ok(password) = entry.get_password() {
        let decoded = hex::decode(password).unwrap();
        Key::from(GenericArray::clone_from_slice(
            &decoded[0..ChaCha20Poly1305::key_size()],
        ))
    } else {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let encoded = hex::encode(key).to_string();
        entry.set_password(encoded.as_str()).unwrap();
        key
    };

    Ok(PreferenceConfig {
        config_file: Mutex::new(config_file),
        secret: Mutex::new(secret),
    })
}

pub fn initial(state: State<PreferenceConfig>) {
    state
        .save_selective("hotkeys".into(), Value::Array(vec![]))
        .unwrap();
    state
        .save_selective("isFirstLaunch".into(), Value::Bool(false))
        .unwrap();
    state
        .save_selective("youtubeAlt".into(), Value::Array(vec![]))
        .unwrap();
}

generate_command!(load_selective, PreferenceConfig, Value, key: String);
generate_command!(save_selective, PreferenceConfig, (), key: String, value: Value);
generate_command!(get_secure, PreferenceConfig, Value, key: String);
generate_command!(set_secure, PreferenceConfig, (), key: String, value: Value);
