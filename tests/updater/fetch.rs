use alfred_workflow_rs::{FileCache, GithubRelease, Updater};
use pretty_assertions::assert_eq;
use tempfile::tempdir;

use crate::support::{closed_localhost_url, github_url, updater_with_mock_api};

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
