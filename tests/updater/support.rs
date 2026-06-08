use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use alfred_workflow_rs::{FileCache, GithubRelease, Opener, Result, Updater};
use serde_json::{Value, json};
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

pub(crate) fn fixture_asset_json(id: u64, name: &str, browser_download_url: &str) -> Value {
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

pub(crate) fn fixture_release_json(tag_name: &str, assets: Vec<Value>) -> Value {
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

pub(crate) fn fixture_release(tag_name: &str, download_url: &str) -> Result<GithubRelease> {
    Ok(serde_json::from_value(fixture_release_json(
        tag_name,
        vec![fixture_asset_json(
            1,
            "workflow.alfredworkflow",
            download_url,
        )],
    ))?)
}

pub(crate) fn github_url(path: &str) -> std::result::Result<Url, url::ParseError> {
    Url::parse(&format!("https://github.com{path}"))
}

pub(crate) fn closed_localhost_url() -> std::result::Result<Url, Box<dyn std::error::Error>> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    drop(listener);
    Ok(Url::parse(&format!("http://{address}"))?)
}

pub(crate) fn updater_with_mock_api(
    server_url: &str,
    cache: FileCache<GithubRelease>,
) -> Result<Updater> {
    Updater::builder(github_url("/example/workflow")?, "1.0.0")?
        .github_api_base_url(Url::parse(server_url)?)
        .file_cache(cache)
        .build()
}

#[derive(Clone, Default)]
pub(crate) struct RecordingOpener {
    opened_paths: Arc<Mutex<Vec<PathBuf>>>,
}

impl RecordingOpener {
    pub(crate) fn opened_paths(&self) -> Vec<PathBuf> {
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
