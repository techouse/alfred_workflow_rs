use alfred_workflow_rs::{FileCache, GithubRelease, Updater};
use pretty_assertions::assert_eq;
use tempfile::tempdir;
use url::Url;

use crate::support::{
    RecordingOpener, fixture_asset_json, fixture_release, fixture_release_json, github_url,
};

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
