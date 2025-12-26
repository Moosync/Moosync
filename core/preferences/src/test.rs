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

use std::{env::temp_dir, fs, path::PathBuf, thread, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::json;
use types::errors::Result;

use crate::preferences::PreferenceConfig;

// Helper for creating test directories
fn setup_test_dir() -> PathBuf {
    let test_dir = temp_dir().join(format!("moosync_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    test_dir
}

// Helper for cleaning up test directories
fn cleanup_test_dir(dir: PathBuf) {
    if dir.exists() {
        let _ = fs::remove_dir_all(dir);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct TestConfig {
    string_value: String,
    number_value: i32,
    boolean_value: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct SecureTestData {
    username: String,
    password: String,
}

#[test]
fn test_preference_init() -> Result<()> {
    let test_dir = setup_test_dir();

    let prefs = PreferenceConfig::new(test_dir.clone())?;

    // Verify the config file exists
    let config_file = prefs.config_file.lock().unwrap();
    assert!(
        config_file.exists(),
        "Config file should be created during initialization"
    );

    // Verify memcache is initialized as an object
    // Note: The memcache might not be empty as it could contain default values
    let memcache = prefs.memcache.lock().unwrap();
    assert!(
        memcache.is_object(),
        "Memcache should be initialized as an object"
    );

    cleanup_test_dir(test_dir);
    Ok(())
}

#[test]
fn test_save_and_load_selective() -> Result<()> {
    let test_dir = setup_test_dir();

    let prefs = PreferenceConfig::new(test_dir.clone())?;

    // Create test data
    let test_data = TestConfig {
        string_value: "test".to_string(),
        number_value: 42,
        boolean_value: true,
    };

    // Save data
    prefs.save_selective("test_key".to_string(), Some(test_data.clone()))?;

    // Load data
    let loaded_data: TestConfig = prefs.load_selective("test_key".to_string())?;

    // Verify data
    assert_eq!(
        loaded_data, test_data,
        "Loaded data should match saved data"
    );

    cleanup_test_dir(test_dir);
    Ok(())
}

#[test]
fn test_save_and_load_array() -> Result<()> {
    let test_dir = setup_test_dir();

    let prefs = PreferenceConfig::new(test_dir.clone())?;

    // Create test array data
    let test_array = vec![
        TestConfig {
            string_value: "item1".to_string(),
            number_value: 1,
            boolean_value: true,
        },
        TestConfig {
            string_value: "item2".to_string(),
            number_value: 2,
            boolean_value: false,
        },
    ];

    // Convert the array to a Value and save it directly
    let array_json = serde_json::to_value(&test_array)?;

    // Save with dot notation for the key path
    prefs.save_selective("test.array_key".to_string(), Some(array_json))?;

    // First approach: Try to load individual items using array index notation
    let item0: TestConfig = prefs.load_selective("test.array_key.0".to_string())?;
    let item1: TestConfig = prefs.load_selective("test.array_key.1".to_string())?;

    // Verify individual items
    assert_eq!(item0.string_value, "item1", "First item should be correct");
    assert_eq!(item1.number_value, 2, "Second item should be correct");

    // Second approach: If direct array loading is supported
    // Try a simpler test first to see if array loading works
    let simple_array = vec![1, 2, 3];
    prefs.save_selective("test.simple_array".to_string(), Some(json!(simple_array)))?;

    // Try to load the simple array
    let loaded_simple: Vec<i32> = prefs.load_selective("test.simple_array".to_string())?;
    assert_eq!(
        loaded_simple,
        vec![1, 2, 3],
        "Simple array should load correctly"
    );

    cleanup_test_dir(test_dir);
    Ok(())
}

#[test]
fn test_secure_preferences() -> Result<()> {
    let test_dir = setup_test_dir();

    let prefs = PreferenceConfig::new(test_dir.clone())?;

    // Create secure test data with distinctive content that would be easy to spot in plaintext
    let secure_data = SecureTestData {
        username: "user123".to_string(),
        password: "VERY_DISTINCTIVE_PASSWORD_STRING_FOR_TESTING".to_string(),
    };

    // Save secure data
    prefs.set_secure("secure_key".to_string(), Some(secure_data.clone()))?;

    // Get the path to the config file
    let config_file_path = prefs.config_file.lock().unwrap().clone();

    // Read the raw file content to check if it contains the plaintext password
    let raw_file_content = fs::read_to_string(&config_file_path).unwrap();

    // Verify the file doesn't contain the plaintext password
    assert!(
        !raw_file_content.contains(&secure_data.password),
        "Config file should not contain plaintext password"
    );

    // Optional: Print file content for debugging
    // println!("File content: {}", raw_file_content);

    // Verify the file does contain some reference to the secure key
    assert!(
        raw_file_content.contains("secure_key"),
        "Config file should contain reference to the secure key"
    );

    // Load secure data using the API
    let loaded_secure: SecureTestData = prefs.get_secure("secure_key".to_string())?;

    // Verify secure data can be correctly decrypted and loaded
    assert_eq!(
        loaded_secure.username, secure_data.username,
        "Username should match"
    );
    assert_eq!(
        loaded_secure.password, secure_data.password,
        "Password should match"
    );

    cleanup_test_dir(test_dir);
    Ok(())
}

#[test]
fn test_update_preferences() -> Result<()> {
    let test_dir = setup_test_dir();

    let prefs = PreferenceConfig::new(test_dir.clone())?;

    // Initial data
    let initial_data = TestConfig {
        string_value: "initial".to_string(),
        number_value: 100,
        boolean_value: false,
    };

    // Save initial data
    prefs.save_selective("update_key".to_string(), Some(initial_data))?;

    // Updated data
    let updated_data = TestConfig {
        string_value: "updated".to_string(),
        number_value: 200,
        boolean_value: true,
    };

    // Update the data
    prefs.save_selective("update_key".to_string(), Some(updated_data.clone()))?;

    // Load updated data
    let loaded_data: TestConfig = prefs.load_selective("update_key".to_string())?;

    // Verify updated data
    assert_eq!(
        loaded_data.string_value, "updated",
        "String value should be updated"
    );
    assert_eq!(
        loaded_data.number_value, 200,
        "Number value should be updated"
    );
    assert!(loaded_data.boolean_value, "Boolean value should be updated");

    cleanup_test_dir(test_dir);
    Ok(())
}

#[test]
fn test_has_key() -> Result<()> {
    let test_dir = setup_test_dir();

    let prefs = PreferenceConfig::new(test_dir.clone())?;

    // Save test data
    prefs.save_selective("exists_key".to_string(), Some("test_value"))?;

    // Check keys
    assert!(prefs.has_key("exists_key"), "Key should exist");
    assert!(!prefs.has_key("nonexistent_key"), "Key should not exist");

    cleanup_test_dir(test_dir);
    Ok(())
}

#[test]
fn test_preference_events() -> Result<()> {
    let test_dir = setup_test_dir();

    let prefs = PreferenceConfig::new(test_dir.clone())?;

    // Get the receiver
    let receiver = prefs.get_receiver();

    // Save data to trigger an event
    let event_key = "event_key";
    prefs.save_selective(event_key.to_string(), Some("event_value"))?;

    // Small delay to ensure event is processed
    thread::sleep(Duration::from_millis(50));

    // Try to receive the event
    if let Ok((key, value)) = receiver.try_recv() {
        // The actual key is prefixed with "prefs."
        let expected_key = format!("prefs.{}", event_key);
        assert_eq!(
            key, expected_key,
            "Event key should match with 'prefs.' prefix"
        );
        assert_eq!(
            value.as_str().unwrap(),
            "event_value",
            "Event value should match"
        );
    } else {
        panic!("No event received");
    }

    cleanup_test_dir(test_dir);
    Ok(())
}

#[test]
fn test_remove_preference() -> Result<()> {
    let test_dir = setup_test_dir();

    let prefs = PreferenceConfig::new(test_dir.clone())?;

    // Save test data
    prefs.save_selective("remove_key".to_string(), Some("test_value"))?;

    // Verify key exists
    assert!(prefs.has_key("remove_key"), "Key should exist initially");

    // Remove the key
    prefs.save_selective::<String>("remove_key".to_string(), None)?;

    // Verify key no longer exists
    assert!(!prefs.has_key("remove_key"), "Key should be removed");

    cleanup_test_dir(test_dir);
    Ok(())
}
