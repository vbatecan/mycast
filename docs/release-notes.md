## Downloads

## v0.1.1 application controls

- Shift+Enter requests **Open as administrator** for supported Windows desktop applications. Change this binding in Settings → Shortcuts. Windows handles UAC; Mycast remains non-elevated. Cancelling does not add a recent item.
- Commands preferences includes Open, administrator launch, and file Reveal/Copy path controls. Alias drafts survive switching between items, and removal returns focus to the alias field.
- Sign-in startup begins directly in the tray; manually opening Mycast shows the launcher. Discovery and indexing remain deferred until opening.
- Automated tests remain paused. Frontend/native compilation and installer builds are checked; interactive UAC and sign-in verification require manual review.

- **Windows x64:** download and run the `-setup.exe` installer. WebView2 is required; the installer handles its setup when needed.
- **Linux x64 (experimental):** install the `.deb` on Debian/Ubuntu systems with WebKitGTK 4.1, or download the `.AppImage`, make it executable with `chmod +x`, and run it. AppImages may need the distribution's FUSE 2 compatibility package.
- `SHA256SUMS.txt` contains checksums for the installers.

These builds are unsigned. Linux packaging is experimental: installed-application discovery/launching, native folder/file opening and picking, PowerShell Run, startup registration, and Windows backdrop materials currently remain Windows-only. Linux builds provide the launcher UI, calculation/clipboard actions, themes, personalization, and in-app shortcuts; desktop shortcut/tray behavior depends on the desktop environment.
