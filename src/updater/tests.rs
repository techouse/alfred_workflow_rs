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
    assert!(validate_repository_url(&Url::parse("https://example.com/owner/repository")?).is_err());
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
