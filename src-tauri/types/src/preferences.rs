use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CheckboxPreference {
    pub key: String,
    pub enabled: bool,
}
