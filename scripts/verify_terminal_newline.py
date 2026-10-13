"""Real Tauri/WebView2 newline regression, isolated newline-test on CDP 9226.

Types a two-line draft in real agents and cancels it, without requesting a
model turn. Checks that plain Enter still executes a harmless CMD echo.
"""
import time
from pathlib import Path
from playwright.sync_api import sync_playwright


def invoke(page, command, args=None):
    return page.evaluate('async ([c,a]) => window.__TAURI_INTERNALS__.invoke(c,a)',
                         [command, args or {}])


def wait_for(predicate, message):
    deadline = time.monotonic() + 25
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(.1)
    raise AssertionError(message)


def main():
    with sync_playwright() as pw:
        browser = pw.chromium.connect_over_cdp('http://127.0.0.1:9226')
        page = browser.contexts[0].pages[0]
        page.wait_for_url('**:1420/**')
        page.wait_for_load_state('networkidle')
        assert invoke(page, 'plugin:app|identifier') == 'com.clihub.newline-test'
        assert not invoke(page, 'list_active_terminals'), 'Use an empty isolated test instance'
        settings = invoke(page, 'load_app_preferences')['preferences']['uiSettings']
        settings.update(language='en', restoreOnStartup=False)
        invoke(page, 'save_ui_settings', {'uiSettings': settings})
        page.reload()
        page.wait_for_load_state('networkidle')
        try:
            page.get_by_role('button', name='Settings', exact=True).click()
            field = page.get_by_role('textbox', name='New line in agent input', exact=True)
            page.get_by_role('button', name='Reset shortcuts', exact=True).click()
            wait_for(lambda: field.input_value() == 'Shift+Enter', 'Default newline missing')
            field.press('Enter')
            assert field.input_value() == 'Shift+Enter', 'Plain Enter must remain reserved'
            field.press('Control+c')
            assert page.get_by_text('This shortcut is already assigned to another action.', exact=True).is_visible()
            assert field.input_value() == 'Shift+Enter', 'Conflict overwrote newline'
            page.get_by_role('button', name='Close settings dialog', exact=True).click()
            for preset in ['codex', 'claude', 'cmd']:
                name = 'Newline ' + preset
                page.get_by_role('button', name='New session', exact=True).click()
                page.locator('#launch-preset').select_option(preset)
                page.locator('#session-name').fill(name)
                page.locator('#working-directory').fill(str(Path(__file__).resolve().parents[1]))
                if preset == 'codex':
                    page.locator('#arguments').fill('--no-daemon')
                page.get_by_role('button', name='Start ' + name, exact=True).click()
                terminal = page.locator('.terminal-container-active .xterm-helper-textarea')
                terminal.wait_for(state='visible')
                wait_for(lambda: any(s['name'] == name for s in invoke(page, 'list_active_terminals')),
                         'Agent process did not launch')
                time.sleep(2)
                if preset != 'cmd':
                    if preset == 'claude':
                        wait_for(lambda: 'Yes, I trust this folder' in page.locator('.terminal-container-active').inner_text()
                                 or '❯' in page.locator('.terminal-container-active').inner_text(),
                                 'Claude startup UI not ready')
                        if 'Yes, I trust this folder' in page.locator('.terminal-container-active').inner_text():
                            # Only the repository under test, never an arbitrary directory.
                            terminal.press('ArrowDown')
                            terminal.press('Enter')
                    wait_for(lambda: any('?' in r or '›' in r or '❯' in r for r in
                                         page.locator('.terminal-container-active .xterm-rows > div').all_text_contents()),
                             preset + ' input UI not ready')
                    time.sleep(2)
                    for index, key in enumerate(['Shift+Enter', 'Control+Enter']):
                        if index == 1:
                            page.get_by_role('button', name='Settings', exact=True).click()
                            field = page.get_by_role('textbox', name='New line in agent input', exact=True)
                            field.press(key)
                            wait_for(lambda: field.input_value() == 'Ctrl+Enter', 'Remapping failed')
                            page.get_by_role('button', name='Close settings dialog', exact=True).click()
                            assert invoke(page, 'load_app_preferences')['preferences']['uiSettings']['terminalShortcuts']['newline'] == 'Ctrl+Enter'
                        alpha = f'NEWLINE_ALPHA_{index}_582'
                        beta = f'NEWLINE_BETA_{index}_582'
                        terminal.press_sequentially(alpha)
                        terminal.press(key)
                        terminal.press_sequentially(beta)
                        def has_two_lines():
                            rows = page.locator('.terminal-container-active .xterm-rows > div').all_text_contents()
                            a = [i for i, row in enumerate(rows) if alpha in row]
                            b = [i for i, row in enumerate(rows) if beta in row]
                            return bool(a and b and b[-1] == a[-1] + 1)
                        try:
                            wait_for(has_two_lines, preset + ' did not display a two-line draft with ' + key)
                        except AssertionError:
                            Path('artifacts').mkdir(exist_ok=True)
                            page.screenshot(path='artifacts/newline-' + preset + '-failure.png')
                            raise
                        terminal.press('Control+c')
                        print('PASS: two-line draft without submission: ' + preset + ' ' + key)
                    page.get_by_role('button', name='Settings', exact=True).click()
                    page.get_by_role('button', name='Reset shortcuts', exact=True).click()
                    wait_for(lambda: page.get_by_role('textbox', name='New line in agent input', exact=True).input_value() == 'Shift+Enter', 'Reset failed')
                    page.get_by_role('button', name='Close settings dialog', exact=True).click()
                else:
                    terminal.press_sequentially('echo ENTER_OK_582')
                    terminal.press('Enter')
                    wait_for(lambda: any(r.strip() == 'ENTER_OK_582' for r in
                                         page.locator('.terminal-container-active .xterm-rows > div').all_text_contents()),
                             'Plain Enter did not execute CMD echo')
                    print('PASS: plain Enter still submits in CMD')
                invoke(page, 'stop_all_terminals')
                wait_for(lambda: not invoke(page, 'list_active_terminals'), 'Test session did not stop')
            page.get_by_role('button', name='Settings', exact=True).click()
            field = page.get_by_role('textbox', name='New line in agent input', exact=True)
            field.press('Control+Enter')
            wait_for(lambda: field.input_value() == 'Ctrl+Enter', 'Remapping failed')
            page.reload()
            page.wait_for_load_state('networkidle')
            page.get_by_role('button', name='Settings', exact=True).click()
            field = page.get_by_role('textbox', name='New line in agent input', exact=True)
            wait_for(lambda: field.input_value() == 'Ctrl+Enter', 'Reloaded UI lost mapping')
            page.get_by_role('button', name='Reset shortcuts', exact=True).click()
            wait_for(lambda: field.input_value() == 'Shift+Enter', 'Reset failed')
            print('PASS: settings conflicts, reserved Enter, reset, and reload persistence')
        finally:
            invoke(page, 'stop_all_terminals')
            invoke(page, 'shutdown_application')


if __name__ == '__main__':
    main()
