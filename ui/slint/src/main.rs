#[cfg(target_os = "linux")]
use prctl::set_thp_disable;
use slint_app::run;
#[cfg(target_os = "android")]
use {
    jni::JavaVM, slint_app::utils::android::init_android_context, std::sync::Arc,
    types::android::AndroidJNIContext,
};

#[cfg(not(any(target_os = "windows", target_os = "android")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(not(target_os = "android"))]
fn main() {
    #[cfg(target_os = "linux")]
    let _ = set_thp_disable(true);

    run();
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
#[tracing::instrument(level = "debug", skip_all)]
fn android_main(app: slint::android::AndroidApp) {
    let _ = slint::android::init(app.clone());

    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    let context = AndroidJNIContext::from_activity(Arc::new(vm), app.activity_as_ptr().cast())
        .expect("Failed to initialize AndroidJNIContext from NativeActivity");

    init_android_context(context);
    run();
}
