mod canvaz;
mod player;
pub mod spirc;
pub mod utils;

use std::sync::{mpsc, Arc, Mutex};

pub use librespot::connect::config::ConnectConfig;
pub use librespot::core::authentication::Credentials;
pub use librespot::core::cache::Cache;
pub use librespot::core::config::DeviceType;
pub use librespot::playback::player::PlayerEvent;

pub use librespot::playback::config::PlayerConfig;
pub use librespot::playback::config::{
    AudioFormat, Bitrate, NormalisationMethod, NormalisationType, VolumeCtrl,
};
pub use librespot::protocol::authentication::AuthenticationType;
use spirc::{ParsedToken, SpircWrapper};
use types::errors::errors::{MoosyncError, Result};

#[derive(Clone)]
pub struct ConfigHolder {
    credentials: Credentials,
    player_config: PlayerConfig,
    connect_config: ConnectConfig,
    cache_config: Cache,
    backend: String,
    volume_ctrl: String,
}

#[macro_export]
macro_rules! generate_methods {
    ($struct_name:ident, $($method_name:ident($($arg:ident: $arg_type:ty),*) -> $return_type:ty),*) => {
        impl $struct_name {
            $(
                pub fn $method_name(&self, $($arg : $arg_type),*) -> Result<$return_type> {
                    println!("Calling method_name $method_name");
                    self.check_initialized()?;
                    let mut instance = self.instance.lock().unwrap();

                    if let Some(instance) = &mut *instance {
                        return instance.$method_name($($arg),*);
                    }
                    Err(MoosyncError::String("Not initialized".to_string()))
                }
            )*
        }
    };
}

#[derive(Default)]
pub struct LibrespotHolder {
    instance: Mutex<Option<SpircWrapper>>,
    config: Mutex<Option<ConfigHolder>>,
}

impl LibrespotHolder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn initialize(
        &self,
        credentials: Credentials,
        player_config: PlayerConfig,
        connect_config: ConnectConfig,
        cache_config: Cache,
        backend: String,
        volume_ctrl: String,
    ) -> Result<()> {
        let mut config = self.config.lock().unwrap();
        *config = Some(ConfigHolder {
            credentials: credentials.clone(),
            player_config: player_config.clone(),
            connect_config: connect_config.clone(),
            cache_config: cache_config.clone(),
            backend: backend.clone(),
            volume_ctrl: volume_ctrl.clone(),
        });

        let instance = SpircWrapper::new(
            credentials,
            player_config,
            connect_config,
            cache_config,
            backend,
            volume_ctrl,
        )?;

        let mut instance_ = self.instance.lock().unwrap();
        *instance_ = Some(instance);

        Ok(())
    }

    fn check_initialized(&self) -> Result<()> {
        if self.instance.lock().unwrap().is_some() {
            return Ok(());
        }
        let config_mutex = self.config.lock().unwrap();
        let config = (*config_mutex).clone();
        drop(config_mutex);
        if let Some(config) = config.as_ref() {
            self.initialize(
                config.credentials.clone(),
                config.player_config.clone(),
                config.connect_config.clone(),
                config.cache_config.clone(),
                config.backend.clone(),
                config.volume_ctrl.clone(),
            )?;
        }

        Ok(())
    }

    pub fn get_events_channel(&self) -> Result<Arc<Mutex<mpsc::Receiver<PlayerEvent>>>> {
        self.check_initialized()?;
        let instance = self.instance.lock().unwrap();

        if let Some(instance) = instance.as_ref() {
            return Ok(instance.events_channel.clone());
        }
        Err(MoosyncError::String("Not initialized".to_string()))
    }
}

generate_methods!(LibrespotHolder, 
    librespot_play() -> (), 
    librespot_pause() -> (),
    librespot_close() -> (), 
    librespot_get_token(scopes: String) -> ParsedToken, 
    librespot_volume(vol: u16) -> (), 
    librespot_load(uri: String, autoplay: bool) -> (), 
    librespot_seek(pos: u32) -> ());
