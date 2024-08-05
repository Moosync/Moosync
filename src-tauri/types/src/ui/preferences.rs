use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsValue {
    pub enabled: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PreferenceTypes {
    Paths,
    Text,
    Number,
    FilePicker,
    Checkbox,
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
pub struct PreferenceUIData {
    #[serde(rename = "type")]
    pub _type: PreferenceTypes,
    pub name: String,
    pub key: String,
    pub tooltip: String,
    pub single: Option<bool>,
    pub items: Option<Vec<CheckboxItems>>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CheckboxItems {
    pub name: String,
    pub key: String,
}
