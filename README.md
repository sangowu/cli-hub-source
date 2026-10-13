# CLI Hub

![CLI Hub — 多个 CLI，一个工作台。Windows 桌面工作台，展示使用虚拟路径的示例会话。](docs/images/cli-hub-banner.png)

**少切窗口，多做事情。｜Fewer windows. More focus.**

[下载 Windows 版本 / Download for Windows](https://github.com/sangowu/cli-hub-releases/releases/latest) · [历史版本 / All releases](https://github.com/sangowu/cli-hub-releases/releases)

### 动态演示 / Animated demo

![CLI Hub demo — session grouping, sidebar collapse/expand, and launch templates](docs/images/cli-hub-demo-en.gif)

[下载高清 MP4 / Download HD MP4](docs/videos/cli-hub-demo-en.mp4) · English · 43s · 1080p

展示 CLI 分组排列、会话栏收起与展开，以及启动模板设置，含鼠标与键盘操作效果。使用示例会话和虚拟路径。

See session grouping, sidebar collapse/expand, and launch template configuration, with visible mouse and keyboard actions. Uses demo sessions and fictional paths.

## 中文

同时使用多个 Codex、Claude Code 或终端，却被满屏窗口困扰，找不到对应的项目？

CLI Hub 是一款基于 Rust 与 Tauri 构建的 Windows 桌面工作台，让你在一个界面中启动、查看和使用多个命令行工具。

- **按项目管理**：多工作区标签页，支持按工作目录、按 CLI 类型和不分组三种视图。每个工作区最多 8 个活动会话，全局最多同时运行 24 个。
- **高效交互**：实时输出预览、完整交互终端，为任意 CLI 提供统一的多行输入区，支持 Shift+Enter 换行。
- **后台继续运行**：关闭、重启或更新界面时，已运行的 CLI 可继续在后台工作；重新打开即可连接原有会话。
- **启动更省心**：保存多套启动配置，保留会话卡片，并可选择在后台进程停止后，下次启动时重新启动此前运行的 CLI。
- **兼容多种工具**：提供 Codex、Claude Code、Pi、Hermes Agent、OpenCode、PowerShell 和 CMD 启动模板，也可自定义可执行文件、参数与工作目录，保存为常用启动配置。提供中英文界面与明暗主题。

### 下载与使用

**前置准备**

- [ ] 使用 **Windows 10/11 x64** 系统。
- [ ] 安装 **Microsoft Edge WebView2 Runtime**；已安装可跳过。
- [ ] 安装需要使用的 **CLI 工具**（例如 Codex、Claude Code 或 PowerShell），并完成该工具要求的配置或登录。
- [ ] 确保 CLI 命令可从系统 `PATH` 找到，或在 CLI Hub 的自定义启动配置中指定可执行文件路径。

CLI Hub 安装版无需 Node.js 或 Rust；所选 CLI 工具的额外依赖请按其官方安装说明准备。

**安装与启动**

1. 前往[最新版本](https://github.com/sangowu/cli-hub-releases/releases/latest)，下载 `.exe` 或 `.msi` 安装包。
2. 安装并打开 CLI Hub，创建工作区，点击 **New session / 新建会话**，选择 CLI 和项目目录。
3. 运行多个会话，通过会话卡片查看输出、切换终端。

**Windows 安装提示**

当前 Windows 安装包尚未使用 Authenticode 代码签名，因此安装时可能出现“Windows 已保护你的电脑”或“未知发布者”提示。这是当前未签名版本可能出现的提示，本身不等同于检测到病毒。

- 确认安装包来自本仓库的 [Releases](https://github.com/sangowu/cli-hub-releases/releases)，并可使用 `Get-FileHash` 核对该版本 `SHA256SUMS.txt` 中的 SHA-256 值。
- 如果是 SmartScreen 的未识别应用提示，并提供继续选项：点击 **更多信息 → 仍要运行**。如果随后出现用户账户控制（UAC）提示，确认这是你刚启动的安装程序后，点击 **是** 继续。
- 如果提示具体病毒／威胁、Smart App Control 拦截，或组织策略禁止运行，请停止安装并反馈完整提示；不要关闭系统安全防护。

关于提示原因，参见 [Microsoft SmartScreen 说明](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation)。

重新启动 CLI 不会恢复工具内部的对话状态。电脑重启或后台进程退出会结束原有任务；如需主动停止全部任务，请使用 **Stop all and exit / 停止全部并退出**。

## English

Running several Codex, Claude Code, or terminal sessions and losing track of which window belongs to which project?

CLI Hub is a Windows desktop workspace built with Rust and Tauri. Launch, monitor, and interact with multiple command-line tools from one interface.

- **Organize by project**: workspace tabs with directory grouping, CLI-type grouping, or an ungrouped view. Run up to 8 active sessions per workspace and 24 across the app.
- **Work comfortably**: live output previews, fully interactive terminals, and a shared multiline composer for any CLI, with Shift+Enter to insert a new line.
- **Keep tasks running**: active CLIs can continue in the background when you close, restart, or update the interface. Reopen CLI Hub to reconnect.
- **Start faster**: save multiple launch profiles, retain session cards, and optionally relaunch previously running CLIs on startup after the background host has stopped.
- **Use your tools**: launch templates for Codex, Claude Code, Pi, Hermes Agent, OpenCode, PowerShell, and CMD. Customize executables, arguments, and working directories, then save them as reusable launch profiles. English and Simplified Chinese interfaces, with light and dark themes.

### Get started

**Prerequisites**

- [ ] Use **Windows 10/11 x64**.
- [ ] Install **Microsoft Edge WebView2 Runtime**, unless it is already installed.
- [ ] Install the **CLI tools** you want to use (such as Codex, Claude Code, or PowerShell), and complete any required configuration or sign-in.
- [ ] Make the CLI commands available on the system `PATH`, or specify the executable path in a custom CLI Hub launch profile.

The installed CLI Hub app does not require Node.js or Rust. Follow each CLI tool's official installation instructions for any additional dependencies.

**Install and launch**

1. Open the [latest release](https://github.com/sangowu/cli-hub-releases/releases/latest) and download the `.exe` or `.msi` installer.
2. Install and open CLI Hub, create a workspace, and choose **New session** to select a CLI and project directory.
3. Run your sessions, preview their output, and switch terminals through the session cards.

**Windows installation prompts**

The current Windows installers are not Authenticode-signed, so Windows may display “Windows protected your PC” or “Unknown publisher.” These prompts can occur with the current unsigned release and do not, by themselves, indicate a virus detection.

- Download only from this repository's [Releases](https://github.com/sangowu/cli-hub-releases/releases). You can use `Get-FileHash` to compare the SHA-256 hash with that release's `SHA256SUMS.txt`.
- For a SmartScreen unrecognized-app prompt that offers an override, choose **More info → Run anyway**. If a User Account Control (UAC) prompt follows, confirm it is the installer you just launched before choosing **Yes**.
- If Windows reports a specific threat, Smart App Control blocks the app, or organizational policy prevents execution, stop and report the full message. Do not disable system security protections.

See [Microsoft's SmartScreen guidance](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).

Relaunching a CLI does not restore its internal conversation state. A computer restart or background-host exit ends the original tasks. Use **Stop all and exit** when you want to stop every task explicitly.

## 从源码构建 / Build from source

开发环境需要 Windows 10/11 x64、Node.js 22.12+ 与 npm、Rust stable（MSVC 工具链）、Visual Studio C++ Build Tools（含 Windows SDK），以及 WebView2 Runtime。安装版用户不需要 Node.js 或 Rust。

Development requires Windows 10/11 x64, Node.js 22.12+ with npm, Rust stable (MSVC toolchain), Visual Studio C++ Build Tools with the Windows SDK, and WebView2 Runtime. Installed-app users do not need Node.js or Rust.

```powershell
git clone https://github.com/sangowu/cli-hub-source.git
cd cli-hub-source
npm ci
npm run tauri dev
```

本地构建安装包，无需维护者的更新签名私钥 / Build local installers without the maintainer's updater signing key:

```powershell
npm run bundle:windows -- --config scripts/local-build.conf.json
npm run verify:windows-bundle
```

本地构建与正式版默认共用应用标识、数据目录及官方更新源；隔离测试请按开发文档使用专用配置。公开源码独立维护，安装包和自动更新继续由 [cli-hub-releases](https://github.com/sangowu/cli-hub-releases) 提供。

Local builds share the official app identifier, data location and update feed by default; use the dedicated configurations in the development guide for isolated testing. Source is maintained here; installers and automatic updates continue through [cli-hub-releases](https://github.com/sangowu/cli-hub-releases).

## 参与贡献 / Contributing

请先阅读 [贡献指南 / Contributing guide](CONTRIBUTING.md)、[开发与测试 / Development and testing](docs/DEVELOPMENT.md) 和 [架构说明 / Architecture](docs/ARCHITECTURE.md)。漏洞反馈请阅读 [安全政策 / Security policy](SECURITY.md)。

## 许可证 / License

Copyright (C) 2026 CLI Hub contributors.

CLI Hub 原创代码按 **GNU GPL v3.0 only（SPDX: GPL-3.0-only）** 发布；完整条款见 [LICENSE](LICENSE)。第三方依赖和字体保留各自许可证，见 [第三方声明 / Third-party notices](docs/THIRD_PARTY_NOTICES.md)。

Original CLI Hub code is licensed under **GNU GPL v3.0 only (SPDX: GPL-3.0-only)**. See [LICENSE](LICENSE). Third-party dependencies and fonts retain their own licenses; see [third-party notices](docs/THIRD_PARTY_NOTICES.md).
