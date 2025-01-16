// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

// Moosync
// Copyright (C) 2025 Moosync
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

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
