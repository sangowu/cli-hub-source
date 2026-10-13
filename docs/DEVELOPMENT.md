# Development and behavior reference / 开发与行为参考

CLI Hub is a local desktop workspace for launching and interacting with command-line tools from one interface.

Latest release: [v0.1.17 — retain previews during generation](RELEASE_NOTES_0.1.17.md).

Interface changes follow the [UI design system](UI_DESIGN_SYSTEM.md), which documents typography, button variants, settings layout, theme states and visual verification.

The current Windows build supports workspace tabs with up to eight active CLI sessions per workspace and a 24-session application safety limit. Sessions can be grouped by working directory, grouped by CLI type, or shown together inside their workspace. They appear as live recent-output cards and open in a full xterm.js terminal for interaction.

## Current scope

- Start up to eight PowerShell, Windows PowerShell, Command Prompt, Codex, Claude Code, or custom CLI sessions per workspace, with a 24-session global safety limit.
- Add, rename, switch, and close workspace tabs; each workspace owns its own session count, directory groups, selection, and stop-page action.
- Resolve built-in programs from the local PATH and report missing commands clearly.
- Launch Windows `.exe`, `.cmd`, `.bat`, and `.ps1` custom entries through the appropriate structured host.
- Pass arguments as an array, entered one argument per line, without shell-string concatenation.
- Apply optional environment variables entered as `NAME=VALUE`, one per line.
- Optionally select its working directory; a blank value uses the current user's home directory.
- Render ANSI/VT output in xterm.js.
- Compose multiline input, use Shift+Enter for a new line, and continue numbered lists automatically.
- Select local files as removable path references without uploading or copying their contents.
- Send composer input and automatically submit it with Enter, while retaining direct keyboard input in the terminal.
- Resize and stop the terminal session.
- Detect normal and failed process exits.
- Switch the session overview between working-directory grouping, CLI-type grouping, and an ungrouped view; the selected mode is saved locally.
- Drag the session/terminal divider to resize the overview from 190 to 360 pixels (one to two 162px cards, including spacing and scrollbar); it remains below 40% of the workspace and collapses to the single-card width when the window narrows.
- Both dividers retain the current terminal grid while dragging, then fit it on release to avoid repeated terminal/CLI redraws. The moving pane may temporarily clip the existing grid until release.
- Drag the terminal/input divider to give the multiline composer more room without reducing the terminal below its usable minimum height.
- Keep session preview cards at a consistent compact width when the window narrows; cards reflow without stretching.
- Show bounded, in-memory previews of the latest recognized Claude/Codex reply, excluding input prompts and common status footers. Other CLI formats fall back to recent filtered output; this is a rendered-text heuristic, not a conversation API. No transcripts are saved.
- Keep the previous preview throughout continuous terminal output. Replace it only after two seconds without new output; first replies keep the waiting placeholder until then. This is a quiet-period heuristic, not an agent completion signal: long pauses may publish an intermediate reply, and continuous background output may delay replacement. Activity indicators continue updating independently. Previews remain bounded and use a fixed two-line area.
- Restore active session cards and up to 256 KiB of recent output after a WebView refresh.
- Confirm before stopping the selected session, stopping a workspace, or closing a workspace that still owns active sessions.
- Keep active CLI sessions in an independent background host when the interface closes, restarts or updates; explicit **Stop all and exit** stops the host and its sessions.
- Save, update, and delete up to 20 local launch profiles without storing environment variables.
- Remember up to 10 successfully used working directories for quick reuse.
- Switch the CLI Hub interface immediately between system language, English, and Simplified Chinese.
- Switch the application chrome between system, light, and dark color modes while keeping the terminal palette optimized for CLI output.
- Bundle JetBrainsMono Nerd Font for Latin text and icons, plus Sarasa Mono SC for coordinated Simplified Chinese rendering.
- Enable or disable operating-system sign-in startup and optionally start directly in the system tray.
- Reuse the existing process when CLI Hub is launched a second time instead of creating duplicate application instances.
- Highlight stopped sessions with a yellow card and expose a red top-right × to remove their panels.
- Keep launch parameters behind a focused **New session** dialog instead of occupying the workspace.
- Use one dark scrollbar treatment across the session overview and full terminal.

On Windows, CLI Hub owns PTYs in a separate per-user background process and reconnects the interface to live sessions. It does not install an operating-system service. See [terminal host lifecycle and verification](TERMINAL_HOST.md).

## Prerequisites

- Windows 10 or 11 x64
- Node.js 22.12+ with npm (or another version supported by the locked Vite package)
- Rust stable with the MSVC target
- Visual Studio C++ Build Tools
- Microsoft Edge WebView2 Runtime

## Verify

```powershell
npm ci
npm run build

$env:Path="$env:USERPROFILE\.cargo\bin;$env:Path"
Push-Location .\src-tauri
cargo fmt -- --check
cargo test
Pop-Location
```

The real Windows usability smoke test uses a separate application identifier and preferences store, so an installed CLI Hub can stay running. With Playwright installed, start `npm run dev` in one terminal, then start the test app in another:

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9222'
npm run tauri -- dev --no-watch --config scripts/usability-test.conf.json
```

Run `python scripts/verify_session_usability.py prepare` in a third terminal. It checks settings, clipboard shortcuts, interruption and grouping, then exits the test app with two saved sessions. Start the test app again with the same command and run `python scripts/verify_session_usability.py restored` to check restoration, no refresh duplication, persisted key mappings and stop cleanup. The test preserves clipboard text and disables restoration before its final exit. If a test fails, `python scripts/verify_session_usability.py cleanup` stops the isolated test instance.

For the agent-newline regression, keep the Vite server running and launch `npm run tauri -- dev --no-watch --config scripts/newline-test.conf.json` with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9226`. Run `python scripts/verify_terminal_newline.py`. This requires installed Codex and Claude Code with their default input bindings. It uses a separate application identity, types and cancels two-line drafts without submitting model requests, checks a harmless CMD echo, and shuts down its test sessions. On Claude's first launch, it trusts only this repository when the directory confirmation is shown. Failure screenshots remain in ignored `artifacts/`.

## Build Windows installers

Build the x64 NSIS and MSI packages on Windows, then use PowerShell 7 to verify their presence, size, version, SHA-256 hash, and signature status:

```powershell
npm run bundle:windows -- --config scripts/local-build.conf.json
npm run verify:windows-bundle
```

The installers are written to `src-tauri/target/release/bundle/nsis/` and `src-tauri/target/release/bundle/msi/`. Version 0.1.0 installation, same-version reinstall, uninstall, and clean reinstall behavior has been validated on Windows 11, including profile persistence. Version 0.1.1 validated the first cross-version upgrade from 0.1.0, version 0.1.2 added signed in-app updater artifacts, version 0.1.3 added flexible session grouping, Claude Code, and terminal focus mode, version 0.1.4 improved the taskbar icon's small-size legibility, and version 0.1.5 adds bounded draggable workspace dividers. The installers remain without Authenticode signatures, so Windows may still display an unknown-publisher warning.

For local testing, run the NSIS `CLI Hub_<version>_x64-setup.exe`, then launch **CLI Hub** from the Windows Start menu. The installed application does not require a development terminal, Node.js, or Rust at runtime.

## Run

```powershell
$env:Path="$env:USERPROFILE\.cargo\bin;$env:Path"
npm run tauri dev
```

In the app, use **＋** to create a workspace, double-click its tab name to rename it, and select **New session** to launch a CLI into the current workspace. Use **Folder**, **CLI**, or **All** above the session cards to change the persisted grouping view. Drag either divider to resize the session overview or input area; double-click a divider to restore its default size. Select a card to open its terminal, then double-click the right-side terminal to enter or leave the full-window focus view. **Stop page** affects only the current workspace; tray Quit affects every workspace.

The composer sends with Enter or the **Send** button, then submits the text to the selected CLI with a separate terminal Enter action. Shift+Enter inserts a new line. Numbered lines using `1.`, `1)` or `1、` continue with the next number; an empty numbered line exits the list. **Attach** selects up to ten local files and sends only their absolute paths as text. CLI Hub does not upload, copy, or read the selected files, and the active CLI decides whether it can use those paths.

Open **Settings** from the header to choose system language, English, or Simplified Chinese, and to select system, light, or dark color mode. These choices apply immediately and are stored locally. The terminal keeps its dedicated high-contrast palette so ANSI output remains readable. The same dialog controls operating-system sign-in startup and signed application updates. When **Start minimized** is enabled, an operating-system startup launch begins in the system tray. Update checks use the public release `sangowu/cli-hub-releases` feed; installation restarts only the interface while managed CLIs keep running in the background. Windows updater artifacts are downloaded to the operating-system temporary directory, and CLI Hub retries cleanup of its own stale updater directories on each launch.

In **New session → CLI preset**, the **Pi**, **Hermes Agent**, and **OpenCode** templates fill the display name and the `pi`, `hermes`, and `opencode` commands respectively. Install the CLI separately and ensure its command is on PATH, or edit the executable to an explicit path. Arguments start empty and remain editable. Save a profile to retain your working directory and arguments. Templates use the existing Custom CLI launch/profile format for compatibility with running hosts; CLI grouping treats these sessions as Custom CLI and terminal shortcuts retain native behavior. Official naming and commands: [Pi](https://pi.dev/docs/latest/quickstart), [Hermes Agent](https://hermes-agent.nousresearch.com/docs/user-guide/cli/), [OpenCode](https://opencode.ai/docs/cli/).

Use **«** at the top of the session sidebar to collapse it into a 52px icon rail, and **»** to expand it. Each icon shows a CLI abbreviation, session number and running/exited/failed status dot. Hover for the session name, CLI type, status and working directory; click to switch sessions without expanding. The rail follows the current grouping order. Expand to access grouping, previews, stop and remove actions. Collapsing preserves live terminals and restores the expanded width and each workspace's list scroll position during the current run. The collapsed/expanded preference applies across workspaces and survives a full restart; older preferences default to expanded.

Enable **Restore CLI sessions on startup** to relaunch the last running CLIs in their original workspaces and resolved launch directories after the background host has stopped. Live host sessions always reconnect without this option. This is disabled by default. Launch types, names, executables and argument arrays are saved locally; arguments can contain sensitive values. Terminal output, temporary environment overrides and CLI conversation state are not saved or restored. Stopping a session removes it from the restoration list; explicitly stopping all and exiting preserves the list before stopping processes. Closing only the interface leaves the host and its sessions running. A launch failure is reported without preventing other sessions from starting.

Inside a terminal, **Ctrl+C** copies when text is selected and interrupts the CLI otherwise. **Ctrl+Shift+C** copies, while **Ctrl+V** and **Ctrl+Shift+V** paste through xterm's bracketed-paste handling. Clear the selection to interrupt with Ctrl+C. Settings explains these actions and lets you focus a shortcut field and press a new Ctrl/Alt/Meta combination. Duplicate mappings are rejected; **Reset shortcuts** restores defaults. You can also disable copying with the interrupt shortcut. These mappings do not change the composer textarea's standard editing shortcuts. Clipboard text is accessed through the native Tauri clipboard plugin only when a copy or paste shortcut is used. Pasting text containing line breaks may execute commands.

In terminals launched with the **Codex** or **Claude Code** preset, the shared newline shortcut defaults to **Shift+Enter**; plain **Enter** still submits. In **Settings → Terminal shortcuts → New line in agent input**, focus the field and press a new combination, for example **Ctrl+Enter**. Changes take effect immediately and persist across restarts; duplicate mappings are rejected and **Reset shortcuts** restores Shift+Enter. Plain Enter cannot be assigned. Old preferences gain the newline default without losing other bindings. A WebView-local fallback preserves this new mapping while a v0.1.9 background host remains running; a supporting host saves it in the preferences store on the next settings save. CLI Hub translates the configured key to Alt+Enter (`ESC CR`) for Codex and Ctrl+J (`LF`) for Claude Code. Native agent shortcuts remain available. This uses the agent's default input bindings; remapped agent bindings, dialogs, and non-input modes can behave differently. Shell and custom presets retain native xterm behavior; this also applies to agents started from inside a shell. The composer keeps its own Shift+Enter editing behavior. Use the multiline composer for unsupported CLIs. CLI Hub does not modify agent configuration files.

Refreshing or reopening the interface reconnects to live sessions, workspace ownership, and up to 256 KiB of recent output per session from the background host without launching duplicate processes. Output recovery is memory-only; no transcript is written to disk. Session-card metadata is saved in local WebView storage and retained across interface updates/restarts until you remove the card or its workspace. This includes stopped and failed cards, but excludes output, error text, arguments and environment overrides. A saved running card whose process is no longer present is shown as stopped. Clearing application data removes these records; records lost before this behavior was introduced cannot be reconstructed.

Closing the window exits the interface and its tray while the background host keeps running. Reopen CLI Hub through its normal shortcut to reconnect. **Settings → Background CLI sessions → Stop all and exit**, or the tray equivalent, confirms before stopping the host and all its tasks. A host crash or computer restart still terminates the original tasks.

Launch profiles, recent directories, UI settings, workspace names, ordering, and the selected workspace are stored in the operating system's application configuration directory as a versioned `preferences.json` file. Existing values from each WebView origin are merged once when that origin next launches, allowing development and packaged builds to migrate into the same file. Profiles include the CLI preset, display name, executable, arguments, and working directory. Environment variables and terminal output are never included. Saved arguments are local but unencrypted, so secrets should remain in the environment field instead of command-line arguments.

On Windows, CLI Hub protects its configuration directory, endpoint token, preferences and temporary files with an explicit ACL for the current user, SYSTEM and Administrators. Existing endpoint/preferences files are repaired before use; a permission failure blocks access instead of silently continuing with broad inherited permissions. This does not encrypt saved arguments or revoke tokens already copied by another process. After finishing running tasks, stop all and exit, then reopen to generate a fresh host token. See [security boundaries and verification](SECURITY.md).

The bundled JetBrainsMono Nerd Font files are version 3.3.0 and retain their embedded SIL Open Font License 1.1 metadata. Sarasa Mono SC is bundled from Sarasa Gothic version 1.0.42. Their licenses are included in the packaged static assets at `licenses/JetBrainsMono-OFL.txt`, `licenses/NerdFonts-LICENSE.txt` and `licenses/SarasaGothic-LICENSE.txt`. See [third-party notices](THIRD_PARTY_NOTICES.md).

PowerShell starts with `-NoLogo -NoProfile` so the baseline is not changed by a user's profile scripts. Agent-specific environment setup belongs in later launch profiles.

## Platform status

Version 0.1.8 adds a collapsible session sidebar with direct icon-based session switching. See [release notes](RELEASE_NOTES_0.1.8.md).

- Windows x64: supported target and real-device validation platform.
- macOS: compile-oriented design only; not yet verified on real hardware.
- Linux: compile-oriented design only; not yet verified on a real desktop.

See [ROADMAP.md](../ROADMAP.md) and [docs/ARCHITECTURE.md](ARCHITECTURE.md) for boundaries and next stages.

## Test dependencies / 测试依赖

Python-based UI checks require Python 3.10+ and Playwright. Install them in a virtual environment from the repository root:

```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install playwright
python -m playwright install chromium
```

仅测试前端布局时可使用 mock 测试；真实终端行为必须使用隔离的原生测试配置。`verify_stage6_real_ui_stress.py` 会操作正常开发实例并启动 24 个真实 CMD 会话，只应在专用测试环境手动运行。不要对正在工作的个人会话运行它。

Browser mock tests cover UI behavior only. Native tests require the documented isolated app identity and a running WebView2 debugging endpoint. The manual `verify_stage6_real_ui_stress.py` operates on the normal development instance and starts 24 real CMD sessions; use a dedicated test environment, never personal work sessions.

## Local packages and official updates / 本地包与官方更新

`scripts/local-build.conf.json` disables creation of updater artifacts for local builds, so contributors do not need the maintainer's signing key. It does not create a new update channel: the default app identifier and official update endpoint remain unchanged. Forks must choose their own app identifier, updater public key and endpoint before distributing independently. Local packages are unsigned and must not be uploaded as official updates.

本地打包配置只关闭更新制品生成，不改变默认应用标识或官方更新地址。本地构建可能共用已安装版本的数据和后台 Host；隔离测试请使用仓库内专用测试配置。独立分发的 fork 应配置自己的应用标识、更新公钥和更新地址。

Maintainer release builds use `npm run bundle:windows` with signing credentials supplied outside the repository. Only final verified artifacts should enter the existing release feed. See the [Tauri configuration reference](https://v2.tauri.app/reference/config/#createupdaterartifacts).
