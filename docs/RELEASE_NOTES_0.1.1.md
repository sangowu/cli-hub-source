# CLI Hub v0.1.1

CLI Hub v0.1.1 is a private Windows patch release that updates the application branding and installer assets.

## Changed

- Replaced the application, taskbar, tray, Start-menu, NSIS, and MSI icons with the transparent CLI cascade design.
- Retained icon source artwork and earlier design iterations at the time of this release. These drafts have since been removed from the public source tree; current application icons remain in `src-tauri/icons/`.
- Increased the package version consistently across the frontend package, Rust crate, Tauri configuration, and lockfiles.

## Validation

- Frontend production build.
- Rust formatting and unit/integration tests.
- Windows x64 NSIS and MSI bundle generation.
- Static bundle version, size, SHA-256, and signature verification.
- Cross-version NSIS upgrade from v0.1.0 to v0.1.1 with profile preservation.
- Installed executable and Start-menu shortcut icon verification.

## Known limitations

- The installers are unsigned private-release assets. Windows may display an unknown-publisher warning.
- macOS and Linux icon assets are generated but remain unverified on real devices.
