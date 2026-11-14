use super::{AndroidKeyringContext, Keyring};

#[test]
fn test_android_keyring_fallback() {
    #[cfg(target_os = "android")]
    let context = AndroidKeyringContext::new(types::android::AndroidJNIContext::default());
    #[cfg(not(target_os = "android"))]
    let context = AndroidKeyringContext::new();

    assert!(context.get_secret().is_err());

    let test_secret = b"12345678901234567890123456789012";
    context.set_secret(test_secret).unwrap();

    let fetched = context.get_secret().unwrap();
    assert_eq!(fetched, test_secret);
}
