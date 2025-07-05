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

use std::{
    fmt::Debug,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    thread,
};

use std::sync::Mutex;

use crossbeam_channel::{bounded, Receiver, Sender};

use chacha20poly1305::{
    aead::{generic_array::GenericArray, Aead, OsRng},
    AeadCore, ChaCha20Poly1305, Key, KeyInit, KeySizeUser,
};
use json_dotpath::DotPaths;
// use jsonschema::Validator;
use keyring::Entry;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use whoami;

use types::errors::{MoosyncError, Result, error_helpers};

// const SCHEMA: &str = include_str!("./schema.json");



#[derive(Debug)]
pub struct PreferenceConfig {
    pub config_file: Mutex<PathBuf>,
    pub secret: Mutex<Key>,
    pub memcache: Mutex<Value>,
    sender: Sender<(String, Value)>,
    receiver: Receiver<(String, Value)>,
}

impl PreferenceConfig {
    #[tracing::instrument(level = "debug", skip(data_dir))]
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let config_file_path = data_dir.join("config.json");

        if !data_dir.exists() {
            fs::create_dir_all(data_dir)?;
        }

        if !config_file_path.exists() {
            let mut file = File::create(config_file_path.clone())?;
            file.write_all(b"{\"prefs\": {}}")?;
        }

        let entry = Entry::new("moosync", whoami::username().as_str())
            .map_err(error_helpers::to_config_error)?;

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let secret = match entry.get_secret() {
            Ok(password) => {
                tracing::debug!("Got keystore password");
                Key::from(GenericArray::clone_from_slice(
                    &password[0..ChaCha20Poly1305::key_size()],
                ))
            }
            Err(e) => {
                tracing::warn!("Error getting keystore password: {:?} (May happen if the app is run for the first time)", e);
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                entry.set_secret(key.as_slice()).map_err(error_helpers::to_config_error)?;

                let entry = Entry::new("moosync", whoami::username().as_str())
                    .map_err(error_helpers::to_config_error)?;
                match entry.get_secret() {
                    Ok(_) => {}
                    Err(_) => panic!("Failed to set secret key"),
                };
                key
            }
        };

        #[cfg(target_os = "android")]
        let secret = ChaCha20Poly1305::generate_key(&mut OsRng);

        let mut config_file = File::open(config_file_path.clone())?;
        let mut prefs = String::new();
        config_file.read_to_string(&mut prefs)?;

        let prefs = serde_json::from_str(&prefs).unwrap_or_default();

        // let schema = serde_json::from_str(SCHEMA).unwrap();
        // let schema = match jsonschema::validator_for(&schema) {
        //     Ok(s) => s,
        //     Err(e) => panic!("{}: {}", e, e.instance_path),
        // };
        // schema.validate(&prefs)?;

        let (sender, receiver) = bounded(1);

        Ok(PreferenceConfig {
            config_file: Mutex::new(config_file_path),
            secret: Mutex::new(secret),
            memcache: Mutex::new(prefs),
            sender,
            receiver,
        })
    }

    #[tracing::instrument(level = "debug", skip(self, key))]
    pub fn load_selective<T>(&self, key: String) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let prefs = self.memcache.lock().unwrap();

        let key = format!("prefs.{}", key);
        tracing::debug!("Loading selective {}", key);

        let val: Option<T> = prefs.dot_get(key.as_str()).map_err(error_helpers::to_parse_error)?;
        drop(prefs);
        if val.is_none() {
            return Err(format!("No value found for {}", key).into());
        }

        Ok(val.unwrap())
    }

    #[tracing::instrument(level = "debug", skip(self, key, value))]
    pub fn save_selective<T>(&self, key: String, value: Option<T>) -> Result<()>
    where
        T: Serialize + Clone + Debug,
    {
        let key = format!("prefs.{}", key);
        tracing::debug!("saving {} - {:?}", key, value);

        let mut prefs = self.memcache.lock().unwrap();

        if value.is_none() {
            prefs.dot_remove(key.as_str()).unwrap();
        } else {
            let old_value: Option<Value> = prefs.dot_get(key.as_str()).unwrap();

            if let Some(old_value) = old_value {
                if old_value == serde_json::to_value(&value).unwrap() {
                    return Ok(());
                }
            }

            {
                let mut prefs_clone = prefs.clone();
                prefs_clone.dot_set(key.as_str(), &value).unwrap();
                // let schema = serde_json::from_str(SCHEMA).unwrap();
                // let schema = match jsonschema::validator_for(&schema) {
                //     Ok(s) => s,
                //     Err(e) => panic!("{}: {}", e, e.instance_path),
                // };
                // schema.validate(&prefs_clone)?;
            }
            prefs.dot_set(key.as_str(), &value).unwrap();
        }

        let writable = prefs.clone();
        drop(prefs);

        let config_file_path = self.config_file.lock().expect("poisoned");
        let mut config_file = File::create(config_file_path.as_os_str())?;
        config_file.write_all(&serde_json::to_vec(&writable)?)?;
        config_file.flush()?;

        let parsed = serde_json::to_value(value).unwrap();

        let sender = self.sender.clone();
        thread::spawn(move || {
            sender.send((key, parsed)).unwrap();
        });
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, key))]
    pub fn load_selective_array<T>(&self, key: String) -> Result<T>
    where
        T: DeserializeOwned,
    {
        tracing::debug!("Loading selective array {}", key);
        let mut split: Vec<&str> = key.split('.').collect();
        let child = split.pop().unwrap();
        let parent = split.join(".");

        let mut preference: Value = self.load_selective(parent.to_string())?;
        if preference.is_array() {
            for item in preference.as_array_mut().unwrap() {
                if let Some(key) = item.get("key") {
                    if key == child {
                        return Ok(serde_json::from_value((*item).take())?);
                    }
                }
            }
        }

        Err(MoosyncError::String("Value is not an array".into()))
    }

    #[tracing::instrument(level = "debug", skip(self, key))]
    pub fn get_secure<T>(&self, key: String) -> Result<T>
    where
        T: DeserializeOwned,
    {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let data: String = self.load_selective(key.clone())?;
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

            Ok(serde_json::from_str(&plaintext)?)
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            self.load_selective(key.clone())
        }
    }

    #[tracing::instrument(level = "debug", skip(self, key, value))]
    pub fn set_secure<T>(&self, key: String, value: Option<T>) -> Result<()>
    where
        T: Serialize + Clone + Debug,
    {
        if value.is_none() {
            tracing::debug!("Clearing {}", key);
            return self.save_selective(key, value);
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let value = value.unwrap();

            let secret = self.secret.lock().unwrap();
            let cipher = ChaCha20Poly1305::new(&secret);
            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let encrypted = cipher
                .encrypt(&nonce, (serde_json::to_string(&value)).unwrap().as_bytes())
                .unwrap();

            let parsed = format!("{}:{}", hex::encode(nonce), hex::encode(encrypted));

            self.save_selective(key, Some(parsed))?;
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            self.save_selective(key, value)?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_receiver(&self) -> Receiver<(String, Value)> {
        self.receiver.clone()
    }

    #[tracing::instrument(level = "debug", skip(self, key))]
    pub fn has_key(&self, key: &str) -> bool {
        let prefs = self.memcache.lock().unwrap();
        let val: Option<Value> = prefs.dot_get(format!("prefs.{}", key).as_str()).unwrap();
        val.is_some()
    }
}
