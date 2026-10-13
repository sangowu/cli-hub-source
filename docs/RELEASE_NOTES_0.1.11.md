# CLI Hub v0.1.11

Session cards now survive interface updates and restarts, including stopped and failed cards. Background CLI sessions in Settings also follows the existing compact typography, spacing and right-aligned action layout.

- Save card names, workspace ownership, working directories and status in local WebView storage. Live host snapshots determine which CLIs are still running; saved running cards whose processes are absent appear as stopped.
- Reconnect live processes and reconcile optional startup relaunches without duplicating their cards. Keep stopped/failed records until you remove the card or its workspace. Clearing application data removes the records.
- Exclude terminal output, error text, arguments and environment overrides from card storage. Recent output for live sessions still comes from the background host's bounded memory.
- Save the latest card state before installing an update, including a CLI that exits during download. Invalid saved records are left untouched, and update installation is blocked if records cannot be loaded or saved.

The new card storage takes effect after installing v0.1.11. It cannot reconstruct records already lost by an older interface or preserve stopped cards that disappear during the initial upgrade from that interface. The reported cause of earlier running-process termination remains unconfirmed; this release fixes the independently verified card-retention defect. Compatible live CLIs can continue through interface updates from host-enabled versions without changing the host protocol.

Windows 10/11 x64 and WebView2 are required. NSIS and MSI installers have Tauri updater signatures but no Authenticode signature.

Validation: frontend build; metadata/reconciliation and keyboard tests; existing UI/update/sidebar and older-host shortcut compatibility regressions; mocked update/reload retention, removal, corrupt-storage protection and exit-during-download checks; isolated native Windows interface restart retaining host/CLI PIDs and live/stopped cards; eight English/Chinese, light/dark and normal/minimum-window settings checks.
