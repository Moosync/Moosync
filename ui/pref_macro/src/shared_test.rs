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

use crate::shared::{generate_expansion, parse_yaml};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_yaml_preferences() {
    let yaml = r#"
preferences:
  - id: music_paths
    component: PathSelector
    title: Music Folders
    subtitle: Folders to scan for audio files
  - id: theme_mode
    component: RadioGroup
    title: Theme Mode
    subtitle: Select dark or light
    options:
      - id: dark
        label: Dark Mode
      - id: light
        label: Light Mode
"#;

    let prefs = parse_yaml(yaml);
    assert_eq!(prefs.len(), 2);
    assert_eq!(prefs[0].id, "music_paths");
    assert_eq!(prefs[0].component, "PathSelector");
    assert_eq!(prefs[0].title, "Music Folders");

    assert_eq!(prefs[1].id, "theme_mode");
    assert_eq!(prefs[1].options_radio.len(), 2);
    assert_eq!(prefs[1].options_radio[0].0, "dark");
    assert_eq!(prefs[1].options_radio[0].1, "Dark Mode");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_generate_expansion_code() {
    let yaml = r#"
preferences:
  - id: enable_scrobbling
    component: ToggleGroup
    title: Enable Scrobbling
    subtitle: Send plays to Last.fm
"#;

    let ident = syn::Ident::new("system_items", proc_macro2::Span::call_site());
    let token_stream = generate_expansion(yaml, &ident);
    let code = token_stream.to_string();

    assert!(code.contains("TempPrefItem"));
    assert!(code.contains("enable_scrobbling"));
    assert!(code.contains("handle_change"));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_yaml_dropdown_and_edge_cases() {
    let yaml = r#"
# Comment header
preferences:
  - id: "audio_device"
    component: 'Dropdown'
    title: "Output Device: Default"
    subtitle: "Select device (1, 2, 3)"
    options:
      - "Device 1 (48000Hz)"
      - "Device 2: USB Audio"
  - id: empty_pref
"#;

    let prefs = parse_yaml(yaml);
    assert_eq!(prefs.len(), 2);
    assert_eq!(prefs[0].id, "audio_device");
    assert_eq!(prefs[0].component, "Dropdown");
    assert_eq!(prefs[0].title, "Output Device: Default");
    assert_eq!(prefs[0].options_dropdown.len(), 2);
    assert_eq!(prefs[0].options_dropdown[0], "Device 1 (48000Hz)");
    assert_eq!(prefs[0].options_dropdown[1], "Device 2: USB Audio");

    assert_eq!(prefs[1].id, "empty_pref");

    // Empty input returns empty vec
    let empty_prefs = parse_yaml("");
    assert!(empty_prefs.is_empty());
}
