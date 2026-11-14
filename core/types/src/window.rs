use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DialogFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileResponse {
    pub name: String,
    pub path: String,
    pub size: usize,
}
