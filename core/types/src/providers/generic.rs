use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
    pub token: Option<String>,
    pub is_first: bool,
    pub is_valid: bool,
}

impl Pagination {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_limit(limit: u32, offset: u32) -> Self {
        Pagination {
            limit,
            offset,
            is_first: true,
            is_valid: true,
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_token(token: Option<String>) -> Self {
        Pagination {
            token,
            is_first: true,
            is_valid: true,
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn next_page(&self) -> Self {
        Pagination {
            limit: self.limit,
            offset: self.offset + self.limit.max(1),
            token: self.token.clone(),
            is_first: false,
            is_valid: true,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn next_page_wtoken(&self, token: Option<String>) -> Self {
        Pagination {
            limit: self.limit,
            offset: self.offset + self.limit,
            token,
            is_first: false,
            is_valid: true,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn invalidate(&mut self) { self.is_valid = false; }
}
