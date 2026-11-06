## Downloads

- **Windows x64:** download and run the `-setup.exe` installer. WebView2 is required; the installer handles its setup when needed.
- **Linux x64 (experimental):** install the `.deb` on Debian/Ubuntu systems with WebKitGTK 4.1, or download the `.AppImage`, make it executable with `chmod +x`, and run it. AppImages may need the distribution's FUSE 2 compatibility package.
- `SHA256SUMS.txt` contains checksums for the installers.

These builds are unsigned. Linux packaging is experimental: installed-application discovery/launching, native folder/file opening and picking, PowerShell Run, startup registration, and Windows backdrop materials currently remain Windows-only. Linux builds provide the launcher UI, calculation/clipboard actions, themes, personalization, and in-app shortcuts; desktop shortcut/tray behavior depends on the desktop environment.
