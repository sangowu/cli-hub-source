# CLI Hub v0.1.2

CLI Hub v0.1.2 is a Windows feature release that introduces signed in-app updates and a larger three-CLI application icon.

## Added

- Added a compact **Application updates** section to Settings with current-version display, update checks, download progress, and one-click installation.
- Added a public binary-only update feed while keeping the source repository private.
- Added Tauri updater signatures and verification. Update installation is blocked until every managed CLI session has stopped.
- Added best-effort cleanup of CLI Hub updater temporary directories on application launch.

## Changed

- Replaced the application, taskbar, tray, Start-menu, NSIS, and MSI icon assets with the enlarged three-CLI fan design.
- Preserved new icon design iterations at the time of this release. These drafts have since been removed from the public source tree; current application icons remain in `src-tauri/icons/`.
- Increased the package version consistently across the frontend package, Rust crate, Tauri configuration, and lockfiles.

## Validation

- Frontend production build.
- Rust formatting and unit/integration tests, including updater-cleanup scope coverage.
- Windows x64 NSIS and MSI bundle generation with updater signatures.
- Static bundle version, size, SHA-256, and Authenticode-status verification.
- Update manifest and public release asset reachability checks.

## Known limitations

- The updater package is cryptographically signed for CLI Hub's in-app verification, but the executable is not Authenticode-signed. Windows may display an unknown-publisher warning.
- v0.1.2 is the first updater-enabled build, so a complete live self-update can first be exercised when v0.1.3 is published.
- macOS and Linux remain compile-oriented and unverified on real devices.
