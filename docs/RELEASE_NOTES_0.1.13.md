# CLI Hub v0.1.13

Add three agent launch templates and refresh the workspace header.

- Add **Pi**, **Hermes Agent**, and **OpenCode** to **New session → CLI preset**, using their official product names and the `pi`, `hermes`, and `opencode` commands. Install each CLI separately and ensure its command is on PATH, or edit its executable path. Arguments remain editable; saved profiles retain the configuration.
- Templates use the existing Custom CLI launch/profile format, preserving compatibility with running terminal hosts. CLI grouping treats them as Custom CLI; native terminal shortcut behavior is retained.
- Pair the existing terminal-stack application icon with a compact 24px CLI Hub wordmark and a cyan Hub accent. Keep the header to a single row, aligned with its controls in both themes.
- Replace the terminal toolbar hint with **Double-click the CLI view to enter fullscreen mode**, and **双击CLI界面可进入全屏模式** in Chinese.

No terminal-host protocol or storage migration. Compatible live CLIs continue through interface updates from host-enabled versions. Windows 10/11 x64 and WebView2 are required. Installers have Tauri updater signatures but no Authenticode signature.

Validation: production build; template selection, saved-profile reload and compatible launch requests; existing interaction, session-history, shortcut, sidebar and update regressions; English/Chinese × light/dark × normal/minimum-window header checks; isolated native Tauri/PTY script launches for all templates. Native template smoke tests use local scripts rather than authenticated agent conversations; Pi and Hermes Agent were not found on the test machine's PATH.
