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
- Shift + Enter opens supported Windows desktop applications as administrator through the Windows UAC prompt. This separate binding is editable in Shortcuts. Packaged apps and unsupported actions cannot be elevated; Mycast itself stays non-elevated.
- Start with `?` to search the web in your default browser. Preferences → Search offers Google, Bing, DuckDuckGo, or a custom HTTP/HTTPS URL with one `{query}` placeholder. Queries are sent only when you press Enter and are not retained.
- Search Windows settings and tools directly: try Add or Remove Programs, Bluetooth, Windows Update, Task Manager, or Device Manager. These appear in All and Commands and support favorites and aliases. Both new providers can be disabled in Modules.
- Type to search; Up/Down wrap results, Enter executes, Escape hides. Escape from Preferences returns to commands. Losing focus hides the window.
- Calculations use the open-source fend engine: try `(50 + 20) / 2`, `1.5e6 / 3`, `sin(pi / 2)`, or `5 ft to cm`. Enter copies the result. Calculations run locally with a bounded evaluation time and a fresh context per expression.
- Search file and folder names in All or the Files filter. Enter opens a result. File favorites, aliases, and Recents persist. Preferences → Files & Run manages search folders, exclusions, indexing status, and the terminal working folder. See [Files and PowerShell](docs/files-and-run.md).
- Start a query with `>` to run a command in a visible PowerShell terminal, for example `> git status`. Enter submits it; `>` alone opens PowerShell. Mycast does not save raw Run commands.
- Pin the selected item with Ctrl + D. Manage up to ten aliases in Preferences → Commands: select an item from the icon list on the left, then use its alias and favorite controls on the right. The home view shows Recents first, followed by Favorites and Suggested items. Recently opened programs, files, and folders launched through Mycast appear newest first, with five shown by default. Preferences → General → Recent items adjusts this limit from 1 to 20 on both the home view and Recents filter. Commands preferences manage favorite order, aliases, and unavailable saved applications. Successful use improves ranking within text relevance classes. General preferences can clear recents and learned ranking.
- Description, animation, density, and text-size preferences persist on this device. Reduced motion is respected. Release builds can launch hidden in the tray at Windows sign-in; startup stays disabled in development builds because they depend on Vite.
- Installed desktop and Windows Store applications appear in search. Use All / Applications / Commands / Files / Favorites / Recents to narrow results. Alt + Shift + 1 through 6 select these filters in order; Alt + Shift + Right moves to the next filter and Alt + Shift + Left moves to the previous filter. Preferences → Shortcuts lets you record new bindings for filters, pinning, result navigation, opening, closing/back, and filter cycling, with duplicate checks and a reset control. The global launcher shortcut is also available there. In-app bindings apply while Mycast is open and persist on this device. Refresh applications in General preferences after installing an app. Discovery waits for the first opening when launched with `--background`.
- Search **Change Theme** to open the theme gallery. Select a preset, create or duplicate your own palette, preview changes, and import/export theme JSON. See [theme creation](docs/themes.md) and the [example theme](static/themes/soft-indigo.json).
- Preferences → Appearance offers Solid, Transparent, Blur, Acrylic, and Mica window materials, background opacity, a highlighted border, and a soft accent glow. Acrylic at 78% opacity is the default. Text remains opaque. Materials apply live and persist on this device; native material failures fall back to Solid with an explanation. Mica uses the Windows 11 wallpaper backdrop. Browser previews approximate the desktop materials.

## Optional modules and updates

Settings → Modules controls Applications, Files, Terminal/Run, Calculator, Currency, Utilities, Web search, Windows settings and tools, and AI assistant. Providers load when needed. Disabling a module hides its commands; disabling Files cancels scanning and releases its watcher/worker, and disabling Applications clears the catalogue/icon caches. Settings and personalization are retained. Native Rust code remains bundled in the executable; disabling modules stops work and releases state rather than shrinking the installed app. This is a built-in module system, not a third-party plugin marketplace.

Use `100 USD to SGD` for currency conversion. Frankfurter reference rates are cached for six hours and retained for offline use, with rate dates shown. Only the currency pair is sent to the provider; the entered amount is calculated locally. See [Frankfurter](https://frankfurter.dev/) and its [data terms](https://frankfurter.dev/license/).

Settings → Updates checks the signed GitHub release manifest. Automatic checks occur shortly after startup and every six hours, without opening the launcher. Download/installation requires a button click and may close/restart Mycast. Windows and Linux AppImage are supported; Debian packages use manual/package-manager updates. Install v0.2.0 manually once when coming from v0.1.x.

Updater signing keys are backed up outside this repository at `%USERPROFILE%/.tauri/mycast/`. Keep both the key and its password backup safe. GitHub Secrets supply signing credentials to CI; the public verification key is bundled with the app. See [releasing](docs/releasing.md).

## Architecture

`src/lib/commands` owns built-in actions, bounded ranked/fuzzy search, and execution dispatch. Rust owns the fend evaluator; optional calculator/currency providers load on demand. The applications store merges native discovery into the registry. Rust `applications.rs` discovers Windows apps and accepts opaque catalogue IDs for launch. Rust `files.rs` owns the persistent filename index, filesystem notifications, native fuzzy matching, file actions, folder picker, and explicit PowerShell Run requests. Themes use a validated color schema, five immutable presets, localStorage persistence, and a temporary preview layer. Components own the palette and settings UI; `src/lib/native.ts` is the typed IPC boundary. Rust `lib.rs` owns window lifecycle, tray recovery, and single-instance activation. Frontend plugin capabilities allow only event subscriptions. Native commands resolve catalogue IDs for app/file actions and accept command text specifically for the user-invoked Run mode; no generic filesystem or clipboard-read plugin is exposed.

## Functionality

Installed-application discovery and launching, 14 built-in commands, known user folders, date/time, UUIDs, arithmetic, Preferences, and About. Five themes and a custom-theme editor support live preview, persistence, import/export, duplication, and deletion. Keyboard and mouse navigation, scoped search, honest errors, empty states, fixed footer, tray, and shortcut fallback are included. Discovery runs once per session and on explicit refresh; there is no filesystem-wide indexing, polling, telemetry, or remote font loading. Local launcher features work offline; cloud AI requires a configured provider and network connection.

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

Extensions, clipboard history, sync, accounts and telemetry remain deferred. Application discovery covers Start Menu shortcuts and packaged Windows apps. File search indexes selected folders and is capped at 250,000 entries, with a visible notice when reached. Search renders at most 40 matches; refine your query to find others. App rows request Windows icons lazily; files use consistent file/folder glyphs. The window supports transparent Windows backdrops with native corners/shadow; their appearance depends on Windows composition and system transparency settings. Launch success confirms dispatch to Windows, not completion of the external app's startup. The previous audit found two low-severity development-only findings in the Svelte Vite plugin's upstream deepmerge dependency.

## AI assistant and animation speed

Type `?? your question` and press Enter to start a conversation. AI uses a bottom composer; Shift + Enter adds a new line. Follow-ups include completed exchanges, and earlier questions and answers stay visible. Stop cancels the request; Retry replaces the failed attempt. The conversation stays in memory when you leave AI or hide Mycast. New conversation, switching providers, or quitting clears it. No requests are made while typing. Context includes up to 16 recent completed exchanges within 256 KB; failed and stopped answers are excluded.

Preferences → AI configures OpenAI, Claude, Gemini, Custom OpenAI-compatible, Ollama, or LM Studio. Enter an exact model ID available to your account/server. Cloud providers require your own API key; Windows Credential Manager stores it, separately per provider and endpoint. Leave the password blank to retain it or use Delete saved key to remove it. Keys never enter localStorage or configuration JSON. Custom endpoints use Chat Completions streaming with a base URL such as `https://your-server/v1`. Ollama defaults to `http://localhost:11434/v1`; LM Studio defaults to `http://localhost:1234/v1`. Start the local server and load/pull your model first. HTTP is accepted only on loopback; remote endpoints require HTTPS. Custom authentication uses a Bearer API key.

Submitted prompts and recent conversation context go to the selected endpoint; cloud usage may incur provider charges. Thinking previews appear only when the model supplies them. Optional [MCP servers](docs/mcp.md) expose local tools to models that support tool calling. Each call requires Allow once in the conversation; connected tool definitions and results are sent to the AI provider. AI has no built-in access to files, shell tools, or clipboard contents. Links open in the default browser; remote images and embedded HTML media are not rendered. Requests have a 180-second timeout and a 2 MB response limit per model round. Key storage currently requires Windows; local keyless endpoints also work on Linux. Disable the AI assistant in Modules to hide AI mode and disconnect MCP servers.

Preferences → Appearance → Animation speed adjusts motion from 0.25× to 2×. Existing animation-off and system reduced-motion preferences override speed.
