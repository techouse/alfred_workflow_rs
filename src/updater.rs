use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use crate::{Error, FileCache, Result};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GithubUser {
    pub login: String,
    pub id: u64,
    pub node_id: String,
    pub avatar_url: Url,
    pub gravatar_id: String,
    pub url: Url,
    pub html_url: Url,
    pub repos_url: Url,
    #[serde(rename = "type")]
    pub user_type: String,
    pub site_admin: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GithubAsset {
    pub url: Url,
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub uploader: GithubUser,
    pub content_type: String,
    pub state: String,
    pub size: u64,
    pub download_count: u64,
    pub created_at: String,
    pub updated_at: String,
    pub browser_download_url: Url,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GithubRelease {
    pub url: Url,
    pub assets_url: Url,
    pub upload_url: Url,
    pub html_url: Url,
    pub id: u64,
    pub author: GithubUser,
    pub node_id: String,
    #[serde(with = "version_tag")]
    pub tag_name: Version,
    pub target_commitish: String,
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: String,
    pub assets: Vec<GithubAsset>,
    pub tarball_url: Url,
    pub zipball_url: Url,
    pub body: Option<String>,
}

pub trait Opener: Send + Sync {
    fn open(&self, path: &Path) -> Result<()>;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CommandOpener;

impl Opener for CommandOpener {
    fn open(&self, path: &Path) -> Result<()> {
        std::process::Command::new("open").arg(path).status()?;
        Ok(())
    }
}

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
    pub const UPDATE_KEY: &'static str = "update";
    pub const UPDATE_CACHE_NAME: &'static str = "update_cache";

    pub fn new(github_repository_url: Url, current_version: &str) -> Result<Self> {
        Self::builder(github_repository_url, current_version)?.build()
    }

    pub fn builder(github_repository_url: Url, current_version: &str) -> Result<UpdaterBuilder> {
        validate_repository_url(&github_repository_url)?;

        Ok(UpdaterBuilder {
            github_repository_url,
            current_version: parse_version_tag(current_version)?,
            update_interval: Duration::ZERO,
            file_cache: None,
            github_api_base_url: Url::parse("https://api.github.com")?,
            download_directory: None,
            opener: Arc::new(CommandOpener),
        })
    }

    pub fn update_cache_key() -> String {
        FileCache::<GithubRelease>::hash_key(Self::UPDATE_KEY)
    }

    pub fn github_repository_url(&self) -> &Url {
        &self.github_repository_url
    }

    pub fn current_version(&self) -> &Version {
        &self.current_version
    }

    pub fn update_interval(&self) -> Duration {
        self.update_interval
    }

    pub fn file_cache(&self) -> &FileCache<GithubRelease> {
        &self.file_cache
    }

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

    pub fn find_alfred_workflow_asset<'a>(
        &self,
        release: &'a GithubRelease,
    ) -> Option<&'a GithubAsset> {
        release
            .assets
            .iter()
            .find(|asset| asset.name.ends_with(".alfredworkflow"))
    }

    pub fn download_asset(&self, asset: &GithubAsset) -> Result<Option<PathBuf>> {
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

        let path = directory.join(&asset.name);
        let bytes = response.body_mut().read_to_vec().map_err(http_error)?;
        std::fs::write(&path, bytes)?;

        Ok(Some(path))
    }

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
    pub fn update_interval(mut self, update_interval: Duration) -> Self {
        self.update_interval = update_interval;
        self
    }

    pub fn file_cache(mut self, file_cache: FileCache<GithubRelease>) -> Self {
        self.file_cache = Some(file_cache);
        self
    }

    pub fn github_api_base_url(mut self, github_api_base_url: Url) -> Self {
        self.github_api_base_url = github_api_base_url;
        self
    }

    pub fn download_directory(mut self, download_directory: impl Into<PathBuf>) -> Self {
        self.download_directory = Some(download_directory.into());
        self
    }

    pub fn opener<O>(mut self, opener: O) -> Self
    where
        O: Opener + 'static,
    {
        self.opener = Arc::new(opener);
        self
    }

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

pub fn parse_version_tag(value: &str) -> Result<Version> {
    let version = find_version_core(value).unwrap_or(value);
    Ok(Version::parse(version)?)
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
