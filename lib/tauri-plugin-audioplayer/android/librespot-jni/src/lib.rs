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

mod java_glue;
use std::{error::Error, sync::Arc, thread, u16};

pub use crate::java_glue::*;
use android_logger::{AndroidLogger, Config};
use lazy_static::lazy_static;
use librespot::{
    utils::event_to_map, Cache, ConnectStateConfig, Credentials, DeviceType, LibrespotHolder,
    PlayerConfig, PlayerEvent,
};
use log::Level;
use rifgen::rifgen_attr::*;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

use jni::{
    signature::ReturnType,
    sys::{jint, jsize, JavaVM},
};
use std::{ffi::c_void, ptr::null_mut};

pub type JniGetCreatedJavaVms =
    unsafe extern "system" fn(vmBuf: *mut *mut JavaVM, bufLen: jsize, nVMs: *mut jsize) -> jint;
pub const JNI_GET_JAVA_VMS_NAME: &[u8] = b"JNI_GetCreatedJavaVMs";

#[generate_interface]
pub trait LibrespotCallbacks {
    fn on_play(&self);
    fn on_stop(&self);
    fn on_pause(&self);
    fn on_time_change(&self, pos: u64);
    fn on_ended(&self);
    fn on_seek(&self, pos: u64);
    fn on_connected(&self);
}

struct BoxedCallbacks {
    callbacks: Box<dyn LibrespotCallbacks>,
}

unsafe impl Send for BoxedCallbacks {}
unsafe impl Sync for BoxedCallbacks {}

pub struct LibrespotWrapper {
    holder: LibrespotHolder,
    callbacks: Arc<BoxedCallbacks>,
}

impl LibrespotWrapper {
    //set up logging
    #[generate_interface(constructor)]
    pub fn new(callbacks: Box<dyn LibrespotCallbacks>) -> LibrespotWrapper {
        #[cfg(target_os = "android")]
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("Hello"),
        );
        log_panics::init(); // log panics rather than printing them
        log::info!("Logging initialised from Rust");

        let filter = EnvFilter::try_new("debug").unwrap();
        let layer = fmt::layer().pretty().with_target(true).with_ansi(false);
        let subscriber = tracing_subscriber::registry().with(layer).with(filter);

        tracing::subscriber::set_global_default(subscriber).unwrap();

        LibrespotWrapper {
            holder: LibrespotHolder::new(),
            callbacks: Arc::new(BoxedCallbacks { callbacks }),
        }
    }

    #[generate_interface]
    pub fn load(&self, src: String, autoplay: bool) {
        log::info!("Loading {}", src);
        if let Err(e) = self.holder.librespot_load(src, true) {
            log::error!("Failed to load audio in librespot {:?}", e);
        }
    }

    #[generate_interface]
    pub fn play(&self) {
        log::info!("Playing");
        if let Err(e) = self.holder.librespot_play() {
            log::error!("Failed to play audio in librespot {:?}", e);
        }
    }

    #[generate_interface]
    pub fn pause(&self) {
        self.holder.librespot_pause();
    }

    #[generate_interface]
    pub fn initialize_librespot(
        &self,
        credentials_path: String,
        audio_path: String,
        access_token: String,
    ) {
        let credentials = Credentials::with_access_token(access_token);

        let player_config = PlayerConfig::default();

        let connect_config = ConnectStateConfig {
            name: "Moosync".into(),
            device_type: DeviceType::Computer,
            initial_volume: u16::MAX as u32,
            is_group: false,
            ..Default::default()
        };

        let cache_config = Cache::new(
            Some(credentials_path.clone()),
            Some(credentials_path),
            Some(audio_path),
            None,
        )
        .unwrap();

        if let Err(e) = self.holder.initialize(
            credentials,
            player_config,
            connect_config,
            cache_config,
            "".to_string(),
            "".to_string(),
        ) {
            log::error!("Failed to initialize librespot {:?}", e);
        }

        let events_channel = self.holder.get_events_channel().unwrap();
        let callbacks = self.callbacks.clone();
        thread::spawn(move || {
            log::info!("In event thread");
            let events_channel = events_channel.lock().unwrap();
            loop {
                log::info!("looping");
                let event = events_channel.recv();
                log::info!("Got event {:?}", event);
                match event {
                    Ok(event) => match event {
                        PlayerEvent::PlayRequestIdChanged { play_request_id } => {}
                        PlayerEvent::Stopped {
                            play_request_id,
                            track_id,
                        } => {
                            callbacks.callbacks.on_stop();
                        }
                        PlayerEvent::Loading {
                            play_request_id,
                            track_id,
                            position_ms,
                        } => {}
                        PlayerEvent::Preloading { track_id } => {}
                        PlayerEvent::Playing {
                            play_request_id,
                            track_id,
                            position_ms,
                        } => {
                            callbacks.callbacks.on_play();
                        }
                        PlayerEvent::Paused {
                            play_request_id,
                            track_id,
                            position_ms,
                        } => {
                            callbacks.callbacks.on_pause();
                        }
                        PlayerEvent::TimeToPreloadNextTrack {
                            play_request_id,
                            track_id,
                        } => {}
                        PlayerEvent::EndOfTrack {
                            play_request_id,
                            track_id,
                        } => {
                            callbacks.callbacks.on_ended();
                        }
                        PlayerEvent::Unavailable {
                            play_request_id,
                            track_id,
                        } => {
                            callbacks.callbacks.on_ended();
                        }
                        PlayerEvent::VolumeChanged { volume } => {}
                        PlayerEvent::PositionCorrection {
                            play_request_id,
                            track_id,
                            position_ms,
                        } => {}
                        PlayerEvent::Seeked {
                            play_request_id,
                            track_id,
                            position_ms,
                        } => {
                            callbacks.callbacks.on_seek(position_ms as u64);
                        }
                        PlayerEvent::TrackChanged { audio_item } => {}
                        PlayerEvent::SessionConnected {
                            connection_id,
                            user_name,
                        } => {
                            callbacks.callbacks.on_connected();
                        }
                        PlayerEvent::SessionDisconnected {
                            connection_id,
                            user_name,
                        } => {}
                        PlayerEvent::SessionClientChanged {
                            client_id,
                            client_name,
                            client_brand_name,
                            client_model_name,
                        } => {}
                        PlayerEvent::ShuffleChanged { shuffle } => {}
                        PlayerEvent::RepeatChanged { context, track } => {}
                        PlayerEvent::AutoPlayChanged { auto_play } => {}
                        PlayerEvent::FilterExplicitContentChanged { filter } => {}
                    },
                    Err(e) => {
                        log::error!("Closing events channel {:?}", e);
                        break;
                    }
                }
            }
        });
    }

    #[generate_interface]
    fn initialize_android_context() {
        unsafe {
            let lib = libloading::os::unix::Library::this();
            let get_created_java_vms: JniGetCreatedJavaVms =
                unsafe { *lib.get(JNI_GET_JAVA_VMS_NAME).unwrap() };
            let mut created_java_vms: [*mut JavaVM; 1] = [null_mut() as *mut JavaVM];
            let mut java_vms_count: i32 = 0;
            unsafe {
                get_created_java_vms(created_java_vms.as_mut_ptr(), 1, &mut java_vms_count);
            }
            let jvm_ptr = *created_java_vms.first().unwrap();
            let jvm = unsafe { jni::JavaVM::from_raw(jvm_ptr) }.unwrap();
            let mut env = jvm.get_env().unwrap();

            let activity_thread = env.find_class("android/app/ActivityThread").unwrap();
            let current_activity_thread = env
                .get_static_method_id(
                    &activity_thread,
                    "currentActivityThread",
                    "()Landroid/app/ActivityThread;",
                )
                .unwrap();
            let at = env
                .call_static_method_unchecked(
                    &activity_thread,
                    current_activity_thread,
                    ReturnType::Object,
                    &[],
                )
                .unwrap();

            let get_application = env
                .get_method_id(
                    activity_thread,
                    "getApplication",
                    "()Landroid/app/Application;",
                )
                .unwrap();
            let context = env
                .call_method_unchecked(at.l().unwrap(), get_application, ReturnType::Object, &[])
                .unwrap();

            ndk_context::initialize_android_context(
                jvm.get_java_vm_pointer() as *mut c_void,
                context.l().unwrap().to_owned() as *mut c_void,
            );
        }
    }
}
