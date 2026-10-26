use std::sync::Arc;

use jni::objects::{Global, JObject};

#[derive(Debug, Clone)]
pub struct AndroidJNIContext {
    pub jvm: Arc<jni::JavaVM>,
    pub activity: Arc<Global<JObject<'static>>>,
    pub service_class: Arc<Global<JObject<'static>>>,
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for AndroidJNIContext {
    fn default() -> Self {
        Self {
            jvm: unsafe { Arc::new(jni::JavaVM::from_raw(1 as *mut _).unwrap()) },
            activity: Arc::new(Global::null()),
            service_class: Arc::new(Global::null()),
        }
    }
}
