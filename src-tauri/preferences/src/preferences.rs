use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    sync::Mutex,
    vec,
};

use chacha20poly1305::{
    aead::{generic_array::GenericArray, Aead, OsRng},
    AeadCore, ChaCha20Poly1305, Key, KeyInit, KeySizeUser,
};
use json_dotpath::DotPaths;
use keyring::Entry;

use serde_json::Value;
use whoami;

use types::errors::errors::{MoosyncError, Result};

pub struct PreferenceConfig {
    pub config_file: Mutex<PathBuf>,
    pub secret: Mutex<Key>,
    pub memcache: Mutex<Value>,
}

impl PreferenceConfig {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let config_file_path = data_dir.join("config.json");

        if !data_dir.exists() {
            fs::create_dir_all(data_dir)?;
        }

        if !config_file_path.exists() {
            let mut file = File::create(config_file_path.clone())?;
            file.write_all(b"{}")?;
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

        let mut config_file = File::open(config_file_path.clone())?;
        let mut prefs = String::new();
        config_file.read_to_string(&mut prefs)?;

        Ok(PreferenceConfig {
            config_file: Mutex::new(config_file_path),
            secret: Mutex::new(secret),
            memcache: Mutex::new(serde_json::from_str(&prefs)?),
        })
    }

    pub fn load_selective(&self, key: String) -> Result<Value> {
        let prefs = self.memcache.lock().unwrap();
        let val: Option<Value> = prefs.dot_get(format!("prefs.{}", key).as_str())?;
        println!("Loading selective {}", key);
        drop(prefs);
        if val.is_none() {
            println!("No value found for {}", key);
            return Ok(Value::Null);
        }

        Ok(val.unwrap())
    }

    pub fn save_selective(&self, key: String, value: Value) -> Result<()> {
        let mut prefs = self.memcache.lock().unwrap();
        println!("saving {}", key);
        prefs
            .dot_set(format!("prefs.{}", key).as_str(), value)
            .unwrap();
        let writable = prefs.clone();
        drop(prefs);

        let config_file_path = self.config_file.lock().expect("poisoned");
        let mut config_file = File::create(config_file_path.as_os_str())?;
        config_file.write_all(&serde_json::to_vec(&writable)?)?;
        config_file.flush()?;
        Ok(())
    }

    pub fn load_selective_array(&self, key: String) -> Result<Value> {
        println!("Loading selective array {}", key);
        let mut split: Vec<&str> = key.split('.').collect();
        let child = split.pop().unwrap();
        let parent = split.join(".");

        let preference = self.load_selective(parent.to_string())?;
        if preference.is_array() {
            for item in preference.as_array().unwrap() {
                if let Some(key) = item.get("key") {
                    if key == child {
                        return Ok(item.clone().take());
                    }
                }
            }
        }

        Err(MoosyncError::String("Invalid key".into()))
    }

    pub fn get_secure(&self, key: String) -> Result<Value> {
        let data = self.load_selective(key)?;
        if data.is_null() {
            return Ok(data);
        }
        let data = data.as_str().unwrap().to_string();
        let mut split = data.split(':');
        let nonce = split.next().unwrap();
        let nonce = GenericArray::clone_from_slice(&hex::decode(nonce).unwrap()[0..12]);
        let ciphertext = hex::decode(split.next().unwrap()).unwrap();

        let secret = self.secret.lock().unwrap();
        let cipher = ChaCha20Poly1305::new(&secret);
        let plaintext = String::from_utf8(
            cipher
                .decrypt(&nonce, ciphertext.as_slice())
                .map_err(|e| MoosyncError::String(e.to_string()))?,
        )?;

        Ok(Value::String(plaintext))
    }

    pub fn set_secure(&self, key: String, value: Value) -> Result<()> {
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

    pub fn get_scan_paths(&self) -> Result<Vec<String>> {
        let tmp = self.load_selective("musicPaths".to_string())?;

        let paths = tmp.as_array().unwrap();

        let mut ret = vec![];
        for p in paths {
            let enabled = p.get("enabled").unwrap().as_bool().unwrap();
            if enabled {
                ret.push(p.get("path").unwrap().as_str().unwrap().to_string());
            }
        }

        Ok(ret)
    }
}
