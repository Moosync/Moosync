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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ThemeItem {
    pub primary: String,
    pub secondary: String,
    pub tertiary: String,
    #[serde(rename = "textPrimary")]
    pub text_primary: String,
    #[serde(rename = "textSecondary")]
    pub text_secondary: String,
    #[serde(rename = "textInverse")]
    pub text_inverse: String,
    pub accent: String,
    pub divider: String,
    #[serde(rename = "customCSS")]
    pub custom_css: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ThemeDetails {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub theme: ThemeItem,
}

impl ThemeDetails {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> Self {
        Self {
            name: "New theme".into(),
            ..Default::default()
        }
    }
}

impl Default for ThemeDetails {
    #[tracing::instrument(level = "debug", skip())]
    fn default() -> Self {
        Self {
            id: "default".into(),
            name: "Default".into(),
            author: Some("Moosync".into()),
            theme: ThemeItem {
                primary: "#212121".into(),
                secondary: "#282828".into(),
                tertiary: "#151515".into(),
                text_primary: "#ffffff".into(),
                text_secondary: "#565656".into(),
                text_inverse: "#000000".into(),
                accent: "#65CB88".into(),
                divider: "rgba(79, 79, 79, 0.67)".into(),
                custom_css: None,
            },
        }
    }
}
