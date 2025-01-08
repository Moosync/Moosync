mod canvaz;
mod player;
pub mod spirc;
pub mod utils;

use std::fmt::Debug;
use std::sync::{mpsc, Arc, Mutex};

use env_logger::Env;
use futures::executor::block_on;
pub use librespot::connect::state::ConnectStateConfig;
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
use types::errors::{MoosyncError, Result};

#[derive(Clone)]
pub struct ConfigHolder {
    credentials: Credentials,
    _player_config: PlayerConfig,
    connect_config: ConnectStateConfig,
    _cache_config: Cache,
    backend: String,
    volume_ctrl: String,
}

impl Debug for ConfigHolder {
    #[tracing::instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigHolder")
            .field("credentials", &self.credentials)
            .field("connect_config", &self.connect_config)
            .field("backend", &self.backend)
            .field("volume_ctrl", &self.volume_ctrl)
            .finish()
    }
}

#[macro_export]
macro_rules! generate_methods {
    ($struct_name:ident, $($method_name:ident($($arg:ident: $arg_type:ty),*) -> $return_type:ty),*) => {
        impl $struct_name {
            $(
                #[tracing::instrument(level = "trace", skip(self))]
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

#[derive(Default, Debug)]
pub struct LibrespotHolder {
    instance: Mutex<Option<SpircWrapper>>,
    config: Mutex<Option<ConfigHolder>>,
}

impl LibrespotHolder {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> Self {
        let env = Env::default().filter_or("MOOSYNC_LOG", "error");
        #[cfg(desktop)]
        env_logger::init_from_env(env);
        Self {
            ..Default::default()
        }
    }

    #[tracing::instrument(
        level = "trace",
        skip(
            self,
            credentials,
            player_config,
            connect_config,
            cache_config,
            backend,
            volume_ctrl
        )
    )]
    pub fn initialize(
        &self,
        credentials: Credentials,
        player_config: PlayerConfig,
        connect_config: ConnectStateConfig,
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
            _player_config: player_config.clone(),
            connect_config: connect_config.clone(),
            _cache_config: cache_config.clone(),
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

    #[tracing::instrument(level = "trace", skip(self))]
    fn check_initialized(&self) -> Result<()> {
        if let Some(instance) = &mut *self.instance.lock().unwrap() {
            let device_id = instance.get_device_id();
            let device_id = block_on(device_id.lock());
            if device_id.is_some() {
                return Ok(());
            }
        }

        Err("Librespot not initialized".into())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_events_channel(&self) -> Result<Arc<Mutex<mpsc::Receiver<PlayerEvent>>>> {
        let instance_lock = self.instance.lock().unwrap();

        if let Some(instance) = instance_lock.as_ref() {
            let events_channel = instance.events_channel.clone();
            drop(instance_lock);
            return Ok(events_channel);
        }
        Err(MoosyncError::String("Not initialized".to_string()))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn register_event(&self, event_name: String) -> Result<()> {
        let mut events = REGISTERED_EVENTS.lock().unwrap();
        events.push(event_name.clone());
        Ok(())
    }

    pub fn is_initialized(&self) -> Result<bool> {
        Ok(self.check_initialized().is_ok())
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
