use alfred_workflow_rs::{FileCache, GithubRelease, Updater};
use pretty_assertions::assert_eq;
use semver::Version;
use tempfile::tempdir;

use crate::support::{fixture_release, github_url, updater_with_mock_api};

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
