# 贡献指南 / Contributing

感谢帮助改进 CLI Hub。请将每次修改聚焦于一个问题；较大的功能或架构调整，请先在 Issues 说明需求与方案。

Keep each change focused on one problem. For substantial features or architecture changes, discuss the intended behavior in an issue first.

## 开发 / Development

环境、运行、打包与隔离测试步骤见 [开发文档](docs/DEVELOPMENT.md)。安装依赖使用 `npm ci`，保留 `package-lock.json` 和 `src-tauri/Cargo.lock`。

See the [development guide](docs/DEVELOPMENT.md) for setup, running, packaging and isolated testing. Use `npm ci` and keep both dependency lockfiles.

## 验证 / Verification

修改前确认相关检查的基线，修改后重新验证。前端修改运行 `npm run build`；Rust 修改在 `src-tauri` 中运行 `cargo fmt -- --check` 和 `cargo test`。终端行为的变化还需要真实 Windows Tauri 窗口验证，浏览器 mock 或编译成功不能替代它。

Establish the relevant baseline before editing and repeat the checks afterward. Run `npm run build` for frontend changes. Run `cargo fmt -- --check` and `cargo test` from `src-tauri` for Rust changes. Terminal behavior changes also need a real Windows Tauri smoke test; browser mocks and compilation do not establish native behavior.

涉及终端预览、会话历史或快捷键时，也运行对应检查：

When touching previews, session history or shortcuts, also run the relevant checks:

```powershell
node scripts/verify_terminal_preview.mjs
node scripts/verify_session_history.mjs
node scripts/verify_terminal_shortcuts.mjs
```

## 提交修改 / Submitting changes

- PR 说明问题、最终行为、验证结果和已知限制。/ Explain the problem, resulting behavior, validation and known limitations.
- 行为变化同步更新文档；修复缺陷时更新 `docs/FIX_LOG.md`。/ Update documentation for behavior changes and `docs/FIX_LOG.md` for bug fixes.
- 不提交凭据、签名私钥、个人路径、终端记录或本地配置；截图和复现示例使用虚拟数据。/ Do not submit credentials, signing keys, personal paths, terminal transcripts or local configuration; use fictional examples.
- 安全问题按 [安全政策](SECURITY.md) 私下反馈。/ Follow the [security policy](SECURITY.md) for private vulnerability reports.

提交贡献表示你有权提供该内容，并同意原创贡献使用项目的 GPL-3.0-only 许可证；第三方内容必须保留来源及其许可声明。

By contributing, you confirm that you have the right to submit the material and agree to license your original contributions under GPL-3.0-only. Preserve provenance and notices for third-party material.
