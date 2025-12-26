use super::preferences::PreferenceConfig;
use std::env::temp_dir;
use types::ui::extensions::PreferenceData;
use serde_json::Value;
use uuid::Uuid;

pub mod mock_keyring {
    use types::errors::MoosyncError;

    pub struct Entry {}

    impl Entry {
        pub fn new(_service: &str, _user: &str) -> Result<Self, MoosyncError> {
            Ok(Self {})
        }

        pub fn get_secret(&self) -> Result<Vec<u8>, MoosyncError> {
            Ok(vec![0; 32])
        }

        pub fn set_secret(&self, _secret: &[u8]) -> Result<(), MoosyncError> {
            Ok(())
        }
    }
}

fn get_test_db_path() -> std::path::PathBuf {
    let file_name = format!("moosync_test_prefs_{}.db", Uuid::new_v4());
    temp_dir().join(file_name)
}

#[test]
fn test_preferences_new() {
    let db_path = get_test_db_path();
    let prefs = PreferenceConfig::new(db_path.clone());
    assert!(prefs.is_ok());
}

#[test]
fn test_set_and_get_preference() {
    let db_path = get_test_db_path();
    let prefs = PreferenceConfig::new(db_path.clone()).unwrap();

    let key = "test.key".to_string();
    let value = "test_value".to_string();

    let res = prefs.save_selective(key.clone(), Some(value.clone()));
    assert!(res.is_ok());

    let get_res: Result<String, _> = prefs.load_selective(key.clone());
    assert!(get_res.is_ok());
    assert_eq!(get_res.unwrap(), value);
}
