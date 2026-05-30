use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type ReplyHandlerFn = std::sync::Arc<
    dyn Fn(
            &str,
            extensions_proto::moosync::types::MainCommand,
        ) -> Result<
            extensions_proto::moosync::types::MainCommandResponse,
            crate::errors::MoosyncError,
        > + Send
        + Sync,
>;

pub type PlayerResolverFn = std::sync::Arc<
    dyn Fn(
            &songs_proto::moosync::types::Song,
        ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

// Context passed to every plugin during initialization
pub struct PluginContext {
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub tmp_dir: PathBuf,
    #[cfg(target_os = "android")]
    pub android_context: crate::android::AndroidJNIContext,
    pub reply_handler: Option<ReplyHandlerFn>,
    pub themes_changed_tx: Option<std::sync::mpsc::Sender<String>>,
    pub player_resolver: Option<PlayerResolverFn>,
}

// Every core plugin must implement this initialization trait
pub trait Plugin: Send + Sync + 'static {
    fn init(context: &PluginContext) -> Self;
}

// A generic Type-Map registry to store and retrieve plugin instances
// Used by state_manager
#[derive(Default)]
pub struct PluginRegistry {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register<P: Plugin>(&mut self, plugin: P) {
        self.map
            .insert(TypeId::of::<P>(), Arc::new(RwLock::new(plugin)));
    }

    // Panics on failure to retrieve/downcast target plugin
    pub fn get<P: Plugin>(&self) -> Arc<RwLock<P>> {
        self.map
            .get(&TypeId::of::<P>())
            .and_then(|boxed| boxed.clone().downcast::<RwLock<P>>().ok())
            .unwrap_or_else(|| panic!("Plugin not registered: {}", std::any::type_name::<P>()))
    }
}

#[derive(Default)]
pub struct CallContext {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl CallContext {
    // Insert any custom state into the context
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(val));
    }

    // Retrieve that state later
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|b| b.downcast_mut())
    }

    // Remove it to take ownership
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|b| b.downcast::<T>().ok())
            .map(|b| *b)
    }
}
