# CLI Hub v0.1.17

Keep the previous session preview while the CLI generates new output.

- Replace the preview only after two seconds without new terminal output, rather than publishing intermediate replies every 500ms. The first reply keeps the waiting placeholder until output settles.
- Update activity timestamps independently, so the running/output indicators remain current while preview text stays fixed.
- Preserve existing reply filtering, bounded in-memory previews and fixed card height. Clean up pending activity and preview timers when sessions are removed or the interface closes.

The quiet period is a heuristic, not an agent completion signal. A long pause during generation may publish intermediate content; continuous background output may delay replacement. No transcripts are persisted.

No terminal-host protocol or storage migration. Compatible running CLIs survive interface updates. Windows 10/11 x64 and WebView2 are required. Installers carry verified Tauri updater signatures but no Authenticode signature.

Validation: production frontend build and preview parser regression; mock/native Windows Tauri/PowerShell tests require one final replacement and stable card geometry; mock generation beyond two seconds and beyond the 360-character limit, first-reply delay and independent activity updates; existing interaction, session-history and shortcut regressions. No production sessions were changed.
