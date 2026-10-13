# CLI Hub v0.1.3

CLI Hub v0.1.3 improves session navigation and adds a first-class Claude Code launch path.

## Added

- Added persisted **Folder**, **CLI**, and **All** views for the session overview.
- Added a Claude Code preset with structured Windows detection for `claude.exe`, `claude.ps1`, and `claude.cmd`, plus the portable `claude` command on macOS and Linux.
- Added a full-window terminal focus mode: double-click the right-side terminal to enter or leave, or use the unobtrusive exit button.

## Changed

- Active-session snapshots now retain their launch preset so CLI-type grouping survives WebView refreshes.
- Existing preferences without a grouping value remain compatible and default to working-directory grouping.
- Terminal focus changes trigger a PTY resize so the selected CLI uses the available rows and columns.

## Validation

- Frontend production build and mocked WebView interaction regression.
- Rust formatting and unit/integration tests, including a real PTY `claude --version` launch.
- Windows x64 NSIS and MSI bundle generation with updater signatures.
- Static bundle version, size, SHA-256, and Authenticode-status verification.
- Public update manifest and release-asset reachability checks.

## Known limitations

- The updater package is cryptographically signed for CLI Hub's in-app verification, but the executable is not Authenticode-signed. Windows may display an unknown-publisher warning.
- macOS and Linux remain compile-oriented and unverified on real devices.
