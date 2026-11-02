use std::sync::Arc;

use jni::{
    Env,
    errors::Error as JniError,
    jni_sig, jni_str,
    objects::{Global, JByteArray, JClass, JObject, JString, JValue, JValueOwned},
    signature::{MethodSignature, RuntimeMethodSignature},
    strings::JNIString,
    sys::{jboolean, jint, jlong},
};

#[derive(Debug, Clone)]
pub struct AndroidJNIContext {
    pub jvm: Arc<jni::JavaVM>,
    pub activity: Arc<Global<JObject<'static>>>,
    pub class_loader: Arc<Global<JObject<'static>>>,
    pub service_class: Arc<Global<JObject<'static>>>,
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for AndroidJNIContext {
    fn default() -> Self {
        Self {
            jvm: unsafe { Arc::new(jni::JavaVM::from_raw(1 as *mut _).unwrap()) },
            activity: Arc::new(Global::null()),
            class_loader: Arc::new(Global::null()),
            service_class: Arc::new(Global::null()),
        }
    }
}

static PERMISSION_TX: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<bool>>> =
    std::sync::Mutex::new(None);

#[tracing::instrument(level = "debug", skip_all)]
pub fn set_permission_waiter(tx: tokio::sync::oneshot::Sender<bool>) {
    if let Ok(mut guard) = PERMISSION_TX.lock() {
        *guard = Some(tx);
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn dispatch_permission_result(request_code: i32, granted: bool) {
    if request_code == 1001 {
        if let Ok(mut guard) = PERMISSION_TX.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(granted);
            }
        }
    }
}

pub mod args;
pub use args::*;

impl AndroidJNIContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn with_env<T, F>(&self, f: F) -> Result<T, JniError>
    where
        F: FnOnce(&mut Env, &Self) -> Result<T, JniError>,
    {
        self.jvm.attach_current_thread(|env| f(env, self))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load_class<'local>(
        &self,
        env: &mut Env<'local>,
        class_name: &str,
    ) -> Result<JClass<'local>, JniError> {
        let class_loader = self.class_loader.as_obj();
        let name_jstr = env.new_string(class_name)?;
        let loaded = env
            .call_method(
                class_loader,
                jni_str!("loadClass"),
                jni_sig!("(Ljava/lang/String;)Ljava/lang/Class;"),
                &[JValue::Object(&name_jstr)],
            )?
            .l()?;
        Ok(unsafe { JClass::from_raw(env, loaded.as_raw().cast()) })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn call_static<Args: ToJniArgs, Ret: FromJniReturn>(
        &self,
        class_name: &str,
        method_name: &str,
        args: Args,
    ) -> Result<Ret, JniError> {
        self.with_env(|env, this| {
            let class = this.load_class(env, class_name)?;
            let method_jni_str = JNIString::from(method_name);
            let sig_string = format!("({}){}", Args::sig(), Ret::sig());
            let runtime_sig = sig_string.parse::<RuntimeMethodSignature>()?;
            let method_sig = MethodSignature::from(&runtime_sig);
            let owned_args = args.to_jvalues(env)?;
            let borrowed_args: Vec<JValue> = owned_args.iter().map(|arg| arg.borrow()).collect();
            let ret =
                env.call_static_method(&class, &method_jni_str, &method_sig, &borrowed_args)?;
            Ret::from_jvalue(env, ret)
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn call_activity<Args: ToJniArgs, Ret: FromJniReturn>(
        &self,
        method_name: &str,
        args: Args,
    ) -> Result<Ret, JniError> {
        self.with_env(|env, this| {
            let activity = this.activity.as_obj();
            let method_jni_str = JNIString::from(method_name);
            let sig_string = format!("({}){}", Args::sig(), Ret::sig());
            let runtime_sig = sig_string.parse::<RuntimeMethodSignature>()?;
            let method_sig = MethodSignature::from(&runtime_sig);
            let owned_args = args.to_jvalues(env)?;
            let borrowed_args: Vec<JValue> = owned_args.iter().map(|arg| arg.borrow()).collect();
            let ret = env.call_method(activity, &method_jni_str, &method_sig, &borrowed_args)?;
            Ret::from_jvalue(env, ret)
        })
    }
}
