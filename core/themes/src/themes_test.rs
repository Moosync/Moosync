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
    sync::{Arc, Mutex},
};

use assertables::{assert_err, assert_ok, assert_some_eq_x};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use themes_proto::moosync::types::{ThemeDetails, ThemeItem};
use tracing_test::traced_test;
use types::prelude::ThemeItemExt;

use crate::themes::ThemeHolder;

struct TestThemeContext {
    pub _temp_dir: TempDir,
    pub theme_holder: ThemeHolder,
}

#[fixture]
fn theme_context() -> TestThemeContext {
    let temp_dir = TempDir::new("moosync_theme_test").expect("failed to create temp dir");
    let theme_dir = temp_dir.path().join("themes");
    let tmp_dir = temp_dir.path().join("tmp");
    std::fs::create_dir_all(&theme_dir).unwrap();
    std::fs::create_dir_all(&tmp_dir).unwrap();

    let theme_holder = ThemeHolder::new(theme_dir, tmp_dir);
    TestThemeContext {
        _temp_dir: temp_dir,
        theme_holder,
    }
}

#[fixture]
fn sample_theme_details() -> ThemeDetails {
    let mut constants = HashMap::new();
    constants.insert("primary".to_string(), "#ff0000".to_string());
    constants.insert("cardWidth".to_string(), "220px".to_string());

    ThemeDetails {
        id: "test_theme_id".to_string(),
        name: "Test Theme".to_string(),
        author: Some("Test Author".to_string()),
        description: Some("Test Description".to_string()),
        theme: Some(ThemeItem {
            constants,
            ..Default::default()
        }),
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_theme_save_load_remove(
    theme_context: TestThemeContext,
    sample_theme_details: ThemeDetails,
) {
    let TestThemeContext { theme_holder, .. } = theme_context;
    let theme_id = "test_theme_id";

    assert_ok!(theme_holder.save_theme(sample_theme_details.clone()));

    let loaded_res = theme_holder.load_theme(theme_id.to_string());
    assert_ok!(loaded_res.as_ref());
    let loaded = loaded_res.unwrap();
    assert_eq!(loaded.id, sample_theme_details.id);
    assert_eq!(loaded.name, sample_theme_details.name);
    assert_eq!(loaded.author, sample_theme_details.author);
    assert_eq!(loaded.description, sample_theme_details.description);
    let loaded_item = loaded.theme.unwrap();
    assert_some_eq_x!(loaded_item.get_constant("primary"), "#ff0000");
    assert_some_eq_x!(loaded_item.get_constant("cardWidth"), "220px");

    let all_res = theme_holder.load_all_themes();
    assert_ok!(all_res.as_ref());
    let all = all_res.unwrap();
    assert!(all.contains_key("default"));
    assert!(all.contains_key(theme_id));

    assert_ok!(theme_holder.remove_theme(theme_id.to_string()));
    assert_err!(theme_holder.load_theme(theme_id.to_string()));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_theme_subscribers(theme_context: TestThemeContext) {
    let TestThemeContext { theme_holder, .. } = theme_context;
    let call_count1 = Arc::new(Mutex::new(0));
    let call_count2 = Arc::new(Mutex::new(0));

    let count1_clone = call_count1.clone();
    let handle1 = theme_holder.on_theme_changed(move |theme| {
        *count1_clone.lock().unwrap() += 1;
        assert_eq!(theme.name, "Notify Test Theme");
    });
    let count2_clone = call_count2.clone();
    let _handle2 = theme_holder.on_theme_changed(move |theme| {
        *count2_clone.lock().unwrap() += 1;
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

    assert_ok!(theme_holder.save_theme(theme_details));
    assert_eq!(*call_count1.lock().unwrap(), 0);
    assert_eq!(*call_count2.lock().unwrap(), 1);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_theme_export_import_cycle(theme_context: TestThemeContext) {
    let TestThemeContext {
        _temp_dir,
        theme_holder,
    } = theme_context;
    let export_path = _temp_dir.path().join("exported_theme.mstx");
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

    assert_ok!(theme_holder.save_theme(theme_details));
    assert_ok!(theme_holder.export_theme(theme_id.to_string(), export_path.clone()));
    assert!(export_path.exists());

    assert_ok!(theme_holder.remove_theme(theme_id.to_string()));
    assert_ok!(theme_holder.import_theme(export_path.to_string_lossy().to_string()));

    let all_themes_res = theme_holder.load_all_themes();
    assert_ok!(all_themes_res.as_ref());
    let all_themes = all_themes_res.unwrap();
    assert!(
        all_themes
            .values()
            .any(|theme| theme.name == "Export Test Theme")
    );
}
