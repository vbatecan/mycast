## Downloads

## v0.3.1

- Record global launcher shortcuts with Alt and either Windows key without opening the Windows system menu or triggering the recorded combination. Key releases, cancellation, and modifiers held before recording are handled natively on Windows.
- Fix the updater resource-close permission that caused `plugin:resources|close not allowed by ACL`. If an older installation still reports this error, install v0.3.1 from GitHub to apply the fix.
- Keep matching file results visible while the next search runs. Delayed progress appears in a fixed footer area, preserving keyboard hints and preventing the result list from jumping while typing.
- Add specific empty states for Files, Applications, Favorites, and Recents, with Search All and Manage folders actions where relevant.
- Add plain-language search errors, Retry actions, and expandable technical details. Pending searches wait before showing a no-matches state, and empty states no longer slide into place.
- Keep indexing counts in Settings only; the launcher shows a quiet preparation message.
- Validation: frontend checks/build, 25 unit tests, 13 native tests, and 13 focused browser tests passed. Broader browser tests have known failures also reproduced on the original code. Physical-key desktop testing and a live updater installation remain unverified.

## v0.3.0

- Web search: type `? search terms` to open your default browser. Preferences → Search offers Google, Bing, DuckDuckGo, and a custom URL template.
- Windows search: find 31 Settings pages and system tools, including Add or Remove Programs, Bluetooth, Windows Update, Task Manager, and Device Manager. Favorites and aliases are supported.
- AI assistant: type `?? your question` and press Enter for a streamed Markdown answer, with empty states, provider-supplied thinking previews, Stop, Retry, and Copy Markdown.
- Bring your own API key for OpenAI, Claude, Gemini, or custom OpenAI-compatible endpoints. Ollama and LM Studio local endpoints are also supported. Configure the provider, exact model ID, endpoint, and key in Preferences → AI. Windows Credential Manager stores keys; cloud usage may incur provider charges.
- AI responses are session-only and clear when leaving AI mode or hiding Mycast. Prompts are sent only on submission. Cloud providers were tested with protocol fixtures, not live paid credentials.
- Adjustable animation speed from 0.25× to 2× under Appearance, respecting animation-off and system reduced-motion settings.
- Separate module switches for Web search, Windows settings and tools, and AI assistant.
- Validation: 24 unit tests, 4 native AI tests, and 5 focused browser tests passed, along with frontend type checks/build and Rust compilation. The broader UI suite and strict Clippy still have known failures outside this feature work.
- Linux remains experimental; AI key storage and Windows system commands require Windows. Local keyless AI endpoints are supported on Linux.

## v0.2.0

- Advanced calculations powered by fend: scientific notation, powers, trigonometry, complex numbers, and unit conversions. Examples: `1.5e6 / 3`, `sin(pi / 2)`, `5 ft to cm`. Expressions run locally with a time limit; calculation contexts are stateless.
- Currency conversion such as `100 USD to SGD`, using Frankfurter reference rates. Rates are cached locally, labelled with their date, and reusable offline. These are reference conversions, not live exchange quotes.
- Settings → Modules enables/disables Applications, Files, Run, Calculator, Currency, and Utilities. Saved personalization stays intact. Files stops its worker and watcher when disabled.
- Settings → Updates checks GitHub Releases for signed updates. Automatic checks ask before download/install. An indicator appears when an update is available.
- **Install v0.2.0 manually once if upgrading from v0.1.x.** Earlier versions do not include the updater. Windows and Linux AppImage support in-app installation; `.deb` installations use manual/package-manager updates.
- Signing verifies updater payloads; installers are still not Authenticode signed. Automated tests remain paused; compilation and packaging are checked.

## Installation

- **Windows x64:** download and run the `-setup.exe` installer. WebView2 is required; the installer handles its setup when needed.
- **Linux x64 (experimental):** install the `.deb` on Debian/Ubuntu systems with WebKitGTK 4.1, or download the `.AppImage`, make it executable with `chmod +x`, and run it. AppImages may need the distribution's FUSE 2 compatibility package.
- `SHA256SUMS.txt` contains checksums for the installers.

These builds are unsigned. Linux packaging is experimental: installed-application discovery/launching, native folder/file opening and picking, PowerShell Run, startup registration, and Windows backdrop materials currently remain Windows-only. Linux builds provide the launcher UI, calculation/clipboard actions, themes, personalization, and in-app shortcuts; desktop shortcut/tray behavior depends on the desktop environment.
