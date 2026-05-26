// Manually find the generated files since bazel doesn't set vars for slint
// slint::include_modules!();
include!(concat!(env!("OUT_DIR"), "/app.rs"));

use std::sync::Arc;

use state_manager::StateManager;
use tracing_subscriber::{fmt, layer::SubscriberExt};

#[cfg(target_os = "android")]
static ANDROID_APP: std::sync::OnceLock<slint::android::AndroidApp> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app.clone()).unwrap();
    ANDROID_APP.set(app).expect("failed to set ANDROID_APP");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    run();
}

pub fn run() {
    setup_tracing();

    #[cfg(not(target_os = "android"))]
    let _state_manager = StateManager::new().expect("StateManager::new failed");

    #[cfg(target_os = "android")]
    let _state_manager = {
        let app = ANDROID_APP.get().expect("ANDROID_APP not initialized");

        // Safety: vm_as_ptr() returns the raw *mut JavaVM for this process.
        let vm = Arc::new(
            unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) }
                .expect("failed to get JavaVM"),
        );
        let (activity, service_class) = {
            let mut env = vm.attach_current_thread().expect("JNI attach");

            let act_ptr = app.activity_as_ptr() as jni::sys::jobject;
            let act_obj = unsafe { jni::objects::JObject::from_raw(act_ptr) };
            let activity_ref = env.new_global_ref(act_obj).expect("new_global_ref");

            // Load Class via Activity's ClassLoader to avoid ClassNotFoundException on native threads
            let class_obj = env.call_method(&activity_ref, "getClass", "()Ljava/lang/Class;", &[])
                .expect("getClass failed")
                .l()
                .expect("getClass returned null/non-object");

            let class_loader = env.call_method(&class_obj, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
                .expect("getClassLoader failed")
                .l()
                .expect("getClassLoader returned null/non-object");

            let class_name_jstr = env.new_string("app.moosync.android.services.MoosyncService")
                .expect("new_string failed");

            let cls_obj = env.call_method(
                &class_loader,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                &[jni::objects::JValue::Object(&class_name_jstr)],
            )
            .expect("loadClass MoosyncService failed")
            .l()
            .expect("loadClass returned null/non-class");

            let service_class_ref = env.new_global_ref(cls_obj).expect("new_global_ref");

            (activity_ref, service_class_ref)
        };

        StateManager::new(vm, activity, service_class).expect("StateManager::new failed")
    };

    let main_window = MainWindow::new().unwrap();

    main_window.on_search_clicked(|text| {
        println!("Search query received: {}", text);
    });

    main_window.run().unwrap();
}

fn setup_tracing() {
    let layer = fmt::layer().pretty().with_target(true).with_ansi(true);

    #[cfg(not(target_os = "android"))]
    let subscriber = {
        use std::fs;
        use tracing_appender::rolling::{RollingFileAppender, Rotation};

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
    let subscriber = tracing_subscriber::registry().with(layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();
}
