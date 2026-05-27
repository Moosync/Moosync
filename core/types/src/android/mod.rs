use jni;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AndroidJNIContext {
    pub jvm: Arc<jni::JavaVM>,
    pub activity: jni::objects::GlobalRef,
    pub service_class: jni::objects::GlobalRef,
}
