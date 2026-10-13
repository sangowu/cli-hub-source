# CLI Hub Roadmap

## Delivery method

Development follows the project SWIFT rules:

- **Small slices:** one usable vertical slice at a time.
- **Working baseline:** every completed stage builds and runs.
- **Immediate verification:** validate changes before expanding scope.
- **Focused scope:** no speculative features or frameworks.
- **Traceable decisions:** record important boundaries and evidence.

## Stage 0 — Project baseline

Status: complete.

- Tauri 2, React, TypeScript, Vite, and Rust scaffold.
- xterm.js and portable-pty selected as terminal boundaries.
- Windows toolchain validated.
- Frontend production build and Rust test path established.
- Cross-platform code boundary documented.

## Stage 1 — Single terminal loop

Status: implemented; automated PTY smoke passed and the real window was visually verified. Manual interaction acceptance remains.

- Start one PowerShell process through portable-pty.
- Stream PTY output to xterm.js.
- Send text input and direct terminal keyboard input.
- Resize the PTY with the terminal viewport.
- Send Ctrl+C, Escape, and Tab.
- Stop the child process and report exit state.

Exit gate:

- Automated: a real PowerShell PTY starts, accepts commands, preserves English and Chinese output, and exits cleanly.
- Automated: Stop terminates a real PowerShell process and confirms its observed exit without the Windows `portable-pty 0.9.0` false error.
- Automated: terminal size sanitization, optional working-directory handling, frontend compilation, and Rust formatting pass.
- Visual: the real Tauri window renders the idle state, working-directory field, xterm area, and controls correctly.
- Manual acceptance: verify direct typing, ANSI cursor movement, resize, Ctrl+C, and Stop in the real Tauri window.

## Stage 2 — Generic CLI launch

Status: complete for the Stage 2 baseline; automated PTY verification and final compact-layout verification passed.

- Add executable, arguments, working directory, and environment fields.
- Support PowerShell, CMD, Codex, Claude Code, and user-defined commands.
- Report missing commands and invalid directories without fallback.

Exit gate:

- Automated: PowerShell, Windows PowerShell, CMD, Codex, and Claude Code presets resolve on Windows.
- Automated: CMD, Codex, and Claude Code `--version` run through real PTYs and exit successfully.
- Automated: custom executable paths preserve argument boundaries and environment values.
- Automated: missing executables, duplicate environment variables, and invalid directories return explicit errors.
- Visual: all Stage 2 launch fields, the terminal, and input controls are visible together at the default 1100×760 window size.
- Manual follow-up: starting each preset from the UI remains a recommended user acceptance check; the same launch paths are covered by real-PTY automated tests.

## Stage 3 — Multiple sessions

Status: implementation complete; automated PTY and frontend build gates passed. Manual real-window interaction acceptance remains.

- Add independent session identifiers and PTY channels.
- Add bounded output queues and batching.
- Verify at least eight concurrent sessions without cross-talk.
- Ensure terminal closure releases its resources.

Exit gate:

- Automated: eight real CMD PTYs run concurrently with unique markers and no cross-talk.
- Automated: output order is preserved through a per-session 64-slot bounded queue and 8 ms / 32 KiB batching.
- Automated: the process wait path removes the session, drains final output, and then emits its exit event.
- Automated: Rust formatting, Rust tests, and the production frontend build pass.
- Manual follow-up: switch between two live sessions, send distinct input to each, stop one, and confirm the other remains interactive.

## Stage 4 — MVP workspace

Status: implemented; mocked WebView interaction and layout verification passed at 1100×760. Manual real-Tauri interaction acceptance remains.

- Default to normalized working-directory grouping, with persisted CLI-type and ungrouped views.
- Add session cards with name, status, and recent output.
- Add selected-session terminal and simple input controls.
- Restore active cards and bounded recent output after a WebView refresh.
- Add a workspace action to stop all active sessions.

Exit gate:

- Automated: Windows grouping keys normalize separators and path case in Rust.
- UI: three sessions render as two directory groups with independent recent-output previews.
- UI: selecting a card changes the full terminal, and an exited session uses a yellow card background.
- UI: exited and failed cards expose a red top-right × that removes the inactive session panel.
- UI: the left overview and right full terminal remain visible together at 1100×760.
- UI: the overview width and composer height have keyboard-accessible drag dividers, with bounded sizes that preserve the terminal workspace.
- UI: launch parameters open from a New session dialog, and overview/xterm scrollbars share one visual treatment.
- UI: refreshing the WebView recovers active cards, recent output, grouping, input, and the correct active count.
- Manual follow-up: repeat the grouping, selection, preview, and stopped-state checks in the real Tauri window.

## Stage 5 — Persistence and safe shutdown

- Status: complete; automated UI verification passed, with manual real-window acceptance recommended.
- Save launch profiles and recent directories locally. Implemented.
- Warn before exiting with active sessions. Implemented.
- Keep full terminal transcripts ephemeral by default.

Safe-shutdown exit gate:

- No active sessions: the native close request proceeds immediately.
- Active sessions: closing opens a confirmation dialog and cancel keeps every session running.
- Confirmed exit: all managed children are stopped and observed as exited before the app exits.

Persistence exit gate:

- Profiles can be saved, updated, deleted, and loaded after a WebView refresh.
- Successfully resolved working directories appear as recent choices, capped at 10.
- Launch profiles, recent directories, and UI settings use a versioned application configuration file instead of origin-scoped WebView storage.
- Existing WebView-local preferences migrate once per development or packaged origin; environment variables and terminal output remain excluded.

## Stage 6 — Windows v0.1.x release readiness

- Status: complete for private distribution; public distribution remains gated on code signing.
- Pre-release interaction slice: multiline composer, Shift+Enter line breaks, numbered-list continuation, and local file path attachments are implemented and covered by mocked WebView verification.
- Composer submission honors terminal bracketed-paste mode and sends Enter after the CLI paste-burst protection window, preventing agent CLIs from turning Send into an inserted newline.
- Managed PTYs replace an inherited missing, empty, or `dumb` `TERM` with `xterm-256color` and default `COLORTERM=truecolor`, while explicit per-session environment values still win.
- Settings slice: system/English/Simplified Chinese UI selection, OS-backed sign-in startup, and optional minimized auto-start are implemented and covered by mocked WebView verification.
- Workspace tabs: persisted add/rename/close behavior, per-workspace directory grouping, independent 8-session limits, page-local stop, and a 24-session global safety guard are implemented. Mocked WebView isolation and Rust limit tests pass. The ignored Windows release-gate stress test passed 10 cycles of 24 simultaneously live interactive CMD PTYs (240 process launches total), including resize, input, isolated marker output, normal exit, and cleanup. A real Tauri/WebView2 run passed the three-by-eight capacity, refresh recovery, tab switching, Send execution, page-local stop, final cleanup, and child working-directory checks. Windows canonical paths now remove the `\\?\` prefix before PTY launch, and a blank working directory resolves to the user home directory.
- Desktop lifecycle: system tray, hide-with-active-sessions behavior, tray reopen/quit actions, auto-start-to-tray, and single-instance reuse are implemented. Installed-app verification passed autostart enable/disable restoration, single-instance exit, window hide, and second-instance reopen. Direct tray-icon clicking remains a manual check because the native UI automation surface did not expose the notification area.
- Run real-device tests for IME, clipboard, DPI, multiple displays, sleep/wake, and browser-based login.
- Windows x64 NSIS and MSI bundles build successfully, and a repeatable static verifier checks artifact presence, size, version, SHA-256 hash, and signature state.
- Windows installer acceptance passed for NSIS install, Start-menu launch target, profile persistence, uninstall cleanup, clean reinstall, and same-version reinstall. Version 0.1.1 validated the first cross-version upgrade from v0.1.0. Version 0.1.2 adds the compact three-CLI icon plus a signed updater backed by a public binary-only release feed; installation requires all managed sessions to be stopped, and stale updater temp directories are retried for cleanup on application launch.
- Version 0.1.3 adds persisted directory/CLI/ungrouped session views, a Claude Code launch preset, and a double-click full-window terminal focus mode.
- Version 0.1.4 enlarges the effective artwork at Windows taskbar sizes while preserving the prior icon source as a retained design revision.
- Version 0.1.5 adds bounded, keyboard-accessible drag dividers for the session overview and composer while keeping the terminal viewport usable.
- Code signing remains required before public distribution. The v0.1.x artifacts are intentionally marked as unsigned private-release assets.
- Before macOS packaging, migrate the current bundle identifier away from the `.app` suffix while preserving or migrating the existing application config directory.
- Keep macOS and Linux labelled unverified until tested on real devices.

## Explicitly deferred

- Attaching to terminals launched outside CLI Hub.
- Agent-specific daemon integration.
- Automatic task naming or waiting-state inference.
- Resuming CLI conversation or execution state after a full app restart. Version 0.1.6 supports opt-in relaunching of the last running CLIs without restoring their internal state.
- SSH, cloud sync, collaboration, and plugins.
