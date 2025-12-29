use std::result::Result;

use keyring::Entry;

use super::Keyring;

#[derive(Debug)]
pub struct KeyringContext {
    entry: Entry,
}

impl KeyringContext {
    pub fn new(service: &str, user: &str) -> Result<Self, keyring::Error> {
        let entry = Entry::new(service, user)?;
        Ok(Self { entry })
    }
}

impl Keyring for KeyringContext {
    fn set_secret(&self, secret: &[u8]) -> Result<(), keyring::Error> {
        self.entry.set_secret(secret)
    }

    fn get_secret(&self) -> Result<Vec<u8>, keyring::Error> {
        self.entry.get_secret()
    }
}
