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

use std::{fs, sync::mpsc::channel};

use types::{
    errors::Result,
    themes::{ThemeDetails, ThemeItem},
};

use crate::themes::{transform_css, ThemeHolder};

#[test]
fn test_transformcss() -> Result<()> {
    // Use system temp directory
    let temp_dir = std::env::temp_dir();
    let root_theme = temp_dir.join("test.css");
    let subroot_theme = temp_dir.join("test1.css");

    fs::write(
        root_theme.clone(),
        "@import \"./test1.css;\"\n\n@import \"./test1.css;\"",
    )?;
    fs::write(subroot_theme.clone(), "hello1")?;

    let (res, imports) = transform_css(root_theme.to_string_lossy().to_string(), Some(temp_dir))?;

    fs::remove_file(root_theme)?;
    fs::remove_file(subroot_theme)?;

    if res == "hello1\n\nhello1" {
        return Ok(());
    }
    panic!("Invalid css transformation");
}

#[test]
fn test_transformcss_with_nested_imports() -> Result<()> {
    // Use system temp directory
    let temp_dir = std::env::temp_dir();
    let root_theme = temp_dir.join("root.css");
    let level1_theme = temp_dir.join("level1.css");
    let level2_theme = temp_dir.join("level2.css");

    fs::write(
        root_theme.clone(),
        "@import \"./level1.css;\"\n\nbody { color: red; }",
    )?;

    fs::write(
        level1_theme.clone(),
        "@import \"./level2.css;\"\n\nh1 { font-size: 20px; }",
    )?;

    fs::write(level2_theme.clone(), "p { margin: 10px; }")?;

    // Test the transformation
    let (res, imports) = transform_css(root_theme.to_string_lossy().to_string(), Some(temp_dir))?;

    // Clean up files
    fs::remove_file(root_theme)?;
    fs::remove_file(level1_theme)?;
    fs::remove_file(level2_theme)?;

    // Verify results
    assert_eq!(
        imports.len(),
        3,
        "Should have 3 imports: root, level1, and level2"
    );
    assert_eq!(
        res,
        "p { margin: 10px; }\n\nh1 { font-size: 20px; }\n\nbody { color: red; }"
    );

    Ok(())
}

#[test]
fn test_transformcss_with_theme_dir_replacement() -> Result<()> {
    // Use system temp directory
    let temp_dir = std::env::temp_dir();
    let theme_file = temp_dir.join("theme_dir_test.css");

    fs::write(
        theme_file.clone(),
        "body { background-image: url('%themeDir%/assets/bg.png'); }",
    )?;

    // Test the transformation
    let (res, _) = transform_css(theme_file.to_string_lossy().to_string(), Some(temp_dir))?;

    // Clean up
    fs::remove_file(theme_file.clone())?;

    // Get the expected parent directory
    let expected_dir = theme_file.parent().unwrap().to_str().unwrap();
    let expected_result = format!(
        "body {{ background-image: url('{}/assets/bg.png'); }}",
        expected_dir
    );

    // Verify results
    assert_eq!(
        res, expected_result,
        "Should replace %themeDir% with the actual theme directory"
    );

    Ok(())
}

#[test]
fn test_transformcss_nonexistent_file() {
    // Use system temp directory
    let temp_dir = std::env::temp_dir();
    let nonexistent_file = temp_dir.join("non_existent_file.css");

    // Test with non-existent file
    let result = transform_css(
        nonexistent_file.to_string_lossy().to_string(),
        Some(temp_dir),
    );

    // Verify it returns an error
    assert!(result.is_err(), "Should return error for non-existent file");

    if let Err(e) = result {
        let error_message = format!("{:?}", e);
        assert!(
            error_message.contains("CSS path does not exist"),
            "Error message should indicate file doesn't exist"
        );
    }
}

#[test]
fn test_transformcss_with_invalid_imports() -> Result<()> {
    // Use system temp directory
    let temp_dir = std::env::temp_dir();
    let theme_file = temp_dir.join("invalid_import.css");

    fs::write(
        theme_file.clone(),
        "@import \"./non_existent_file.css;\"\nbody { color: blue; }",
    )?;

    // Test the transformation (should fail due to invalid import)
    let result = transform_css(theme_file.to_string_lossy().to_string(), Some(temp_dir));

    // Clean up
    fs::remove_file(theme_file)?;

    // Verify it returns an error
    assert!(result.is_err(), "Should return error for invalid import");

    Ok(())
}

#[test]
fn test_theme_export_import_cycle() -> Result<()> {
    // Use system temp directory
    let temp_dir = std::env::temp_dir();
    let temp_theme_dir = temp_dir.join("temp_themes_export");
    let temp_tmp_dir = temp_dir.join("temp_tmp_export");
    let export_path = temp_dir.join("exported_theme.mstx");

    fs::create_dir_all(&temp_theme_dir)?;
    fs::create_dir_all(&temp_tmp_dir)?;

    // Setup channel for theme change notifications
    let (tx, _rx) = channel();

    // Create ThemeHolder instance
    let theme_holder = ThemeHolder::new(temp_theme_dir.clone(), temp_tmp_dir.clone(), tx);

    // Theme ID
    let theme_id = "export_test_theme";

    // Create a test theme without CSS for simplicity
    let theme_details = ThemeDetails {
        id: theme_id.to_string(),
        name: "Export Test Theme".to_string(),
        author: Some("Export Author".to_string()),
        theme: ThemeItem {
            primary: "#aabbcc".to_string(),
            secondary: "#ddeeff".to_string(),
            tertiary: "#112233".to_string(),
            text_primary: "#445566".to_string(),
            text_secondary: "#778899".to_string(),
            text_inverse: "#aabbcc".to_string(),
            accent: "#ddeeff".to_string(),
            divider: "#112233".to_string(),
            custom_css: None, // No CSS for simplicity
        },
    };

    // Save the theme
    theme_holder.save_theme(theme_details.clone())?;

    // Export the theme
    theme_holder.export_theme(theme_id.to_string(), export_path.clone())?;

    // Verify export file exists
    assert!(export_path.exists(), "Export file should exist");

    // Remove the original theme
    theme_holder.remove_theme(theme_id.to_string())?;

    // Import the theme back
    theme_holder.import_theme(export_path.to_string_lossy().to_string())?;

    // Load all themes and verify the imported theme exists
    let all_themes = theme_holder.load_all_themes()?;
    assert!(
        all_themes.values().any(|t| t.name == "Export Test Theme"),
        "Imported theme should be in all themes"
    );

    // Clean up
    fs::remove_file(&export_path)?;
    fs::remove_dir_all(&temp_theme_dir)?;
    fs::remove_dir_all(&temp_tmp_dir)?;

    Ok(())
}

#[test]
fn test_get_css_functionality() -> Result<()> {
    // Use system temp directory
    let temp_dir = std::env::temp_dir();
    let temp_theme_dir = temp_dir.join("temp_themes_css");
    let temp_tmp_dir = temp_dir.join("temp_tmp_css");

    fs::create_dir_all(&temp_theme_dir)?;
    fs::create_dir_all(&temp_tmp_dir)?;

    // Setup channel for theme change notifications
    let (tx, _rx) = channel();

    // Create ThemeHolder instance
    let theme_holder = ThemeHolder::new(temp_theme_dir.clone(), temp_tmp_dir.clone(), tx);

    // Create a test CSS file
    let css_content = "body { color: green; }";
    let css_path = temp_theme_dir.join("css_test_theme").join("style.css");
    fs::create_dir_all(css_path.parent().unwrap())?;
    fs::write(&css_path, css_content)?;

    // Create a test theme
    let theme_details = ThemeDetails {
        id: "css_test_theme".to_string(),
        name: "CSS Test Theme".to_string(),
        author: Some("CSS Author".to_string()),
        theme: ThemeItem {
            primary: "#123456".to_string(),
            secondary: "#234567".to_string(),
            tertiary: "#345678".to_string(),
            text_primary: "#456789".to_string(),
            text_secondary: "#56789a".to_string(),
            text_inverse: "#6789ab".to_string(),
            accent: "#789abc".to_string(),
            divider: "#89abcd".to_string(),
            custom_css: Some("style.css".to_string()),
        },
    };

    // Save the theme
    theme_holder.save_theme(theme_details.clone())?;

    // Get CSS
    let css = theme_holder.get_css("css_test_theme".to_string())?;

    // Verify CSS content
    assert_eq!(css, css_content, "CSS content should match");

    // Clean up
    fs::remove_dir_all(temp_theme_dir)?;
    fs::remove_dir_all(temp_tmp_dir)?;

    Ok(())
}
