# CLI Hub v0.1.5

CLI Hub v0.1.5 adds adjustable workspace dividers while preserving practical limits for the terminal area.

## Added

- Drag the vertical divider to resize the session overview between 190 and 420 pixels, capped at 40% of the workspace.
- Drag the horizontal divider to resize the input panel between 160 and 360 pixels, capped at 45% of the available workspace height.
- Use arrow keys on a focused divider for precise adjustment, Home or End for its bounds, and double-click to restore the default size.

## Changed

- The terminal now observes its container size and refits xterm.js after either divider moves.
- Narrow windows retain the existing single-card-width session overview.
- The multiline composer expands with the resized input panel while keeping Send aligned to the input area.

## Validation

- Production frontend build and TypeScript compilation pass.
- Mocked WebView interaction verification covers both drag directions, maximum bounds, double-click reset, narrow-window layout, and terminal resizing.
- Rust formatting and unit tests pass.
- Windows x64 NSIS and MSI bundles are generated and verified with updater signatures.

## Known limitations

- Divider sizes reset to their defaults after the application restarts.
- The updater package is cryptographically signed for CLI Hub's in-app verification, but the executable is not Authenticode-signed. Windows may display an unknown-publisher warning.
