# CLI Hub v0.1.8

CLI Hub v0.1.8 adds a collapsible session sidebar to give the terminal more room while keeping session switching available.

- Click **«** to collapse the sidebar into a 52px icon rail, and **»** to expand it.
- Icons show CLI abbreviations, session numbers, running/exited/failed status dots and selected highlighting. Hover for the name, CLI type, status and directory; click to switch without expanding.
- The rail follows the current grouping order and scrolls vertically when needed. Expand to use grouping, previews, stop and remove actions.
- Folding preserves live terminal instances and restores the expanded width and each workspace's list scroll position during the current run. The global collapse preference survives a full restart; older preferences default to expanded.
- The minimum-window layout keeps the rail at 52px. Labels, focus states and theme styling follow the documented UI design system.

Validation: frontend build; existing mocked UI regression; eight English/Chinese, light/dark, normal/minimum-window sidebar checks; Rust formatting and 33 passing tests with one intentionally ignored manual stress test. An isolated real Windows Tauri/WebView2 instance verified ConPTY column changes, unchanged process IDs, preserved terminal selection, icon switching, direct/composer input, expanded width restoration and the saved collapse preference after a full process restart.

Windows 10/11 x64 with WebView2 Runtime is required. NSIS and MSI installers use Tauri updater signatures; they do not have Authenticode signatures, so Windows may show an unknown-publisher warning.
