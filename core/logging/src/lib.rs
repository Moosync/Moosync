use std::{
    env,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use cache::CacheManager;
use plugin_macro::generate;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer, Registry, fmt, layer::SubscriberExt, reload};
#[cfg(not(target_os = "android"))]
use tracing_tracy::TracyLayer;
use types::plugin::{Plugin, PluginContext};

#[cfg(test)]
mod lib_test;

type BaseRegistry = Registry;
type BoxedProfilingLayer = Box<dyn Layer<BaseRegistry> + Send + Sync>;
type ReloadLevelFn = Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>;
type ReloadProfilingFn = Box<dyn Fn(ProfilingMode) -> Result<(), String> + Send + Sync>;

pub enum ProfilingMode {
    Disabled,
    Tracy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracyToggleOutcome {
    pub enabled: bool,
    pub status_message: String,
}

struct TracingReloadHandles {
    reload_level: ReloadLevelFn,
    reload_profiling: ReloadProfilingFn,
    current_level: Mutex<String>,
    tracy_enabled: AtomicBool,
    tracy_status: Mutex<String>,
}

static TRACING_HANDLES: OnceLock<TracingReloadHandles> = OnceLock::new();

#[tracing::instrument(level = "debug", skip_all)]
pub fn default_log_dir() -> PathBuf {
    match platform_dirs::AppDirs::new(Some("moosync"), false) {
        Some(dirs) => dirs.data_dir.join("logs"),
        None => env::temp_dir().join("moosync-logs"),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn build_profiling_layer(mode: ProfilingMode) -> Option<BoxedProfilingLayer> {
    match mode {
        ProfilingMode::Disabled => None,
        ProfilingMode::Tracy => {
            #[cfg(not(target_os = "android"))]
            {
                Some(Box::new(TracyLayer::default()))
            }
            #[cfg(target_os = "android")]
            {
                None
            }
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn toggle_tracy_profiling_with<R>(
    currently_enabled: bool,
    reload_profiling: R,
) -> Result<TracyToggleOutcome, String>
where
    R: FnOnce(ProfilingMode) -> Result<(), String>,
{
    if currently_enabled {
        reload_profiling(ProfilingMode::Disabled)?;
        return Ok(TracyToggleOutcome {
            enabled: false,
            status_message: "Tracy profiling disabled".to_string(),
        });
    }

    reload_profiling(ProfilingMode::Tracy)?;
    Ok(TracyToggleOutcome {
        enabled: true,
        status_message: "Tracy profiling enabled".to_string(),
    })
}

#[derive(Clone)]
pub struct LogController {
    log_dir: PathBuf,
}

#[generate]
impl LogController {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(log_dir: PathBuf) -> Self { Self { log_dir } }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn init_default_tracing() {
        let log_dir = default_log_dir();
        Self::init_tracing_with_dir(&log_dir);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn init_tracing_with_dir(log_dir: &Path) {
        if TRACING_HANDLES.get().is_some() {
            return;
        }

        let initial_level = env::var("MOOSYNC_LOG").unwrap_or_else(|_| "info".to_string());
        let env_filter = EnvFilter::try_new(&initial_level).unwrap_or_else(|err| {
            eprintln!("Invalid MOOSYNC_LOG filter '{initial_level}': {err}");
            EnvFilter::new("info")
        });

        let (filter_layer, filter_handle) = reload::Layer::new(env_filter);
        let (profiling_layer, profiling_handle) = reload::Layer::new(None::<BoxedProfilingLayer>);

        let console_layer = fmt::layer().pretty().with_target(true).with_ansi(true);

        #[cfg(not(target_os = "android"))]
        let subscriber = {
            let cache_manager = CacheManager::new(log_dir.to_path_buf());
            if let Err(err) = cache_manager.ensure_dir(log_dir) {
                eprintln!("Failed to create log dir {}: {err}", log_dir.display());
            }
            let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "moosync");
            let file_layer = fmt::layer()
                .pretty()
                .with_ansi(false)
                .with_target(true)
                .with_writer(file_appender);

            tracing_subscriber::registry()
                .with(profiling_layer.with_filter(EnvFilter::new("debug")))
                .with(console_layer.and_then(file_layer).with_filter(filter_layer))
        };

        #[cfg(target_os = "android")]
        let subscriber = tracing_subscriber::registry()
            .with(profiling_layer.with_filter(EnvFilter::new("debug")))
            .with(console_layer.with_filter(filter_layer));

        let _ = tracing::subscriber::set_global_default(subscriber);

        let handles = TracingReloadHandles {
            reload_level: Box::new(move |level: &str| {
                let new_filter = EnvFilter::try_new(level)
                    .map_err(|err| format!("Invalid log filter '{level}': {err}"))?;
                filter_handle
                    .reload(new_filter)
                    .map_err(|err| format!("Failed to reload log level: {err}"))
            }),
            reload_profiling: Box::new(move |mode: ProfilingMode| {
                let layer = build_profiling_layer(mode);
                profiling_handle
                    .reload(layer)
                    .map_err(|err| format!("Failed to reload profiling layer: {err}"))
            }),
            current_level: Mutex::new(initial_level),
            tracy_enabled: AtomicBool::new(false),
            tracy_status: Mutex::new("Tracy profiling is disabled".to_string()),
        };

        let _ = TRACING_HANDLES.set(handles);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_log_level(&self) -> String {
        match TRACING_HANDLES.get() {
            Some(handles) => match handles.current_level.lock() {
                Ok(guard) => guard.clone(),
                Err(err) => {
                    tracing::error!("Failed to lock current_level: {err}");
                    "info".to_string()
                }
            },
            None => env::var("MOOSYNC_LOG").unwrap_or_else(|_| "info".to_string()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_log_level(&self, level: &str) -> Result<(), String> {
        let trimmed = level.trim();
        if trimmed.is_empty() {
            return Err("Log level cannot be empty".to_string());
        }

        let Some(handles) = TRACING_HANDLES.get() else {
            return Err("Logging controller is not initialized".to_string());
        };

        (handles.reload_level)(trimmed)?;
        match handles.current_level.lock() {
            Ok(mut guard) => {
                *guard = trimmed.to_string();
            }
            Err(err) => {
                tracing::error!("Failed to update current_level state: {err}");
            }
        }
        tracing::info!("Runtime log level changed to '{}'", trimmed);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn is_tracy_enabled(&self) -> bool {
        TRACING_HANDLES
            .get()
            .is_some_and(|h| h.tracy_enabled.load(Ordering::Relaxed))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_tracy_status(&self) -> String {
        match TRACING_HANDLES.get() {
            Some(handles) => match handles.tracy_status.lock() {
                Ok(guard) => guard.clone(),
                Err(err) => {
                    tracing::error!("Failed to lock tracy_status: {err}");
                    String::new()
                }
            },
            None => "Tracy profiling is disabled".to_string(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_log_dir(&self) -> String { self.log_dir.to_string_lossy().to_string() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn build_env_filter_string(
        global_level: &str,
        component_levels: &[(&str, &str)],
        custom_filter: &str,
    ) -> String {
        let mut parts = Vec::new();
        let trimmed_global = global_level.trim();
        if !trimmed_global.is_empty() {
            parts.push(trimmed_global.to_string());
        } else {
            parts.push("info".to_string());
        }

        for (component, level) in component_levels {
            let trimmed_level = level.trim();
            if !trimmed_level.is_empty() && trimmed_level != "inherit" {
                parts.push(format!("{component}={trimmed_level}"));
            }
        }

        let trimmed_custom = custom_filter.trim();
        if !trimmed_custom.is_empty() {
            parts.push(trimmed_custom.to_string());
        }

        parts.join(",")
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_tracy_profiling(&self, enable: bool) -> Result<TracyToggleOutcome, String> {
        if self.is_tracy_enabled() == enable {
            return Ok(TracyToggleOutcome {
                enabled: enable,
                status_message: self.get_tracy_status(),
            });
        }
        self.toggle_tracy_profiling()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn toggle_tracy_profiling(&self) -> Result<TracyToggleOutcome, String> {
        let Some(handles) = TRACING_HANDLES.get() else {
            return Err("Logging controller is not initialized".to_string());
        };

        let currently_enabled = handles.tracy_enabled.load(Ordering::Relaxed);
        let outcome = toggle_tracy_profiling_with(currently_enabled, &handles.reload_profiling)?;

        handles
            .tracy_enabled
            .store(outcome.enabled, Ordering::Relaxed);
        match handles.tracy_status.lock() {
            Ok(mut guard) => {
                *guard = outcome.status_message.clone();
            }
            Err(err) => {
                tracing::error!("Failed to update tracy_status: {err}");
            }
        }
        tracing::info!("Tracy profiling toggled: {}", outcome.status_message);
        Ok(outcome)
    }
}

impl Plugin for LogController {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(context: &PluginContext) -> Arc<Self> {
        let log_dir = context.data_dir.join("logs");
        Self::init_tracing_with_dir(&log_dir);
        Arc::new(LogController::new(log_dir))
    }
}
