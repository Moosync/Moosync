pub mod cache;
#[cfg(feature = "core")]
pub mod cache_schema;
pub mod canvaz;
pub mod common;
pub mod entities;
pub mod errors;

#[cfg(not(feature = "extensions"))]
pub mod mpris;
pub mod preferences;
#[cfg(feature = "core")]
pub mod schema;
pub mod songs;

pub mod providers;

#[cfg(not(feature = "extensions"))]
pub mod ui;

#[cfg(feature = "core")]
pub mod oauth;

pub mod extensions;

#[cfg(not(feature = "extensions"))]
pub mod themes;

#[cfg(not(feature = "extensions"))]
pub mod window;
