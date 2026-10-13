# CLI Hub v0.1.9

CLI Hub now keeps running CLIs in an independent Windows background host. Closing, restarting or compatibly updating the interface reconnects to the original terminal processes, preserving their running tasks and in-memory state.

- Closing the window exits only the interface. Reopen CLI Hub to reconnect to live sessions, regardless of the optional startup-restoration setting.
- Settings explains background sessions and provides **Stop all and exit**. This explicit action, or the tray equivalent, stops all terminals and the host.
- Updates no longer require stopping live CLIs. The host uses a distinct executable name and a private versioned copy, so replacing the interface does not replace its running code.
- Local IPC is authenticated, limited to loopback and bounded. The host remains the single writer for preferences; lost connections block launches and updates instead of silently creating replacement sessions.
- Reopening after full shutdown correctly handles Windows retaining an exited process handle, and tray quit also shuts down an idle host.
- Computer or host restarts still end the original tasks. Optional startup restoration relaunches saved CLIs but cannot restore their memory, conversations or temporary environment.

**First migration from v0.1.8 or earlier:** finish or explicitly stop existing tasks before installing this release. Their PTYs belong to the old application and cannot be transferred to the new host. Process continuity applies to sessions launched by this host-enabled release and subsequent compatible interface updates.

Windows 10/11 x64 and WebView2 are required. Installers have Tauri updater signatures, but no Authenticode signature; Windows may show an unknown-publisher warning. CLI Hub does not install a Windows service or save terminal transcripts.

Validation: frontend build, Rust formatting and 39 passing tests (one manual stress test ignored); existing UI regression and eight sidebar combinations; update success/failure regression; real isolated Tauri task/PID/memory preservation; NSIS/MSI test-package replacement and reconnect while tasks ran (MSI current-user mode); and full-stop/reopen verification. Production application instances were not replaced or closed during testing.
