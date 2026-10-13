# CLI Hub v0.1.0

CLI Hub v0.1.0 is the first Windows release of the local desktop workspace for launching, monitoring, grouping, and interacting with multiple CLI sessions.

## Highlights

- Manage up to eight active sessions in each workspace and up to 24 across the application.
- Group sessions by their resolved working directory and switch between persistent workspace tabs.
- Launch PowerShell, Windows PowerShell, Command Prompt, Codex, or a custom executable with structured arguments and environment variables.
- Interact through the full xterm.js terminal or the multiline Send composer.
- Keep active sessions running when the main window is hidden to the system tray.
- Save local launch profiles, recent directories, workspace layout, language, color mode, and startup preferences.
- Use English or Simplified Chinese with bundled JetBrainsMono Nerd Font and Sarasa Mono SC typography.

## Validation

- 29 automated Rust tests passed; the manual release-gate test passed 10 cycles of 24 simultaneous CMD/ConPTY sessions, totalling 240 process launches.
- A real Tauri/WebView2 run passed three workspaces with eight sessions each, refresh recovery, tab switching, Send execution, page-local stop, working-directory retention, and final cleanup.
- Windows 11 installation acceptance passed NSIS installation, installed-app launch, profile persistence, autostart restoration, single-instance reuse, window hide/reopen, uninstall cleanup, clean reinstall, and same-version reinstall.
- NSIS and MSI artifacts passed version, size, SHA-256, and signature-state verification.

## Distribution notes

- The v0.1.0 installers are unsigned private-release assets. Windows may display an unknown-publisher warning.
- Cross-version upgrade validation begins with the next release because v0.1.0 has no predecessor.
- macOS and Linux remain compile-oriented targets and have not been verified on real devices.
- Fully quitting CLI Hub terminates managed sessions; full-process session restoration and attachment to externally launched terminals are not included.
