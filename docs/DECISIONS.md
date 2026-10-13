# Decisions

## 2026-09-26 — Windows-first, cross-platform codebase

The first validated release targets Windows, but the project uses Tauri and portable-pty rather than WPF and direct ConPTY calls. macOS and Linux remain unverified until real-device tests are available.

## 2026-09-26 — Rust owns terminal processes

Rust owns PTY handles, input/output streams, resize operations, and child termination. TypeScript owns presentation and user interaction. This keeps native lifecycle work outside the WebView without requiring an all-Rust UI.

## 2026-09-26 — xterm.js renders terminal state

The UI uses xterm.js instead of a plain text control because interactive CLIs depend on VT sequences, cursor movement, colors, alternate screens, and terminal keyboard modes.

## 2026-09-26 — No Agent-specific integration in the baseline

Stage 1 treats PowerShell as a generic terminal process. Codex daemon inspection, task-state inference, and tool-specific adapters are deferred until the generic multi-session workspace is stable.

## 2026-09-26 — Deterministic PowerShell baseline

Windows prefers `pwsh.exe` when it is available and falls back to `powershell.exe`. Both start with `-NoLogo -NoProfile`. This avoids profile-specific prompts and side effects without weakening execution policy; future saved launch profiles can opt into custom setup explicitly.

## 2026-09-26 — Windows Stop uses observed process exit

`portable-pty 0.9.0` reverses the Windows `TerminateProcess` return-value check in its cloned killer, so a successful termination can be reported as an arbitrary stale OS error while a failure can be reported as success. CLI Hub still requests termination through the portable abstraction, then treats the child wait thread's observed exit as the source of truth. A two-second timeout preserves a real error when the process does not exit. Remove this compatibility boundary after adopting a released dependency version with the upstream correction.

## 2026-09-26 — Structured CLI launch requests

Stage 2 sends preset, display name, executable, argument array, working directory, and environment entries as separate values. Rust resolves and validates the executable before creating the PTY. The UI uses one argument per line instead of parsing a shell command string, so paths and values containing spaces keep their intended boundaries. On Windows the Codex preset prefers a native `codex.exe`, then uses the installed PowerShell or CMD launcher as a structured fallback. Custom `.ps1`, `.cmd`, and `.bat` entries use the same host-specific argument arrays instead of direct `CreateProcess` calls. Resolved Windows executable paths remove the `\\?\` verbatim prefix before passing script paths to CMD because CMD cannot execute that path form.

## 2026-09-26 — Bounded per-session output pipelines

Stage 3 stores active terminals in a map keyed by monotonically increasing session identifiers. Each session has a 64-slot bounded output queue and emits batches capped at 32 KiB or 8 ms. Blocking the reader when its queue is full is intentional backpressure; unbounded transcript growth and silent output loss are both avoided. The exit event is emitted only after final queued output drains.

## 2026-09-26 — Directory groups come from resolved native paths

Stage 4 makes the resolved working directory explicit for every launch. A blank value resolves to the current user's home directory instead of the desktop process directory. Rust supplies both the display path and a platform-aware group key; Windows keys normalize separators and case, and resolved paths remove the `\\?\` verbatim prefix before PTY launch so CMD retains the requested directory, while non-Windows keys preserve case. The frontend displays in-memory recent-output cards and a selected full terminal. Cards are navigation and monitoring views, not persisted transcripts.

## 2026-09-26 — No tray mode in the MVP

CLI Hub is a normal taskbar application in the current MVP. It does not create a system-tray icon or remain hidden in the background. Safe close behavior for active sessions remains Stage 5 work.

## 2026-09-28 — Tray lifecycle supersedes the earlier MVP boundary

Stage 6 adds a system-tray icon, hide-with-active-sessions behavior, explicit tray reopen and quit actions, optional minimized auto-start, and single-instance window reuse. The earlier no-tray decision remains above as historical context but no longer describes the shipped Windows lifecycle.

## 2026-10-02 — Session restoration and terminal usability

Startup restoration is opt-in and relaunches only the last running sessions. Persist structured launch requests and workspace ownership, excluding terminal output and temporary environment overrides. The Rust manager owns snapshots and guards restoration against WebView refresh duplication. Clipboard shortcuts use the native Tauri text clipboard plugin to avoid WebView2 permission waits; paste passes through xterm's paste API. Copy, two paste bindings and interrupt are configurable, with duplicate mapping rejection and optional selection-aware interrupt copying. The dark theme's generic button rules previously overrode grouping-button selection colors; explicit selected/hover rules and a checkmark now keep the current grouping visible.

Validation passed: frontend build, Rust formatting and 32 tests (one manual stress test remains intentionally ignored), the existing mocked UI regression, and an isolated real Windows Tauri/WebView2 smoke test. The real test covered clipboard copy/paste, selection-aware Ctrl+C, interrupting a running ping command, live key remapping and conflicts, grouping selection colors, two-workspace restoration after a full exit, persisted key mappings, refresh without duplicate sessions, and removing stopped sessions from the restoration list. Temporary environment values are excluded by a preferences round-trip test. This does not establish conversation resumption in Codex or Claude.

## 2026-10-02 — Settings visual consistency

The shortcut reset button previously relied on generic button defaults and separate shortcut styling, leaving its typography and dimensions inconsistent with other settings actions. Reset and update actions now share `settings-action-button`, including hover, keyboard focus, disabled and dark-theme states. Shortcut headings, descriptions, inputs and spacing follow the existing compact settings hierarchy. Shortcut behavior is unchanged.

Added `docs/UI_DESIGN_SYSTEM.md` and a README entry to document the current typography, button variants, settings layout, theme states and verification rules. Frontend build and the existing mocked UI regression passed. Eight visual checks covered English and Chinese, light and dark themes, and 1280 × 900 and 720 × 600 windows. Computed styles confirmed matching action-button dimensions, typography and hover colors; settings had no horizontal overflow. Screenshots were inspected for layout consistency. These visual changes do not require a new terminal-behavior smoke test.

## 2026-10-02 — Workbench and confirmation visual repairs

Generic button rules overrode the terminal focus-exit hover background, producing nearly white text on a nearly white background in light mode. Generic dark-mode rules also hid workspace tab color differences and replaced close/add button states. Exclude these components from the affected generic rules so their existing dedicated states remain intact, including keyboard focus.

Stop and application-exit confirmation dialogs now share compact typography and 30px action buttons. Give the final danger-confirmation variant precedence over ordinary danger buttons so it keeps a filled red background, white text and the intended right-aligned action spacing in light and dark themes. Document these rules in the UI design system.

Validation: frontend build and the existing mocked UI regression passed. Eight English/Chinese, light/dark and normal/minimum-window combinations verified distinct workspace tab colors, danger-close hover, consistent confirmation dimensions and filled danger states, readable focus-exit hover, visible keyboard focus and no horizontal overflow. Settings visual checks also passed. Screenshots were reviewed; this is browser/Tauri-mock rendering validation, with no terminal behavior or Rust changes.

## 2026-10-03 — Collapsible session sidebar

Add a sidebar toggle that switches between the existing overview and a 52px icon rail. The rail follows grouping order and retains session switching, selected highlighting, CLI abbreviations, session IDs, status dots and descriptive tooltips. Grouping, previews and stop/remove actions remain in the expanded view. Preserve terminal containers, expanded width and per-workspace list scroll positions during the current run. Persist the global collapsed preference in existing UI settings with a serde default so older preferences remain expanded.

The narrow-window layout rule previously fixed the sidebar at 190px; the collapsed selector now has explicit precedence to retain 52px at the minimum window size. New rail buttons retain dedicated dark-workbench states rather than generic theme-button styling.

Validation passed: frontend build, existing mocked UI regression, eight English/Chinese × light/dark × normal/minimum sidebar checks, Rust formatting and 33 tests (one manual stress test intentionally ignored). Sidebar checks cover live status changes, grouping order, selection, width/scroll restoration, workspace switching, refresh persistence, hidden expanded actions, unchanged terminal DOM instances and save-error handling. An isolated real Windows Tauri/WebView2 instance verified actual ConPTY columns via `mode con`, preserved process IDs and terminal selection, icon switching, direct and composer input, expanded width restoration and persisted collapse after a full process restart. The test restored the clipboard, stopped its own CMD sessions and closed its own instance.

## 2026-10-03 — Preserve live terminals across interface updates

The old updater required users to stop all sessions first, which deleted their restart requests. Startup restoration therefore could not restore those sessions; it also never preserved actual running processes. Move the existing terminal manager and the sole preferences writer into a per-user Windows background host. The GUI forwards existing commands through authenticated, bounded local IPC and subscribes to events. Closing/updating the GUI leaves the host alive; explicit stop-all-and-exit shuts it down. Reconnection works independently of optional startup relaunch.

Keep the host in a content-addressed application-data copy with its own executable name: the generated NSIS installer closes processes by the GUI executable name. Its code is not replaced during a GUI update. Retain protocol checks, a singleton host lock, fail-closed handling of an unreachable live host, bounded event queues and explicit disconnect errors. Document first-version migration, host/computer restart limitations and future protocol compatibility in `TERMINAL_HOST.md`.

Isolated native Windows checks verified unchanged host and PTY PIDs, continued PowerShell task output, CMD memory, recent-output replay, input, resize, six refreshes, rejected IPC credentials and normal explicit shutdown. Real NSIS/MSI test installers replaced the host-enabled v0.1.8 fixture with v0.1.9 while the original tasks continued; MSI used current-user mode in an isolated directory. Mocked update regression covers active sessions and download/preflight/install failures.

Release validation found that Windows can retain an exited host's process object while image-path queries already fail. Read its exit status before classifying an unreadable image as a live inaccessible host. A retained-handle regression reproduced the failure and passed after the fix; the installed v0.1.9 test application also reopened after full shutdown. Tray quit now shuts down an idle host as well as the interface.

## 2026-10-05 — Retain session-card metadata independently of live terminals

The background host intentionally drops exited terminals, so reconstructing the interface only from its live snapshots loses stopped/failed cards. Save a versioned metadata-only card list in WebView-local storage, compatible with a running older host. Preserve stable record keys, reconcile live IDs/PIDs and optional startup relaunches, and use separate UI IDs for historical cards. The host remains authoritative for running state; absent saved running sessions become stopped. Output, error text, arguments and environment are excluded. Explicit card/workspace removal updates the stored list. Do not overwrite invalid history; require a successful save before installing an interface update. This retains records going forward and cannot restore previously lost cards or transcripts.

Background CLI sessions now follows the existing settings title, description, padding and right-aligned action rules instead of browser heading defaults and an undefined layout class.

Validation: frontend build, metadata/reconciliation checks and existing keyboard/UI/update/sidebar regressions passed. New UI checks cover update/reload retention, removal, metadata privacy and corrupt-storage protection. An isolated native Windows interface restart preserved host and live CLI PIDs and both live/stopped cards. Eight language/theme/window combinations passed layout checks; screenshots were inspected. The earlier reported running-process termination cause remains unconfirmed.
