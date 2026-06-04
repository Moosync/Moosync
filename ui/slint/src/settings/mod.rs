use slint::ComponentHandle;
use state_manager::StateManager;

use crate::{MainWindow, PreferenceChange, pages::PageHandler};

pub mod extensions;
pub mod paths;
pub mod system;
pub mod themes;

#[cfg(not(target_os = "android"))]
fn select_directory() -> String {
    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        tracing::info!("Selected directory: {:?}", path);
        path.to_string_lossy().to_string()
    } else {
        String::new()
    }
}

#[cfg(not(target_os = "android"))]
fn select_file(filter: String) -> String {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Custom files", &filter.split(",").collect::<Vec<&str>>())
        .pick_file()
    {
        tracing::info!("Selected file: {:?}", path);
        path.to_string_lossy().to_string()
    } else {
        String::new()
    }
}

#[cfg(target_os = "android")]
fn select_directory() -> String { String::new() }

#[cfg(target_os = "android")]
fn select_file(filter: String) -> String { String::new() }

pub fn setup_settings(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    let paths_handler = paths::PathsPageHandler::new(main_window, state_manager);
    paths_handler.initialize();

    let system_handler = system::SystemPageHandler::new(main_window, state_manager);
    system_handler.initialize();

    let extensions_handler = extensions::ExtensionsPageHandler::new(main_window, state_manager);
    extensions_handler.initialize();

    let themes_handler = themes::ThemesPageHandler::new(main_window, state_manager);
    themes_handler.initialize();

    let main_window_weak = main_window.as_weak();
    main_window
        .global::<crate::AppCallbacks>()
        .on_preference_changed(move |change| {
            handle_preference_change(change, &main_window_weak, state_manager);
        });

    main_window
        .global::<crate::AppCallbacks>()
        .on_open_directory_picker(move || select_directory().into());

    main_window
        .global::<crate::AppCallbacks>()
        .on_open_file_picker(move |filter| select_file(filter.into()).into());
}

pub fn handle_preference_change(
    change: PreferenceChange,
    main_window_weak: &slint::Weak<MainWindow>,
    state_manager: &'static StateManager,
) {
    let main_window_weak = main_window_weak.clone();
    tokio::spawn(async move {
        if paths::PathsPageHandler::handle_change(&change, &main_window_weak, state_manager) {
            return;
        }
        if system::SystemPageHandler::handle_change(&change, &main_window_weak, state_manager) {
            return;
        }
    });
}
