use std::{
    collections::BTreeMap,
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use extism::Plugin;
use interprocess::local_socket::Stream as LocalSocketStream;
use types::{
    errors::MoosyncError,
    extensions::{ExtensionManifest, MainCommand, MainCommandResponse},
};

use crate::{
    errors::ExtensionError,
    models::{ExtensionCommand, ExtensionCommandResponse},
};

pub use extism_context::ExtismContext;

mod extism_context;

pub type ReplyHandler =
    Arc<Box<dyn Fn(&str, MainCommand) -> Result<MainCommandResponse, MoosyncError> + Sync + Send>>;

struct MainCommandUserData {
    package_name: String,
    reply_handler: ReplyHandler,
}

struct SocketUserData {
    socks: Vec<LocalSocketStream>,
    allowed_paths: Option<BTreeMap<String, PathBuf>>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait Extism: Debug + Send + Sync {
    fn spawn_extension(&self, manifest: &ExtensionManifest) -> Arc<Mutex<Plugin>>;
    async fn execute_command(
        &self,
        package_name: String,
        plugin: Arc<Mutex<Plugin>>,
        command: ExtensionCommand,
    ) -> Result<ExtensionCommandResponse, ExtensionError>;
}
