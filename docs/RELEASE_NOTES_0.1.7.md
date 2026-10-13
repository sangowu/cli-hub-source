# CLI Hub v0.1.7

CLI Hub v0.1.7 fixes visual consistency across settings, workspace tabs and confirmation dialogs.

- Reset shortcuts shares the settings action-button style, including typography, dimensions, hover, keyboard focus and light/dark themes. Shortcut headings, descriptions, inputs and spacing follow the existing settings hierarchy.
- Workspace tabs retain distinct selected colors in dark mode; close and add buttons keep their dedicated states.
- Exit focus stays readable when hovered and when focused by keyboard in either theme.
- Stop and exit confirmations share compact typography and 30px action buttons. Final danger confirmations keep a filled red background, white text and consistent action spacing.
- Added a UI design-system document covering typography, button variants, settings layout, theme states and verification.

Validation: frontend build; Rust formatting and 32 passing tests with one intentionally ignored manual stress test; existing mocked UI regression; and rendering checks for English/Chinese, light/dark themes, normal/minimum windows, hover and keyboard focus. This patch changes visual styling; native terminal behavior is unchanged.

Windows x64 NSIS and MSI installers use Tauri updater signatures. They do not have Authenticode signatures, so Windows may show an unknown-publisher warning. Windows 10/11 x64 with WebView2 Runtime is required.
