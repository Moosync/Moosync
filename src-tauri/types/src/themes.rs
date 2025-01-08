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
    #[tracing::instrument(level = "trace", skip())]
    #[cfg(any(feature = "core", feature = "ui"))]
    pub fn new() -> Self {
        Self {
            name: "New theme".into(),
            id: uuid::Uuid::new_v4().to_string(),
            ..Default::default()
        }
    }
}

impl Default for ThemeDetails {
    #[tracing::instrument(level = "trace", skip())]
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
