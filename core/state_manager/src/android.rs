use jni::{Env, jni_str, objects::JClass};

pub type NativeRegistrar = dyn Fn(&mut Env, &JClass) -> Result<(), jni::errors::Error>;

#[tracing::instrument(level = "debug", skip_all)]
pub fn register_all_natives(
    env: &mut Env,
    extra_registrars: &[&NativeRegistrar],
) -> Result<(), jni::errors::Error> {
    let class = env.find_class(jni_str!("app/moosync/android/NativeBridge"))?;
    mpris::mpris_android::register_mpris_natives(env, &class)?;
    file_scanner::context::android::register_scanner_natives(env, &class)?;
    for registrar in extra_registrars {
        registrar(env, &class)?;
    }
    Ok(())
}
