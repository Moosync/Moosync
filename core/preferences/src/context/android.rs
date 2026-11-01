// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Mutex;

#[cfg(target_os = "android")]
use jni::{
    jni_sig, jni_str,
    objects::{JByteArray, JClass, JValue},
};
#[cfg(target_os = "android")]
use types::android::AndroidJNIContext;

use super::Keyring;

pub struct AndroidKeyringContext {
    #[cfg(target_os = "android")]
    #[allow(dead_code)]
    context: AndroidJNIContext,
    fallback: Mutex<Option<Vec<u8>>>,
}

impl AndroidKeyringContext {
    #[cfg(target_os = "android")]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(context: AndroidJNIContext) -> Self {
        Self {
            context,
            fallback: Mutex::new(None),
        }
    }

    #[cfg(not(target_os = "android"))]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            fallback: Mutex::new(None),
        }
    }
}

#[cfg(not(target_os = "android"))]
impl Default for AndroidKeyringContext {
    fn default() -> Self { Self::new() }
}

impl Keyring for AndroidKeyringContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_secret(&self, secret: &[u8]) -> Result<(), keyring::Error> {
        #[cfg(target_os = "android")]
        {
            if let Ok(res) = self.context.jvm.attach_current_thread(|env| {
                let activity = self.context.activity.as_obj();
                let class_obj = env
                    .call_method(
                        activity,
                        jni_str!("getClass"),
                        jni_sig!("()Ljava/lang/Class;"),
                        &[],
                    )?
                    .l()?;

                let class_loader = env
                    .call_method(
                        &class_obj,
                        jni_str!("getClassLoader"),
                        jni_sig!("()Ljava/lang/ClassLoader;"),
                        &[],
                    )?
                    .l()?;

                let class_name = env.new_string("app.moosync.android.security.KeyStoreHelper")?;

                let loaded_class = env
                    .call_method(
                        &class_loader,
                        jni_str!("loadClass"),
                        jni_sig!("(Ljava/lang/String;)Ljava/lang/Class;"),
                        &[JValue::Object(&class_name)],
                    )?
                    .l()?;

                let helper_class = unsafe { JClass::from_raw(env, loaded_class.as_raw().cast()) };
                let secret_jbytes = env.byte_array_from_slice(secret)?;

                let success = env
                    .call_static_method(
                        &helper_class,
                        jni_str!("setSecret"),
                        jni_sig!("(Landroid/content/Context;[B)Z"),
                        &[JValue::Object(activity), JValue::Object(&secret_jbytes)],
                    )?
                    .z()?;

                Ok::<_, jni::errors::Error>(success)
            }) {
                if res {
                    return Ok(());
                }
            }
        }

        let mut guard = self.fallback.lock().unwrap();
        *guard = Some(secret.to_vec());
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_secret(&self) -> Result<Vec<u8>, keyring::Error> {
        #[cfg(target_os = "android")]
        {
            if let Ok(sec) = self.context.jvm.attach_current_thread(|env| {
                let activity = self.context.activity.as_obj();
                let class_obj = env
                    .call_method(
                        activity,
                        jni_str!("getClass"),
                        jni_sig!("()Ljava/lang/Class;"),
                        &[],
                    )?
                    .l()?;

                let class_loader = env
                    .call_method(
                        &class_obj,
                        jni_str!("getClassLoader"),
                        jni_sig!("()Ljava/lang/ClassLoader;"),
                        &[],
                    )?
                    .l()?;

                let class_name = env.new_string("app.moosync.android.security.KeyStoreHelper")?;

                let loaded_class = env
                    .call_method(
                        &class_loader,
                        jni_str!("loadClass"),
                        jni_sig!("(Ljava/lang/String;)Ljava/lang/Class;"),
                        &[JValue::Object(&class_name)],
                    )?
                    .l()?;

                let helper_class = unsafe { JClass::from_raw(env, loaded_class.as_raw().cast()) };

                let result_obj = env
                    .call_static_method(
                        &helper_class,
                        jni_str!("getSecret"),
                        jni_sig!("(Landroid/content/Context;)[B"),
                        &[JValue::Object(activity)],
                    )?
                    .l()?;

                if result_obj.is_null() {
                    return Ok::<_, jni::errors::Error>(None);
                }

                let byte_array = unsafe { JByteArray::from_raw(env, result_obj.as_raw().cast()) };
                let bytes = env.convert_byte_array(&byte_array)?;
                Ok::<_, jni::errors::Error>(Some(bytes))
            }) {
                if let Some(bytes) = sec {
                    return Ok(bytes);
                }
            }
        }

        let guard = self.fallback.lock().unwrap();
        if let Some(ref sec) = *guard {
            return Ok(sec.clone());
        }
        Err(keyring::Error::NoEntry)
    }
}
