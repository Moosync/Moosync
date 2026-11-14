use std::sync::Mutex;

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
            if let Ok(true) = self.context.call_static::<_, bool>(
                "app.moosync.android.security.KeyStoreHelper",
                "setSecret",
                (&self.context.activity, secret),
            ) {
                return Ok(());
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
            if let Ok(Some(bytes)) = self.context.call_static::<_, Option<Vec<u8>>>(
                "app.moosync.android.security.KeyStoreHelper",
                "getSecret",
                &self.context.activity,
            ) {
                return Ok(bytes);
            }
        }

        let guard = self.fallback.lock().unwrap();
        if let Some(ref sec) = *guard {
            return Ok(sec.clone());
        }
        Err(keyring::Error::NoEntry)
    }
}
