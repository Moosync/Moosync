use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

#[cfg(desktop)]
use desktop::Audioplayer;
#[cfg(mobile)]
use mobile::Audioplayer;

#[cfg(mobile)]
pub use mobile::{PermissionResponse, RequestPermission};

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the audioplayer APIs.
pub trait AudioplayerExt<R: Runtime> {
    fn audioplayer(&self) -> &Audioplayer<R>;
}

impl<R: Runtime, T: Manager<R>> crate::AudioplayerExt<R> for T {
    fn audioplayer(&self) -> &Audioplayer<R> {
        self.state::<Audioplayer<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("audioplayer")
        .setup(|app, api| {
            #[cfg(mobile)]
            let audioplayer = mobile::init(app, api)?;
            #[cfg(desktop)]
            let audioplayer = desktop::init(app, api)?;
            app.manage(audioplayer);
            Ok(())
        })
        .build()
}
