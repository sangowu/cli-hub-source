# CLI Hub v0.1.6

CLI Hub v0.1.6 adds optional startup session restoration and configurable terminal shortcuts, and fixes the current grouping indicator in dark mode.

- Enable **Restore CLI sessions on startup** in Settings to restart the last running CLIs in their original workspaces and launch directories. The setting is off by default. Launch arguments are stored locally; terminal output, temporary environment overrides and CLI conversation state are not restored.
- With the default shortcuts, **Ctrl+C** copies selected text and interrupts the CLI when there is no selection. **Ctrl+Shift+C** copies; **Ctrl+V** and **Ctrl+Shift+V** paste using xterm's bracketed-paste handling. Clear the selection before using Ctrl+C to interrupt.
- Settings includes shortcut explanations, editable copy/paste/interrupt mappings, duplicate mapping checks, a reset button and an option to disable selection-aware copying. Native clipboard access avoids WebView2 permission waits.
- The current **Folder / CLI / All** grouping now has a visible checkmark, highlighted background and bottom indicator, including in dark mode.

Validation: frontend build; Rust formatting and 32 passing tests with one intentionally ignored manual stress test; existing UI regression; and real Windows Tauri/WebView2 checks for clipboard operations, interrupting a running command, live key remapping, persisted mappings, two-workspace restoration after a full exit, refresh without duplicate sessions and stopped-session cleanup.

Windows x64 installers use Tauri updater signatures. They do not have Authenticode signatures, so Windows may show an unknown-publisher warning.
