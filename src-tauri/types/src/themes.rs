use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeItem {
    pub primary: String,
    pub secondary: String,
    pub tertiary: String,
    pub textPrimary: String,
    pub textSecondary: String,
    pub textInverse: String,
    pub accent: String,
    pub divider: String,
    pub customCSS: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeDetails {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub theme: ThemeItem,
}
