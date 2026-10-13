# CLI Hub v0.1.12

More space for CLI sessions, correctly fitted terminal output, and useful reply previews.

- Fix terminal content being clipped at the bottom/right by applying terminal padding where xterm's FitAddon accounts for it. Terminal rows and columns now fit the available content area after panel resizing and focus-mode transitions.
- Limit the expanded session sidebar to 190–360px: one to two fixed-width mini cards, including spacing and the scrollbar. Narrow windows retain a single column; the collapsible icon rail is unchanged.
- Prefer the latest recognized Claude/Codex reply in mini cards, excluding input prompts and common status footers. Other CLI formats fall back to recent filtered output. Recognition uses bounded rendered-text heuristics, so arbitrary CLI formats and remapped agent interfaces may not be recognized. Previews remain in memory and are not saved as transcripts.
- Remove the two introductory header lines, leaving more vertical space for the workspace.

Compatible live CLIs can continue through interface updates from host-enabled versions. This release does not change the terminal-host protocol or card metadata storage.

Windows 10/11 x64 and WebView2 are required. NSIS and MSI installers have Tauri updater signatures but no Authenticode signature.

Validation: frontend build; latest-reply parsing, shortcut and session-history tests; combined UI/update/sidebar regressions; eight English/Chinese, light/dark and normal/minimum-window sidebar checks; one/two-column and minimum/large viewport geometry checks; real isolated Windows Tauri/PowerShell output, sidebar resize, terminal focus/restore and bottom/right canvas bounds.
