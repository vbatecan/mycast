# Mycast

A compact command launcher for Windows 11, built with Tauri 2, Rust, Svelte 5, TypeScript, Vite, Tailwind CSS 4, and Lucide. Inspired by [Raycast](https://www.raycast.com/), with original branding and assets.

## Develop

Prerequisites: Node.js 24+ and npm; Rust stable (MSVC, minimum 1.90); Visual Studio Build Tools with Desktop development with C++ and a Windows 11 SDK; Edge WebView2 Runtime.

```powershell
npm ci
npm run tauri dev
```

`npm run dev` opens a frontend-only preview. Native actions explicitly report that the desktop app is required.

## Build

```powershell
npm run tauri build
```

Executable: `src-tauri/target/release/mycast.exe`. NSIS installer: `src-tauri/target/release/bundle/nsis`. Use `npm run tauri build -- --no-bundle` for only the executable. Installers are unsigned.

GitHub Actions builds Windows and Linux packages on pushes to `main` and pull requests. Pushing a matching version tag such as `v0.1.0` automatically publishes a GitHub Release with the Windows installer, Linux `.deb` and `.AppImage`, and checksums after both builds succeed. Linux packages are experimental; several native features remain Windows-only. See [builds and releases](docs/releasing.md) for versioning, downloads, and pipeline behavior. Tests remain paused in the workflow.

## Use

- Alt + Space toggles the launcher. General preferences let you record, save, cancel, or reset this shortcut. Conflicts keep the working shortcut. Startup tries your saved shortcut, then Alt + Space and Ctrl + Alt + Space as recovery options. The tray can reopen or quit Mycast.
- Type to search; Up/Down wrap results, Enter executes, Escape hides. Escape from Preferences returns to commands. Losing focus hides the window.
- Type arithmetic such as `(50 + 20) / 2`; Enter copies the result. Display uses 12 significant digits and standard floating point arithmetic.
- Search file and folder names in All or the Files filter. Enter opens a result. File favorites, aliases, and Recents persist. Preferences → Files & Run manages search folders, exclusions, indexing status, and the terminal working folder. See [Files and PowerShell](docs/files-and-run.md).
- Start a query with `>` to run a command in a visible PowerShell terminal, for example `> git status`. Enter submits it; `>` alone opens PowerShell. Mycast does not save raw Run commands.
- Pin the selected item with Ctrl + D. Manage up to ten aliases in Preferences → Commands: select an item from the icon list on the left, then use its alias and favorite controls on the right. The home view shows Recents first, followed by Favorites and Suggested items. Recently opened programs, files, and folders launched through Mycast appear newest first, with five shown by default. Preferences → General → Recent items adjusts this limit from 1 to 20 on both the home view and Recents filter. Commands preferences manage favorite order, aliases, and unavailable saved applications. Successful use improves ranking within text relevance classes. General preferences can clear recents and learned ranking.
- Description, animation, density, and text-size preferences persist on this device. Reduced motion is respected. Release builds can launch hidden in the tray at Windows sign-in; startup stays disabled in development builds because they depend on Vite.
- Installed desktop and Windows Store applications appear in search. Use All / Applications / Commands / Files / Favorites / Recents to narrow results. Ctrl + Alt + 1 through 6 select these filters in order; Ctrl + Tab and Ctrl + Shift + Tab cycle them. Preferences → Shortcuts lets you record new bindings for filters, pinning, result navigation, opening, closing/back, and filter cycling, with duplicate checks and a reset control. The global launcher shortcut is also available there. In-app bindings apply while Mycast is open and persist on this device. Refresh applications in General preferences after installing an app. Discovery waits for the first opening when launched with `--background`.
- Search **Change Theme** to open the theme gallery. Select a preset, create or duplicate your own palette, preview changes, and import/export theme JSON. See [theme creation](docs/themes.md) and the [example theme](static/themes/soft-indigo.json).
- Preferences → Appearance offers Solid, Transparent, Blur, Acrylic, and Mica window materials, background opacity, a highlighted border, and a soft accent glow. Acrylic at 78% opacity is the default. Text remains opaque. Materials apply live and persist on this device; native material failures fall back to Solid with an explanation. Mica uses the Windows 11 wallpaper backdrop. Browser previews approximate the desktop materials.

## Architecture

`src/lib/commands` owns built-in actions, bounded ranked/fuzzy search, the safe calculator, and execution dispatch. The applications store merges native discovery into the registry. Rust `applications.rs` discovers Windows apps and accepts opaque catalogue IDs for launch. Rust `files.rs` owns the persistent filename index, filesystem notifications, native fuzzy matching, file actions, folder picker, and explicit PowerShell Run requests. Themes use a validated color schema, five immutable presets, localStorage persistence, and a temporary preview layer. Components own the palette and settings UI; `src/lib/native.ts` is the typed IPC boundary. Rust `lib.rs` owns window lifecycle, tray recovery, and single-instance activation. Frontend plugin capabilities allow only event subscriptions. Native commands resolve catalogue IDs for app/file actions and accept command text specifically for the user-invoked Run mode; no generic filesystem or clipboard-read plugin is exposed.

## Functionality

Installed-application discovery and launching, 14 built-in commands, known user folders, date/time, UUIDs, arithmetic, Preferences, and About. Five themes and a custom-theme editor support live preview, persistence, import/export, duplication, and deletion. Keyboard and mouse navigation, scoped search, honest errors, empty states, fixed footer, tray, and shortcut fallback are included. Discovery runs once per session and on explicit refresh; there is no filesystem-wide indexing, polling, telemetry, remote font loading, or cloud service. Release assets work offline.

The updated application compiles and has been launched from `src-tauri/target/release/mycast.exe` with bundled frontend assets. Full test suites remain paused. Previous memory benchmarks predate these additions. See [verification notes](VERIFICATION.md) for current checks and remaining manual Windows checks.

## Verify

```powershell
npm run check
npm test
npm run build
npm run test:ui
npm run format:check
cargo check --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

Browser tests use installed Microsoft Edge in headless mode against production assets; build first. Screenshots go to `artifacts/`. See [VERIFICATION.md](VERIFICATION.md) for results.

## Memory

Measure a release build and its entire process tree, including WebView2, in both visible-idle and hidden states:

```powershell
./scripts/measure-memory.ps1 -LauncherProcessId (Get-Process mycast).Id -State visible-idle
# Hide with Escape, then repeat with -State hidden.
```

The script reports summed working set and private commit separately. Shared pages can be counted more than once in summed working set. Targets of <50 MB hidden, <100 MB visible idle, and <150 MB active are engineering targets, not guarantees.

The script also reports private resident memory when Windows performance counters are available. The preceding MVP's release snapshot was 86.1 MiB private resident memory and 371.6 MiB summed working set across seven processes; visible/hidden state benchmarks and this phase's memory use remain unverified. See the verification notes for measurement limits.

Optional `npm run test:native` creates a separate debug build with a test-only debugging port, writes test values to the clipboard, and opens a blank Notepad window. Close any running Mycast instance first. The production executable is unchanged. This optional script's final convenience-build update was not rerun after testing was stopped.

## Deferred and limitations

Extensions, AI, clipboard history, sync, auto-update, accounts, and telemetry remain deferred. Application discovery covers Start Menu shortcuts and packaged Windows apps. File search indexes selected folders and is capped at 250,000 entries, with a visible notice when reached. Search renders at most 40 matches; refine your query to find others. App rows request Windows icons lazily; files use consistent file/folder glyphs. The window supports transparent Windows backdrops with native corners/shadow; their appearance depends on Windows composition and system transparency settings. Launch success confirms dispatch to Windows, not completion of the external app's startup. The previous audit found two low-severity development-only findings in the Svelte Vite plugin's upstream deepmerge dependency.
