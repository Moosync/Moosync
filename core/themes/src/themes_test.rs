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
    collections::HashMap,
    env::temp_dir,
    fs,
    sync::{Arc, Mutex},
};

use themes_proto::moosync::types::{ThemeDetails, ThemeItem};
use types::prelude::ThemeItemExt;
use uuid::Uuid;

use crate::{error::ThemesError, themes::ThemeHolder};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_theme_save_load_remove() -> Result<(), ThemesError> {
    let temp_base = temp_dir().join(format!("moosync_theme_test_{}", Uuid::new_v4()));
    let temp_theme_dir = temp_base.join("themes");
    let temp_tmp_dir = temp_base.join("tmp");

    fs::create_dir_all(&temp_theme_dir).unwrap();
    fs::create_dir_all(&temp_tmp_dir).unwrap();

    let theme_holder = ThemeHolder::new(temp_theme_dir.clone(), temp_tmp_dir.clone());
    let theme_id = "test_theme_id";

    let mut constants = HashMap::new();
    constants.insert("primary".to_string(), "#ff0000".to_string());
    constants.insert("cardWidth".to_string(), "220px".to_string());

    let theme_details = ThemeDetails {
        id: theme_id.to_string(),
        name: "Test Theme".to_string(),
        author: Some("Test Author".to_string()),
        description: Some("Test Description".to_string()),
        theme: Some(ThemeItem {
            constants,
            ..Default::default()
        }),
    };

    theme_holder.save_theme(theme_details.clone())?;

    let loaded = theme_holder.load_theme(theme_id.to_string())?;
    assert_eq!(loaded.id, theme_details.id);
    assert_eq!(loaded.name, theme_details.name);
    assert_eq!(loaded.author, theme_details.author);
    assert_eq!(loaded.description, theme_details.description);

    let loaded_item = loaded.theme.unwrap();
    assert_eq!(loaded_item.get_constant("primary").unwrap(), "#ff0000");
    assert_eq!(loaded_item.get_constant("cardWidth").unwrap(), "220px");

    let all = theme_holder.load_all_themes()?;
    assert!(all.contains_key("default"));
    assert!(all.contains_key(theme_id));

    theme_holder.remove_theme(theme_id.to_string())?;
    assert!(theme_holder.load_theme(theme_id.to_string()).is_err());

    let _ = fs::remove_dir_all(&temp_base);
    Ok(())
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_theme_subscribers() -> Result<(), ThemesError> {
    let temp_base = temp_dir().join(format!("moosync_theme_subs_{}", Uuid::new_v4()));
    let temp_theme_dir = temp_base.join("themes");
    let temp_tmp_dir = temp_base.join("tmp");

    fs::create_dir_all(&temp_theme_dir).unwrap();
    fs::create_dir_all(&temp_tmp_dir).unwrap();

    let theme_holder = ThemeHolder::new(temp_theme_dir.clone(), temp_tmp_dir.clone());

    let call_count1 = Arc::new(Mutex::new(0));
    let call_count2 = Arc::new(Mutex::new(0));

    let c1 = call_count1.clone();
    let handle1 = theme_holder.on_theme_changed(move |theme| {
        let mut count = c1.lock().unwrap();
        *count += 1;
        assert_eq!(theme.name, "Notify Test Theme");
    });

    let c2 = call_count2.clone();
    let _handle2 = theme_holder.on_theme_changed(move |theme| {
        let mut count = c2.lock().unwrap();
        *count += 1;
        assert_eq!(theme.name, "Notify Test Theme");
    });

    handle1.cancel();

    let theme_details = ThemeDetails {
        id: "notify_test".to_string(),
        name: "Notify Test Theme".to_string(),
        author: Some("Author".to_string()),
        description: Some("Desc".to_string()),
        theme: Some(ThemeItem {
            constants: HashMap::new(),
            ..Default::default()
        }),
    };

    theme_holder.save_theme(theme_details)?;

    assert_eq!(*call_count1.lock().unwrap(), 0);
    assert_eq!(*call_count2.lock().unwrap(), 1);

    let _ = fs::remove_dir_all(&temp_base);
    Ok(())
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_theme_export_import_cycle() -> Result<(), ThemesError> {
    let temp_base = temp_dir().join(format!("moosync_theme_export_{}", Uuid::new_v4()));
    let temp_theme_dir = temp_base.join("themes");
    let temp_tmp_dir = temp_base.join("tmp");
    let export_path = temp_base.join("exported_theme.mstx");

    fs::create_dir_all(&temp_theme_dir).unwrap();
    fs::create_dir_all(&temp_tmp_dir).unwrap();

    let theme_holder = ThemeHolder::new(temp_theme_dir.clone(), temp_tmp_dir.clone());
    let theme_id = "export_test_theme";

    let mut constants = HashMap::new();
    constants.insert("primary".to_string(), "#aabbcc".to_string());

    let theme_details = ThemeDetails {
        id: theme_id.to_string(),
        name: "Export Test Theme".to_string(),
        author: Some("Export Author".to_string()),
        description: Some("Export Desc".to_string()),
        theme: Some(ThemeItem {
            constants,
            ..Default::default()
        }),
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

    let _ = fs::remove_dir_all(&temp_base);
    Ok(())
}
