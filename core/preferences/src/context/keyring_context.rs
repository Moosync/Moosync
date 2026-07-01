use std::result::Result;

use keyring::Entry;

use super::Keyring;

#[derive(Debug)]
pub struct KeyringContext {
    entry: Entry,
}

impl KeyringContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(service: &str, user: &str) -> Result<Self, keyring::Error> {
        let entry = Entry::new(service, user)?;
        Ok(Self { entry })
    }
}

impl Keyring for KeyringContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_secret(&self, secret: &[u8]) -> Result<(), keyring::Error> {
        self.entry.set_secret(secret)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_secret(&self) -> Result<Vec<u8>, keyring::Error> { self.entry.get_secret() }
}
