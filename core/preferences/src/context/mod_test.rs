use assertables::assert_err;
use rstest::{fixture, rstest};
use tracing_test::traced_test;
use uuid::Uuid;

use crate::context::{Keyring, KeyringContext};

#[fixture]
fn fresh_keyring_context() -> KeyringContext {
    let service = format!("moosync_test_empty_{}", Uuid::new_v4());
    let user = format!("moosync_test_user_{}", Uuid::new_v4());
    KeyringContext::new(&service, &user).expect("failed to init fresh keyring context")
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_keyring_context_get_nonexistent_returns_no_entry(fresh_keyring_context: KeyringContext) {
    assert_err!(fresh_keyring_context.get_secret());
}
