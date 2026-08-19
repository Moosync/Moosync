use std::{result::Result, sync::Mutex};

use keyring::Entry;

use super::Keyring;

#[derive(Debug)]
pub struct KeyringContext {
    entry: Option<Entry>,
    fallback: Mutex<Option<Vec<u8>>>,
}

impl KeyringContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(service: &str, user: &str) -> Result<Self, keyring::Error> {
        let entry = Entry::new(service, user).ok();
        Ok(Self {
            entry,
            fallback: Mutex::new(None),
        })
    }
}

impl Keyring for KeyringContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_secret(&self, secret: &[u8]) -> Result<(), keyring::Error> {
        if let Some(ref entry) = self.entry {
            if entry.set_secret(secret).is_ok() {
                return Ok(());
            }
        }
        let mut guard = self.fallback.lock().unwrap();
        *guard = Some(secret.to_vec());
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_secret(&self) -> Result<Vec<u8>, keyring::Error> {
        if let Some(ref entry) = self.entry {
            if let Ok(sec) = entry.get_secret() {
                return Ok(sec);
            }
        }
        let guard = self.fallback.lock().unwrap();
        if let Some(ref sec) = *guard {
            return Ok(sec.clone());
        }
        Err(keyring::Error::NoEntry)
    }
}
