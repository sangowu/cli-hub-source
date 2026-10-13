# Changelog / 更新日志

User-visible changes, newest first. Version entries describe behavior at the time of that release; current behavior and setup are documented in the [README](../README.md), [development guide](DEVELOPMENT.md) and [architecture](ARCHITECTURE.md).

按版本记录用户可见变化，最新版本在前。旧条目反映当时的行为；当前用法以 README 和开发文档为准。原独立版本说明和修复记录已整理到本文件，不再分别维护。

Official installers remain at [cli-hub-releases](https://github.com/sangowu/cli-hub-releases/releases). Windows 10/11 x64 and WebView2 are required. Current installers are not Authenticode-signed; see the README for installation prompts and [security boundaries](SECURITY.md) for updater verification and local data handling.

## Unreleased

- Consolidate release notes and user-facing fixes into this changelog. Summarize current design decisions in the architecture document and focus the development guide on setup, builds and testing. No application behavior changes.

## 0.1.17

Keep the previous session preview while the CLI generates new output.

- Replace the preview only after two seconds without new terminal output, rather than publishing intermediate replies every 500ms. The first reply keeps the waiting placeholder until output settles.
- Update activity timestamps independently, so the running/output indicators remain current while preview text stays fixed.
- Preserve existing reply filtering, bounded in-memory previews and fixed card height. Clean up pending activity and preview timers when sessions are removed or the interface closes.

The quiet period is a heuristic, not an agent completion signal. A long pause during generation may publish intermediate content; continuous background output may delay replacement. No transcripts are persisted.

## 0.1.16

Match terminal padding and content backgrounds to remove the contrasting black strips at the left and bottom edges.

- Set the xterm root and legacy viewport backgrounds to the same `#101418` used by the terminal content layer.
- Preserve existing padding, character grid dimensions, scrollbar styling and divider resizing behavior. Explicit CLI color commands and colored text cells are unchanged.

## 0.1.15

Keep session previews stable while CLIs redraw their terminal screens.

- Retain the last recognized reply during temporary clear-screen, thinking, progress and tool-output frames. Replace it when a new recognized reply appears; streaming replies continue to update.
- Coalesce preview updates every 500ms instead of 80ms, reducing distracting text changes during active output.
- Keep plain-output previews through empty/status-only frames, while allowing new filtered shell output to update normally.
- Fix the preview area to 31px (two lines) so content changes do not change card height.

Previews remain bounded and in memory only. Reply recognition uses rendered-text heuristics; unknown CLI formats may not expose reliable message boundaries. Once a reply is recognized, unmarked fallback output is held back until another recognized reply appears.

## 0.1.14

Protect local application files and fix session panel resizing.

- Restrict the terminal host directory, endpoint token, preferences and temporary preference files to the current Windows user, SYSTEM and Administrators. Repair existing files when connecting to the host, and reject reparse points at protected storage paths.
- Enable production Content Security Policy to block inline scripts and remote frontend connections. Update the development-only source-map-js dependency to address its reported denial-of-service issue.
- Restore two session-card columns reliably after shrinking and expanding the sidebar, including fractional display scaling with classic scrollbars.
- Keep the terminal grid stable while dragging either divider, then fit it on release. Avoid redundant PTY resize calls to reduce CLI redraw flicker.
- Match Settings action buttons to the 170 × 34 px input and dropdown controls in English and Chinese.

Existing session and profile formats remain compatible. File access restrictions do not revoke an endpoint token already copied before upgrading. After active work finishes, use **Stop all and exit**, then reopen CLI Hub to start the updated terminal host and rotate its token. Updating the interface alone intentionally preserves running sessions.

Saved launch arguments remain plaintext local configuration; avoid putting secrets in saved arguments. No terminal transcripts are added to persistent storage. See [Security](SECURITY.md) for scope and remaining limitations.

## 0.1.13

Add three agent launch templates and refresh the workspace header.

- Add **Pi**, **Hermes Agent**, and **OpenCode** to **New session → CLI preset**, using their official product names and the `pi`, `hermes`, and `opencode` commands. Install each CLI separately and ensure its command is on PATH, or edit its executable path. Arguments remain editable; saved profiles retain the configuration.
- Templates use the existing Custom CLI launch/profile format, preserving compatibility with running terminal hosts. CLI grouping treats them as Custom CLI; native terminal shortcut behavior is retained.
- Pair the existing terminal-stack application icon with a compact 24px CLI Hub wordmark and a cyan Hub accent. Keep the header to a single row, aligned with its controls in both themes.
- Replace the terminal toolbar hint with **Double-click the CLI view to enter fullscreen mode**, and **双击CLI界面可进入全屏模式** in Chinese.

## 0.1.12

More space for CLI sessions, correctly fitted terminal output, and useful reply previews.

- Fix terminal content being clipped at the bottom/right by applying terminal padding where xterm's FitAddon accounts for it. Terminal rows and columns now fit the available content area after panel resizing and focus-mode transitions.
- Limit the expanded session sidebar to 190–360px: one to two fixed-width mini cards, including spacing and the scrollbar. Narrow windows retain a single column; the collapsible icon rail is unchanged.
- Prefer the latest recognized Claude/Codex reply in mini cards, excluding input prompts and common status footers. Other CLI formats fall back to recent filtered output. Recognition uses bounded rendered-text heuristics, so arbitrary CLI formats and remapped agent interfaces may not be recognized. Previews remain in memory and are not saved as transcripts.
- Remove the two introductory header lines, leaving more vertical space for the workspace.

Compatible live CLIs can continue through interface updates from host-enabled versions. This release does not change the terminal-host protocol or card metadata storage.

## 0.1.11

Session cards now survive interface updates and restarts, including stopped and failed cards. Background CLI sessions in Settings also follows the existing compact typography, spacing and right-aligned action layout.

- Save card names, workspace ownership, working directories and status in local WebView storage. Live host snapshots determine which CLIs are still running; saved running cards whose processes are absent appear as stopped.
- Reconnect live processes and reconcile optional startup relaunches without duplicating their cards. Keep stopped/failed records until you remove the card or its workspace. Clearing application data removes the records.
- Exclude terminal output, error text, arguments and environment overrides from card storage. Recent output for live sessions still comes from the background host's bounded memory.
- Save the latest card state before installing an update, including a CLI that exits during download. Invalid saved records are left untouched, and update installation is blocked if records cannot be loaded or saved.

The new card storage takes effect after installing v0.1.11. It cannot reconstruct records already lost by an older interface or preserve stopped cards that disappear during the initial upgrade from that interface. The reported cause of earlier running-process termination remains unconfirmed; this release fixes the independently verified card-retention defect. Compatible live CLIs can continue through interface updates from host-enabled versions without changing the host protocol.

## 0.1.10

Codex and Claude Code terminals now use **Shift+Enter** for a new line while **Enter** continues to submit. Previously, terminal keyboard encoding made both keys act like Enter.

- Customize the shared agent newline key in **Settings → Terminal shortcuts → New line in agent input**, for example Ctrl+Enter. Changes take effect immediately and survive reopening; reset restores Shift+Enter.
- Duplicate shortcut assignments are rejected, plain Enter stays reserved for submission, and IME composition does not capture or trigger shortcut mappings.
- Existing preferences gain the new default without losing other custom shortcuts. When an older v0.1.9 background host remains running during an update, a local WebView fallback preserves the new mapping; a supporting host saves it in the preferences store on the next settings save.
- Codex and Claude Code receive their native newline sequences. Shell/custom presets and agents started inside a shell retain their native input behavior. Agent-specific remapped bindings and non-input modes can differ. The composer keeps its own Shift+Enter editing shortcut.

Live CLI sessions can stay running during the interface update from v0.1.9. No agent configuration files are modified, and no terminal transcripts are saved.

## 0.1.9

CLI Hub now keeps running CLIs in an independent Windows background host. Closing, restarting or compatibly updating the interface reconnects to the original terminal processes, preserving their running tasks and in-memory state.

- Closing the window exits only the interface. Reopen CLI Hub to reconnect to live sessions, regardless of the optional startup-restoration setting.
- Settings explains background sessions and provides **Stop all and exit**. This explicit action, or the tray equivalent, stops all terminals and the host.
- Updates no longer require stopping live CLIs. The host uses a distinct executable name and a private versioned copy, so replacing the interface does not replace its running code.
- Local IPC is authenticated, limited to loopback and bounded. The host remains the single writer for preferences; lost connections block launches and updates instead of silently creating replacement sessions.
- Reopening after full shutdown correctly handles Windows retaining an exited process handle, and tray quit also shuts down an idle host.
- Computer or host restarts still end the original tasks. Optional startup restoration relaunches saved CLIs but cannot restore their memory, conversations or temporary environment.

**First migration from v0.1.8 or earlier:** finish or explicitly stop existing tasks before installing this release. Their PTYs belong to the old application and cannot be transferred to the new host. Process continuity applies to sessions launched by this host-enabled release and subsequent compatible interface updates.

## 0.1.8

CLI Hub v0.1.8 adds a collapsible session sidebar to give the terminal more room while keeping session switching available.

- Click **«** to collapse the sidebar into a 52px icon rail, and **»** to expand it.
- Icons show CLI abbreviations, session numbers, running/exited/failed status dots and selected highlighting. Hover for the name, CLI type, status and directory; click to switch without expanding.
- The rail follows the current grouping order and scrolls vertically when needed. Expand to use grouping, previews, stop and remove actions.
- Folding preserves live terminal instances and restores the expanded width and each workspace's list scroll position during the current run. The global collapse preference survives a full restart; older preferences default to expanded.
- The minimum-window layout keeps the rail at 52px. Labels, focus states and theme styling follow the documented UI design system.

## 0.1.7

CLI Hub v0.1.7 fixes visual consistency across settings, workspace tabs and confirmation dialogs.

- Reset shortcuts shares the settings action-button style, including typography, dimensions, hover, keyboard focus and light/dark themes. Shortcut headings, descriptions, inputs and spacing follow the existing settings hierarchy.
- Workspace tabs retain distinct selected colors in dark mode; close and add buttons keep their dedicated states.
- Exit focus stays readable when hovered and when focused by keyboard in either theme.
- Stop and exit confirmations share compact typography and 30px action buttons. Final danger confirmations keep a filled red background, white text and consistent action spacing.
- Added a UI design-system document covering typography, button variants, settings layout, theme states and verification.

## 0.1.6

CLI Hub v0.1.6 adds optional startup session restoration and configurable terminal shortcuts, and fixes the current grouping indicator in dark mode.

- Enable **Restore CLI sessions on startup** in Settings to restart the last running CLIs in their original workspaces and launch directories. The setting is off by default. Launch arguments are stored locally; terminal output, temporary environment overrides and CLI conversation state are not restored.
- With the default shortcuts, **Ctrl+C** copies selected text and interrupts the CLI when there is no selection. **Ctrl+Shift+C** copies; **Ctrl+V** and **Ctrl+Shift+V** paste using xterm's bracketed-paste handling. Clear the selection before using Ctrl+C to interrupt.
- Settings includes shortcut explanations, editable copy/paste/interrupt mappings, duplicate mapping checks, a reset button and an option to disable selection-aware copying. Native clipboard access avoids WebView2 permission waits.
- The current **Folder / CLI / All** grouping now has a visible checkmark, highlighted background and bottom indicator, including in dark mode.

## 0.1.5

CLI Hub v0.1.5 adds adjustable workspace dividers while preserving practical limits for the terminal area.

### Added

- Drag the vertical divider to resize the session overview between 190 and 420 pixels, capped at 40% of the workspace.
- Drag the horizontal divider to resize the input panel between 160 and 360 pixels, capped at 45% of the available workspace height.
- Use arrow keys on a focused divider for precise adjustment, Home or End for its bounds, and double-click to restore the default size.

### Changed

- The terminal now observes its container size and refits xterm.js after either divider moves.
- Narrow windows retain the existing single-card-width session overview.
- The multiline composer expands with the resized input panel while keeping Send aligned to the input area.

### Known limitations

- Divider sizes reset to their defaults after the application restarts.

## 0.1.4

CLI Hub v0.1.4 improves the application icon at Windows taskbar and shortcut sizes.

### Changed

- Enlarged the three-CLI artwork by approximately 10% while retaining deliberate breathing room at the top and bottom.
- Increased effective artwork height from 12 to 14 pixels at 16 px, 18 to 20 pixels at 24 px, and 24 to 27 pixels at 32 px.
- Regenerated the Windows ICO, application PNGs, Store assets, and macOS ICNS from the retained v12 source artwork.

### Known limitations

- Windows may retain a cached taskbar or Start-menu icon until the application is updated and restarted.

## 0.1.3

CLI Hub v0.1.3 improves session navigation and adds a first-class Claude Code launch path.

### Added

- Added persisted **Folder**, **CLI**, and **All** views for the session overview.
- Added a Claude Code preset with structured Windows detection for `claude.exe`, `claude.ps1`, and `claude.cmd`, plus the portable `claude` command on macOS and Linux.
- Added a full-window terminal focus mode: double-click the right-side terminal to enter or leave, or use the unobtrusive exit button.

### Changed

- Active-session snapshots now retain their launch preset so CLI-type grouping survives WebView refreshes.
- Existing preferences without a grouping value remain compatible and default to working-directory grouping.
- Terminal focus changes trigger a PTY resize so the selected CLI uses the available rows and columns.

## 0.1.2

CLI Hub v0.1.2 is a Windows feature release that introduces signed in-app updates and a larger three-CLI application icon.

### Added

- Added a compact **Application updates** section to Settings with current-version display, update checks, download progress, and one-click installation.
- Added a public binary-only update feed while keeping the source repository private.
- Added Tauri updater signatures and verification. Update installation is blocked until every managed CLI session has stopped.
- Added best-effort cleanup of CLI Hub updater temporary directories on application launch.

### Changed

- Replaced the application, taskbar, tray, Start-menu, NSIS, and MSI icon assets with the enlarged three-CLI fan design.
- Increased the package version consistently across the frontend package, Rust crate, Tauri configuration, and lockfiles.

### Known limitations

- v0.1.2 is the first updater-enabled build, so a complete live self-update can first be exercised when v0.1.3 is published.

## 0.1.1

CLI Hub v0.1.1 is a private Windows patch release that updates the application branding and installer assets.

### Changed

- Replaced the application, taskbar, tray, Start-menu, NSIS, and MSI icons with the transparent CLI cascade design.
- Increased the package version consistently across the frontend package, Rust crate, Tauri configuration, and lockfiles.

### Known limitations

- The installers are unsigned private-release assets. Windows may display an unknown-publisher warning.
- macOS and Linux icon assets are generated but remain unverified on real devices.

## 0.1.0

CLI Hub v0.1.0 is the first Windows release of the local desktop workspace for launching, monitoring, grouping, and interacting with multiple CLI sessions.

### Highlights

- Manage up to eight active sessions in each workspace and up to 24 across the application.
- Group sessions by their resolved working directory and switch between persistent workspace tabs.
- Launch PowerShell, Windows PowerShell, Command Prompt, Codex, or a custom executable with structured arguments and environment variables.
- Interact through the full xterm.js terminal or the multiline Send composer.
- Keep active sessions running when the main window is hidden to the system tray.
- Save local launch profiles, recent directories, workspace layout, language, color mode, and startup preferences.
- Use English or Simplified Chinese with bundled JetBrainsMono Nerd Font and Sarasa Mono SC typography.

### Distribution notes

- The v0.1.0 installers are unsigned private-release assets. Windows may display an unknown-publisher warning.
- Cross-version upgrade validation begins with the next release because v0.1.0 has no predecessor.
- Fully quitting CLI Hub terminates managed sessions; full-process session restoration and attachment to externally launched terminals are not included.
