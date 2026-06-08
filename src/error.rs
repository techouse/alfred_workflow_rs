/// Convenient crate result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type used by all fallible crate APIs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Automatic cache seconds were outside Alfred's accepted range.
    #[error("automatic cache seconds must be between {min} and {max}, got {seconds}")]
    InvalidAutomaticCacheSeconds {
        /// Provided seconds value.
        seconds: u64,
        /// Inclusive minimum accepted seconds.
        min: u64,
        /// Inclusive maximum accepted seconds.
        max: u64,
    },

    /// Typed action had no `text`, `url`, `file`, or `auto` value.
    #[error("typed action must include at least one of text, url, file, or auto")]
    EmptyTypedAction,

    /// An item tried to contain both `arg` and `action`.
    #[error("item cannot include both arg and action")]
    ArgAndAction,

    /// A modifier key set was empty.
    #[error("modifier key set cannot be empty")]
    EmptyModifierKeySet,

    /// A modifier key was not one of Alfred's supported keys.
    #[error("unknown modifier key `{0}`")]
    UnknownModifierKey(String),

    /// File cache max entries was zero.
    #[error("cache max entries must be greater than zero, got {entries}")]
    InvalidMaxCacheEntries {
        /// Provided max-entry count.
        entries: usize,
    },

    /// File cache backend or metadata access failed.
    #[error("failed to access cache: {0}")]
    Cache(String),

    /// Plist user configuration parsing failed.
    #[error("failed to parse user configuration plist: {0}")]
    UserConfiguration(String),

    /// GitHub updater repository URL was not a valid `github.com/owner/repo` URL.
    #[error(
        "GitHub repository URL must be hosted on github.com and include owner/repo, got `{url}`"
    )]
    InvalidGithubRepositoryUrl {
        /// Provided repository URL.
        url: String,
    },

    /// GitHub updater current version was not a strict semantic version.
    #[error("current version must be a strict semantic version, got `{version}`")]
    InvalidCurrentVersion {
        /// Provided current version.
        version: String,
    },

    /// Version parsing failed.
    #[error("failed to parse version")]
    Version(#[from] semver::Error),

    /// URL parsing failed.
    #[error("failed to parse URL")]
    Url(#[from] url::ParseError),

    /// HTTP request or response body handling failed.
    #[error("HTTP request failed: {0}")]
    Http(String),

    /// JSON rendering or parsing failed.
    #[error("failed to render JSON")]
    Json(#[from] serde_json::Error),

    /// Filesystem, process, or output writer operation failed.
    #[error("filesystem, process, or output writer operation failed: {0}")]
    Io(#[from] std::io::Error),
}
