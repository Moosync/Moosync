/// Error type for the types crate itself.
#[derive(Debug, thiserror::Error)]
pub enum TypesError {
    #[error("Negative protobuf duration")]
    NegativeDuration,
}
