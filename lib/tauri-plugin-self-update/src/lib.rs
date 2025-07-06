use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod models;
pub use models::{PlatformInfo, Release};

#[cfg(desktop)]
use desktop::SelfUpdate;
#[cfg(mobile)]
use mobile::SelfUpdate;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the self-update APIs.
pub trait SelfUpdateExt<R: Runtime> {
    fn self_update(&self) -> &SelfUpdate<R>;
}

impl<R: Runtime, T: Manager<R>> crate::SelfUpdateExt<R> for T {
    fn self_update(&self) -> &SelfUpdate<R> {
        self.state::<SelfUpdate<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("self-update")
        .setup(|_app, _api| {
            #[cfg(mobile)]
            {
                let self_update = mobile::init(_app, _api)?;
                _app.manage(self_update);
            }
            Ok(())
        })
        .build()
}
