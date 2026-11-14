pub mod error;
pub mod keys;
pub mod preferences;

pub use keys::PreferenceItemExt;
pub use preferences::PreferenceConfig;

#[cfg(test)]
mod keys_test;
#[cfg(test)]
mod lib_test_smoke;
#[cfg(test)]
mod preferences_test;

pub mod context;
