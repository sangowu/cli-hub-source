# 安全政策 / Security policy

请勿在公开 Issue、PR 或截图中发布漏洞利用细节、终端记录、访问令牌或真实配置文件。CLI Hub 的安全边界和本地数据处理见 [安全设计说明](docs/SECURITY.md)。

Do not post exploit details, terminal transcripts, access tokens or real configuration files in public issues, pull requests or screenshots. See the [security boundaries](docs/SECURITY.md) for the threat model and local data handling.

## 漏洞反馈 / Reporting a vulnerability

公开仓启用 GitHub 私密漏洞反馈后，请通过 [Report a vulnerability](https://github.com/sangowu/cli-hub-source/security/advisories/new) 提交。若该入口尚不可用，只创建不含技术细节的 Issue 请求维护者提供私密沟通渠道；请勿公开漏洞细节。

Once private vulnerability reporting is enabled for the public repository, use [Report a vulnerability](https://github.com/sangowu/cli-hub-source/security/advisories/new). If unavailable, open an issue requesting a private contact channel without technical details; do not disclose the vulnerability publicly.

报告请提供受影响版本、Windows 版本、影响范围、最小复现步骤，以及已脱敏的证据。只在自己的隔离测试环境中复现。

Include the affected release, Windows version, impact, minimal reproduction steps and sanitized evidence. Reproduce only in an isolated environment you control.

## 修复范围 / Maintenance scope

项目优先在最新版本中提供修复；旧版长期维护不作承诺。更新界面不会替换仍运行的旧后台 Host：完成任务后使用 **Stop all and exit**，再重新打开以启用新版 Host。

Fixes target the latest release; long-term maintenance of older releases is not promised. Updating the interface preserves an already-running host. Finish your tasks, choose **Stop all and exit**, and reopen to activate the updated host.
