pub mod android;
#[cfg(test)]
mod android_test;
mod keyring_context;
#[cfg(test)]
mod keyring_context_test;
#[cfg(test)]
mod mod_test;
pub use android::AndroidKeyringContext;
pub use keyring_context::KeyringContext;

#[cfg_attr(test, mockall::automock)]
pub trait Keyring: Send + Sync {
    fn set_secret(&self, secret: &[u8]) -> std::result::Result<(), keyring::Error>;
    fn get_secret(&self) -> std::result::Result<Vec<u8>, keyring::Error>;
}
