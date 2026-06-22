use std::env::temp_dir;

use uuid::Uuid;

use super::{context::MockKeyring, preferences::PreferenceConfig};

fn get_test_db_path() -> std::path::PathBuf {
    let file_name = format!("moosync_test_prefs_{}", Uuid::new_v4());
    temp_dir().join(file_name)
}

#[test]
fn test_preferences_new() {
    let mut mock_context = Box::new(MockKeyring::new());
    mock_context
        .expect_get_secret()
        .returning(|| Ok(vec![0; 32]));

    let db_path = get_test_db_path();
    let prefs = PreferenceConfig::new_with_context(db_path.clone(), mock_context);
    assert!(prefs.is_ok());
}

#[test]
fn test_exhaustive_keys() {
    let mut mock_context = Box::new(MockKeyring::new());
    mock_context
        .expect_get_secret()
        .returning(|| Ok(vec![0; 32]));

    let db_path = get_test_db_path();
    let prefs = PreferenceConfig::new_with_context(db_path.clone(), mock_context).unwrap();

    use crate::keys::*;

    let paths = vec!["/path1".to_string(), "/path2".to_string()];
    assert!(prefs.save(MusicPaths, paths.clone()).is_ok());
    assert_eq!(prefs.load(MusicPaths).unwrap(), paths);
    assert!(prefs.remove_key(MusicPaths).is_ok());
    assert!(prefs.load(MusicPaths).is_err());

    let excl_paths = vec!["/excl1".to_string()];
    assert!(prefs.save(ExcludeMusicPaths, excl_paths.clone()).is_ok());
    assert_eq!(prefs.load(ExcludeMusicPaths).unwrap(), excl_paths);
    assert!(prefs.remove_key(ExcludeMusicPaths).is_ok());
    assert!(prefs.load(ExcludeMusicPaths).is_err());

    let threads = 4i32;
    assert!(prefs.save(ScanThreads, threads).is_ok());
    assert_eq!(prefs.load(ScanThreads).unwrap(), threads);
    assert!(prefs.remove_key(ScanThreads).is_ok());
    assert!(prefs.load(ScanThreads).is_err());

    let splitter = ",".to_string();
    assert!(prefs.save(ArtistSplitter, splitter.clone()).is_ok());
    assert_eq!(prefs.load(ArtistSplitter).unwrap(), splitter);
    assert!(prefs.remove_key(ArtistSplitter).is_ok());
    assert!(prefs.load(ArtistSplitter).is_err());

    let interval = 60i32;
    assert!(prefs.save(ScanInterval, interval).is_ok());
    assert_eq!(prefs.load(ScanInterval).unwrap(), interval);
    assert!(prefs.remove_key(ScanInterval).is_ok());
    assert!(prefs.load(ScanInterval).is_err());

    let thumb = "/thumb".to_string();
    assert!(prefs.save(ThumbnailPath, thumb.clone()).is_ok());
    assert_eq!(prefs.load(ThumbnailPath).unwrap(), thumb);
    assert!(prefs.remove_key(ThumbnailPath).is_ok());
    assert!(prefs.load(ThumbnailPath).is_err());

    let art = "/art".to_string();
    assert!(prefs.save(ArtworkPath, art.clone()).is_ok());
    assert_eq!(prefs.load(ArtworkPath).unwrap(), art);
    assert!(prefs.remove_key(ArtworkPath).is_ok());
    assert!(prefs.load(ArtworkPath).is_err());

    let startup = true;
    assert!(prefs.save(AutoStartup, startup).is_ok());
    assert_eq!(prefs.load(AutoStartup).unwrap(), startup);
    assert!(prefs.remove_key(AutoStartup).is_ok());
    assert!(prefs.load(AutoStartup).is_err());

    let tray = false;
    assert!(prefs.save(MinimizeToTray, tray).is_ok());
    assert_eq!(prefs.load(MinimizeToTray).unwrap(), tray);
    assert!(prefs.remove_key(MinimizeToTray).is_ok());
    assert!(prefs.load(MinimizeToTray).is_err());

    let jukebox = true;
    assert!(prefs.save(JukeboxMode, jukebox).is_ok());
    assert_eq!(prefs.load(JukeboxMode).unwrap(), jukebox);
    assert!(prefs.remove_key(JukeboxMode).is_ok());
    assert!(prefs.load(JukeboxMode).is_err());

    let clear = false;
    assert!(prefs.save(ClearQueue, clear).is_ok());
    assert_eq!(prefs.load(ClearQueue).unwrap(), clear);
    assert!(prefs.remove_key(ClearQueue).is_ok());
    assert!(prefs.load(ClearQueue).is_err());

    let vol = "clamp".to_string();
    assert!(prefs.save(VolumePersistMode, vol.clone()).is_ok());
    assert_eq!(prefs.load(VolumePersistMode).unwrap(), vol);
    assert!(prefs.remove_key(VolumePersistMode).is_ok());
    assert!(prefs.load(VolumePersistMode).is_err());

    let lang = "en".to_string();
    assert!(prefs.save(I18nLanguage, lang.clone()).is_ok());
    assert_eq!(prefs.load(I18nLanguage).unwrap(), lang);
    assert!(prefs.remove_key(I18nLanguage).is_ok());
    assert!(prefs.load(I18nLanguage).is_err());

    let theme = uuid::Uuid::new_v4().to_string();
    assert!(prefs.save(ActiveThemeId, theme.clone()).is_ok());
    assert_eq!(prefs.load(ActiveThemeId).unwrap(), theme);
    assert!(prefs.remove_key(ActiveThemeId).is_ok());
    assert!(prefs.load(ActiveThemeId).is_err());

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
    let loaded_val = loaded_ext.unwrap();
    assert_eq!(loaded_val.kind, val.kind);
    assert!(prefs.remove_key(ext_key.clone()).is_ok());
    assert!(prefs.load(ext_key).is_err());
}
