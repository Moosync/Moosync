use std::fmt::Debug;

mod keyring_context;
pub use keyring_context::KeyringContext;

#[cfg_attr(test, mockall::automock)]
pub trait Keyring: Send + Sync + Debug {
    fn set_secret(&self, secret: &[u8]) -> std::result::Result<(), keyring::Error>;
    fn get_secret(&self) -> std::result::Result<Vec<u8>, keyring::Error>;
}
