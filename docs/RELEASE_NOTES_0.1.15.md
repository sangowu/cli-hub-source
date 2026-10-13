# CLI Hub v0.1.15

Keep session previews stable while CLIs redraw their terminal screens.

- Retain the last recognized reply during temporary clear-screen, thinking, progress and tool-output frames. Replace it when a new recognized reply appears; streaming replies continue to update.
- Coalesce preview updates every 500ms instead of 80ms, reducing distracting text changes during active output.
- Keep plain-output previews through empty/status-only frames, while allowing new filtered shell output to update normally.
- Fix the preview area to 31px (two lines) so content changes do not change card height.

Previews remain bounded and in memory only. Reply recognition uses rendered-text heuristics; unknown CLI formats may not expose reliable message boundaries. Once a reply is recognized, unmarked fallback output is held back until another recognized reply appears.

No terminal-host protocol or storage migration. Compatible running CLIs survive interface updates. Windows 10/11 x64 and WebView2 are required. Installers carry verified Tauri updater signatures but no Authenticode signature.

Validation: production frontend build; temporal preview tests for clear/status/tool frames, streaming and successive replies, shell fallback and bounded storage; existing shortcut/history and interaction regressions; eight sidebar language/theme/viewport cases; isolated native Windows Tauri/PowerShell clear-screen animations and streaming output with stable card height and coalesced preview updates. No production sessions were changed.
