// Manually find the generated files since bazel doesn't set vars for slint
// slint::include_modules!();
include!(concat!(env!("OUT_DIR"), "/app.rs"));

use std::fs;

use state_manager::StateManager;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt};

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    run();
}

fn setup_tracing() {
    let layer = fmt::layer().pretty().with_target(true).with_ansi(true);

    #[cfg(not(target_os = "android"))]
    let subscriber = {
        let log_path = platform_dirs::AppDirs::new(Some("moosync"), false)
            .unwrap()
            .data_dir
            .join("logs");
        if !log_path.exists() {
            fs::create_dir_all(log_path.clone()).unwrap();
        }
        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_path, "moosync");
        let log_layer = fmt::layer()
            .pretty()
            .with_ansi(false)
            .with_target(true)
            .with_writer(file_appender);
        tracing_subscriber::registry().with(layer).with(log_layer)
    };

    #[cfg(target_os = "android")]
    let subscriber = {
        tracing_subscriber::registry().with(layer)
    };

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

pub fn run() {
    setup_tracing();

    let state_manager = StateManager::new();
    let main_window = MainWindow::new().unwrap();

    main_window.on_search_clicked(|text| {
        println!("Search query received: {}", text);
    });

    main_window.run().unwrap();
}
