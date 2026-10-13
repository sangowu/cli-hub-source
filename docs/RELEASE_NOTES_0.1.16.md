# CLI Hub v0.1.16

Match terminal padding and content backgrounds to remove the contrasting black strips at the left and bottom edges.

- Set the xterm root and legacy viewport backgrounds to the same `#101418` used by the terminal content layer.
- Preserve existing padding, character grid dimensions, scrollbar styling and divider resizing behavior. Explicit CLI color commands and colored text cells are unchanged.

No terminal-host protocol or storage migration. Compatible running CLIs survive interface updates. Windows 10/11 x64 and WebView2 are required. Installers carry verified Tauri updater signatures but no Authenticode signature.

Validation: production frontend build; preview regression; mock minimum/large-window and one/two-column layout checks; isolated native Windows Tauri/PowerShell content, background-color equality, terminal bounds and fullscreen/restore checks. Screenshots inspected. No production sessions were changed.
