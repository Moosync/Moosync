#![allow(unused_variables)]
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

use macros::generate_command;
use tauri::{AppHandle, State};
use types::errors::Result;

#[cfg(mobile)]
use tauri_plugin_audioplayer::AudioplayerExt;

pub struct MobilePlayer {}

impl MobilePlayer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn mobile_load(
        &self,
        app: AppHandle,
        key: String,
        src: String,
        autoplay: bool,
    ) -> Result<()> {
        #[cfg(mobile)]
        {
            let player = app.audioplayer();
            player.load(key, src, autoplay)?;
        }

        Ok(())
    }

    pub fn mobile_play(&self, app: AppHandle, key: String) -> Result<()> {
        #[cfg(mobile)]
        {
            let player = app.audioplayer();
            player.play(key)?;
        }

        Ok(())
    }

    pub fn mobile_pause(&self, app: AppHandle, key: String) -> Result<()> {
        #[cfg(mobile)]
        {
            let player = app.audioplayer();
            player.pause(key)?;
        }
        Ok(())
    }

    pub fn mobile_stop(&self, app: AppHandle, key: String) -> Result<()> {
        #[cfg(mobile)]
        {
            let player = app.audioplayer();
            player.stop(key)?;
        }
        Ok(())
    }

    pub fn mobile_seek(&self, app: AppHandle, key: String, pos: f64) -> Result<()> {
        #[cfg(mobile)]
        {
            let player = app.audioplayer();
            player.seek(key, pos * 1000f64)?;
        }
        Ok(())
    }
}

generate_command!(mobile_load, MobilePlayer, (), app: AppHandle, key: String, src: String, autoplay: bool);
generate_command!(mobile_play, MobilePlayer, (), app: AppHandle, key: String);
generate_command!(mobile_pause, MobilePlayer, (), app: AppHandle, key: String);
generate_command!(mobile_stop, MobilePlayer, (), app: AppHandle, key: String);
generate_command!(mobile_seek, MobilePlayer, (), app: AppHandle, key: String, pos: f64);
