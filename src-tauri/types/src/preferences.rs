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
