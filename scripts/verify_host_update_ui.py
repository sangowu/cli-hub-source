"""Mocked update regression with active background sessions; Vite on port 1420."""
from playwright.sync_api import sync_playwright, expect
from verify_stage4_ui import TAURI_MOCK
from verify_session_sidebar import seed

UPDATE_MOCK = r"""
(() => {
  const original = window.__TAURI_INTERNALS__.invoke;
  window.__hostUpdateCalls = [];
  window.__TAURI_INTERNALS__.invoke = async (command, args) => {
    if (command.startsWith('plugin:updater|') || command === 'prepare_interface_update' || command === 'plugin:process|restart') {
      window.__hostUpdateCalls.push(command);
      if (command === 'plugin:updater|check') return {rid:100,currentVersion:'0.1.8',version:'0.1.9',body:'Test update',rawJson:{}};
      if (command === 'plugin:updater|download') {
        if (window.__fault === 'download') throw new Error('Download test failure');
        return 101;
      }
      if (command === 'prepare_interface_update') {
        if (window.__fault === 'preflight') throw new Error('Host test unavailable');
        return {protocol:1,pid:1234};
      }
      if (command === 'plugin:updater|install' && window.__fault === 'install') throw new Error('Installer test failure');
      return null;
    }
    return original(command,args);
  };
})();
"""


def main():
    with sync_playwright() as pw:
        browser = pw.chromium.launch()
        for fault in ['success','download','preflight','install']:
            page = browser.new_page()
            page.add_init_script(seed('en','dark') + TAURI_MOCK + UPDATE_MOCK)
            page.goto('http://localhost:1420', wait_until='networkidle')
            page.evaluate('(fault)=>window.__fault=fault',fault)
            page.get_by_role('button',name='Settings',exact=True).click()
            page.get_by_role('button',name='Check for updates',exact=True).click()
            install = page.get_by_role('button',name='Install 0.1.9',exact=True)
            expect(install).to_be_enabled()
            expect(page.get_by_text('Running CLIs stay in the background while this interface updates and restarts.')).to_be_visible()
            install.click()
            if fault == 'success':
                page.wait_for_function("window.__hostUpdateCalls.includes('plugin:process|restart')")
            else:
                expect(page.locator('.settings-body .error-message')).to_be_visible()
                expect(install).to_be_enabled()
            calls = page.evaluate('window.__hostUpdateCalls')
            expected = ['plugin:updater|check','plugin:updater|download']
            if fault != 'download': expected.append('prepare_interface_update')
            if fault not in ['download','preflight']: expected.append('plugin:updater|install')
            if fault == 'success': expected.append('plugin:process|restart')
            assert calls == expected, calls
            assert not page.evaluate("window.__commands.some(c=>c.startsWith('stop_') || c==='shutdown_application')")
            if fault == 'preflight':
                page.evaluate("window.__emitTauriEvent('terminal-host-disconnected','Test host connection lost')")
                expect(install).to_be_disabled()
                expect(page.get_by_text('Test host connection lost',exact=True)).to_be_visible()
            print(f'Active-session update {fault}: passed')
            page.close()
        browser.close()


if __name__ == '__main__': main()
