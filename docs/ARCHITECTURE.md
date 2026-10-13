# Architecture / 架构

CLI Hub separates the desktop interface from the Windows process that owns terminal sessions. This document records current design choices; version history is in the [changelog](CHANGELOG.md).

界面负责展示和交互，独立后台 Host 负责终端与子进程。本文描述当前架构；历史变化见更新日志。

## Components

```text
React interface + xterm.js
          |
          | Tauri commands and events
          v
GUI-side HostClient
          |
          | authenticated, bounded loopback IPC
          v
Windows terminal host
  TerminalManager + PreferencesStore
          |
          | portable-pty
          v
PowerShell, CMD, Codex, Claude Code, or a custom CLI
```

| Component | Responsibility |
| --- | --- |
| React and xterm.js | Workspace navigation, interactive VT rendering, input composition, shortcuts and bounded previews |
| GUI-side `HostClient` | Forward terminal/preferences commands and subscribe to host events |
| Host `TerminalManager` | Own PTY handles, child processes, workspace ownership, output queues and recovery snapshots |
| Host `PreferencesStore` | Write versioned local preferences and optional startup relaunch requests |
| `portable-pty` | Provide the operating-system PTY and child lifecycle boundary |

See [terminal-host details](TERMINAL_HOST.md) for executable isolation, protocol compatibility, connection failures and native verification.

## Launch and input decisions

- Executable, argument array, working directory and environment entries remain separate values. The hub does not construct a shell command string. Windows script entries use the appropriate structured PowerShell/CMD launcher.
- Rust resolves and validates launch paths. A blank working directory resolves to the user's home directory; directory grouping uses a native normalized key. Windows launch paths remove the verbatim prefix that CMD cannot execute.
- PowerShell starts with `-NoLogo -NoProfile` for predictable behavior. Authentication and agent configuration remain the launched CLI's responsibility.
- xterm.js renders VT sequences, cursor movement, alternate screens and terminal keyboard modes. Output travels as byte arrays so UTF-8 split across operating-system reads can be decoded correctly.
- Codex/Claude preset newline translation is a narrow input adaptation, not a conversation API. Shell/custom presets retain native input behavior. See [development notes](DEVELOPMENT.md) for the shortcut sequences and verification scope.

## Lifecycle and updates

- The host enforces eight active sessions per workspace and 24 globally. Each start gets a new host session ID; commands reject stale IDs.
- Closing the main window exits the interface and tray while the host keeps its CLIs running. Reopening reconnects to existing processes. Sign-in startup and Start minimized apply to the interface.
- **Stop page** affects one workspace. **Stop all and exit** saves startup relaunch requests before stopping the host and all its tasks. A computer or host restart ends the original processes.
- Optional startup restoration relaunches saved commands only when needed; it does not restore CLI conversations, process memory or temporary environment values. Startup/shutdown snapshot guards prevent partially restored or terminating sessions from overwriting the saved list.
- The host has a distinct executable name and a private content-addressed copy, so replacing the GUI does not overwrite running host code. A full stop/reopen activates a newer host.
- Compatible interface updates allow live sessions to continue. Before installation the interface saves card metadata and prepares the host; unavailable/corrupt card storage or host communication can block installation. The updater verifies artifacts against the embedded public key and uses the existing public release feed.
- The single-instance plugin focuses an existing interface. Temporary update cleanup is restricted to CLI Hub's updater-specific entries.

On Windows, stop handling uses observed child exit as the final result after requesting termination. This works around the cloned-killer return-value problem in the pinned `portable-pty` version; a two-second deadline preserves failures when a child does not exit. Recheck this compatibility code when upgrading that dependency.

## Output and persistence

Each OS read is capped at 8 KiB. Per-session 64-slot queues apply backpressure; events batch for at most 8 ms or 32 KiB. Exit notification follows final output drainage. The host's recent-output history and the frontend remount cache are each capped at 256 KiB per session. Output offsets merge snapshots and live bytes without duplication; only the selected workspace mounts terminal panes.

Session-card metadata is persisted separately from live processes. Host snapshots remain authoritative: a missing saved running process becomes a stopped card. Stopped/failed cards remain until removal. Card records exclude output, errors, arguments and environment overrides. UI IDs are kept separate from host IDs so a later process cannot overwrite a historical card through ID reuse.

Preferences keep workspace ordering/selection, UI settings, up to 20 launch profiles and 10 recent directories. Profiles and relaunch requests can include plaintext arguments, but never temporary environment overrides or terminal transcripts. Configuration and WebView card storage have different purposes; deleting application data removes saved records.

Previews scan bounded rendered text, retain recognized replies through status redraws, and replace text after two seconds without new output. Activity updates independently. This quiet period is a heuristic, not an agent completion signal; unknown CLI formats may not expose reliable reply boundaries. No transcripts are persisted.

## Security and platform scope

The host binds only to loopback and authenticates local requests. Protected Windows file ACLs restrict endpoint tokens and configuration. The frontend uses a production CSP and explicit Tauri capabilities. These controls do not sandbox launched CLIs or protect against administrators and malware running as the same user. See [security boundaries](SECURITY.md) and the root [reporting policy](../SECURITY.md).

Windows x64 is the supported and real-device validation target. `portable-pty` keeps the native process boundary portable; macOS/Linux are not supported until real-device interaction tests cover IME, clipboard, shortcuts and cleanup. Compilation alone is insufficient.
