# Architecture

## Current slice

The Windows v0.1.x release assigns terminal sessions to persistent workspace tabs. Each workspace supports up to eight active sessions, while the Rust backend enforces a 24-session application safety limit:

```text
React input and xterm.js
          │
          │ Tauri commands and events
          ▼
Rust TerminalManager (session map)
          │
          ▼
portable-pty
          │
          ▼
PowerShell, CMD, Codex, Claude Code, or a custom executable
```

The UI sends a structured launch request plus session-scoped write, resize, and stop commands. Rust resolves executables, validates working directories and environment variables, and owns every PTY handle and child lifecycle. Output is emitted as raw byte arrays so UTF-8 characters split across OS reads remain valid when xterm.js parses the stream.

Each session carries a stable workspace identifier through the React launch request, Rust session map, and active-session recovery snapshot. The selected workspace groups its sessions using a normalized directory key returned by Rust. Only the selected workspace mounts xterm.js panes; a capped frontend byte cache rebuilds a pane after switching tabs. Rust retains at most 256 KiB of raw output per active session so a WebView refresh can rebuild cards and replay recent output. Output offsets deduplicate cached, snapshot, and live bytes. Output is never persisted to disk. Optional startup restoration stores launch requests without environment overrides, using resolved launch directories and separate argument arrays. Rust restores once per process after event listeners register; startup and shutdown suppress intermediate snapshot writes so partially restored or terminating sessions do not overwrite the list. Normal exits and explicit stops update the saved list.

The versioned operating-system configuration file keeps workspace names and selection, up to 20 launch profiles, 10 recent working directories, and UI settings. Profile records deliberately exclude environment variables, terminal output, status, PID, and session identifiers. Storage is local but not encrypted, so command-line arguments must not contain secrets.

## Cross-platform boundary

`portable-pty` is the operating-system boundary. Windows provides explicit PowerShell, Windows PowerShell, CMD, Codex, and Claude Code presets. PowerShell 7 is preferred when available. Codex and Claude Code resolve native executables first, then structured PowerShell or CMD launchers. Non-Windows code keeps the same structured launch boundary but the Windows-only presets return explicit errors. No Windows API is called from session or UI code.

The product is not considered supported on macOS or Linux until real-device interaction tests pass. Compilation alone cannot validate IME, clipboard, shortcuts, desktop environment behavior, or process cleanup.

## Lifecycle

- Up to eight sessions may be active in one workspace and up to 24 across the application. The Rust limit tests validate the per-workspace and global boundaries. The ignored Windows release-gate test validates 24 simultaneously live interactive CMD PTYs over 10 cycles (240 process launches total), including resize, input, isolated output, normal exit, and cleanup. `scripts/verify_stage6_real_ui_stress.py` drives the real Tauri/WebView2 IPC surface and validates three 8-session workspaces, refresh recovery, tab switching, UI Send, page-local stop, final cleanup, and the child process working directory. Windows launch paths use ordinary drive or UNC syntax instead of the canonical `\\?\` form; blank working directories resolve to the user home directory. Native tray clicks remain a separate manual acceptance item.
- Each start receives a monotonically increasing session identifier.
- Commands reject stale session identifiers.
- A wait thread removes only its own session, waits for final output to drain, and then reports the exit code.
- Each reader uses a 64-slot bounded queue; an emitter batches output for at most 8 ms or 32 KiB.
- Stop requests terminate the child through the PTY library's child killer.
- The frontend subscribes to output and exit events before requesting the active-session snapshot, then merges snapshot and live output by byte offset.
- **Stop page** selects children by workspace identifier and leaves other workspaces untouched. Explicit application shutdown still stops every managed child.
- A native window close request reads the Rust session map. With active sessions it hides the main window while the process and tray remain alive; without active sessions it exits. Tray Quit reopens the window and emits the existing stop-all confirmation request.
- The single-instance plugin focuses the existing main window when another application launch is attempted.
- The updater plugin checks a public binary-only GitHub release feed, verifies every installer with the embedded updater public key, and refuses installation while managed CLI sessions are active. Windows launches the passive NSIS updater and restarts the application after installation. On every launch, cleanup is restricted to temporary entries with CLI Hub's updater-specific prefix so unrelated downloads are never touched.

## Security and privacy

- CLI Hub launches only a command selected by the local user.
- Executables and arguments remain separate values; CLI Hub does not concatenate a shell command string.
- Environment overrides exist only for the launched process and are not persisted.
- Terminal output stays in memory and is not persisted by default.
- Authentication remains owned by the launched CLI.
- The frontend has Tauri core, dialog, autostart, process-restart, updater, and text-only clipboard read/write permissions. Updater downloads are accepted only after signature verification against the embedded public key.

## Performance boundary

Each operating-system read is at most 8 KiB. Per-session bounded queues apply backpressure, and short time/size batching reduces frontend IPC pressure. Both the Rust recovery history and frontend remount cache are capped at 256 KiB per session. Inactive workspaces do not retain mounted xterm.js canvases.
