use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[cfg(mobile)]
mod mobile;

#[cfg(desktop)]
mod desktop;

#[cfg(mobile)]
use mobile::FileScanner;

#[cfg(desktop)]
use desktop::FileScanner;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the file-scanner APIs.
pub trait FileScannerExt<R: Runtime> {
    fn file_scanner(&self) -> &FileScanner<R>;
}

impl<R: Runtime, T: Manager<R>> crate::FileScannerExt<R> for T {
    fn file_scanner(&self) -> &FileScanner<R> {
        self.state::<FileScanner<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("file-scanner")
        .setup(|app, api| {
            #[cfg(mobile)]
            {
                let file_scanner = mobile::init(app, api)?;
                app.manage(file_scanner);
            }
            Ok(())
        })
        .build()
}
