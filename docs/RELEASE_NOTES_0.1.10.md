# CLI Hub v0.1.10

Codex and Claude Code terminals now use **Shift+Enter** for a new line while **Enter** continues to submit. Previously, terminal keyboard encoding made both keys act like Enter.

- Customize the shared agent newline key in **Settings → Terminal shortcuts → New line in agent input**, for example Ctrl+Enter. Changes take effect immediately and survive reopening; reset restores Shift+Enter.
- Duplicate shortcut assignments are rejected, plain Enter stays reserved for submission, and IME composition does not capture or trigger shortcut mappings.
- Existing preferences gain the new default without losing other custom shortcuts. When an older v0.1.9 background host remains running during an update, a local WebView fallback preserves the new mapping; a supporting host saves it in the preferences store on the next settings save.
- Codex and Claude Code receive their native newline sequences. Shell/custom presets and agents started inside a shell retain their native input behavior. Agent-specific remapped bindings and non-input modes can differ. The composer keeps its own Shift+Enter editing shortcut.

Live CLI sessions can stay running during the interface update from v0.1.9. No agent configuration files are modified, and no terminal transcripts are saved.

Windows 10/11 x64 and WebView2 are required. Installers have Tauri updater signatures but no Authenticode signature.

Validation: frontend build, Rust formatting and 41 passing tests (one manual stress test ignored); keyboard/IME guards; existing UI/update regressions; isolated real Windows Codex/Claude two-line drafts with default and Ctrl+Enter mappings, settings conflict/reset/reload persistence, CMD Enter submission; old-host fallback and replacement-host persistence regression.
