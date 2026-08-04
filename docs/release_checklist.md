# Release Checklist

Before creating a later `1.x` tag:

- confirm the working tree is clean and the release version is committed on `main`;
- run `make pre-release` outside a sandbox that blocks localhost test servers;
- review the version, README install snippet, and matching changelog section;
- ensure `CI Required` is green for the release commit;
- verify the `crates.io` environment has a reviewer and valid publishing credential;
- create and push an annotated `v<version>` tag that exactly matches `Cargo.toml`.

After approving the publish job:

- confirm crates.io, docs.rs, the GitHub release artifact, and GitHub Pages all show the intended version;
- if publishing failed before upload, delete the failed tag, fix the release commit, and create a new tag;
- if crates.io accepted the version, do not move the tag or republish it—repair the
  release/docs job or issue a patch release instead.
