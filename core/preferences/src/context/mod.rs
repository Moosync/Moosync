#[cfg(not(test))]
use keyring::Entry;
#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    pub Entry {
        pub fn new(service: &str, user: &str) -> std::result::Result<Self, keyring::Error>;
        pub fn set_secret(&self, secret: &[u8]) -> std::result::Result<(), keyring::Error>;
        pub fn get_secret(&self) -> std::result::Result<Vec<u8>, keyring::Error>;
    }
}

#[cfg(test)]
use MockEntry as Entry;

#[cfg_attr(test, mockall::automock)]
pub trait KeyringContext: Send + Sync {
    fn set_secret(&self, secret: &[u8]) -> std::result::Result<(), keyring::Error>;
    fn get_secret(&self) -> std::result::Result<Vec<u8>, keyring::Error>;
}

pub struct RealKeyringContext {
    entry: Entry,
}

impl RealKeyringContext {
    pub fn new(service: &str, user: &str) -> std::result::Result<Self, keyring::Error> {
        let entry = Entry::new(service, user)?;
        Ok(Self { entry })
    }
}

impl KeyringContext for RealKeyringContext {
    fn set_secret(&self, secret: &[u8]) -> std::result::Result<(), keyring::Error> {
        self.entry.set_secret(secret)
    }

    fn get_secret(&self) -> std::result::Result<Vec<u8>, keyring::Error> {
        self.entry.get_secret()
    }
}
