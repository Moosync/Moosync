#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod mpris;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use mpris::{MediaControlEvent, MprisHolder};

#[cfg(target_os = "android")]
pub mod mpris_android;

#[cfg(target_os = "android")]
pub use mpris_android::{MediaControlEvent, MprisHolder};
