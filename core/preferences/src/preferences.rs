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
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

#[cfg(not(target_os = "android"))]
use chacha20poly1305::{
    AeadCore, KeySizeUser,
    aead::{Aead, generic_array::GenericArray},
};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, aead::OsRng};
use preferences_proto::moosync::types::{PreferenceItem, PreferenceValue, preference_value};
use prost::Message;
use serde::{Serialize, de::DeserializeOwned};
use types::{
    plugin::{Plugin, PluginContext, RwLock as AsyncRwLock},
    subscription::{SubscriberList, ToFilterKeys},
};
use whoami;

use crate::{
    context::{Keyring, KeyringContext},
    error::PreferencesError,
};

pub type OnPreferenceChangedCallback = Box<dyn Fn(String) + Send + Sync + 'static>;

pub struct PreferenceConfig {
    pub config_file: Mutex<PathBuf>,
    pub secret: Mutex<Key>,
    pub memcache: RwLock<HashMap<String, PreferenceItem>>,
    pub on_preference_changed: SubscriberList<OnPreferenceChangedCallback>,
}

impl fmt::Debug for PreferenceConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreferenceConfig")
            .field("config_file", &self.config_file)
            .field("secret", &self.secret)
            .field("memcache", &self.memcache)
            .finish()
    }
}

#[plugin_macro::generate]
impl PreferenceConfig {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(data_dir: PathBuf) -> Result<Self, PreferencesError> {
        let context = KeyringContext::new("moosync", whoami::username().as_str())
            .map_err(PreferencesError::Keyring)?;
        Self::new_with_context(data_dir, Box::new(context))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_context(
        data_dir: PathBuf,
        context: Box<dyn Keyring>,
    ) -> Result<Self, PreferencesError> {
        let config_file_path = data_dir.join("preferences.bin");

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).map_err(PreferencesError::Io)?;
        }

        #[cfg(not(target_os = "android"))]
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
                    .map_err(PreferencesError::Keyring)?;

                match context.get_secret() {
                    Ok(_) => {}
                    Err(_) => panic!("Failed to set secret key"),
                };
                key
            }
        };

        #[cfg(target_os = "android")]
        let secret = ChaCha20Poly1305::generate_key(&mut OsRng);

        let mut map = HashMap::new();
        if config_file_path.exists() {
            let mut file = File::open(&config_file_path).map_err(PreferencesError::Io)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map_err(PreferencesError::Io)?;
            let mut slice = &buf[..];
            while !slice.is_empty() {
                match PreferenceItem::decode_length_delimited(&mut slice) {
                    Ok(item) => {
                        map.insert(item.id.clone(), item);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to decode preference item: {:?}", e);
                        break;
                    }
                }
            }
        } else {
            let mut file = File::create(&config_file_path).map_err(PreferencesError::Io)?;
            file.write_all(b"").map_err(PreferencesError::Io)?;
        }

        Ok(PreferenceConfig {
            config_file: Mutex::new(config_file_path),
            secret: Mutex::new(secret),
            memcache: RwLock::new(map),
            on_preference_changed: SubscriberList::new(),
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_secure<T>(&self, key: String) -> Result<T, PreferencesError>
    where
        T: DeserializeOwned,
    {
        #[cfg(not(target_os = "android"))]
        {
            let item = self
                .get(&key)
                .ok_or_else(|| PreferencesError::KeyNotFound(key.clone()))?;
            let data = match item.value.and_then(|v| v.value) {
                Some(preference_value::Value::StringValue(s)) => s,
                _ => return Err(PreferencesError::KeyNotFound(key)),
            };

            let mut split = data.split(':');
            let nonce_str = split
                .next()
                .ok_or_else(|| PreferencesError::KeyNotFound(key.clone()))?;
            let nonce = GenericArray::clone_from_slice(
                &hex::decode(nonce_str).map_err(PreferencesError::HexDecode)?[0..12],
            );
            let ciphertext_str = split
                .next()
                .ok_or_else(|| PreferencesError::KeyNotFound(key))?;
            let ciphertext = hex::decode(ciphertext_str).map_err(PreferencesError::HexDecode)?;

            let secret = self.secret.lock().unwrap();
            let cipher = ChaCha20Poly1305::new(&secret);
            let plaintext = String::from_utf8(
                cipher
                    .decrypt(&nonce, ciphertext.as_slice())
                    .map_err(PreferencesError::Decryption)?,
            )?;

            Ok(serde_json::from_str(&plaintext)?)
        }

        #[cfg(target_os = "android")]
        {
            let item = self
                .get(&key)
                .ok_or_else(|| PreferencesError::KeyNotFound(key.clone()))?;
            let data = match item.value.and_then(|v| v.value) {
                Some(preference_value::Value::StringValue(s)) => s,
                _ => return Err(PreferencesError::KeyNotFound(key)),
            };
            Ok(serde_json::from_str(&data)?)
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_secure<T>(&self, key: String, value: Option<T>) -> Result<(), PreferencesError>
    where
        T: Serialize + Clone + Debug,
    {
        if value.is_none() {
            let mut prefs = self.memcache.write().unwrap();
            prefs.remove(&key);
            drop(prefs);
            self.persist()?;
            self.on_preference_changed.run_all(|sub| {
                sub(key.clone());
            });
            return Ok(());
        }

        #[cfg(not(target_os = "android"))]
        {
            let val = value.unwrap();
            let secret = self.secret.lock().unwrap();
            let cipher = ChaCha20Poly1305::new(&secret);
            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let serialized = serde_json::to_string(&val)?;
            let encrypted = cipher
                .encrypt(&nonce, serialized.as_bytes())
                .map_err(PreferencesError::Encryption)?;

            let parsed = format!("{}:{}", hex::encode(nonce), hex::encode(encrypted));
            let item = PreferenceItem {
                id: key.clone(),
                value: Some(PreferenceValue {
                    value: Some(preference_value::Value::StringValue(parsed)),
                }),
                ..Default::default()
            };
            self.save(item)?;
        }

        #[cfg(target_os = "android")]
        {
            let val = value.unwrap();
            let serialized = serde_json::to_string(&val)?;
            let item = PreferenceItem {
                id: key.clone(),
                value: Some(PreferenceValue {
                    value: Some(preference_value::Value::StringValue(serialized)),
                }),
                ..Default::default()
            };
            self.save(item)?;
        }

        Ok(())
    }
}

impl PreferenceConfig {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load(&self, item: &PreferenceItem) -> PreferenceItem {
        let prefs = self.memcache.read().unwrap();
        if let Some(saved) = prefs.get(&item.id) {
            let mut res = item.clone();
            if saved.value.is_some() {
                res.value = saved.value.clone();
            }
            res
        } else {
            item.clone()
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get(&self, id: &str) -> Option<PreferenceItem> {
        let prefs = self.memcache.read().unwrap();
        prefs.get(id).cloned()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn save(&self, pref: PreferenceItem) -> Result<(), PreferencesError> {
        let key = pref.id.clone();
        let mut prefs = self.memcache.write().unwrap();
        prefs.insert(key.clone(), pref);
        drop(prefs);

        self.persist()?;

        self.on_preference_changed.run_all(|sub| {
            sub(key.clone());
        });

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove(&self, item: &PreferenceItem) -> Result<(), PreferencesError> {
        let key = item.id.clone();
        let mut prefs = self.memcache.write().unwrap();
        prefs.remove(&key);
        drop(prefs);

        self.persist()?;

        self.on_preference_changed.run_all(|sub| {
            sub(key.clone());
        });

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn persist(&self) -> Result<(), PreferencesError> {
        let prefs = self.memcache.read().unwrap();
        let mut buf = Vec::new();
        for item in prefs.values() {
            item.encode_length_delimited(&mut buf)
                .map_err(PreferencesError::Encode)?;
        }

        let config_file_path = self.config_file.lock().expect("poisoned");
        let mut config_file =
            File::create(config_file_path.as_os_str()).map_err(PreferencesError::Io)?;
        config_file.write_all(&buf).map_err(PreferencesError::Io)?;
        config_file.flush().map_err(PreferencesError::Io)?;

        Ok(())
    }
}

impl Plugin for PreferenceConfig {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(context: &PluginContext) -> Arc<AsyncRwLock<Self>> {
        Arc::new(AsyncRwLock::new(
            PreferenceConfig::new(context.data_dir.clone())
                .expect("Failed to initialize PreferenceConfig"),
        ))
    }
}

types::generate_on_event_impl!(
    PreferenceConfig;
    on_preference_changed, on_preference_changed_immediate, String, ToFilterKeys<String>;
);
