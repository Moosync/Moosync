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

// Moosync
// Copyright (C) 2025 Moosync
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
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use std::hash::Hasher;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CheckboxPreference {
    pub key: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsValue {
    pub enabled: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum PreferenceTypes {
    DirectoryGroup,
    EditText,
    FilePicker,
    CheckboxGroup,
    ThemeSelector,
    Extensions,
    ButtonGroup,
    ProgressBar,
    TextField,
    InfoField,
    Dropdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceUIFile {
    pub page: Vec<Page>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Page {
    pub data: Vec<PreferenceUIData>,
    pub title: String,
    pub path: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    Text,
    Number,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferenceUIData {
    #[serde(rename = "type")]
    pub _type: PreferenceTypes,
    pub title: String,
    pub key: String,
    pub description: String,
    pub input_type: Option<InputType>,
    pub single: Option<bool>,
    pub items: Option<Vec<CheckboxItems>>,
    pub default: Option<Value>,
    pub mobile: Option<bool>,
}

impl PartialEq for PreferenceUIData {
    #[tracing::instrument(level = "trace", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl std::cmp::Eq for PreferenceUIData {}

impl std::hash::Hash for PreferenceUIData {
    #[tracing::instrument(level = "trace", skip(self, state))]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state)
    }
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct CheckboxItems {
    pub title: String,
    pub key: String,
}
