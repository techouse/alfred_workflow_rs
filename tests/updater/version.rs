use alfred_workflow_rs::{GithubRelease, parse_version_tag};
use pretty_assertions::assert_eq;
use semver::Version;

use crate::support::{fixture_asset_json, fixture_release_json};

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
