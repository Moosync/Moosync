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
    let _handle = prefs.on_preference_changed.listen_filtered(
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

    let token = extensions_proto::moosync::types::PreferenceData {
        key: "token_key".to_string(),
        value: Some(extensions_proto::struct_proto::google::protobuf::Value {
            kind: Some(
                extensions_proto::struct_proto::google::protobuf::value::Kind::StringValue(
                    "super_secret_123".to_string(),
                ),
            ),
        }),
    };

    assert_ok!(prefs.set_secure("auth_token".to_string(), Some(token.clone())));
    let loaded: Result<extensions_proto::moosync::types::PreferenceData, _> =
        prefs.get_secure("auth_token".to_string());
    assert_ok_eq_x!(&loaded, &token);

    assert_ok!(
        prefs.set_secure::<extensions_proto::moosync::types::PreferenceData>(
            "auth_token".to_string(),
            None
        )
    );
    let deleted: Result<extensions_proto::moosync::types::PreferenceData, _> =
        prefs.get_secure("auth_token".to_string());
    assert!(deleted.is_err());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_debug_format(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;

    let debug_str = format!("{:?}", prefs);

    assert!(debug_str.contains("PreferenceConfig"));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_corrupted_file_recovery() {
    let temp_dir = TempDir::new("prefs_corrupt").expect("failed to create temp dir");
    let config_file = temp_dir.path().join("preferences.bin");
    std::fs::write(&config_file, b"corrupted bytes that fail prost decoding").unwrap();
    let mut mock = Box::new(MockKeyring::new());
    mock.expect_get_secret().returning(|| Ok(vec![0; 32]));

    let prefs = PreferenceConfig::new_with_context(temp_dir.path().to_path_buf(), mock);

    assert_ok!(prefs.as_ref());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_get_secure_key_not_found(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;

    let result: Result<extensions_proto::moosync::types::PreferenceData, _> =
        prefs.get_secure("nonexistent_key_12345".to_string());

    assert!(result.is_err());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_secure_roundtrip(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;
    let data = StringList {
        values: vec!["secret_token_123".to_string()],
    };

    prefs
        .set_secure("my_secure_key".to_string(), Some(data.clone()))
        .unwrap();

    let loaded: StringList = prefs.get_secure("my_secure_key".to_string()).unwrap();
    assert_eq!(loaded, data);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_get_secure_legacy_unencrypted_fallback(prefs_context: TestPrefsContext) {
    use prost::Message;
    let TestPrefsContext { prefs, .. } = prefs_context;
    let data = StringList {
        values: vec!["legacy_unencrypted_token".to_string()],
    };
    let hex_encoded = hex::encode(data.encode_to_vec());

    let item = preferences_proto::moosync::types::PreferenceItem {
        id: "legacy_key".to_string(),
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::StringValue(hex_encoded)),
        }),
        ..Default::default()
    };
    prefs.save(item).unwrap();

    let loaded: StringList = prefs.get_secure("legacy_key".to_string()).unwrap();
    assert_eq!(loaded, data);
}
