# CLI Hub v0.1.14

Protect local application files and fix session panel resizing.

- Restrict the terminal host directory, endpoint token, preferences and temporary preference files to the current Windows user, SYSTEM and Administrators. Repair existing files when connecting to the host, and reject reparse points at protected storage paths.
- Enable production Content Security Policy to block inline scripts and remote frontend connections. Update the development-only source-map-js dependency to address its reported denial-of-service issue.
- Restore two session-card columns reliably after shrinking and expanding the sidebar, including fractional display scaling with classic scrollbars.
- Keep the terminal grid stable while dragging either divider, then fit it on release. Avoid redundant PTY resize calls to reduce CLI redraw flicker.
- Match Settings action buttons to the 170 × 34 px input and dropdown controls in English and Chinese.

Existing session and profile formats remain compatible. File access restrictions do not revoke an endpoint token already copied before upgrading. After active work finishes, use **Stop all and exit**, then reopen CLI Hub to start the updated terminal host and rotate its token. Updating the interface alone intentionally preserves running sessions.

Saved launch arguments remain plaintext local configuration; avoid putting secrets in saved arguments. No terminal transcripts are added to persistent storage. See [Security](SECURITY.md) for scope and remaining limitations.

Windows 10/11 x64 and WebView2 are required. Installers carry verified Tauri updater signatures, but no Authenticode signature; Windows may show publisher or SmartScreen warnings.

Validation: production build; 43 Rust tests passed (one manual stress test ignored); dependency audit with zero reported vulnerabilities; shortcut, session history, preview, sidebar and interaction regressions; isolated native Windows Tauri tests for CSP enforcement, host authentication, CMD/PowerShell input and output, panel resizing, fullscreen and terminal bounds; English/Chinese Settings control geometry. Native tests use isolated application storage and test sessions.
