use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use crate::{Error, FileCache, Result};

/// GitHub release author/uploader payload.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GithubUser {
    /// GitHub login.
    pub login: String,
    /// Numeric GitHub user ID.
    pub id: u64,
    /// GitHub GraphQL node ID.
    pub node_id: String,
    /// Avatar URL.
    pub avatar_url: Url,
    /// Gravatar ID.
    pub gravatar_id: String,
    /// API URL.
    pub url: Url,
    /// Browser URL.
    pub html_url: Url,
    /// Repositories API URL.
    pub repos_url: Url,
    /// GitHub user type.
    #[serde(rename = "type")]
    pub user_type: String,
    /// Whether the user is a site admin.
    pub site_admin: bool,
}

/// GitHub release asset payload.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GithubAsset {
    /// Asset API URL.
    pub url: Url,
    /// Numeric asset ID.
    pub id: u64,
    /// GitHub GraphQL node ID.
    pub node_id: String,
    /// Asset file name.
    pub name: String,
    /// Optional asset label.
    pub label: Option<String>,
    /// Uploading GitHub user.
    pub uploader: GithubUser,
    /// Asset content type.
    pub content_type: String,
    /// Asset state.
    pub state: String,
    /// Asset size in bytes.
    pub size: u64,
    /// Asset download count.
    pub download_count: u64,
    /// Asset creation timestamp.
    pub created_at: String,
    /// Asset update timestamp.
    pub updated_at: String,
    /// Browser download URL.
    pub browser_download_url: Url,
}

/// GitHub release payload.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GithubRelease {
    /// Release API URL.
    pub url: Url,
    /// Release assets API URL.
    pub assets_url: Url,
    /// Upload URL template.
    pub upload_url: Url,
    /// Browser URL.
    pub html_url: Url,
    /// Numeric release ID.
    pub id: u64,
    /// Release author.
    pub author: GithubUser,
    /// GitHub GraphQL node ID.
    pub node_id: String,
    /// Release tag parsed as a semantic version.
    #[serde(with = "version_tag")]
    pub tag_name: Version,
    /// Target branch or commit.
    pub target_commitish: String,
    /// Release name.
    pub name: String,
    /// Whether the release is a draft.
    pub draft: bool,
    /// Whether the release is a prerelease.
    pub prerelease: bool,
    /// Release creation timestamp.
    pub created_at: String,
    /// Release publication timestamp.
    pub published_at: String,
    /// Release assets.
    pub assets: Vec<GithubAsset>,
    /// Tarball URL.
    pub tarball_url: Url,
    /// Zipball URL.
    pub zipball_url: Url,
    /// Optional release body.
    pub body: Option<String>,
}

/// Opens a downloaded workflow update.
pub trait Opener: Send + Sync {
    /// Opens the downloaded `.alfredworkflow` file.
    fn open(&self, path: &Path) -> Result<()>;
}

/// Opener that delegates to a system command, defaulting to macOS `open`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOpener {
    command: OsString,
}

impl Default for CommandOpener {
    fn default() -> Self {
        Self {
            command: OsString::from("open"),
        }
    }
}

impl CommandOpener {
    /// Creates an opener that delegates to the macOS `open` command.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an opener that delegates to a custom command program.
    pub fn with_command(command: impl Into<OsString>) -> Self {
        Self {
            command: command.into(),
        }
    }
}

impl Opener for CommandOpener {
    fn open(&self, path: &Path) -> Result<()> {
        let status = std::process::Command::new(&self.command).arg(path).status()?;
        if !status.success() {
            return Err(std::io::Error::other(format!(
                "{} command failed with status {status}",
                self.command.to_string_lossy()
            ))
            .into());
        }

        Ok(())
    }
}

/// GitHub release updater for Alfred workflow bundles.
#[derive(Clone)]
pub struct Updater {
    github_repository_url: Url,
    current_version: Version,
    update_interval: Duration,
    file_cache: FileCache<GithubRelease>,
    http_agent: ureq::Agent,
    github_api_base_url: Url,
    download_directory: Option<PathBuf>,
    opener: Arc<dyn Opener>,
}

impl std::fmt::Debug for Updater {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Updater")
            .field("github_repository_url", &self.github_repository_url)
            .field("current_version", &self.current_version)
            .field("update_interval", &self.update_interval)
            .field("file_cache", &self.file_cache)
            .field("github_api_base_url", &self.github_api_base_url)
            .field("download_directory", &self.download_directory)
            .finish_non_exhaustive()
    }
}

impl PartialEq for Updater {
    fn eq(&self, other: &Self) -> bool {
        self.github_repository_url == other.github_repository_url
            && self.current_version == other.current_version
            && self.update_interval == other.update_interval
            && self.file_cache == other.file_cache
            && self.github_api_base_url == other.github_api_base_url
            && self.download_directory == other.download_directory
    }
}

impl Eq for Updater {}

impl Updater {
    /// Cache key used for latest-release metadata.
    pub const UPDATE_KEY: &'static str = "update";
    /// Cache name used for latest-release metadata.
    pub const UPDATE_CACHE_NAME: &'static str = "update_cache";

    /// Creates an updater with default configuration.
    pub fn new(github_repository_url: Url, current_version: &str) -> Result<Self> {
        Self::builder(github_repository_url, current_version)?.build()
    }

    /// Creates an updater builder.
    pub fn builder(github_repository_url: Url, current_version: &str) -> Result<UpdaterBuilder> {
        validate_repository_url(&github_repository_url)?;

        Ok(UpdaterBuilder {
            github_repository_url,
            current_version: parse_current_version(current_version)?,
            update_interval: Duration::ZERO,
            file_cache: None,
            github_api_base_url: Url::parse("https://api.github.com")?,
            download_directory: None,
            opener: Arc::new(CommandOpener::new()),
        })
    }

    /// Returns the hashed update cache key.
    pub fn update_cache_key() -> String {
        FileCache::<GithubRelease>::hash_key(Self::UPDATE_KEY)
    }

    /// Returns the GitHub repository URL.
    pub fn github_repository_url(&self) -> &Url {
        &self.github_repository_url
    }

    /// Returns the current workflow version.
    pub fn current_version(&self) -> &Version {
        &self.current_version
    }

    /// Returns the update check interval.
    pub fn update_interval(&self) -> Duration {
        self.update_interval
    }

    /// Returns the release metadata cache.
    pub fn file_cache(&self) -> &FileCache<GithubRelease> {
        &self.file_cache
    }

    /// Returns whether a newer GitHub release is available.
    pub fn update_available(&self) -> Result<bool> {
        let cache_key = Self::update_cache_key();
        if let Some(cached_release) = self.file_cache.get(&cache_key)? {
            return Ok(cached_release.tag_name > self.current_version);
        }

        if let Some(release) = self.fetch_latest_release()? {
            let update_available = release.tag_name > self.current_version;
            self.file_cache.put(&cache_key, release)?;
            return Ok(update_available);
        }

        Ok(false)
    }

    /// Fetches the latest GitHub release.
    pub fn fetch_latest_release(&self) -> Result<Option<GithubRelease>> {
        let url = self.latest_release_url()?;
        let mut response = match self.http_agent.get(url.as_str()).call() {
            Ok(response) => response,
            Err(ureq::Error::StatusCode(status)) if status >= 400 => return Ok(None),
            Err(error) => return Err(Error::Http(error.to_string())),
        };

        let body = response.body_mut().read_to_string().map_err(http_error)?;
        Ok(Some(serde_json::from_str(&body)?))
    }

    /// Finds the first `.alfredworkflow` asset in a release.
    pub fn find_alfred_workflow_asset<'a>(
        &self,
        release: &'a GithubRelease,
    ) -> Option<&'a GithubAsset> {
        release
            .assets
            .iter()
            .find(|asset| asset.name.ends_with(".alfredworkflow"))
    }

    /// Downloads a release asset and returns its local path.
    pub fn download_asset(&self, asset: &GithubAsset) -> Result<Option<PathBuf>> {
        let file_name = safe_asset_file_name(&asset.name)?;
        let mut response = match self
            .http_agent
            .get(asset.browser_download_url.as_str())
            .call()
        {
            Ok(response) => response,
            Err(ureq::Error::StatusCode(status)) if status >= 400 => return Ok(None),
            Err(error) => return Err(Error::Http(error.to_string())),
        };

        let directory = match &self.download_directory {
            Some(directory) => directory.clone(),
            None => unique_temp_directory(),
        };
        std::fs::create_dir_all(&directory)?;

        let path = directory.join(file_name);
        let bytes = response.body_mut().read_to_vec().map_err(http_error)?;
        std::fs::write(&path, bytes)?;

        Ok(Some(path))
    }

    /// Downloads and opens an available workflow update.
    pub fn update(&self) -> Result<()> {
        if !self.update_available()? {
            return Ok(());
        }

        let cache_key = Self::update_cache_key();
        let release = match self.file_cache.get(&cache_key)? {
            Some(release) => release,
            None => match self.fetch_latest_release()? {
                Some(release) => release,
                None => return Ok(()),
            },
        };

        if release.tag_name <= self.current_version {
            return Ok(());
        }

        if let Some(asset) = self.find_alfred_workflow_asset(&release)
            && let Some(path) = self.download_asset(asset)?
        {
            self.opener.open(&path)?;
        }

        Ok(())
    }

    fn latest_release_url(&self) -> Result<Url> {
        let repository_path = repository_path(&self.github_repository_url)?;
        self.github_api_base_url
            .join(&format!("/repos/{repository_path}/releases/latest"))
            .map_err(Into::into)
    }
}

/// Builder for [`Updater`] configuration.
pub struct UpdaterBuilder {
    github_repository_url: Url,
    current_version: Version,
    update_interval: Duration,
    file_cache: Option<FileCache<GithubRelease>>,
    github_api_base_url: Url,
    download_directory: Option<PathBuf>,
    opener: Arc<dyn Opener>,
}

impl UpdaterBuilder {
    /// Sets the release check cache TTL.
    pub fn update_interval(mut self, update_interval: Duration) -> Self {
        self.update_interval = update_interval;
        self
    }

    /// Sets a custom release metadata cache.
    pub fn file_cache(mut self, file_cache: FileCache<GithubRelease>) -> Self {
        self.file_cache = Some(file_cache);
        self
    }

    /// Sets a custom GitHub API base URL.
    pub fn github_api_base_url(mut self, github_api_base_url: Url) -> Self {
        self.github_api_base_url = github_api_base_url;
        self
    }

    /// Sets the directory for downloaded workflow assets.
    pub fn download_directory(mut self, download_directory: impl Into<PathBuf>) -> Self {
        self.download_directory = Some(download_directory.into());
        self
    }

    /// Sets the opener used for downloaded workflow assets.
    pub fn opener<O>(mut self, opener: O) -> Self
    where
        O: Opener + 'static,
    {
        self.opener = Arc::new(opener);
        self
    }

    /// Builds the updater.
    pub fn build(self) -> Result<Updater> {
        let file_cache = match self.file_cache {
            Some(file_cache) => file_cache,
            None => FileCache::with_config_unchecked(
                FileCache::<GithubRelease>::default_path(),
                Updater::UPDATE_CACHE_NAME,
                1,
                self.update_interval.as_secs(),
                false,
            ),
        };

        Ok(Updater {
            github_repository_url: self.github_repository_url,
            current_version: self.current_version,
            update_interval: self.update_interval,
            file_cache,
            http_agent: ureq::Agent::new_with_defaults(),
            github_api_base_url: self.github_api_base_url,
            download_directory: self.download_directory,
            opener: self.opener,
        })
    }
}

/// Parses a Dart-compatible release tag into a semantic version.
pub fn parse_version_tag(value: &str) -> Result<Version> {
    let version = find_version_core(value).unwrap_or(value);
    Ok(Version::parse(version)?)
}

fn parse_current_version(value: &str) -> Result<Version> {
    if !value.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        return Err(Error::InvalidCurrentVersion {
            version: value.to_owned(),
        });
    }

    Ok(Version::parse(value)?)
}

fn find_version_core(value: &str) -> Option<&str> {
    let bytes = value.as_bytes();

    for start in 0..bytes.len() {
        if !bytes[start].is_ascii_digit() {
            continue;
        }

        let mut index = start;
        let mut matched = true;

        for part in 0..3 {
            let part_start = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }

            if part_start == index {
                matched = false;
                break;
            }

            if part < 2 {
                if bytes.get(index) != Some(&b'.') {
                    matched = false;
                    break;
                }
                index += 1;
            }
        }

        if matched {
            return Some(&value[start..index]);
        }
    }

    None
}

fn validate_repository_url(url: &Url) -> Result<()> {
    if url.host_str() != Some("github.com") || repository_path(url).is_err() {
        return Err(Error::InvalidGithubRepositoryUrl {
            url: url.to_string(),
        });
    }

    Ok(())
}

fn repository_path(url: &Url) -> Result<String> {
    let mut segments = url
        .path_segments()
        .into_iter()
        .flatten()
        .filter(|segment| !segment.is_empty());

    let Some(owner) = segments.next() else {
        return Err(Error::InvalidGithubRepositoryUrl {
            url: url.to_string(),
        });
    };
    let Some(repository) = segments.next() else {
        return Err(Error::InvalidGithubRepositoryUrl {
            url: url.to_string(),
        });
    };

    Ok(format!("{owner}/{repository}"))
}

fn unique_temp_directory() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    std::env::temp_dir().join(format!(
        "alfred_workflow_update_{}_{}",
        std::process::id(),
        timestamp
    ))
}

fn safe_asset_file_name(name: &str) -> Result<&str> {
    if name.is_empty() || name == "." || name == ".." || name.contains('/') || name.contains('\\') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("release asset name must be a plain file name, got `{name}`"),
        )
        .into());
    }

    Ok(name)
}

fn http_error(error: impl std::fmt::Display) -> Error {
    Error::Http(error.to_string())
}

mod version_tag {
    use super::*;

    pub fn serialize<S>(version: &Version, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&version.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        parse_version_tag(&value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    fn github_url(path: &str) -> Result<Url> {
        Ok(Url::parse(&format!("https://github.com{path}"))?)
    }

    #[test]
    fn private_helpers_cover_updater_utility_paths() -> Result<()> {
        assert_eq!(
            repository_path(&github_url("/owner/repository")?)?,
            "owner/repository"
        );
        assert!(
            validate_repository_url(&Url::parse("https://example.com/owner/repository")?).is_err()
        );
        assert!(validate_repository_url(&github_url("/owner/repository")?).is_ok());
        assert_eq!(
            safe_asset_file_name("workflow.alfredworkflow")?,
            "workflow.alfredworkflow"
        );
        assert!(
            unique_temp_directory()
                .to_string_lossy()
                .contains("alfred_workflow_update_")
        );
        assert!(matches!(http_error("network"), Error::Http(message) if message == "network"));

        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn command_opener_uses_open_command_status() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempdir()?;
        let command_path = directory.path().join("open");
        std::fs::write(
            &command_path,
            "#!/bin/sh\nif [ \"$1\" = success ]; then exit 0; fi\nexit 42\n",
        )?;
        let mut permissions = std::fs::metadata(&command_path)?.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&command_path, permissions)?;

        let opener = CommandOpener::with_command(command_path);
        let success = opener.open(Path::new("success"));
        let failure = opener.open(Path::new("failure"));

        success?;
        assert!(failure.is_err());

        Ok(())
    }
}
