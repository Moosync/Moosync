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

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use assertables::{assert_ok, assert_ok_eq_x};
use preferences_proto::moosync::types::{PreferenceValue, StringList, preference_value};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{context::MockKeyring, keys::*, preferences::PreferenceConfig};

struct TestPrefsContext {
    pub temp_dir: TempDir,
    pub prefs: PreferenceConfig,
}

#[fixture]
fn mock_keyring() -> Box<MockKeyring> {
    let mut mock = Box::new(MockKeyring::new());
    mock.expect_get_secret().returning(|| Ok(vec![0; 32]));
    mock
}

#[fixture]
fn prefs_context(mock_keyring: Box<MockKeyring>) -> TestPrefsContext {
    let temp_dir = TempDir::new("prefs_test").expect("failed to create temp dir");
    let prefs = PreferenceConfig::new_with_context(temp_dir.path().to_path_buf(), mock_keyring)
        .expect("failed to init preferences");
    TestPrefsContext { temp_dir, prefs }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_new_and_defaults(prefs_context: TestPrefsContext) {
    let TestPrefsContext { temp_dir, prefs } = prefs_context;
    let config_file = temp_dir.path().join("preferences.bin");

    assert!(config_file.exists());
    assert!(prefs.get("scan_threads").is_none());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_music_paths_crud(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;
    let paths = vec!["/path1".to_string(), "/path2".to_string()];

    let mut item = prefs.load(&MUSIC_PATHS);
    assert_eq!(item.value::<Vec<String>>(), None);

    item.value = Some(PreferenceValue {
        value: Some(preference_value::Value::ListValue(StringList {
            values: paths.clone(),
        })),
    });

    assert_ok!(prefs.save(item));
    let loaded = prefs.load(&MUSIC_PATHS);
    assert_eq!(loaded.value::<Vec<String>>(), Some(paths));

    assert_ok!(prefs.remove(&MUSIC_PATHS));
    assert!(prefs.get("music_paths").is_none());
    assert_eq!(prefs.load(&MUSIC_PATHS).value::<Vec<String>>(), None);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_primitive_values(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;

    let mut scan_threads = prefs.load(&SCAN_THREADS);
    scan_threads.value = Some(PreferenceValue {
        value: Some(preference_value::Value::NumberValue(8.0)),
    });
    assert_ok!(prefs.save(scan_threads));
    assert_eq!(prefs.load(&SCAN_THREADS).value::<i32>(), Some(8));

    let mut auto_startup = prefs.load(&AUTO_STARTUP);
    auto_startup.value = Some(PreferenceValue {
        value: Some(preference_value::Value::BoolValue(true)),
    });
    assert_ok!(prefs.save(auto_startup));
    assert_eq!(prefs.load(&AUTO_STARTUP).value::<bool>(), Some(true));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_persistence_reload(prefs_context: TestPrefsContext) {
    let TestPrefsContext { temp_dir, prefs } = prefs_context;

    let mut item = prefs.load(&ARTIST_SPLITTER);
    item.value = Some(PreferenceValue {
        value: Some(preference_value::Value::StringValue(";".to_string())),
    });
    assert_ok!(prefs.save(item));

    let mut mock = Box::new(MockKeyring::new());
    mock.expect_get_secret().returning(|| Ok(vec![0; 32]));

    let reloaded = PreferenceConfig::new_with_context(temp_dir.path().to_path_buf(), mock);
    assert_ok!(reloaded.as_ref());
    let reloaded_prefs = reloaded.unwrap();

    let loaded = reloaded_prefs.load(&ARTIST_SPLITTER);
    assert_eq!(loaded.value::<String>().as_deref(), Some(";"));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_on_preference_changed(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;
    let call_count = Arc::new(AtomicUsize::new(0));

    let c = call_count.clone();
    let _handle = prefs.on_preference_changed(
        move |key| {
            c.fetch_add(1, Ordering::SeqCst);
            assert_eq!(key, "scan_threads");
        },
        SCAN_THREADS.id.as_str(),
    );

    let mut item = prefs.load(&SCAN_THREADS);
    item.value = Some(PreferenceValue {
        value: Some(preference_value::Value::NumberValue(4.0)),
    });
    assert_ok!(prefs.save(item));
    assert_eq!(call_count.load(Ordering::SeqCst), 1);

    assert_ok!(prefs.remove(&SCAN_THREADS));
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_secure_storage(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
    struct SecretToken {
        token: String,
    }

    let token = SecretToken {
        token: "super_secret_123".to_string(),
    };

    assert_ok!(prefs.set_secure("auth_token".to_string(), Some(token.clone())));
    let loaded: Result<SecretToken, _> = prefs.get_secure("auth_token".to_string());
    assert_ok_eq_x!(&loaded, &token);

    assert_ok!(prefs.set_secure::<SecretToken>("auth_token".to_string(), None));
    let deleted: Result<SecretToken, _> = prefs.get_secure("auth_token".to_string());
    assert!(deleted.is_err());
}
