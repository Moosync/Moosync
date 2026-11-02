use std::time::Duration;

use jni::{
    Env, EnvUnowned, NativeMethod, jni_str,
    objects::{JClass, JString},
    sys::jboolean,
};

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_back_pressed(_env: EnvUnowned, _class: JClass) -> jboolean {
    let (tx, rx) = std::sync::mpsc::channel();
    let res = slint::invoke_from_event_loop(move || {
        let handled = crate::pages::handle_native_back_press();
        let _ = tx.send(handled);
    });
    if res.is_err() {
        return false;
    }
    rx.recv_timeout(Duration::from_millis(500)).unwrap_or(false)
}

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_deep_link(mut env: EnvUnowned, _class: JClass, url: JString) {
    let outcome = env.with_env(|env| -> Result<(), jni::errors::Error> {
        let url_str = url.try_to_string(env)?;
        crate::accounts::on_deep_link(url_str);
        Ok(())
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>();
}

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_directory_picked(
    mut env: EnvUnowned,
    _class: JClass,
    path: JString,
) {
    let outcome = env.with_env(|env| -> Result<(), jni::errors::Error> {
        let path_str = path.try_to_string(env)?;
        crate::settings::on_directory_picked(path_str);
        Ok(())
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>();
}

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_file_picked(mut env: EnvUnowned, _class: JClass, path: JString) {
    let outcome = env.with_env(|env| -> Result<(), jni::errors::Error> {
        let path_str = path.try_to_string(env)?;
        crate::settings::on_file_picked(path_str);
        Ok(())
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>();
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn register_ui_natives(env: &mut Env, class: &JClass) -> Result<(), jni::errors::Error> {
    let methods = [
        unsafe {
            NativeMethod::from_raw_parts(
                jni_str!("nativeOnBackPressed"),
                jni_str!("()Z"),
                native_on_back_pressed as *mut _,
            )
        },
        unsafe {
            NativeMethod::from_raw_parts(
                jni_str!("nativeOnDeepLink"),
                jni_str!("(Ljava/lang/String;)V"),
                native_on_deep_link as *mut _,
            )
        },
        unsafe {
            NativeMethod::from_raw_parts(
                jni_str!("nativeOnDirectoryPicked"),
                jni_str!("(Ljava/lang/String;)V"),
                native_on_directory_picked as *mut _,
            )
        },
        unsafe {
            NativeMethod::from_raw_parts(
                jni_str!("nativeOnFilePicked"),
                jni_str!("(Ljava/lang/String;)V"),
                native_on_file_picked as *mut _,
            )
        },
    ];
    unsafe { env.register_native_methods(class, &methods) }
}

#[unsafe(no_mangle)]
#[tracing::instrument(level = "debug", skip_all)]
pub unsafe extern "system" fn JNI_OnLoad(
    raw_vm: *mut jni::sys::JavaVM,
    _reserved: *mut std::ffi::c_void,
) -> jni::sys::jint {
    let vm = unsafe { jni::JavaVM::from_raw(raw_vm) };
    let outcome = vm.attach_current_thread(|env| {
        state_manager::android::register_all_natives(env, &[&register_ui_natives])
    });
    match outcome {
        Ok(_) => {
            tracing::info!("JNI_OnLoad: successfully registered all native methods");
            jni::sys::JNI_VERSION_1_6
        }
        Err(e) => {
            tracing::error!("JNI_OnLoad: failed to register native methods: {:?}", e);
            jni::sys::JNI_ERR
        }
    }
}
