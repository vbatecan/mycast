# Installed applications

Mycast discovers applications from the per-user and all-users Start Menus and the Windows AppsFolder catalogue for packaged/Store apps. This includes redirected Start Menu locations supplied by Windows. The first lookup runs on a native worker so the launcher and its built-in commands remain usable while discovery completes.

Type an application's name, executable keyword, or Start Menu group to find it. **Applications** narrows the list to apps; **Commands** keeps utilities and preferences easy to reach. Ctrl + Tab cycles these filters. Enter or a click asks Windows to open the selected application and hides the launcher after dispatch.

Use the refresh icon after installing or removing software. Refresh is explicit: Mycast does not watch folders, scan entire drives, or poll for changes. Existing cached launches stay available while a refresh runs. Up to 40 matching entries are rendered at once; a more specific query can reach any indexed entry.

Desktop entries launch the original `.lnk` through the Windows Shell, preserving its arguments, working directory, and activation behavior. Store apps use Windows application activation. The frontend passes an opaque catalogue ID; native code looks up the target it discovered, rather than accepting an executable path or shell command from the UI.

Duplicate targets with the same arguments and working directory are combined. Entries with different arguments remain separate. Discovery skips unreadable shortcuts and reparse points and applies bounds to depth, count, and shortcut size. Warnings appear beside the app count, with details on hover. Failed refreshes preserve the previous catalogue where possible. Missing/removed shortcuts return an error when launched.

Portable apps without a Start Menu entry are not automatically discovered. Add a normal Windows Start Menu shortcut and refresh to include one. Catalogue IDs are stable through refresh within one process and are not intended as persistent favourites identifiers.

Personalization uses a separate versioned SHA-256 fingerprint of each native launch identity. Built-in commands use fixed keys; a discovered Windows app that replaces a built-in fallback inherits its saved favorites, aliases, and usage. Search queries, calculations, generated values, and clipboard contents are never saved in personalization. Missing favorites and aliases remain manageable as unavailable items under Preferences → Commands.

Empty search shows up to eight Favorites in pin order, eight nonduplicate Recents, then Suggested items within the 40-row limit. Favorites and Recents filters search their full collections; refine your query to find entries beyond the visible limit. The latest 50 distinct successful items are retained in Recents. Text relevance takes priority over learning; exact aliases come first, followed by label/prefix/substring/fuzzy classes, then favorites and usage within each class. Use Ctrl + D to pin or unpin, or use the selected-item footer controls.

Aliases are case-insensitive, unique across items, and allow up to ten per item. Each has at most 32 characters, no whitespace/control characters, and at least one letter. General preferences can clear recents and learned ranking while preserving favorites and aliases. Background startup defers application discovery until the palette first opens.

Application rows use Windows-provided icons, loaded lazily as rows become visible. Only three icon requests run concurrently, and caches are bounded to keep memory predictable. A consistent application glyph appears while an icon loads or when Windows cannot provide one. Built-in commands retain their own Lucide icons.
