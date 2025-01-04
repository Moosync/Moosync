use macros::generate_command;
use tauri::{AppHandle, State};
use tauri_plugin_audioplayer::AudioplayerExt;
use types::errors::Result;

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
            player.seek(key, pos)?;
        }
        Ok(())
    }
}

generate_command!(mobile_load, MobilePlayer, (), app: AppHandle, key: String, src: String, autoplay: bool);
generate_command!(mobile_play, MobilePlayer, (), app: AppHandle, key: String);
generate_command!(mobile_pause, MobilePlayer, (), app: AppHandle, key: String);
generate_command!(mobile_stop, MobilePlayer, (), app: AppHandle, key: String);
generate_command!(mobile_seek, MobilePlayer, (), app: AppHandle, key: String, pos: f64);
