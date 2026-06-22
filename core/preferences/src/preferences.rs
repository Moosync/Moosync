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
    sync::Mutex,
};

use chacha20poly1305::{
    AeadCore, ChaCha20Poly1305, Key, KeyInit, KeySizeUser,
    aead::{Aead, OsRng, generic_array::GenericArray},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use types::{
    errors::{
        MoosyncError, Result,
        error_helpers::{self, to_file_system_error},
    },
    subscription::SubscriberList,
};
use whoami;

use crate::context::{Keyring, KeyringContext};

pub type OnPreferenceChangedCallback = Box<dyn Fn(String) + Send + Sync + 'static>;

use crate::keys::PreferenceKey;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PreferenceConfigData {
    pub prefs: std::collections::HashMap<String, Value>,
}

pub struct PreferenceConfig {
    pub config_file: Mutex<PathBuf>,
    pub secret: Mutex<Key>,
    pub memcache: std::sync::RwLock<PreferenceConfigData>,
    _keyring_context: Box<dyn Keyring>,
    pub on_preference_changed: SubscriberList<OnPreferenceChangedCallback>,
}

impl std::fmt::Debug for PreferenceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreferenceConfig")
            .field("config_file", &self.config_file)
            .field("secret", &self.secret)
            .field("memcache", &self.memcache)
            .finish()
    }
}

#[plugin_macro::generate]
impl PreferenceConfig {
    #[tracing::instrument(level = "debug", skip(data_dir))]
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let context = KeyringContext::new("moosync", whoami::username().as_str())
            .map_err(error_helpers::to_config_error)?;
        Self::new_with_context(data_dir, Box::new(context))
    }

    pub fn new_with_context(data_dir: PathBuf, context: Box<dyn Keyring>) -> Result<Self> {
        let config_file_path = data_dir.join("config.json");

        if !data_dir.exists() {
            fs::create_dir_all(data_dir).map_err(to_file_system_error)?;
        }

        if !config_file_path.exists() {
            let mut file = File::create(config_file_path.clone()).map_err(to_file_system_error)?;
            file.write_all(b"{\"prefs\": {}}")
                .map_err(to_file_system_error)?;
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let secret = match context.get_secret() {
            Ok(password) => {
                tracing::debug!("Got keystore password");
                Key::from(GenericArray::clone_from_slice(
                    &password[0..ChaCha20Poly1305::key_size()],
                ))
            }
            Err(e) => {
                tracing::warn!(
                    "Error getting keystore password: {:?} (May happen if the app is run for the first time)",
                    e
                );
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                context
                    .set_secret(key.as_slice())
                    .map_err(error_helpers::to_config_error)?;

                match context.get_secret() {
                    Ok(_) => {}
                    Err(_) => panic!("Failed to set secret key"),
                };
                key
            }
        };

        #[cfg(target_os = "android")]
        let secret = ChaCha20Poly1305::generate_key(&mut OsRng);

        let mut config_file = File::open(config_file_path.clone()).map_err(to_file_system_error)?;
        let mut prefs_str = String::new();
        config_file
            .read_to_string(&mut prefs_str)
            .map_err(to_file_system_error)?;

        let prefs: PreferenceConfigData = serde_json::from_str(&prefs_str).unwrap_or_default();

        Ok(PreferenceConfig {
            config_file: Mutex::new(config_file_path),
            secret: Mutex::new(secret),
            memcache: std::sync::RwLock::new(prefs),
            _keyring_context: context,
            on_preference_changed: SubscriberList::new(),
        })
    }

    #[tracing::instrument(level = "debug", skip(self, key))]
    pub fn get_secure<T>(&self, key: String) -> Result<T>
    where
        T: DeserializeOwned,
    {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            use types::errors::error_helpers::to_auth_error;

            let data: String = self.load_selective(key.clone())?;
            let mut split = data.split(':');
            let nonce = split.next().unwrap();
            let nonce =
                GenericArray::clone_from_slice(&hex::decode(nonce).map_err(to_auth_error)?[0..12]);
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
            return self.remove_selective(key);
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

            self.save_selective(key, parsed)?;
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            self.save_selective(key, value.unwrap())?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, key))]
    pub fn has_key(&self, key: &str) -> bool {
        let prefs = self.memcache.read().unwrap();
        prefs.prefs.contains_key(key)
    }
}

impl PreferenceConfig {
    pub fn load<K: PreferenceKey>(&self, key: K) -> Result<K::Value> {
        self.load_selective::<K::Value>(key.key())
    }

    pub fn save<K: PreferenceKey>(&self, key: K, value: K::Value) -> Result<()> {
        self.save_selective::<K::Value>(key.key(), value)
    }

    pub fn remove_key<K: PreferenceKey>(&self, key: K) -> Result<()> {
        self.remove_selective(key.key())
    }
}

impl PreferenceConfig {
    fn load_selective<T>(&self, key: String) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let prefs = self.memcache.read().unwrap();
        if let Some(val) = prefs.prefs.get(&key) {
            let t: T = serde_json::from_value(val.clone())?;
            return Ok(t);
        }
        Err(format!("No value found for {}", key).into())
    }

    fn save_selective<T>(&self, key: String, value: T) -> Result<()>
    where
        T: Serialize,
    {
        let clean_key = key.clone();
        let mut prefs = self.memcache.write().unwrap();
        let json_val = serde_json::to_value(value)?;
        prefs.prefs.insert(key, json_val);
        let writable = prefs.clone();
        drop(prefs);

        let config_file_path = self.config_file.lock().expect("poisoned");
        let mut config_file =
            File::create(config_file_path.as_os_str()).map_err(to_file_system_error)?;
        config_file
            .write_all(&serde_json::to_vec(&writable)?)
            .map_err(to_file_system_error)?;
        config_file.flush().map_err(to_file_system_error)?;

        self.on_preference_changed.run_all(|sub| {
            sub(clean_key.clone());
        });

        Ok(())
    }

    fn remove_selective(&self, key: String) -> Result<()> {
        let clean_key = key.clone();
        let mut prefs = self.memcache.write().unwrap();
        prefs.prefs.remove(&key);
        let writable = prefs.clone();
        drop(prefs);

        let config_file_path = self.config_file.lock().expect("poisoned");
        let mut config_file =
            File::create(config_file_path.as_os_str()).map_err(to_file_system_error)?;
        config_file
            .write_all(&serde_json::to_vec(&writable)?)
            .map_err(to_file_system_error)?;
        config_file.flush().map_err(to_file_system_error)?;

        self.on_preference_changed.run_all(|sub| {
            sub(clean_key.clone());
        });

        Ok(())
    }
}

impl types::plugin::Plugin for PreferenceConfig {
    fn init(
        context: &types::plugin::PluginContext,
    ) -> types::plugin::Arc<types::plugin::RwLock<Self>> {
        types::plugin::Arc::new(types::plugin::RwLock::new(
            PreferenceConfig::new(context.data_dir.clone())
                .expect("Failed to initialize PreferenceConfig"),
        ))
    }
}

types::generate_on_event_impl!(
    PreferenceConfig;
    on_preference_changed, on_preference_changed_immediate, String, ::types::subscription::ToFilterKeys<String>;
);
