# 第三方声明 / Third-party notices

CLI Hub 原创代码使用 GPL-3.0-only。下列第三方内容保留其原许可证；项目许可证不替代第三方许可。完整依赖版本由 `package-lock.json` 与 `src-tauri/Cargo.lock` 记录，分发时应保留所分发依赖要求的许可及版权声明。

Original CLI Hub code uses GPL-3.0-only. Third-party material retains its own license; the project license does not replace those terms. Exact dependency versions are recorded in `package-lock.json` and `src-tauri/Cargo.lock`. Preserve the notices required by the dependencies included in a distribution.

## 随应用提供的字体 / Bundled fonts

| Material | Source and version | License notices |
| --- | --- | --- |
| JetBrainsMono Nerd Font, Regular / Bold | JetBrains Mono 2.304, patched by Nerd Fonts 3.3.0; versions and copyright verified from the bundled fonts' name tables | [JetBrains Mono OFL](../public/licenses/JetBrainsMono-OFL.txt), [Nerd Fonts notices](../public/licenses/NerdFonts-LICENSE.txt) |
| Sarasa Mono SC, Regular / Bold | Sarasa Gothic 1.0.42; version and copyright verified from the bundled fonts' name tables | [Sarasa Gothic OFL](../public/licenses/SarasaGothic-LICENSE.txt) |

字体许可文件位于 `public/licenses/`，由前端构建复制进应用静态资源。字体不改用项目的 GPL 许可证。

The files in `public/licenses/` are copied into packaged frontend assets. The fonts are not relicensed under the project's GPL license.

Upstream references: [JetBrains Mono 2.304 license](https://github.com/JetBrains/JetBrainsMono/blob/v2.304/OFL.txt), [Nerd Fonts 3.3.0](https://github.com/ryanoasis/nerd-fonts/tree/v3.3.0), [Sarasa Gothic](https://github.com/be5invis/Sarasa-Gothic).

## 代码依赖 / Code dependencies

The application uses React, xterm.js, Tauri and its plugins, portable-pty, and other dependencies listed in the package manifests and lockfiles. See the [locked dependency license inventory](DEPENDENCY_LICENSES.md) and [collected license/notice texts](../public/licenses/dependency-notices.txt), which are included in packaged static assets. Recheck this inventory when changing dependencies or preparing a binary distribution.

应用使用 React、xterm.js、Tauri 及其插件、portable-pty 等依赖。锁定依赖的许可清单及汇总声明见上述链接，声明原文随应用静态资源打包；调整依赖或准备分发二进制版本时应重新核对。

## 项目声明 / Project notice

Copyright (C) 2026 CLI Hub contributors. CLI Hub is free software under GPL-3.0-only, without warranty. The complete [GPL text](../LICENSE) is also bundled at `licenses/CLI-Hub-GPL-3.0.txt`. Source is intended to be published at https://github.com/sangowu/cli-hub-source; versioned binary releases should link to the corresponding source revision and include build instructions and lockfiles.

项目 GPL 原文随静态资源提供。后续二进制发布应链接到对应版本的源码修订，保留构建步骤和依赖锁文件。
