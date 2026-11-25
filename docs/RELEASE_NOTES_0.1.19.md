# CLI Hub v0.1.19

Paste clipboard images directly inside supported agent terminals.

- Keep CLI Hub's existing text-paste behavior when the clipboard contains text.
- When the clipboard contains an image or another non-text format, forward native `Ctrl+V` to Codex, Claude Code, Pi, Hermes Agent, and OpenCode so the agent can read the clipboard itself.
- Keep PowerShell, Windows PowerShell, CMD, and custom CLI behavior unchanged.
- Add regression coverage for every enabled and excluded preset.

Image handling still depends on the selected agent's own clipboard support and input mode. No terminal-host protocol or preference-storage migration is required. Compatible running CLIs survive interface updates. Windows 10/11 x64 and WebView2 are required. Installers carry Tauri updater signatures but no Authenticode signature.

Validation: production frontend build, terminal shortcut regression checks, session-history and terminal-preview checks, UI interaction regression, Rust formatting and tests, and Windows bundle verification.
