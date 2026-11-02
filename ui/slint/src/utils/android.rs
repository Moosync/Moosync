#![cfg(target_os = "android")]

use std::sync::RwLock;

use types::android::AndroidJNIContext;

static ANDROID_CONTEXT: RwLock<Option<AndroidJNIContext>> = RwLock::new(None);

#[tracing::instrument(level = "debug", skip_all)]
pub fn init_android_context(context: AndroidJNIContext) {
    let mut lock = ANDROID_CONTEXT
        .write()
        .expect("ANDROID_CONTEXT lock poisoned");
    if let Some(existing) = lock.as_ref() {
        existing.update_activity(context.take_activity());
    } else {
        *lock = Some(context);
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_android_context() -> AndroidJNIContext {
    let guard = ANDROID_CONTEXT
        .read()
        .expect("ANDROID_CONTEXT lock poisoned");
    guard.clone().expect("ANDROID_CONTEXT not initialized")
}

#[cfg(test)]
#[tracing::instrument(level = "debug", skip_all)]
pub fn reset_android_context() {
    let mut lock = ANDROID_CONTEXT
        .write()
        .expect("ANDROID_CONTEXT lock poisoned");
    *lock = None;
}
