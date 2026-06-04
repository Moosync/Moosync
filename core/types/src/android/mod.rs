use std::sync::Arc;

use jni;

#[derive(Debug, Clone)]
pub struct AndroidJNIContext {
    pub jvm: Arc<jni::JavaVM>,
    pub activity: jni::objects::GlobalRef,
    pub service_class: jni::objects::GlobalRef,
}
