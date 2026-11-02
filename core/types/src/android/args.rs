use std::sync::Arc;

use jni::{
    Env,
    errors::Error as JniError,
    objects::{Global, JByteArray, JObject, JString, JValueOwned},
    sys::{jboolean, jint, jlong},
};

pub trait ToJniArg {
    fn sig() -> &'static str;
    fn to_jvalue_owned<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError>;
}

impl ToJniArg for bool {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "Z" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalue_owned<'local>(
        &self,
        _env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError> {
        Ok(JValueOwned::Bool(*self as jboolean))
    }
}

impl ToJniArg for i32 {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "I" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalue_owned<'local>(
        &self,
        _env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError> {
        Ok(JValueOwned::Int(*self as jint))
    }
}

impl ToJniArg for i64 {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "J" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalue_owned<'local>(
        &self,
        _env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError> {
        Ok(JValueOwned::Long(*self as jlong))
    }
}

impl ToJniArg for &str {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "Ljava/lang/String;" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalue_owned<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError> {
        let jstr = env.new_string(self)?;
        Ok(JValueOwned::Object(jstr.into()))
    }
}

impl ToJniArg for String {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "Ljava/lang/String;" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalue_owned<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError> {
        self.as_str().to_jvalue_owned(env)
    }
}

impl ToJniArg for Option<&str> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "Ljava/lang/String;" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalue_owned<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError> {
        match self {
            Some(s) => s.to_jvalue_owned(env),
            None => Ok(JValueOwned::Object(JObject::null())),
        }
    }
}

impl ToJniArg for &[u8] {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "[B" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalue_owned<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError> {
        let jbytes = env.byte_array_from_slice(self)?;
        Ok(JValueOwned::Object(jbytes.into()))
    }
}

impl ToJniArg for &Global<JObject<'static>> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "Landroid/content/Context;" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalue_owned<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError> {
        let local_ref = env.new_local_ref(self.as_obj())?;
        Ok(JValueOwned::Object(local_ref))
    }
}

impl ToJniArg for &Arc<Global<JObject<'static>>> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "Landroid/content/Context;" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalue_owned<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError> {
        let local_ref = env.new_local_ref(self.as_obj())?;
        Ok(JValueOwned::Object(local_ref))
    }
}

impl ToJniArg for Arc<Global<JObject<'static>>> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "Landroid/content/Context;" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalue_owned<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> Result<JValueOwned<'local>, JniError> {
        let local_ref = env.new_local_ref(self.as_obj())?;
        Ok(JValueOwned::Object(local_ref))
    }
}

pub trait ToJniArgs {
    fn sig() -> String;
    fn to_jvalues<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> Result<Vec<JValueOwned<'local>>, JniError>;
}

impl ToJniArgs for () {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> String { String::new() }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalues<'local>(
        &self,
        _env: &mut Env<'local>,
    ) -> Result<Vec<JValueOwned<'local>>, JniError> {
        Ok(Vec::new())
    }
}

impl<T: ToJniArg> ToJniArgs for T {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> String { T::sig().to_string() }
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_jvalues<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> Result<Vec<JValueOwned<'local>>, JniError> {
        Ok(vec![self.to_jvalue_owned(env)?])
    }
}

macro_rules! impl_to_jni_args_tuple {
    ($($idx:tt: $T:ident),+) => {
        impl<$($T: ToJniArg),+> ToJniArgs for ($($T,)+) {
            #[tracing::instrument(level = "debug", skip_all)]
            fn sig() -> String {
                let mut s = String::new();
                $(s.push_str($T::sig());)+
                s
            }
            #[tracing::instrument(level = "debug", skip_all)]
            fn to_jvalues<'local>(&self, env: &mut Env<'local>) -> Result<Vec<JValueOwned<'local>>, JniError> {
                Ok(vec![
                    $(self.$idx.to_jvalue_owned(env)?,)+
                ])
            }
        }
    };
}

impl_to_jni_args_tuple!(0: A, 1: B);
impl_to_jni_args_tuple!(0: A, 1: B, 2: C);
impl_to_jni_args_tuple!(0: A, 1: B, 2: C, 3: D);
impl_to_jni_args_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);

pub trait FromJniReturn: Sized {
    fn sig() -> &'static str;
    fn from_jvalue<'local>(
        env: &mut Env<'local>,
        value: JValueOwned<'local>,
    ) -> Result<Self, JniError>;
}

impl FromJniReturn for () {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "V" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn from_jvalue<'local>(
        _env: &mut Env<'local>,
        _value: JValueOwned<'local>,
    ) -> Result<Self, JniError> {
        Ok(())
    }
}

impl FromJniReturn for bool {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "Z" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn from_jvalue<'local>(
        _env: &mut Env<'local>,
        value: JValueOwned<'local>,
    ) -> Result<Self, JniError> {
        value.z()
    }
}

impl FromJniReturn for i32 {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "I" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn from_jvalue<'local>(
        _env: &mut Env<'local>,
        value: JValueOwned<'local>,
    ) -> Result<Self, JniError> {
        value.i()
    }
}

impl FromJniReturn for i64 {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "J" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn from_jvalue<'local>(
        _env: &mut Env<'local>,
        value: JValueOwned<'local>,
    ) -> Result<Self, JniError> {
        value.j()
    }
}

impl FromJniReturn for Vec<u8> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "[B" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn from_jvalue<'local>(
        env: &mut Env<'local>,
        value: JValueOwned<'local>,
    ) -> Result<Self, JniError> {
        let obj = value.l()?;
        let byte_array = unsafe { JByteArray::from_raw(env, obj.as_raw().cast()) };
        env.convert_byte_array(&byte_array)
    }
}

impl FromJniReturn for Option<Vec<u8>> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "[B" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn from_jvalue<'local>(
        env: &mut Env<'local>,
        value: JValueOwned<'local>,
    ) -> Result<Self, JniError> {
        let obj = value.l()?;
        if obj.is_null() {
            return Ok(None);
        }
        let byte_array = unsafe { JByteArray::from_raw(env, obj.as_raw().cast()) };
        let bytes = env.convert_byte_array(&byte_array)?;
        Ok(Some(bytes))
    }
}

impl FromJniReturn for String {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sig() -> &'static str { "Ljava/lang/String;" }
    #[tracing::instrument(level = "debug", skip_all)]
    fn from_jvalue<'local>(
        env: &mut Env<'local>,
        value: JValueOwned<'local>,
    ) -> Result<Self, JniError> {
        let obj = value.l()?;
        let jstr = unsafe { JString::from_raw(env, obj.as_raw().cast()) };
        Ok(jstr.to_string())
    }
}
