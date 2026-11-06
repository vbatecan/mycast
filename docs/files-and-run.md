# Files and PowerShell

Search file and folder names from **All**, or select **Files** to show only file results. Search supports case-insensitive, accent-insensitive names, fuzzy abbreviations, aliases, and parent paths. Enter opens the selected result with Windows. **Reveal** selects it in Explorer; **Copy path** copies its full path.

The initial folders are Desktop, Documents, Downloads, Pictures, Music, and Videos, resolved through Windows Known Folders so redirected locations are respected. **Preferences → Files & Run** lets you add or remove search folders, include hidden/system entries, change excluded folder names, and rebuild the index. `.git` and `node_modules` folders are excluded by default. Subfolders beneath overlapping roots are indexed once. Symlinks and junctions are not traversed; OneDrive cloud placeholders are handled as metadata and are not opened to read contents during indexing.

The native Rust backend uses parallel `ignore` traversal and `nucleo-matcher` fuzzy matching. It builds the filename index when file search first becomes active, then persists metadata in the per-user application data folder. Background tray startup defers this work. A cached index is available while the app reconciles changes after restart. Windows filesystem notifications update the affected paths, with batched work and full reconciliation after notification overflow. There is no recurring folder polling. Permissions and watcher errors appear in Files & Run settings. An unavailable watcher can be refreshed using Rebuild.

Only the best 40 results reach the frontend. Text relevance is considered before favorites and learned usage; exact aliases rank first. Older requests are cancelled or discarded when the query changes. The index is bounded to 250,000 entries; reaching that limit produces a notice so the selected folders or exclusions can be adjusted.

Files support **Ctrl+D**, favorites, multiple aliases, Recents, and learned ranking. File keys persist across refresh and restart. Personalized files record a Windows volume/file identifier and creation time where available, allowing same-volume renames to retain their personalization, including renames while Mycast is closed. Files copied or moved across volumes may become new items. Missing saved files remain manageable as unavailable entries. Native actions use opaque session IDs and resolve their paths from the native index.

## Run mode

Start the query with `>`:

```text
> git status
> Get-ChildItem
> Write-Output "Hello"
```

The palette shows a Run row containing the command, detected PowerShell executable, and working folder. Enter opens a visible terminal and submits the command; `>` by itself opens PowerShell. PowerShell 7 is preferred when installed at its standard location, with Windows PowerShell as the fallback. Windows Terminal is used when available, with a console window as the fallback. The session stays open after the command completes unless the command itself exits.

Change the working folder under **Preferences → Files & Run**. Its default is your home folder. Commands are passed using PowerShell's UTF-16 encoded-command interface so quotes, pipes, semicolons, and Unicode survive the terminal launch. Mycast saves no raw Run commands or their output in personalization. Execution uses the current user's privileges and existing PowerShell policy/profile settings.

## Focused verification

`cargo test --release --manifest-path src-tauri/Cargo.toml file_search_ -- --include-ignored --nocapture` runs the targeted ranking, boundary/cache, and opt-in 100,000-entry profile checks. It does not run the complete application suites.

`scripts/file-search-smoke.mjs` attaches to a fresh debug executable launched with `--background` and an explicit test-only WebView2 debugging port of 9223. The override used during implementation is `artifacts/file-search-smoke.conf.json`; normal builds contain no debugging port. The helper temporarily selects an isolated fixture under `artifacts`, restores the prior settings and personalization, checks event updates and restart persistence, and launches a visible PowerShell probe that exits after writing an owned verification file. Full test suites remain paused.
