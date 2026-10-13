# Development / 开发指南

This guide covers setup, building and verification. Product features and installation steps are in the [README](../README.md); version changes are in the [changelog](CHANGELOG.md). See [architecture](ARCHITECTURE.md), [terminal-host lifecycle](TERMINAL_HOST.md) and [UI design rules](UI_DESIGN_SYSTEM.md) when changing those areas.

本文聚焦开发与测试；产品功能见 README，版本变化见更新日志。所有命令从仓库根目录执行，另有说明除外。

## Prerequisites / 开发环境

- Windows 10/11 x64, the supported validation platform.
- Node.js 22.12+ and npm, or another Node version supported by the locked Vite package.
- Rust stable with the MSVC toolchain, Visual Studio C++ Build Tools and Windows SDK.
- Microsoft Edge WebView2 Runtime.
- PowerShell 7 (`pwsh`) for the Windows bundle verification script.
- Installed/configured CLI tools for tests that exercise those tools; they are not needed for a frontend-only build.

macOS/Linux remain unverified on real devices. Installed-app users do not need Node.js or Rust.

## Install and run / 安装依赖与运行

```powershell
npm ci
$env:Path="$env:USERPROFILE\.cargo\bin;$env:Path"
npm run tauri dev
```

Keep `package-lock.json` and `src-tauri/Cargo.lock`. Normal development builds share the official application identifier, data location and update feed by default. Use the dedicated configurations below for isolated tests.

默认开发实例可能共用正式版的数据和后台 Host；测试应使用专用配置，不要操作正在工作的个人会话。

## Baseline checks / 基础检查

```powershell
npm run build
node scripts/verify_terminal_preview.mjs
node scripts/verify_session_history.mjs
node scripts/verify_terminal_shortcuts.mjs

Push-Location .\src-tauri
cargo fmt -- --check
cargo test
Pop-Location
```

Repeat the checks relevant to your change. Rust tests include real Windows PTY checks; one expensive stress test is intentionally ignored. Compilation or browser mocks do not establish native terminal behavior. Terminal/input/lifecycle changes require a real isolated Tauri smoke test.

## UI test dependencies / 界面测试依赖

Python-based checks require Python 3.10+ and Playwright:

```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install playwright
python -m playwright install chromium
```

Start `npm run dev` in one terminal. Frontend checks such as `python scripts/verify_stage4_ui.py` use a browser with mocked Tauri IPC. Select additional checks for the changed area, for example `verify_session_sidebar.py`, `verify_panel_resize.py`, `verify_agent_templates.py` or `verify_preview_stability.py`. Screenshots go to ignored `artifacts/`.

### Isolated native usability check / 隔离原生测试

With Vite running, launch this in a second terminal; its configuration disables the duplicate Vite startup command:

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9222'
npm run tauri -- dev --no-watch --config scripts/usability-test.conf.json
```

In a third terminal run `python scripts/verify_session_usability.py prepare`. It checks settings, clipboard, interruption and grouping, then exits with two saved sessions. Relaunch the same test app and run `python scripts/verify_session_usability.py restored` to verify restoration, no refresh duplication and persisted mappings. On failure, `python scripts/verify_session_usability.py cleanup` stops the isolated instance. The test preserves clipboard text and disables restoration before final exit.

For Codex/Claude newline behavior, use `scripts/newline-test.conf.json` and CDP port `9226`, then run `python scripts/verify_terminal_newline.py`. The test requires installed agents with default input bindings, types and cancels drafts without model requests, checks CMD Enter and stops its sessions. It trusts only this repository if Claude asks for directory confirmation.

For host reconnect/update checks, follow [TERMINAL_HOST.md](TERMINAL_HOST.md). For bundled CSP/file-permission checks, follow [SECURITY.md](SECURITY.md). Wait for `window.__TAURI_INTERNALS__` before native IPC assertions.

`verify_stage6_real_ui_stress.py` is a manual release gate: it operates on the normal development instance and starts 24 real CMD sessions. Use a dedicated environment, never a personal work instance. A debugging endpoint is for local testing; do not expose it to the network.

### Behavior to preserve / 需要保持的行为

- Native terminal shortcuts: Ctrl+C copies a selection or interrupts without one; Ctrl+Shift+C copies; Ctrl+V and Ctrl+Shift+V paste through xterm. Settings mappings reject duplicates. Clipboard access occurs on shortcut use; multiline paste may execute commands.
- Codex/Claude presets default to Shift+Enter for newline and Enter for submit. The hub sends Codex's Alt+Enter (`ESC CR`) or Claude's Ctrl+J (`LF`) sequence. Remapped agents, non-input modes and agents launched inside shells can differ. Shell/custom presets keep native behavior; the composer has its own Shift+Enter editing shortcut. No agent configuration is modified.
- Old host versions may not persist the shared newline mapping; the UI keeps a local fallback until a supporting host acknowledges the saved field. Run `verify_newline_host_compatibility.py` when changing this path.
- Launch profiles and optional relaunch requests can contain plaintext arguments; temporary environment overrides are not saved. Card records contain metadata only; output stays in bounded host/UI memory. See architecture and security documents for the storage boundaries.

## Build Windows installers / 构建安装包

```powershell
npm run bundle:windows -- --config scripts/local-build.conf.json
npm run verify:windows-bundle
```

Local build configuration disables updater-artifact creation so contributors do not need the maintainer's signing key. It does not change the app identifier or official update feed. These packages are unsigned and must not be uploaded as official updates.

Outputs are `src-tauri/target/release/bundle/nsis/CLI Hub_<version>_x64-setup.exe` and `src-tauri/target/release/bundle/msi/CLI Hub_<version>_x64_en-US.msi`. Verification checks presence, size, version, SHA-256 and Authenticode status; the unsigned warning is expected for current local builds.

本地包不改变默认应用标识、数据目录或更新地址。独立分发的 fork 应配置自己的应用标识、更新公钥和更新地址。

## Maintaining releases and documentation / 发布与文档维护

Maintainer builds use `npm run bundle:windows` with updater signing credentials supplied outside the repository. Keep secrets out of files and commits. Verify final installers, then generate final updater signatures/hashes and publish to the existing [release repository](https://github.com/sangowu/cli-hub-releases). Link each binary release to its corresponding source revision and preserve required license notices.

Use the Unreleased section of `docs/CHANGELOG.md` for user-visible fixes and features. Update architecture/security/UI documents when their boundaries change, and recheck the [dependency inventory and notices](THIRD_PARTY_NOTICES.md) when dependencies change. Fonts retain their own licenses.

Closing/updating only the interface preserves old host code. Finish tasks, choose **Stop all and exit**, and reopen when testing a new host version. Optional startup relaunch does not preserve process memory or conversations.
