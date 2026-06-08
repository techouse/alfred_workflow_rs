use alfred_workflow_rs::{FileCache, GithubAsset, GithubRelease, Updater};
use pretty_assertions::assert_eq;
use tempfile::tempdir;
use url::Url;

use crate::support::{closed_localhost_url, fixture_asset_json, fixture_release_json, github_url};

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
