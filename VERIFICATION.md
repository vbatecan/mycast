# Verification recorded during implementation

## v0.1.1 controls and administrator launching

Implemented editable Shift+Enter administrator launching, native executable capability checks, preserved shortcut arguments/working directories, UAC cancellation without history updates, focus-loss protection while elevation is pending, and foreground restoration after cancellation/failure. Application controls now expose Open/admin and file Reveal/Copy path actions, retain per-item alias drafts, restore alias-input focus after removal, and retain the nearest remaining selection when filtered. Startup copy now explains the existing hidden sign-in and visible manual-launch behavior.

Svelte/TypeScript and Windows Rust compilation passed. Automated tests remained paused. Both browser and native desktop review tools were unavailable (browser host unavailable; native pipe unavailable), so visual walkthroughs, actual sign-in, and interactive UAC consent/cancellation were not performed. Do not interpret installer build success as completion of those manual checks.

## Alt + Shift filter shortcuts — 2026-11-07

Changed the six filter defaults to Alt + Shift + 1–6 and previous/next filter defaults to Alt + Shift + Left/Right. The existing shortcut editor and tooltips use these defaults automatically. A one-time preferences migration upgrades the previous Ctrl + Alt number bindings and Ctrl + Tab cycling bindings without changing custom bindings; a conflicting custom binding takes priority. The migration version is persisted so users can subsequently reassign any valid binding without it being migrated again. Existing shortcut regression sources and the README were updated. Tests remained paused as requested.

## Commands selection and controls — 2026-11-07

Replaced the narrow all-in-one column with a two-pane editor. The 200-pixel left pane contains searchable commands with native application icons, selection highlighting, pin indicators, and keyboard selection. The right pane shows the selected item's icon, name, description, availability, favorite toggle and position controls, and inline alias chips with add/remove controls. Alias validation uses the existing shared rules and saves immediately. Each pane scrolls independently beneath the fixed settings navigation.

Tests remained paused at the user's request. The frontend was compiled as part of building the release executable.

## Narrow Commands column — 2026-11-07

Commands preferences now uses a 180-pixel left column with a subtle right divider. The heading, search, saved-only filter, command names, categories, aliases, and action buttons stay within that column. Categories stack below names, buttons wrap when necessary, and the command list scrolls below the fixed controls. The description and search placeholder were shortened to fit. No tests were run, as requested; the production frontend was compiled as part of rebuilding the desktop executable.

## Simplified launcher and editable shortcuts — 2026-11-07

Removed the Local search badge, installed-application count/refresh area, and idle indexed-file count from the launcher. Application refresh is available in General preferences. Transient search/index progress and errors remain visible when needed.

Added Ctrl + Alt + 1–6 for All, Applications, Commands, Files, Favorites, and Recents. Preferences → Shortcuts manages these bindings plus pinning, filter cycling, result navigation, opening, closing/back, and the existing global launcher shortcut. Changes persist locally, duplicate bindings are rejected, and reset restores defaults. Filter tooltips and footer hints follow the saved bindings. Standard text editing stays available.

The user requested stopping tests during implementation. No additional tests were run after that instruction; the remaining work was the release build and relaunch.

## Recents first and configurable limit — 2026-11-07

The release executable rebuilt successfully and was relaunched with this change.

Recently opened programs, files, and folders now lead the empty-query home view in newest-first order, followed by nonduplicate Favorites and Suggested items. Preferences and clipboard actions do not occupy the Recent section. General → Recent items defaults to 5, accepts 1–20, persists locally, and caps both home Recents and the Recents filter. Existing successful-use history is retained so increasing the limit can reveal more entries; text matching outside the Recents filter keeps its relevance ordering.

Svelte/TypeScript passed without errors or warnings; the frontend production build passed. Twelve focused search/personalization tests passed, covering ordering, limits, folder eligibility, deduplication, relevance, and history. A focused browser integration check passed for recent folders preceding pinned programs, the default cap, changing the cap to three, and persistence after reload. The screenshot `artifacts/recent-first.png` was visually reviewed. Full suites were not run.

## Selected-command bar removal — 2026-11-07

Removed the launcher strip containing the selected command name and Pin/Edit aliases (and file action) buttons, along with its unused dialog state and styles. The command list now extends to the main footer. Ctrl + D still pins the selected item; Preferences → Commands manages pins and aliases.

Svelte/TypeScript passed with zero errors and warnings, and the frontend production build passed. Two focused browser checks passed: fixed-footer layout/no external requests, and keyboard pinning plus alias persistence through Commands preferences. The updated launcher screenshot was visually reviewed in `artifacts/launcher.png`.

## Commands layout and window materials — 2026-11-07

Commands uses a fixed heading/search/filter area above a scrolling list, with names and actions on the left and categories on the right. Appearance now includes Solid, Transparent, Blur, Acrylic, and Mica, background opacity, a highlighted border, and a soft accent glow. Background tint is separate from text opacity. Preferences persist, older preference objects migrate, and failed native materials fall back to Solid with a visible explanation.

Focused checks passed:

- Svelte/TypeScript: zero errors and warnings; frontend production build and formatting passed.
- Rust formatting and `cargo check` passed. A bundled debug desktop build passed.
- Three focused Playwright tests passed: Commands scrolling with fixed search/footer plus pin/alias actions; material/opacity/finish persistence; legacy and malformed preference recovery, including 125% text without horizontal overflow.
- Native WebView2 integration switched Solid → Transparent → Blur → Acrylic → Mica → Acrylic without errors. An opacity change updated the CSS tint without invoking the native material command again. The probe restored the user's original preferences before exiting.

Screenshots were visually reviewed in `artifacts/commands-settings.png`, `artifacts/appearance-effects.png`, `artifacts/native-commands-settings.png`, and `artifacts/native-appearance-effects.png`. The final native probe preserved the production transparent-window configuration and asserted a 720 × 480 viewport. Browser previews approximate native backdrops. The isolated native probe used a debug-only port override; production configuration has no debugging port. Full suites and installer generation were not run for this change. Native checks confirm successful API calls; material appearance still depends on Windows composition and system transparency preferences.

The bundled release executable rebuilt successfully and was relaunched from `src-tauri/target/release/mycast.exe`.

## File search and PowerShell Run — 2026-11-06

Implemented native filename/folder search across the six Windows user folders, configurable roots/exclusions, persisted index metadata, event-driven updates, All/Files integration, file personalization, and explicit `>` Run mode. Full suites remain paused. Checks for this milestone were limited to compilation and focused feature checks.

| Check                           | Result                                                                                                                                                                                                                                                                  |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Svelte/TypeScript               | Passed, zero errors or warnings                                                                                                                                                                                                                                         |
| Frontend production build       | Passed; approximately 146 kB JavaScript and 44.2 kB CSS before compression                                                                                                                                                                                              |
| Rust formatting and compilation | Passed                                                                                                                                                                                                                                                                  |
| Filtered native checks          | Three passed: ranking/aliases before result limiting, path boundaries/cache roundtrip, and the opt-in 100,000-entry profile                                                                                                                                             |
| Warm filename search profile    | 100,000 synthetic entries, 18 queries: median 44–46 ms, 95th percentile 56–57 ms after optimizing ranking; initial profile was 135 ms p95                                                                                                                               |
| Profile memory                  | Peak standalone test-process working set 61.4 MiB; this excludes a live WebView2 app and is not a whole-application memory measurement                                                                                                                                  |
| Native feature integration      | Passed fuzzy and Unicode matching, multiple aliases, create/delete notifications, same-volume rename identity, invalid ID rejection, All/Files UI, pin selection, refresh persistence, and process-restart persistence after a file was renamed while Mycast was closed |
| PowerShell integration          | A visible terminal executed an owned probe, preserved quotes/semicolons/Unicode, used the selected working folder, and exited when explicitly instructed                                                                                                                |
| Appearance compatibility        | All 16 combinations of density/text size/descriptions rendered Files results, action controls, and Files & Run settings without horizontal overflow; zero page runtime errors                                                                                           |

The native smoke helper temporarily used a fixture in `artifacts` and restored the prior roots, terminal directory, and personalization. Its debug-only WebView2 port was supplied through an artifact override; the production configuration has no debugging port. Screenshots are `artifacts/file-search-results.png`, `artifacts/run-mode.png`, and `artifacts/files-settings-125.png`. The updated settings tab IDs and Files refresh control were included in the final compilation/layout checks.

The final bundled release executable built successfully (5,259,264 bytes) and was left running. A Windows startup check confirmed one hidden palette after `--background`, normal second-instance reopening, and the second process exiting. The restored six-folder index contained 1,045 entries and a 295,092-byte metadata cache. No production debugging port was enabled.

A single current-release startup/index-idle snapshot across Mycast and six WebView2 descendants measured 107.9 MiB private working set, 190.9 MiB private commit, and 433.1 MiB summed working set. The palette had a visible main window at inspection. Shared pages can be counted repeatedly in summed working sets; this snapshot is not a hidden-state or 100,000-entry live-app benchmark.

The 100,000-entry result measures warmed native matching, rather than cold scanning 100,000 physical files or end-to-end rendering latency. Full test suites, broad filesystem/provider coverage, manual folder-picker/default-file-open/Explorer-reveal/clipboard checks, and classic-console/Windows-PowerShell fallback execution remain unperformed. Installer generation and whole-app memory targets were not revalidated. See [Files and PowerShell](docs/files-and-run.md) for behavior and targeted check commands.

## Personalized launching and appearance — 2026-11-06

Implemented Favorites, Recents, aliases, learned ranking, release-only startup registration, configurable launcher shortcut, density, text sizing, and Manual/Follow Windows theme pairs. Full test suites remain paused; the following were focused compilation, startup, and integration checks.

| Check                                 | Result                                                                                                                                                                                     |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Svelte/TypeScript                     | Passed, zero errors or warnings                                                                                                                                                            |
| Frontend production build             | Passed; approximately 136 kB JavaScript and 42.6 kB CSS before compression                                                                                                                 |
| Rust formatting and cargo check       | Passed                                                                                                                                                                                     |
| Bundled debug desktop build           | Passed; `src-tauri/target/debug/mycast.exe`                                                                                                                                                |
| Hidden background startup             | Running process with one Mycast palette window; palette visibility false after `--background`                                                                                              |
| Normal second-instance activation     | Second process exited and the existing palette was observed visible                                                                                                                        |
| Browser startup                       | Rendered successfully, zero page runtime errors                                                                                                                                            |
| Density/text/description combinations | All 16 combinations rendered without horizontal overflow in the fixed 720 × 480 window; minimum rows 52/40 px, growing as needed                                                           |
| Focused persistence integration       | Pin preserved selected command after reranking; multiple aliases and favorites survived refresh; failed native dispatch added no history; successful About navigation persisted in Recents |
| Focused appearance state checks       | Legacy migration, media change events, custom theme pairs, preview cancellation, deletion fallback, and text-size validation passed with isolated mocked storage/media                     |

Added regression source in `tests/personalization.test.ts` and `e2e/launcher.spec.ts` for ranking precedence, alias validation/conflicts, home deduplication, fallback identity continuity, unavailable entries, recent limits, selection stability, and successful/failed execution history. The complete unit, UI, and native smoke suites were not run. Screenshots are in `artifacts/personalized-*.png`.

The bundled release build also passed: `src-tauri/target/release/mycast.exe`, 3,718,656 bytes. Hidden background startup and normal second-instance reopening were confirmed for both debug and release executables. The updated release app was left running; startup registration remains off. Debug builds intentionally disable the startup switch.

Windows sign-in, actual startup enable/disable registration, physical shortcut presses, and live shortcut-conflict rollback remain unperformed. Capture cancellation and registration rollback were reviewed in source and compiled. The installer and memory benchmarks below predate this milestone.

## Earlier verification

**Historical phase two status:** installed-app discovery/launching and custom themes were implemented after the request to stop testing. At the subsequent request to run the app, the Tauri development build caught a Windows string-conversion type mismatch; it was fixed and the app started successfully. Full test suites remain paused. The measurements and passes below describe the preceding MVP only; they are not verification of the current release.

Testing was stopped at the user's request. The following results had already been obtained on Windows on 2026-11-06.

| Check                                 | Result                                                                      |
| ------------------------------------- | --------------------------------------------------------------------------- |
| Dependency installation               | Passed; npm lockfile included                                               |
| Svelte and TypeScript checks          | Passed, zero errors or warnings                                             |
| Calculator/search unit tests          | 10 passed, including a 500-command search fixture                           |
| Production browser integration tests  | 8 passed using Microsoft Edge                                               |
| Frontend production build             | Passed; approximately 87.4 kB JavaScript and 21.8 kB CSS before compression |
| Prettier                              | Passed before final documentation and optional smoke-script edits           |
| Rust cargo check                      | Passed                                                                      |
| Rust formatting                       | Passed                                                                      |
| Clippy, all targets, warnings denied  | Passed                                                                      |
| Tauri release build without installer | Passed; executable 3,552,256 bytes                                          |
| Production npm dependency audit       | Zero vulnerabilities                                                        |

Browser checks covered input focus, wrapping arrow navigation, search selection, arithmetic examples, division by zero, empty states, mouse selection/execution, normal select-all behavior, About navigation, settings persistence, disabled startup control, fixed footer, and no external page requests. Launcher, calculator, settings, and empty-state screenshots were rendered and visually reviewed. A native WebView2 screenshot was also reviewed.

Native integration used a separate debug build with an explicit test-only CDP option. The production executable and configuration contain no debugging port or diagnostic logging. Successful checks included actual Alt + Space registration, calculator Enter-handler-to-clipboard integration, date/time/UUID clipboard writes, invalid action rejection, clipboard length enforcement, denied clipboard-read permission, second-instance activation, and approved Notepad dispatch. The optional smoke script was subsequently updated to create its own temporary build override; that final convenience update was not rerun after testing was stopped.

The automation host steals foreground focus, triggering the required native focus-loss hiding. CDP keyboard routing therefore needed explicit DOM dispatch in the native integration test; normal keyboard behavior was tested separately in the browser. Physical global keypresses, visual tray behavior, and native focus/visibility transitions still require manual desktop QA. The desktop inspection bridge was unavailable. Windows Settings and all known-folder launches were implemented and compiled but were not individually exercised. NSIS installer generation was not tested. A blank Notepad window was opened by the smoke test.

## Memory snapshot

A separate unmodified **release** process was measured after a 15-second warm-up, including all six WebView2 descendants. No debugger was attached to that release process.

| Metric, entire seven-process tree |   MiB |
| --------------------------------- | ----: |
| Summed working set                | 371.6 |
| Private working set               |  86.1 |
| Private committed memory          | 242.7 |

Summed working sets can count shared pages more than once; private commit includes memory that is not resident. These metrics are deliberately reported separately. Native window visibility could not be independently established in this automation environment, so this is a startup-idle snapshot, **not** a validated visible/hidden benchmark. The hidden 50 MB target and active-use target remain unverified; the summed-working-set target was not met. The measurement script supports repeatable manual snapshots in both states and reports unavailable performance counters as unavailable.

No polling, filesystem indexing, telemetry, external fonts, application network calls, or repeating frontend timers were introduced. Vite development HMR and the optional test-only debugging endpoint are development tools; neither is part of a normal release launch. Two low-severity development-only npm findings remain in the Svelte Vite plugin's upstream deepmerge dependency.

The release executable was rebuilt and relaunched after removing the selected-command bar.
