use super::{preferences::PreferenceConfig, context::MockKeyringContext};
use std::env::temp_dir;
use types::ui::extensions::PreferenceData;
use serde_json::Value;
use uuid::Uuid;

fn get_test_db_path() -> std::path::PathBuf {
    let file_name = format!("moosync_test_prefs_{}.db", Uuid::new_v4());
    temp_dir().join(file_name)
}

#[test]
fn test_preferences_new() {
    let mut mock_context = Box::new(MockKeyringContext::new());
    mock_context.expect_get_secret().returning(|| Ok(vec![0; 32]));

    let db_path = get_test_db_path();
    let prefs = PreferenceConfig::new_with_context(db_path.clone(), mock_context);
    assert!(prefs.is_ok());
}

#[test]
fn test_set_and_get_preference() {
    let mut mock_context = Box::new(MockKeyringContext::new());
    mock_context.expect_get_secret().returning(|| Ok(vec![0; 32]));

    let db_path = get_test_db_path();
    let prefs = PreferenceConfig::new_with_context(db_path.clone(), mock_context).unwrap();

    let key = "test.key".to_string();
    let value = "test_value".to_string();

    let res = prefs.save_selective(key.clone(), Some(value.clone()));
    assert!(res.is_ok());

    let get_res: Result<String, _> = prefs.load_selective(key.clone());
    assert!(get_res.is_ok());
    assert_eq!(get_res.unwrap(), value);
}
