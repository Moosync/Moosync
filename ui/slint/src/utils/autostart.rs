#[cfg(not(target_os = "android"))]
use auto_launch::{AutoLaunch, AutoLaunchBuilder, MacOSLaunchMode};

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_auto_launch() -> Option<AutoLaunch> {
    #[cfg(not(target_os = "android"))]
    {
        let current_exe = match std::env::current_exe() {
            Ok(exe) => exe,
            Err(e) => {
                tracing::error!("Failed to get current executable path: {:?}", e);
                return None;
            }
        };

        let app_path = current_exe.to_string_lossy().to_string();
        match AutoLaunchBuilder::new()
            .set_app_name("Moosync")
            .set_app_path(&app_path)
            .set_macos_launch_mode(MacOSLaunchMode::LaunchAgent)
            .build()
        {
            Ok(auto) => Some(auto),
            Err(e) => {
                tracing::error!("Failed to build AutoLaunch instance: {:?}", e);
                None
            }
        }
    }
    #[cfg(target_os = "android")]
    {
        None
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn update_auto_startup(enabled: bool) {
    #[cfg(not(target_os = "android"))]
    {
        let Some(auto) = get_auto_launch() else {
            return;
        };

        if enabled {
            if let Err(e) = auto.enable() {
                tracing::error!("Failed to enable auto startup: {:?}", e);
            }
        } else if let Err(e) = auto.disable() {
            tracing::error!("Failed to disable auto startup: {:?}", e);
        }
    }
}
