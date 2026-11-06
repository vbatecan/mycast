# Builds and releases

[Build and release workflow](https://github.com/vbatecan/mycast/actions/workflows/build-release.yml)

GitHub Actions compiles and packages Mycast on Windows Server 2022 and Ubuntu 22.04, using Node.js 24, stable Rust, and the committed npm/Cargo lockfiles. The jobs install the Linux WebKitGTK 4.1 and packaging dependencies, cache dependencies, and upload installers as workflow artifacts for 14 days. No unit, UI, or native tests are run; tests remain paused.

## Build triggers

- Pull requests targeting `main`: compile frontend types and build both platforms, without publishing a release.
- Pushes to `main`: compile and package both platforms, with downloadable workflow artifacts.
- Manual **Run workflow**: build the selected branch, or publish when selecting a matching version tag.
- Push a `v<version>` tag: build both platforms and automatically publish a GitHub Release after both succeed.

## Publish a version

Keep these versions identical before tagging:

- `package.json`
- `package-lock.json` (both the top-level and root-package version)
- `src-tauri/Cargo.toml`
- The `mycast` package entry in `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

For example, after updating the files to `0.2.0`:

```sh
git add --all
git commit -m "Release Mycast 0.2.0"
git tag v0.2.0
git push origin main
git push origin v0.2.0
```

The workflow rejects tags whose version does not match the application. A version containing a prerelease suffix, such as `0.2.0-beta.1`, creates a GitHub prerelease. Normal branch builds never publish releases or create tags.

## Release downloads

- Windows x64: NSIS `-setup.exe` installer.
- Linux x64: Debian/Ubuntu `.deb` package and portable `.AppImage`.
- `SHA256SUMS.txt`: SHA-256 hashes for all installers.

Windows uses the main Tauri configuration; Linux automatically merges `src-tauri/tauri.linux.conf.json`. The platform jobs must produce every expected package before the release job starts. Assets are attached to a draft and the draft is published only after upload succeeds. A rerun can recover an unfinished draft but leaves an already published release unchanged. Fix published versions by releasing a new version, rather than moving tags.

The workflow uses the repository's automatic `GITHUB_TOKEN`; no personal access token or signing secret is required. Only the tag-release job receives repository write permission. External actions are pinned to commit SHAs. Update those pins deliberately when upgrading actions.

Builds are unsigned. Windows signing and app auto-update are not configured. Linux packaging is experimental: the application has several Windows-only native features, listed in [the release notes](release-notes.md). Creating Linux installers does not port those features.

Implementation follows the [Tauri distribution guide](https://v2.tauri.app/distribute/pipelines/github/) and [AppImage compatibility guidance](https://v2.tauri.app/distribute/appimage/).
