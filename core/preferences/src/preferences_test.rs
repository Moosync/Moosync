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

use std::{env::temp_dir, fs};

use uuid::Uuid;

use crate::{context::MockKeyring, keys::*, preferences::PreferenceConfig};

#[tracing::instrument(level = "debug", skip_all)]
fn get_test_prefs_dir() -> std::path::PathBuf {
    let dir = temp_dir().join(format!("moosync_test_prefs_{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_new_and_defaults() {
    let mut mock_context = Box::new(MockKeyring::new());
    mock_context
        .expect_get_secret()
        .returning(|| Ok(vec![0; 32]));

    let dir = get_test_prefs_dir();
    let prefs = PreferenceConfig::new_with_context(dir.clone(), mock_context).unwrap();

    let config_file = dir.join("config.json");
    assert!(config_file.exists());
    assert!(!prefs.has_key("scan_threads"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_music_paths_crud() {
    let mut mock_context = Box::new(MockKeyring::new());
    mock_context
        .expect_get_secret()
        .returning(|| Ok(vec![0; 32]));

    let dir = get_test_prefs_dir();
    let prefs = PreferenceConfig::new_with_context(dir.clone(), mock_context).unwrap();

    let paths = vec!["/path1".to_string(), "/path2".to_string()];
    assert!(prefs.save(MusicPaths, paths.clone()).is_ok());
    assert_eq!(prefs.load(MusicPaths).unwrap(), paths);
    assert!(prefs.has_key("music_paths"));
    assert!(prefs.remove_key(MusicPaths).is_ok());
    assert!(!prefs.has_key("music_paths"));
    assert!(prefs.load(MusicPaths).is_err());

    let excl_paths = vec!["/excl1".to_string()];
    assert!(prefs.save(ExcludeMusicPaths, excl_paths.clone()).is_ok());
    assert_eq!(prefs.load(ExcludeMusicPaths).unwrap(), excl_paths);
    assert!(prefs.remove_key(ExcludeMusicPaths).is_ok());

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_primitive_values() {
    let mut mock_context = Box::new(MockKeyring::new());
    mock_context
        .expect_get_secret()
        .returning(|| Ok(vec![0; 32]));

    let dir = get_test_prefs_dir();
    let prefs = PreferenceConfig::new_with_context(dir.clone(), mock_context).unwrap();

    assert!(prefs.save(ScanThreads, 8).is_ok());
    assert_eq!(prefs.load(ScanThreads).unwrap(), 8);
    assert!(prefs.remove_key(ScanThreads).is_ok());

    assert!(prefs.save(ArtistSplitter, "/".to_string()).is_ok());
    assert_eq!(prefs.load(ArtistSplitter).unwrap(), "/");
    assert!(prefs.remove_key(ArtistSplitter).is_ok());

    assert!(prefs.save(AutoStartup, true).is_ok());
    assert_eq!(prefs.load(AutoStartup).unwrap(), true);
    assert!(prefs.remove_key(AutoStartup).is_ok());

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_extension_protobuf_value() {
    let mut mock_context = Box::new(MockKeyring::new());
    mock_context
        .expect_get_secret()
        .returning(|| Ok(vec![0; 32]));

    let dir = get_test_prefs_dir();
    let prefs = PreferenceConfig::new_with_context(dir.clone(), mock_context).unwrap();

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

    assert!(prefs.save(ext_key.clone(), val.clone()).is_ok());
    let loaded_ext = prefs.load(ext_key.clone());
    assert!(loaded_ext.is_ok());
    assert_eq!(loaded_ext.unwrap().kind, val.kind);
    assert!(prefs.remove_key(ext_key.clone()).is_ok());

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_corrupted_json_recovery() {
    let mut mock_context = Box::new(MockKeyring::new());
    mock_context
        .expect_get_secret()
        .returning(|| Ok(vec![0; 32]));

    let dir = get_test_prefs_dir();
    let config_file = dir.join("preferences.json");
    fs::write(&config_file, b"{\"invalid_json: [").unwrap();

    // Opening corrupted JSON should gracefully fall back to defaults or overwrite
    let prefs = PreferenceConfig::new_with_context(dir.clone(), mock_context);
    assert!(prefs.is_ok());

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_special_characters_extension_key() {
    let mut mock_context = Box::new(MockKeyring::new());
    mock_context
        .expect_get_secret()
        .returning(|| Ok(vec![0; 32]));

    let dir = get_test_prefs_dir();
    let prefs = PreferenceConfig::new_with_context(dir.clone(), mock_context).unwrap();

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
    assert!(prefs.save(special_key.clone(), val.clone()).is_ok());
    assert_eq!(prefs.load(special_key).unwrap().kind, val.kind);

    let _ = fs::remove_dir_all(dir);
}
