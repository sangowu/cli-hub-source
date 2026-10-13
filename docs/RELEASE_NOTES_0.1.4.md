# CLI Hub v0.1.4

CLI Hub v0.1.4 improves the application icon at Windows taskbar and shortcut sizes.

## Changed

- Enlarged the three-CLI artwork by approximately 10% while retaining deliberate breathing room at the top and bottom.
- Increased effective artwork height from 12 to 14 pixels at 16 px, 18 to 20 pixels at 24 px, and 24 to 27 pixels at 32 px.
- Regenerated the Windows ICO, application PNGs, Store assets, and macOS ICNS from the retained v12 source artwork.

## Validation

- Inspected enlarged 16 px, 24 px, and 32 px ICO frames at pixel scale.
- Verified that the ICO contains 16, 24, 32, 48, 64, and 256 px frames.
- Completed a Tauri release build with the regenerated resources.
- Generated and verified Windows x64 NSIS and MSI bundles with updater signatures.
- Verified the public update manifest and release-asset reachability.

## Known limitations

- The updater package is cryptographically signed for CLI Hub's in-app verification, but the executable is not Authenticode-signed. Windows may display an unknown-publisher warning.
- Windows may retain a cached taskbar or Start-menu icon until the application is updated and restarted.
