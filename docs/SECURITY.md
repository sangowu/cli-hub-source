# Security boundaries

CLI Hub runs terminals with the desktop user's permissions. CLI programs can read files and use the network within those permissions; the hub does not sandbox them or control an agent provider's data retention. Never run untrusted launch profiles or executables.

## Local data

On Windows, the configuration directory and endpoint/preferences files grant access only to the current user's SID, SYSTEM and Administrators. Protected DACLs prevent broad inherited groups from reading the host token. Startup repairs existing endpoint/preferences files and leftover temporary files before use. Preference reads and writes also enforce file permissions. Failures are reported rather than falling back to broad access. Reparse points at protected paths are rejected. Private storage requires a dedicated directory, never the shared temporary root.

This protects files from other unprivileged local accounts; it does not protect against administrators, malware already running as the desktop user, previously opened file handles or copied tokens. Finish tasks and fully stop/restart the host after upgrading to rotate its token. Updating/reopening the interface alone preserves the old host and its running terminals.

Launch arguments and project metadata remain plaintext locally. Put credentials in temporary environment overrides, not saved arguments. No new persistent credential store is introduced. Terminal output/previews are bounded in memory; saved session cards contain metadata only. Removing a secret from a profile does not erase old backups; rotate any secret known to have been exposed.

## Frontend policy

The bundled frontend permits local scripts/assets and Tauri IPC. Unapproved inline scripts, external script sources, remote frontend connections, frames and objects are blocked. Inline styles remain allowed because React and xterm use dynamic styling. Development has a separate policy permitting Vite's inline refresh script and its localhost WebSocket; that policy is not used by the bundled frontend. Updates are downloaded by the native updater, not a frontend network exception.

## Release signatures

Tauri updater signatures remain enabled. Windows Authenticode requires a trusted code-signing certificate or a configured signing service; updater keys are not Authenticode certificates. Do not claim a verified Windows publisher until the application and installers have valid Authenticode signatures and trusted timestamps. Check those files with `Get-AuthenticodeSignature` after signing, and generate updater signatures and release hashes only after all binary signing is finished.

## Verification

- `cargo test --offline` includes Windows tests repairing a deliberately broad ACL on an isolated directory, existing configuration and temporary files. The tests inspect the resulting native DACL, exact allowed SIDs, inheritance protection and preserved content.
- `npm audit` checks frontend/build dependencies; `source-map-js` is locked to fixed version 1.2.2.
- `npm run tauri -- build --debug --no-bundle --config scripts/host-test.conf.json` builds an isolated executable with the bundled frontend. Launch it with CDP port 9224, then run `python scripts/verify_security_native.py`. It verifies active production CSP, blocked inline script/remote fetch, invalid host authentication, CMD input/output, reconnect, resize and preference saving. It never targets production sessions or installs an update.

These checks are regression tests, not a guarantee that all vulnerabilities have been found.
