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
use types::canvaz::CanvazResponse;
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

pub static REGISTERED_EVENTS: Mutex<Vec<String>> = Mutex::new(vec![]);

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
        let mut events = REGISTERED_EVENTS.lock().unwrap();
        events.clear();
        drop(events);

        let mut config = self.config.lock().unwrap();
        *config = Some(ConfigHolder {
            credentials: credentials.clone(),
            player_config: player_config.clone(),
            connect_config: connect_config.clone(),
            cache_config: cache_config.clone(),
            backend: backend.clone(),
            volume_ctrl: volume_ctrl.clone(),
        });
        let mut instance_ = self.instance.lock().unwrap();
        if let Some(instance) = &mut *instance_ {
            instance.librespot_close()?;
        }
        drop(instance_);

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
        if let Some(instance) = &mut *self.instance.lock().unwrap() {
            let device_id = instance.get_device_id();
            let device_id = device_id.lock().unwrap();
            if device_id.is_some() {
                return Ok(());
            }
        }

        Err("Librespot not initialized".into())
    }

    pub fn get_events_channel(&self) -> Result<Arc<Mutex<mpsc::Receiver<PlayerEvent>>>> {
        let instance_lock = self.instance.lock().unwrap();

        if let Some(instance) = instance_lock.as_ref() {
            let events_channel = instance.events_channel.clone();
            drop(instance_lock);
            return Ok(events_channel);
        }
        Err(MoosyncError::String("Not initialized".to_string()))
    }

    pub fn register_event(&self, event_name: String) -> Result<()> {
        let mut events = REGISTERED_EVENTS.lock().unwrap();
        events.push(event_name.clone());
        Ok(())
    }
}

generate_methods!(LibrespotHolder,
    librespot_play() -> (),
    librespot_pause() -> (),
    librespot_close() -> (),
    librespot_get_token(scopes: String) -> ParsedToken,
    librespot_volume(vol: u16) -> (),
    librespot_load(uri: String, autoplay: bool) -> (),
    librespot_seek(pos: u32) -> (),
    get_lyrics(uri: String) -> String,
    get_canvaz(uri: String) -> CanvazResponse);
