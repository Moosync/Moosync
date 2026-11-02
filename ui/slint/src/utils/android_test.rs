use std::sync::Arc;

use types::android::AndroidJNIContext;

use super::android::{get_android_context, init_android_context, reset_android_context};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_android_context_lifecycle() {
    reset_android_context();

    let result = std::panic::catch_unwind(|| {
        let _ = get_android_context();
    });
    assert!(result.is_err());

    let ctx = AndroidJNIContext::default();
    init_android_context(ctx);
    let retrieved = get_android_context();
    assert!(Arc::strong_count(&retrieved.jvm) >= 1);

    let second_ctx = AndroidJNIContext::default();
    init_android_context(second_ctx);
    let updated = get_android_context();
    assert!(Arc::strong_count(&updated.jvm) >= 1);

    reset_android_context();

    let post_reset = std::panic::catch_unwind(|| {
        let _ = get_android_context();
    });
    assert!(post_reset.is_err());
}
