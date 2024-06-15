#[derive(Debug, Default, Clone)]
pub struct ProviderStatus {
    pub key: String,
    pub name: String,
    pub user_name: Option<String>,
    pub logged_in: bool,
}

impl ProviderStatus {
    pub fn with_name(name: &str) -> Self {
        Self { key: name.to_ascii_lowercase(), name: name.to_string(), user_name: None, logged_in: false }
    }
}