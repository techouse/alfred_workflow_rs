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

    #[error("cache max entries must be greater than zero, got {entries}")]
    InvalidMaxCacheEntries { entries: usize },

    #[error("failed to access cache: {0}")]
    Cache(String),

    #[error(
        "GitHub repository URL must be hosted on github.com and include owner/repo, got `{url}`"
    )]
    InvalidGithubRepositoryUrl { url: String },

    #[error("failed to parse version")]
    Version(#[from] semver::Error),

    #[error("failed to parse URL")]
    Url(#[from] url::ParseError),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("failed to render JSON")]
    Json(#[from] serde_json::Error),

    #[error("failed to write output")]
    Io(#[from] std::io::Error),
}
