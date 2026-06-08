use std::time::Duration;

use alfred_workflow_rs::{CommandOpener, FileCache, GithubRelease, Updater};
use pretty_assertions::assert_eq;
use semver::Version;
use tempfile::tempdir;
use url::Url;

use crate::support::{RecordingOpener, github_url};

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
    assert_eq!(CommandOpener::new(), CommandOpener::new());
    assert!(format!("{:?}", CommandOpener::new()).contains("CommandOpener"));

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
