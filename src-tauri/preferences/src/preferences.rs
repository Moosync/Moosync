use std::{
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
use jsonschema::JSONSchema;
use keyring::Entry;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use whoami;

use types::errors::{MoosyncError, Result};

#[derive(Debug)]
pub struct PreferenceConfig {
    pub config_file: Mutex<PathBuf>,
    pub secret: Mutex<Key>,
    pub memcache: Mutex<Value>,
    schema: JSONSchema,
    sender: Sender<(String, Value)>,
    receiver: Receiver<(String, Value)>,
}

impl PreferenceConfig {
    #[tracing::instrument(level = "trace", skip(data_dir))]
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let schema = serde_json::from_str(include_str!("./schema.json")).unwrap();
        let schema = match JSONSchema::options().compile(&schema) {
            Ok(s) => s,
            Err(e) => panic!("{}: {}", e, e.instance_path),
        };

        let config_file_path = data_dir.join("config.json");

        if !data_dir.exists() {
            fs::create_dir_all(data_dir)?;
        }

        if !config_file_path.exists() {
            let mut file = File::create(config_file_path.clone())?;
            file.write_all(b"{\"prefs\": {}}")?;
        }

        let entry = Entry::new("moosync", whoami::username().as_str())?;

        let secret = match entry.get_secret() {
            Ok(password) => {
                tracing::info!("Got keystore password");
                Key::from(GenericArray::clone_from_slice(
                    &password[0..ChaCha20Poly1305::key_size()],
                ))
            }
            Err(e) => {
                tracing::info!("Error getting keystore password: {:?}", e);
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                entry.set_secret(key.as_slice())?;

                let entry = Entry::new("moosync", whoami::username().as_str())?;
                match entry.get_secret() {
                    Ok(_) => {}
                    Err(_) => panic!("Failed to set secret key"),
                };
                key
            }
        };

        let mut config_file = File::open(config_file_path.clone())?;
        let mut prefs = String::new();
        config_file.read_to_string(&mut prefs)?;

        let prefs = serde_json::from_str(&prefs).unwrap_or_default();

        schema.validate(&prefs)?;

        let (sender, receiver) = bounded(1);

        Ok(PreferenceConfig {
            config_file: Mutex::new(config_file_path),
            secret: Mutex::new(secret),
            memcache: Mutex::new(prefs),
            sender,
            receiver,
            schema,
        })
    }

    #[tracing::instrument(level = "trace", skip(self, key))]
    pub fn load_selective<T>(&self, key: String) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let prefs = self.memcache.lock().unwrap();

        let key = format!("prefs.{}", key);
        tracing::info!("Loading selective {}", key);

        let val: Option<T> = prefs.dot_get(key.as_str())?;
        drop(prefs);
        if val.is_none() {
            return Err(format!("No value found for {}", key).into());
        }

        Ok(val.unwrap())
    }

    #[tracing::instrument(level = "trace", skip(self, key, value))]
    pub fn save_selective<T>(&self, key: String, value: Option<T>) -> Result<()>
    where
        T: Serialize + Clone,
    {
        let key = format!("prefs.{}", key);
        tracing::info!("saving {}", key);

        let mut prefs = self.memcache.lock().unwrap();

        if value.is_none() {
            prefs.dot_remove(key.as_str()).unwrap();
        } else {
            let old_value: Option<Value> = prefs.dot_get(key.as_str()).unwrap();

            if let Some(old_value) = old_value {
                if old_value == serde_json::to_value(value.clone()).unwrap() {
                    return Ok(());
                }
            }

            let mut prefs_clone = prefs.clone();
            prefs_clone.dot_set(key.as_str(), value.clone()).unwrap();
            self.schema.validate(&prefs_clone)?;
            prefs.dot_set(key.as_str(), value.clone()).unwrap();
        }

        let writable = prefs.clone();
        drop(prefs);

        let config_file_path = self.config_file.lock().expect("poisoned");
        let mut config_file = File::create(config_file_path.as_os_str())?;
        config_file.write_all(&serde_json::to_vec(&writable)?)?;
        config_file.flush()?;

        let sender = self.sender.clone();
        let parsed = serde_json::to_value(value).unwrap();
        thread::spawn(move || {
            sender.send((key, parsed)).unwrap();
        });
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, key))]
    pub fn load_selective_array<T>(&self, key: String) -> Result<T>
    where
        T: DeserializeOwned,
    {
        tracing::info!("Loading selective array {}", key);
        let mut split: Vec<&str> = key.split('.').collect();
        let child = split.pop().unwrap();
        let parent = split.join(".");

        let preference: Value = self.load_selective(parent.to_string())?;
        if preference.is_array() {
            for item in preference.as_array().unwrap() {
                if let Some(key) = item.get("key") {
                    if key == child {
                        return Ok(serde_json::from_value(item.clone().take())?);
                    }
                }
            }
        }

        Err(MoosyncError::String("Value is not an array".into()))
    }

    #[tracing::instrument(level = "trace", skip(self, key))]
    pub fn get_secure<T>(&self, key: String) -> Result<T>
    where
        T: DeserializeOwned,
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

    #[tracing::instrument(level = "trace", skip(self, key, value))]
    pub fn set_secure<T>(&self, key: String, value: Option<T>) -> Result<()>
    where
        T: Serialize + Clone,
    {
        if value.is_none() {
            return self.save_selective(key, value);
        }

        let value = value.unwrap();

        let secret = self.secret.lock().unwrap();
        let cipher = ChaCha20Poly1305::new(&secret);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let encrypted = cipher
            .encrypt(&nonce, (serde_json::to_string(&value)).unwrap().as_bytes())
            .unwrap();

        let parsed = format!("{}:{}", hex::encode(nonce), hex::encode(encrypted));

        self.save_selective(key, Some(parsed))?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_receiver(&self) -> Receiver<(String, Value)> {
        self.receiver.clone()
    }

    #[tracing::instrument(level = "trace", skip(self, key))]
    pub fn has_key(&self, key: &str) -> bool {
        let prefs = self.memcache.lock().unwrap();
        let val: Option<Value> = prefs.dot_get(format!("prefs.{}", key).as_str()).unwrap();
        val.is_some()
    }
}
