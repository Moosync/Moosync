use serde_json::Value;

#[derive(Debug)]
pub enum MoosyncError {
    String(String),
}

impl From<wasm_bindgen::JsValue> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let parsed: Value = serde_wasm_bindgen::from_value(value).unwrap();
        Self::String(format!("{}", parsed))
    }
}

impl From<serde_wasm_bindgen::Error> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(err))]
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        Self::String(format!("serde_wasm_bindgen::Error: {}", err))
    }
}

impl std::fmt::Display for MoosyncError {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoosyncError::String(s) => write!(f, "{}", s),
        }
    }
}

pub type Result<T> = std::result::Result<T, MoosyncError>;
