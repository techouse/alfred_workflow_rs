pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("automatic cache seconds must be between {min} and {max}, got {seconds}")]
    InvalidAutomaticCacheSeconds { seconds: u64, min: u64, max: u64 },

    #[error("typed action must include at least one of text, url, file, or auto")]
    EmptyTypedAction,

    #[error("item cannot include both arg and action")]
    ArgAndAction,

    #[error("modifier key set cannot be empty")]
    EmptyModifierKeySet,

    #[error("unknown modifier key `{0}`")]
    UnknownModifierKey(String),

    #[error("failed to render JSON")]
    Json(#[from] serde_json::Error),

    #[error("failed to write output")]
    Io(#[from] std::io::Error),
}
