use std::{
    ffi::c_void,
    mem,
    sync::{Arc, Mutex, RwLock},
};

use jni::{
    Env, JavaVM,
    errors::Error as JniError,
    jni_sig, jni_str,
    objects::{Global, JClass, JObject, JValue},
    signature::{MethodSignature, RuntimeMethodSignature},
    strings::JNIString,
};
use tokio::sync::oneshot::Sender;

#[derive(Debug, Clone)]
pub struct AndroidJNIContext {
    pub jvm: Arc<JavaVM>,
    pub activity: Arc<RwLock<Global<JObject<'static>>>>,
    pub class_loader: Arc<Global<JObject<'static>>>,
    pub service_class: Arc<Global<JObject<'static>>>,
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for AndroidJNIContext {
    fn default() -> Self {
        Self {
            jvm: unsafe { Arc::new(JavaVM::from_raw(1 as *mut _).unwrap()) },
            activity: Arc::new(RwLock::new(Global::null())),
            class_loader: Arc::new(Global::null()),
            service_class: Arc::new(Global::null()),
        }
    }
}

static PERMISSION_TX: Mutex<Option<Sender<bool>>> = Mutex::new(None);

#[tracing::instrument(level = "debug", skip_all)]
pub fn set_permission_waiter(tx: Sender<bool>) {
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
    pub fn get_object_class<'local>(
        &self,
        env: &mut Env<'local>,
        obj: &JObject,
    ) -> Result<JClass<'local>, JniError> {
        let class_obj = env
            .call_method(
                obj,
                jni_str!("getClass"),
                jni_sig!("()Ljava/lang/Class;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { JClass::from_raw(env, class_obj.as_raw().cast()) })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_class_loader<'local>(
        &self,
        env: &mut Env<'local>,
        class: &JClass,
    ) -> Result<JObject<'local>, JniError> {
        env.call_method(
            class,
            jni_str!("getClassLoader"),
            jni_sig!("()Ljava/lang/ClassLoader;"),
            &[],
        )?
        .l()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_activity(vm: Arc<JavaVM>, activity_ptr: *mut c_void) -> Result<Self, JniError> {
        let activity = vm.attach_current_thread(|env| {
            let act_obj = unsafe { JObject::from_raw(env, activity_ptr.cast()) };
            env.new_global_ref(act_obj)
        })?;

        let mut context = Self {
            jvm: vm,
            activity: Arc::new(RwLock::new(activity)),
            class_loader: Arc::new(Global::null()),
            service_class: Arc::new(Global::null()),
        };

        let class_loader = context.with_env(|env, this| {
            let activity_lock = this.activity.read().unwrap();
            let class_obj = this.get_object_class(env, &activity_lock)?;
            let loader = this.get_class_loader(env, &class_obj)?;
            let loader_ref = env.new_global_ref(loader)?;
            Ok(loader_ref)
        })?;
        context.class_loader = Arc::new(class_loader);

        let service_class = context.with_env(|env, this| {
            let cls = this.load_class(env, "app.moosync.android.services.MoosyncService")?;
            let service_ref = env.new_global_ref(JObject::from(cls))?;
            Ok(service_ref)
        })?;
        context.service_class = Arc::new(service_class);

        Ok(context)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn update_activity(&self, new_activity: Global<JObject<'static>>) {
        if let Ok(mut lock) = self.activity.write() {
            *lock = new_activity;
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn take_activity(&self) -> Global<JObject<'static>> {
        if let Ok(mut lock) = self.activity.write() {
            mem::replace(&mut *lock, Global::null())
        } else {
            Global::null()
        }
    }

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
            let activity_lock = this.activity.read().unwrap();
            let activity = activity_lock.as_obj();
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

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn open_url(&self, url: &str) -> Result<(), JniError> {
        self.with_env(|env, this| {
            let activity_lock = this.activity.read().unwrap();
            let activity = activity_lock.as_obj();

            let url_jstr = env.new_string(url)?;
            let uri_class = env.find_class(jni_str!("android/net/Uri"))?;
            let uri_val = env.call_static_method(
                uri_class,
                jni_str!("parse"),
                jni_sig!("(Ljava/lang/String;)Landroid/net/Uri;"),
                &[JValue::Object(&url_jstr)],
            )?;
            let uri_obj = uri_val.l()?;

            let action_view = env.new_string("android.intent.action.VIEW")?;
            let intent_class = env.find_class(jni_str!("android/content/Intent"))?;
            let intent_obj = env.new_object(
                intent_class,
                jni_sig!("(Ljava/lang/String;Landroid/net/Uri;)V"),
                &[JValue::Object(&action_view), JValue::Object(&uri_obj)],
            )?;

            env.call_method(
                activity,
                jni_str!("startActivity"),
                jni_sig!("(Landroid/content/Intent;)V"),
                &[JValue::Object(&intent_obj)],
            )?;
            Ok(())
        })
    }
}
