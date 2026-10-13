"""Mock the v0.1.9 host's unknown-field dropping during a live interface update."""
from playwright.sync_api import sync_playwright
from verify_stage4_ui import TAURI_MOCK

OLD_HOST = r"""
(() => {
  const original = window.__TAURI_INTERNALS__.invoke;
  window.__TAURI_INTERNALS__.invoke = async (command, args) => {
    const old = sessionStorage.getItem('test-modern-host') !== 'true';
    if (old && command === 'save_ui_settings') {
      args = structuredClone(args);
      delete args.uiSettings.terminalShortcuts.newline;
    }
    const result = await original(command, args);
    if (old && result) {
      const prefs = command === 'load_app_preferences' ? result.preferences : result;
      if (prefs.uiSettings?.terminalShortcuts) delete prefs.uiSettings.terminalShortcuts.newline;
    }
    return result;
  };
})();
"""


def main():
    with sync_playwright() as pw:
        browser = pw.chromium.launch(headless=True)
        page = browser.new_page(locale='en-US')
        page.add_init_script(TAURI_MOCK + "\n" + OLD_HOST)
        page.goto('http://localhost:1420')
        page.get_by_role('button', name='Settings', exact=True).click()
        field = page.get_by_role('textbox', name='New line in agent input', exact=True)
        assert field.input_value() == 'Shift+Enter'
        field.press('Control+Enter')
        page.wait_for_function("localStorage.getItem('cli-hub.newline-shortcut.v1') === 'Ctrl+Enter'")
        page.reload()
        page.get_by_role('button', name='Settings', exact=True).click()
        assert field.input_value() == 'Ctrl+Enter', 'Old host lost custom newline after reload'
        page.evaluate("sessionStorage.setItem('test-modern-host', 'true')")
        page.reload()
        page.get_by_role('button', name='Settings', exact=True).click()
        assert field.input_value() == 'Ctrl+Enter', 'Replacement host lost fallback'
        field.press('Control+Shift+Enter')
        page.wait_for_function("window.__mockPreferences.uiSettings.terminalShortcuts.newline === 'Ctrl+Shift+Enter'")
        page.wait_for_function("localStorage.getItem('cli-hub.newline-shortcut.v1') === null")
        page.reload()
        page.get_by_role('button', name='Settings', exact=True).click()
        assert field.input_value() == 'Ctrl+Shift+Enter'
        page.get_by_role('button', name='Reset shortcuts', exact=True).click()
        page.wait_for_function("window.__mockPreferences.uiSettings.terminalShortcuts.newline === 'Shift+Enter'")
        print('PASS: old-host fallback, reload, replacement-host persistence, fallback cleanup, reset')
        browser.close()


if __name__ == '__main__':
    main()
