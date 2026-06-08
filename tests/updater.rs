use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use alfred_workflow_rs::{
    CommandOpener, FileCache, GithubAsset, GithubRelease, Opener, Result, Updater,
    parse_version_tag,
};
use pretty_assertions::assert_eq;
use semver::Version;
use serde_json::{Value, json};
use tempfile::tempdir;
use url::Url;

fn fixture_user_json(login: &str, id: u64) -> Value {
    json!({
        "login": login,
        "id": id,
        "node_id": format!("USER_{id}"),
        "avatar_url": format!("https://avatars.githubusercontent.com/u/{id}"),
        "gravatar_id": "",
        "url": format!("https://api.github.com/users/{login}"),
        "html_url": format!("https://github.com/{login}"),
        "repos_url": format!("https://api.github.com/users/{login}/repos"),
        "type": "User",
        "site_admin": false
    })
}

fn fixture_asset_json(id: u64, name: &str, browser_download_url: &str) -> Value {
    json!({
        "url": format!("https://api.github.com/repos/example/workflow/releases/assets/{id}"),
        "id": id,
        "node_id": format!("ASSET_{id}"),
        "name": name,
        "label": null,
        "uploader": fixture_user_json("maintainer", 42),
        "content_type": "application/octet-stream",
        "state": "uploaded",
        "size": 1234,
        "download_count": 5,
        "created_at": "2026-06-01T00:00:00Z",
        "updated_at": "2026-06-01T00:00:00Z",
        "browser_download_url": browser_download_url
    })
}

fn fixture_release_json(tag_name: &str, assets: Vec<Value>) -> Value {
    json!({
        "url": "https://api.github.com/repos/example/workflow/releases/1",
        "assets_url": "https://api.github.com/repos/example/workflow/releases/1/assets",
        "upload_url": "https://uploads.github.com/repos/example/workflow/releases/1/assets{?name,label}",
        "html_url": "https://github.com/example/workflow/releases/tag/v2.0.0",
        "id": 1,
        "author": fixture_user_json("maintainer", 42),
        "node_id": "RELEASE_1",
        "tag_name": tag_name,
        "target_commitish": "main",
        "name": "Release",
        "draft": false,
        "prerelease": false,
        "created_at": "2026-06-01T00:00:00Z",
        "published_at": "2026-06-01T00:00:00Z",
        "assets": assets,
        "tarball_url": "https://api.github.com/repos/example/workflow/tarball/v2.0.0",
        "zipball_url": "https://api.github.com/repos/example/workflow/zipball/v2.0.0",
        "body": "Release notes"
    })
}

fn fixture_release(tag_name: &str, download_url: &str) -> Result<GithubRelease> {
    Ok(serde_json::from_value(fixture_release_json(
        tag_name,
        vec![fixture_asset_json(
            1,
            "workflow.alfredworkflow",
            download_url,
        )],
    ))?)
}

fn github_url(path: &str) -> std::result::Result<Url, url::ParseError> {
    Url::parse(&format!("https://github.com{path}"))
}

fn closed_localhost_url() -> std::result::Result<Url, Box<dyn std::error::Error>> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    drop(listener);
    Ok(Url::parse(&format!("http://{address}"))?)
}

fn updater_with_mock_api(server_url: &str, cache: FileCache<GithubRelease>) -> Result<Updater> {
    Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .github_api_base_url(Url::parse(server_url)?)
        .file_cache(cache)
        .build()
}

#[derive(Clone, Default)]
struct RecordingOpener {
    opened_paths: Arc<Mutex<Vec<PathBuf>>>,
}

impl RecordingOpener {
    fn opened_paths(&self) -> Vec<PathBuf> {
        self.opened_paths
            .lock()
            .expect("recording opener mutex should not be poisoned")
            .clone()
    }
}

impl Opener for RecordingOpener {
    fn open(&self, path: &Path) -> Result<()> {
        self.opened_paths
            .lock()
            .expect("recording opener mutex should not be poisoned")
            .push(path.to_path_buf());
        Ok(())
    }
}

#[test]
fn parse_version_tag_accepts_dart_version_converter_forms()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let expected = Version::parse("12.34.56")?;
    let fixtures = [
        "12.34.56",
        "v12.34.56",
        "V12.34.56",
        "x12.34.56",
        "name-12.34.56",
        "name-v12.34.56",
        "12.34.56-name",
        "v12.34.56-name",
        "name-12.34.56-name",
    ];

    for fixture in fixtures {
        assert_eq!(parse_version_tag(fixture)?, expected);
    }

    Ok(())
}

#[test]
fn parse_version_tag_rejects_versions_without_major_minor_patch() {
    assert!(parse_version_tag("12").is_err());
    assert!(parse_version_tag("12.34").is_err());
    assert!(parse_version_tag("release").is_err());
    assert!(parse_version_tag("1..2.3").is_err());
}

#[test]
fn github_release_json_uses_github_keys_and_version_converter()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let value = fixture_release_json(
        "workflow-v2.0.1-beta",
        vec![fixture_asset_json(
            1,
            "workflow.alfredworkflow",
            "https://example.com/workflow.alfredworkflow",
        )],
    );

    let release: GithubRelease = serde_json::from_value(value)?;
    assert_eq!(release.tag_name, Version::parse("2.0.1")?);

    let serialized = serde_json::to_value(release)?;
    assert_eq!(serialized["tag_name"], "2.0.1");
    assert_eq!(
        serialized["assets"][0]["browser_download_url"],
        "https://example.com/workflow.alfredworkflow"
    );
    assert_eq!(serialized["author"]["site_admin"], false);

    Ok(())
}

#[test]
fn updater_new_rejects_non_github_repository_url()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse("https://example.com/example/workflow")?;

    assert!(Updater::new(url, "1.0.0").is_err());
    assert!(Updater::new(github_url("/")?, "1.0.0").is_err());
    assert!(Updater::new(github_url("/example")?, "1.0.0").is_err());

    Ok(())
}

#[test]
fn updater_new_requires_strict_current_semver()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    for current_version in ["v1.0.0", "V1.0.0", "x1.0.0", "name-1.0.0", "1.0"] {
        assert!(Updater::new(github_url("/example/workflow")?, current_version).is_err());
    }

    Ok(())
}

#[test]
fn updater_builder_accessors_debug_and_equality_use_public_configuration()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let cache_dir = tempdir()?;
    let download_dir = tempdir()?;
    let cache = FileCache::<GithubRelease>::try_with_config(
        cache_dir.path(),
        "update_cache",
        1,
        300,
        true,
    )?;
    let api_base_url = Url::parse("https://api.example.com")?;
    let opener = RecordingOpener::default();
    let updater = Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .update_interval(Duration::from_secs(300))
        .file_cache(cache.clone())
        .github_api_base_url(api_base_url.clone())
        .download_directory(download_dir.path())
        .opener(opener)
        .build()?;

    assert_eq!(
        updater.github_repository_url().as_str(),
        "https://github.com/example/workflow"
    );
    assert_eq!(updater.current_version(), &Version::parse("1.0.0")?);
    assert_eq!(updater.update_interval(), Duration::from_secs(300));
    assert_eq!(updater.file_cache(), &cache);
    assert!(format!("{updater:?}").contains("Updater"));
    assert_eq!(CommandOpener, CommandOpener);
    assert!(format!("{CommandOpener:?}").contains("CommandOpener"));

    let same = Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .update_interval(Duration::from_secs(300))
        .file_cache(cache.clone())
        .github_api_base_url(api_base_url)
        .download_directory(download_dir.path())
        .opener(RecordingOpener::default())
        .build()?;
    assert_eq!(updater, same);

    let different = Updater::builder(github_url("/example/workflow")?, "1.0.1")?
        .update_interval(Duration::from_secs(300))
        .file_cache(cache)
        .download_directory(download_dir.path())
        .build()?;
    assert_ne!(updater, different);

    Ok(())
}

#[test]
fn updater_default_cache_matches_dart_update_interval_defaults()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let updater = Updater::new(github_url("/example/workflow")?, "1.0.0")?;

    assert_eq!(updater.update_interval(), Duration::ZERO);
    assert_eq!(updater.file_cache().name(), Updater::UPDATE_CACHE_NAME);
    assert_eq!(updater.file_cache().max_entries(), 1);
    assert_eq!(updater.file_cache().time_to_live_seconds(), 0);

    Ok(())
}

#[test]
fn update_available_fetches_latest_release_and_caches_it()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let release = fixture_release(
        "v2.0.0",
        &format!("{}/download/workflow.alfredworkflow", server.url()),
    )?;
    let _mock = server
        .mock("GET", "/repos/example/workflow/releases/latest")
        .with_status(200)
        .with_body(serde_json::to_string(&release)?)
        .create();
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = updater_with_mock_api(&server.url(), cache.clone())?;

    assert!(updater.update_available()?);
    assert_eq!(
        cache
            .get(&Updater::update_cache_key())?
            .map(|release| release.tag_name),
        Some(Version::parse("2.0.0")?)
    );

    Ok(())
}

#[test]
fn update_available_uses_cached_release_without_fetching()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    cache.put(
        &Updater::update_cache_key(),
        fixture_release("name-v2.0.1", "https://example.com/workflow.alfredworkflow")?,
    )?;
    let updater = Updater::builder(github_url("/example/workflow")?, "2.0.0")?
        .file_cache(cache)
        .build()?;

    assert!(updater.update_available()?);

    Ok(())
}

#[test]
fn update_available_returns_false_when_release_is_not_newer()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    cache.put(
        &Updater::update_cache_key(),
        fixture_release("v1.9.9", "https://example.com/workflow.alfredworkflow")?,
    )?;
    let updater = Updater::builder(github_url("/example/workflow")?, "2.0.0")?
        .file_cache(cache)
        .build()?;

    assert!(!updater.update_available()?);

    Ok(())
}

#[test]
fn update_available_fetches_and_caches_release_that_is_not_newer()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let release = fixture_release(
        "v1.0.0",
        &format!("{}/download/workflow.alfredworkflow", server.url()),
    )?;
    let _mock = server
        .mock("GET", "/repos/example/workflow/releases/latest")
        .with_status(200)
        .with_body(serde_json::to_string(&release)?)
        .create();
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = updater_with_mock_api(&server.url(), cache.clone())?;

    assert!(!updater.update_available()?);
    assert_eq!(
        cache
            .get(&Updater::update_cache_key())?
            .map(|release| release.tag_name),
        Some(Version::parse("1.0.0")?)
    );

    Ok(())
}

#[test]
fn update_available_returns_false_when_latest_release_fetch_returns_none()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let _mock = server
        .mock("GET", "/repos/example/workflow/releases/latest")
        .with_status(404)
        .create();
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = updater_with_mock_api(&server.url(), cache)?;

    assert!(!updater.update_available()?);

    Ok(())
}

#[test]
fn fetch_latest_release_returns_none_for_http_errors()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let _mock = server
        .mock("GET", "/repos/example/workflow/releases/latest")
        .with_status(404)
        .create();
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = updater_with_mock_api(&server.url(), cache)?;

    assert_eq!(updater.fetch_latest_release()?, None);

    Ok(())
}

#[test]
fn fetch_latest_release_reports_invalid_json_response()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let _mock = server
        .mock("GET", "/repos/example/workflow/releases/latest")
        .with_status(200)
        .with_body("{not-json")
        .create();
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = updater_with_mock_api(&server.url(), cache)?;

    assert!(updater.fetch_latest_release().is_err());

    Ok(())
}

#[test]
fn fetch_latest_release_reports_transport_errors()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .github_api_base_url(closed_localhost_url()?)
        .file_cache(cache)
        .build()?;

    assert!(updater.fetch_latest_release().is_err());

    Ok(())
}

#[test]
fn find_alfred_workflow_asset_returns_first_matching_asset()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let release: GithubRelease = serde_json::from_value(fixture_release_json(
        "v2.0.0",
        vec![
            fixture_asset_json(1, "readme.txt", "https://example.com/readme.txt"),
            fixture_asset_json(
                2,
                "workflow.alfredworkflow",
                "https://example.com/workflow.alfredworkflow",
            ),
            fixture_asset_json(
                3,
                "other.alfredworkflow",
                "https://example.com/other.alfredworkflow",
            ),
        ],
    ))?;
    let updater = Updater::new(github_url("/example/workflow")?, "1.0.0")?;

    assert_eq!(
        updater
            .find_alfred_workflow_asset(&release)
            .map(|asset| asset.id),
        Some(2)
    );

    Ok(())
}

#[test]
fn download_asset_writes_response_bytes_to_configured_directory()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let _mock = server
        .mock("GET", "/download/workflow.alfredworkflow")
        .with_status(200)
        .with_body("workflow-bytes")
        .create();
    let dir = tempdir()?;
    let download_dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .github_api_base_url(Url::parse(&server.url())?)
        .file_cache(cache)
        .download_directory(download_dir.path())
        .build()?;
    let asset: GithubAsset = serde_json::from_value(fixture_asset_json(
        1,
        "workflow.alfredworkflow",
        &format!("{}/download/workflow.alfredworkflow", server.url()),
    ))?;

    let path = updater.download_asset(&asset)?.expect("asset downloaded");

    assert_eq!(path, download_dir.path().join("workflow.alfredworkflow"));
    assert_eq!(std::fs::read(path)?, b"workflow-bytes");

    Ok(())
}

#[test]
fn download_asset_returns_none_for_http_errors()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let _mock = server
        .mock("GET", "/download/workflow.alfredworkflow")
        .with_status(404)
        .create();
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .github_api_base_url(Url::parse(&server.url())?)
        .file_cache(cache)
        .build()?;
    let asset: GithubAsset = serde_json::from_value(fixture_asset_json(
        1,
        "workflow.alfredworkflow",
        &format!("{}/download/workflow.alfredworkflow", server.url()),
    ))?;

    assert_eq!(updater.download_asset(&asset)?, None);

    Ok(())
}

#[test]
fn download_asset_reports_transport_errors() -> std::result::Result<(), Box<dyn std::error::Error>>
{
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .file_cache(cache)
        .build()?;
    let asset_url = closed_localhost_url()?.join("/download/workflow.alfredworkflow")?;
    let asset: GithubAsset = serde_json::from_value(fixture_asset_json(
        1,
        "workflow.alfredworkflow",
        asset_url.as_str(),
    ))?;

    assert!(updater.download_asset(&asset).is_err());

    Ok(())
}

#[test]
fn download_asset_uses_unique_temp_directory_when_no_directory_is_configured()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let _mock = server
        .mock("GET", "/download/workflow.alfredworkflow")
        .with_status(200)
        .with_body("workflow-bytes")
        .create();
    let dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .github_api_base_url(Url::parse(&server.url())?)
        .file_cache(cache)
        .build()?;
    let asset: GithubAsset = serde_json::from_value(fixture_asset_json(
        1,
        "workflow.alfredworkflow",
        &format!("{}/download/workflow.alfredworkflow", server.url()),
    ))?;

    let path = updater.download_asset(&asset)?.expect("asset downloaded");

    assert_eq!(
        path.file_name().and_then(|name| name.to_str()),
        Some("workflow.alfredworkflow")
    );
    assert_eq!(std::fs::read(&path)?, b"workflow-bytes");
    if let Some(parent) = path.parent() {
        std::fs::remove_dir_all(parent)?;
    }

    Ok(())
}

#[test]
fn download_asset_rejects_names_with_path_components()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let download_dir = tempdir()?;
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    let updater = Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .file_cache(cache)
        .download_directory(download_dir.path())
        .build()?;

    for name in [
        "../workflow.alfredworkflow",
        "nested/workflow.alfredworkflow",
        r"nested\workflow.alfredworkflow",
        ".",
        "..",
        "",
    ] {
        let asset: GithubAsset = serde_json::from_value(fixture_asset_json(
            1,
            name,
            "https://example.com/workflow.alfredworkflow",
        ))?;

        assert!(updater.download_asset(&asset).is_err());
    }

    assert_eq!(std::fs::read_dir(download_dir.path())?.count(), 0);

    Ok(())
}

#[test]
fn update_downloads_asset_and_invokes_injected_opener()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let release = fixture_release(
        "v2.0.0",
        &format!("{}/download/workflow.alfredworkflow", server.url()),
    )?;
    let _release_mock = server
        .mock("GET", "/repos/example/workflow/releases/latest")
        .with_status(200)
        .with_body(serde_json::to_string(&release)?)
        .create();
    let _download_mock = server
        .mock("GET", "/download/workflow.alfredworkflow")
        .with_status(200)
        .with_body("workflow-bytes")
        .create();
    let cache_dir = tempdir()?;
    let download_dir = tempdir()?;
    let opener = RecordingOpener::default();
    let cache = FileCache::<GithubRelease>::try_with_config(
        cache_dir.path(),
        "update_cache",
        1,
        60,
        false,
    )?;
    let updater = Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .github_api_base_url(Url::parse(&server.url())?)
        .download_directory(download_dir.path())
        .file_cache(cache)
        .opener(opener.clone())
        .build()?;

    updater.update()?;

    let opened_paths = opener.opened_paths();
    assert_eq!(opened_paths.len(), 1);
    assert_eq!(
        opened_paths[0],
        download_dir.path().join("workflow.alfredworkflow")
    );
    assert_eq!(std::fs::read(&opened_paths[0])?, b"workflow-bytes");

    Ok(())
}

#[test]
fn update_returns_without_opening_when_cached_release_is_not_newer()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let opener = RecordingOpener::default();
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    cache.put(
        &Updater::update_cache_key(),
        fixture_release("v1.0.0", "https://example.com/workflow.alfredworkflow")?,
    )?;
    let updater = Updater::builder(github_url("/example/workflow")?, "2.0.0")?
        .file_cache(cache)
        .opener(opener.clone())
        .build()?;

    updater.update()?;

    assert!(opener.opened_paths().is_empty());

    Ok(())
}

#[test]
fn update_returns_without_opening_when_release_has_no_workflow_asset()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let opener = RecordingOpener::default();
    let cache =
        FileCache::<GithubRelease>::try_with_config(dir.path(), "update_cache", 1, 60, false)?;
    cache.put(
        &Updater::update_cache_key(),
        serde_json::from_value(fixture_release_json(
            "v2.0.0",
            vec![fixture_asset_json(
                1,
                "readme.txt",
                "https://example.com/readme.txt",
            )],
        ))?,
    )?;
    let updater = Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .file_cache(cache)
        .opener(opener.clone())
        .build()?;

    updater.update()?;

    assert!(opener.opened_paths().is_empty());

    Ok(())
}
