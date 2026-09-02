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

use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use assertables::{assert_err, assert_ok, assert_ok_eq_x};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{context::MockKeyring, keys::*, preferences::PreferenceConfig};

struct TestPrefsContext {
    pub _temp_dir: TempDir,
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
    TestPrefsContext {
        _temp_dir: temp_dir,
        prefs,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_new_and_defaults(prefs_context: TestPrefsContext) {
    let TestPrefsContext { _temp_dir, prefs } = prefs_context;
    let config_file = _temp_dir.path().join("config.json");

    assert!(config_file.exists());
    assert!(!prefs.has_key("scan_threads"));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_music_paths_crud(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;
    let paths = vec!["/path1".to_string(), "/path2".to_string()];
    let excl_paths = vec!["/excl1".to_string()];

    assert_ok!(prefs.save(MusicPaths, paths.clone()));
    assert_ok_eq_x!(prefs.load(MusicPaths).as_ref(), &paths);
    assert!(prefs.has_key("music_paths"));

    assert_ok!(prefs.remove_key(MusicPaths));
    assert!(!prefs.has_key("music_paths"));
    assert_err!(prefs.load(MusicPaths));

    assert_ok!(prefs.save(ExcludeMusicPaths, excl_paths.clone()));
    assert_ok_eq_x!(prefs.load(ExcludeMusicPaths).as_ref(), &excl_paths);
    assert_ok!(prefs.remove_key(ExcludeMusicPaths));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_primitive_values(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;

    assert_ok!(prefs.save(ScanThreads, 8));
    assert_ok_eq_x!(prefs.load(ScanThreads), 8);
    assert_ok!(prefs.remove_key(ScanThreads));

    assert_ok!(prefs.save(ArtistSplitter, "/".to_string()));
    assert_ok_eq_x!(prefs.load(ArtistSplitter).as_deref(), "/");
    assert_ok!(prefs.remove_key(ArtistSplitter));

    assert_ok!(prefs.save(AutoStartup, true));
    assert_ok_eq_x!(prefs.load(AutoStartup), true);
    assert_ok!(prefs.remove_key(AutoStartup));
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
        ScanThreads,
    );

    assert_ok!(prefs.save(ScanThreads, 4));
    assert_eq!(call_count.load(Ordering::SeqCst), 1);

    assert_ok!(prefs.remove_key(ScanThreads));
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_extension_protobuf_value(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;
    let ext_key = ExtensionKey {
        package_name: "test_pkg".to_string(),
        key: "test_key".to_string(),
    };
    let mut struct_val = extensions_proto::struct_proto::google::protobuf::Struct::default();
    struct_val.fields.insert(
        "inner_key".to_string(),
        extensions_proto::struct_proto::google::protobuf::Value {
            kind: Some(
                extensions_proto::struct_proto::google::protobuf::value::Kind::StringValue(
                    "inner_val".to_string(),
                ),
            ),
        },
    );
    let val = extensions_proto::struct_proto::google::protobuf::Value {
        kind: Some(
            extensions_proto::struct_proto::google::protobuf::value::Kind::StructValue(struct_val),
        ),
    };

    assert_ok!(prefs.save(ext_key.clone(), val.clone()));
    let loaded = prefs.load(ext_key.clone());
    assert_ok!(loaded.as_ref());
    assert_eq!(loaded.unwrap().kind, val.kind);
    assert_ok!(prefs.remove_key(ext_key));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_corrupted_json_recovery(mock_keyring: Box<MockKeyring>) {
    let temp_dir = TempDir::new("prefs_corrupt_test").unwrap();
    let config_file = temp_dir.path().join("preferences.json");
    fs::write(&config_file, b"{\"invalid_json: [").unwrap();

    assert_ok!(PreferenceConfig::new_with_context(
        temp_dir.path().to_path_buf(),
        mock_keyring
    ));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_special_characters_extension_key(prefs_context: TestPrefsContext) {
    let TestPrefsContext { prefs, .. } = prefs_context;
    let special_key = ExtensionKey {
        package_name: "org.moosync.plugin-v2_sub:item".to_string(),
        key: "token.special_value/123 🎵".to_string(),
    };
    let val = extensions_proto::struct_proto::google::protobuf::Value {
        kind: Some(
            extensions_proto::struct_proto::google::protobuf::value::Kind::StringValue(
                "special_val".to_string(),
            ),
        ),
    };

    assert_ok!(prefs.save(special_key.clone(), val.clone()));
    let load_res = prefs.load(special_key);
    assert_ok!(load_res.as_ref());
    assert_eq!(load_res.unwrap().kind, val.kind);
}
