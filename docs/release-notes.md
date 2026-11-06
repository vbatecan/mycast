## Downloads

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
