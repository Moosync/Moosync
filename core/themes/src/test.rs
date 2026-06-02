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

use std::{fs, sync::{Arc, Mutex}};

use themes_proto::moosync::types::{ThemeDetails, ThemeItem};
use types::errors::Result;

use crate::themes::ThemeHolder;

#[test]
fn test_theme_save_load() -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let temp_theme_dir = temp_dir.join("temp_themes_save_load");
    let temp_tmp_dir = temp_dir.join("temp_tmp_save_load");

    fs::create_dir_all(&temp_theme_dir).unwrap();
    fs::create_dir_all(&temp_tmp_dir).unwrap();

    let theme_holder = ThemeHolder::new(temp_theme_dir.clone(), temp_tmp_dir.clone());
    let theme_id = "test_theme_id";

    let mut constants = std::collections::HashMap::new();
    constants.insert("primary".to_string(), "#ff0000".to_string());
    constants.insert("cardWidth".to_string(), "220px".to_string());

    let theme_details = ThemeDetails {
        id: theme_id.to_string(),
        name: "Test Theme".to_string(),
        author: Some("Test Author".to_string()),
        description: Some("Test Description".to_string()),
        theme: Some(ThemeItem { constants, ..Default::default() }),
    };

    theme_holder.save_theme(theme_details.clone())?;

    let loaded = theme_holder.load_theme(theme_id.to_string())?;
    assert_eq!(loaded.id, theme_details.id);
    assert_eq!(loaded.name, theme_details.name);
    assert_eq!(loaded.author, theme_details.author);
    assert_eq!(loaded.description, theme_details.description);
    
    let loaded_item = loaded.theme.unwrap();
    use types::prelude::ThemeItemExt;
    assert_eq!(loaded_item.get_constant("primary").unwrap(), "#ff0000");
    assert_eq!(loaded_item.get_constant("cardWidth").unwrap(), "220px");

    fs::remove_dir_all(&temp_theme_dir).unwrap();
    fs::remove_dir_all(&temp_tmp_dir).unwrap();
    Ok(())
}

#[test]
fn test_theme_subscribers() -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let temp_theme_dir = temp_dir.join("temp_themes_subs");
    let temp_tmp_dir = temp_dir.join("temp_tmp_subs");

    fs::create_dir_all(&temp_theme_dir).unwrap();
    fs::create_dir_all(&temp_tmp_dir).unwrap();

    let theme_holder = ThemeHolder::new(temp_theme_dir.clone(), temp_tmp_dir.clone());

    let call_count1 = Arc::new(Mutex::new(0));
    let call_count2 = Arc::new(Mutex::new(0));

    let c1 = call_count1.clone();
    theme_holder.on_theme_changed(move |theme| {
        let mut count = c1.lock().unwrap();
        *count += 1;
        assert_eq!(theme.name, "Notify Test Theme");
    });

    let c2 = call_count2.clone();
    theme_holder.on_theme_changed(move |theme| {
        let mut count = c2.lock().unwrap();
        *count += 1;
        assert_eq!(theme.name, "Notify Test Theme");
    });

    let theme_details = ThemeDetails {
        id: "notify_test".to_string(),
        name: "Notify Test Theme".to_string(),
        author: Some("Author".to_string()),
        description: Some("Desc".to_string()),
        theme: Some(ThemeItem { constants: std::collections::HashMap::new(), ..Default::default() }),
    };

    theme_holder.save_theme(theme_details)?;

    assert_eq!(*call_count1.lock().unwrap(), 1);
    assert_eq!(*call_count2.lock().unwrap(), 1);

    fs::remove_dir_all(&temp_theme_dir).unwrap();
    fs::remove_dir_all(&temp_tmp_dir).unwrap();
    Ok(())
}

#[test]
fn test_theme_export_import_cycle() -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let temp_theme_dir = temp_dir.join("temp_themes_export");
    let temp_tmp_dir = temp_dir.join("temp_tmp_export");
    let export_path = temp_dir.join("exported_theme.mstx");

    fs::create_dir_all(&temp_theme_dir).unwrap();
    fs::create_dir_all(&temp_tmp_dir).unwrap();

    let theme_holder = ThemeHolder::new(temp_theme_dir.clone(), temp_tmp_dir.clone());
    let theme_id = "export_test_theme";

    let mut constants = std::collections::HashMap::new();
    constants.insert("primary".to_string(), "#aabbcc".to_string());

    let theme_details = ThemeDetails {
        id: theme_id.to_string(),
        name: "Export Test Theme".to_string(),
        author: Some("Export Author".to_string()),
        description: Some("Export Desc".to_string()),
        theme: Some(ThemeItem { constants, ..Default::default() }),
    };

    theme_holder.save_theme(theme_details.clone())?;

    theme_holder.export_theme(theme_id.to_string(), export_path.clone())?;
    assert!(export_path.exists(), "Export file should exist");

    theme_holder.remove_theme(theme_id.to_string())?;

    theme_holder.import_theme(export_path.to_string_lossy().to_string())?;

    let all_themes = theme_holder.load_all_themes()?;
    assert!(
        all_themes.values().any(|t| t.name == "Export Test Theme"),
        "Imported theme should be in all themes"
    );

    fs::remove_file(&export_path).unwrap();
    fs::remove_dir_all(&temp_theme_dir).unwrap();
    fs::remove_dir_all(&temp_tmp_dir).unwrap();

    Ok(())
}

#[test]
fn test_theme_backwards_compatibility() -> Result<()> {
    use types::prelude::ThemeItemExt;

    // 1. Old theme format (colors and customCss at the root level)
    let old_json = r##"{
        "id": "old_theme",
        "name": "Old Theme",
        "author": "Old Author",
        "description": "Old Description",
        "theme": {
            "primary": "#111111",
            "secondary": "#222222",
            "tertiary": "#333333",
            "textPrimary": "#444444",
            "textSecondary": "#555555",
            "textInverse": "#666666",
            "accent": "#777777",
            "divider": "#888888",
            "customCss": "body { background: red; }"
        }
    }"##;

    let loaded_old: ThemeDetails = serde_json::from_str(old_json).unwrap();
    assert_eq!(loaded_old.id, "old_theme");
    assert_eq!(loaded_old.name, "Old Theme");
    assert_eq!(loaded_old.author.as_deref(), Some("Old Author"));
    assert_eq!(loaded_old.description.as_deref(), Some("Old Description"));

    let theme_item = loaded_old.theme.unwrap();
    assert_eq!(theme_item.get_constant("primary").unwrap(), "#111111");
    assert_eq!(theme_item.get_constant("secondary").unwrap(), "#222222");
    assert_eq!(theme_item.get_constant("tertiary").unwrap(), "#333333");
    assert_eq!(theme_item.get_constant("textPrimary").unwrap(), "#444444");
    assert_eq!(theme_item.get_constant("textSecondary").unwrap(), "#555555");
    assert_eq!(theme_item.get_constant("textInverse").unwrap(), "#666666");
    assert_eq!(theme_item.get_constant("accent").unwrap(), "#777777");
    assert_eq!(theme_item.get_constant("divider").unwrap(), "#888888");
    assert_eq!(theme_item.custom_css.as_deref(), Some("body { background: red; }"));

    // 2. Transitional theme format (colors in constants map)
    let transitional_json = r##"{
        "id": "transitional_theme",
        "name": "Transitional Theme",
        "theme": {
            "constants": {
                "primary": "#123456",
                "accent": "#654321",
                "cardWidth": "240px"
            }
        }
    }"##;

    let loaded_transitional: ThemeDetails = serde_json::from_str(transitional_json).unwrap();
    let trans_item = loaded_transitional.theme.unwrap();
    assert_eq!(trans_item.get_constant("primary").unwrap(), "#123456");
    assert_eq!(trans_item.get_constant("accent").unwrap(), "#654321");
    assert_eq!(trans_item.get_constant("cardWidth").unwrap(), "240px");

    // 3. Modifying and saving a theme puts colors at root, others in map
    let mut item_to_save = trans_item;
    item_to_save.set_constant("primary", "#abcdef".to_string());
    item_to_save.set_constant("cardWidth", "250px".to_string());

    assert_eq!(item_to_save.primary, "#abcdef");
    assert_eq!(item_to_save.get_constant("cardWidth").unwrap(), "250px");

    Ok(())
}

