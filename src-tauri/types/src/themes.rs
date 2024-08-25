use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeDetails {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub theme: ThemeItem,
}
