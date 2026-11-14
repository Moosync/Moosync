use std::fs;

use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::remote::RemoteExtensions;

struct TestRemoteSmokeContext {
    pub _temp_dir: TempDir,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn smoke_context() -> TestRemoteSmokeContext {
    let temp_dir = TempDir::new("moosync_remote_smoke").expect("failed to create temp dir");
    let tmp_dir = temp_dir.path().join("tmp");
    fs::create_dir_all(&tmp_dir).unwrap();

    TestRemoteSmokeContext {
        _temp_dir: temp_dir,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_remote_extensions_init(smoke_context: TestRemoteSmokeContext) {
    let tmp_dir = smoke_context._temp_dir.path().join("tmp");

    let _remote = RemoteExtensions::new(tmp_dir);
}
