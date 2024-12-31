#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod playlist_scanner;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod scanner;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use scanner::{ScanState, ScannerHolder};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod song_scanner;
mod types;
mod utils;

#[cfg(target_os = "android")]
mod scanner_android;
#[cfg(target_os = "android")]
pub use scanner_android::{ScanState, ScannerHolder};
