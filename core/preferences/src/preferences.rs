use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use chacha20poly1305::{
    ChaCha20Poly1305, Key, KeyInit, Nonce,
    aead::{Aead, Generate},
};
use preferences_proto::moosync::types::{PreferenceItem, PreferenceValue, preference_value};
use prost::Message;
use types::{
    plugin::{Plugin, PluginContext},
    subscription::SubscriberList,
};
#[cfg(not(target_os = "android"))]
use whoami;

#[cfg(target_os = "android")]
use crate::context::AndroidKeyringContext;
#[cfg(not(target_os = "android"))]
use crate::context::KeyringContext;
use crate::{context::Keyring, error::PreferencesError};

pub struct PreferenceConfig {
    pub config_file: Mutex<PathBuf>,
    pub secret: Mutex<Key>,
    pub memcache: RwLock<HashMap<String, PreferenceItem>>,
    pub on_preference_changed: SubscriberList<String>,
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
    pub fn new(
        data_dir: PathBuf,
        #[cfg(target_os = "android")] android_context: types::android::AndroidJNIContext,
    ) -> Result<Self, PreferencesError> {
        #[cfg(not(target_os = "android"))]
        let context: Box<dyn Keyring> = {
            let username = whoami::username().unwrap_or_else(|_| "moosync_user".to_string());
            Box::new(KeyringContext::new("moosync", &username).map_err(PreferencesError::Keyring)?)
        };
        #[cfg(target_os = "android")]
        let context: Box<dyn Keyring> = Box::new(AndroidKeyringContext::new(android_context));

        Self::new_with_context(data_dir, context)
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

        let secret = match context.get_secret() {
            Ok(password) => {
                tracing::debug!("Got keystore password");
                Key::try_from(&password[0..32]).unwrap()
            }
            Err(e) => {
                tracing::warn!(
                    "Error getting keystore password: {:?} (May happen if the app is run for the first time)",
                    e
                );
                let key = Key::generate();
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
        T: Message + Default,
    {
        let item = self
            .get(&key)
            .ok_or_else(|| PreferencesError::KeyNotFound(key.clone()))?;
        let data = match item.value.and_then(|v| v.value) {
            Some(preference_value::Value::StringValue(s)) => s,
            _ => return Err(PreferencesError::KeyNotFound(key)),
        };

        if !data.contains(':') {
            let bytes = hex::decode(&data).map_err(PreferencesError::HexDecode)?;
            return Ok(T::decode(&bytes[..])?);
        }

        let mut split = data.split(':');
        let nonce_str = split
            .next()
            .ok_or_else(|| PreferencesError::KeyNotFound(key.clone()))?;
        let nonce =
            Nonce::try_from(&hex::decode(nonce_str).map_err(PreferencesError::HexDecode)?[0..12])
                .unwrap();
        let ciphertext_str = split
            .next()
            .ok_or_else(|| PreferencesError::KeyNotFound(key))?;
        let ciphertext = hex::decode(ciphertext_str).map_err(PreferencesError::HexDecode)?;

        let secret = self.secret.lock().unwrap();
        let cipher = ChaCha20Poly1305::new(&secret);
        let decrypted = cipher
            .decrypt(&nonce, ciphertext.as_slice())
            .map_err(PreferencesError::Decryption)?;

        Ok(T::decode(&decrypted[..])?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_secure<T>(&self, key: String, value: Option<T>) -> Result<(), PreferencesError>
    where
        T: Message + Clone + Debug,
    {
        if value.is_none() {
            let mut prefs = self.memcache.write().unwrap();
            prefs.remove(&key);
            drop(prefs);
            self.persist()?;
            self.on_preference_changed.emit(key.clone());
            return Ok(());
        }

        let val = value.unwrap();
        let secret = self.secret.lock().unwrap();
        let cipher = ChaCha20Poly1305::new(&secret);
        let nonce = Nonce::generate();
        let serialized = val.encode_to_vec();
        let encrypted = cipher
            .encrypt(&nonce, serialized.as_slice())
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

        self.on_preference_changed.emit(key);

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove(&self, item: &PreferenceItem) -> Result<(), PreferencesError> {
        let key = item.id.clone();
        let mut prefs = self.memcache.write().unwrap();
        prefs.remove(&key);
        drop(prefs);

        self.persist()?;

        self.on_preference_changed.emit(key);

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
    fn init(context: &PluginContext) -> Arc<Self> {
        let instance = PreferenceConfig::new(
            context.data_dir.clone(),
            #[cfg(target_os = "android")]
            context.android_context.clone(),
        );
        Arc::new(instance.expect("Failed to initialize PreferenceConfig"))
    }
}
