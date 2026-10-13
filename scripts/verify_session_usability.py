"""Real Tauri smoke test. Use scripts/usability-test.conf.json and CDP port 9222.

prepare: verify settings, grouping, clipboard, and leave two sessions for restart.
restored: verify a fresh process restored both sessions; stop them and disable restore.
"""
import argparse
import time
from contextlib import contextmanager
from pathlib import Path
from playwright.sync_api import sync_playwright

PROJECT_DIR = str(Path(__file__).resolve().parents[1])


@contextmanager
def preserved_clipboard(page):
    original = invoke(page, 'plugin:clipboard-manager|read_text')
    try:
        yield
    finally:
        invoke(page, 'plugin:clipboard-manager|write_text', {'text': original})


def invoke(page, command, args=None):
    return page.evaluate("async ([c,a]) => window.__TAURI_INTERNALS__.invoke(c,a)", [command, args or {}])


def wait_for(predicate, message):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(.1)
    raise AssertionError(message)


def snapshot_text(page):
    return '\n'.join(bytes(s['output']).decode('utf-8', errors='replace') for s in invoke(page, 'list_active_terminals'))


def launch(page, name):
    page.get_by_role('button', name='New session', exact=True).click()
    page.locator('#launch-preset').select_option('cmd')
    page.locator('#session-name').fill(name)
    page.locator('#working-directory').fill(PROJECT_DIR)
    page.get_by_role('button', name=f'Start {name}', exact=True).click()
    wait_for(lambda: any(s['name'] == name for s in invoke(page, 'list_active_terminals')), 'session did not launch')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('phase', choices=['prepare', 'restored', 'cleanup', 'reset'])
    args = parser.parse_args()
    with sync_playwright() as pw:
        browser = pw.chromium.connect_over_cdp('http://127.0.0.1:9222')
        page = next(p for c in browser.contexts for p in c.pages if 'localhost:1420' in p.url)
        page.wait_for_load_state('networkidle')
        prefs = invoke(page, 'load_app_preferences')['preferences']
        settings = prefs['uiSettings']
        settings.update(language='en', colorMode='dark')
        if args.phase in ('cleanup', 'reset'):
            invoke(page, 'stop_all_terminals')
            settings['restoreOnStartup'] = False
            invoke(page, 'save_ui_settings', {'uiSettings': settings})
            if args.phase == 'cleanup': invoke(page, 'shutdown_application')
            return
        invoke(page, 'save_ui_settings', {'uiSettings': settings})
        page.reload()
        page.wait_for_load_state('networkidle')
        if args.phase == 'restored':
            wait_for(lambda: len(invoke(page, 'list_active_terminals')) == 2, 'restart did not restore two sessions')
            snapshots = invoke(page, 'list_active_terminals')
            assert {s['name'] for s in snapshots} == {'Restore A', 'Restore B'}
            assert len({s['pageId'] for s in snapshots}) == 2
            assert all(s['workingDirectory'] == PROJECT_DIR for s in snapshots)
            assert settings['terminalShortcuts']['copy'] == 'Ctrl+Shift+X', 'shortcut did not persist across restart'
            page.reload()
            page.wait_for_load_state('networkidle')
            assert len(invoke(page, 'list_active_terminals')) == 2, 'refresh duplicated sessions'
            invoke(page, 'stop_all_terminals')
            wait_for(lambda: not invoke(page, 'list_active_terminals'), 'cleanup failed')
            wait_for(lambda: not invoke(page, 'load_app_preferences')['preferences']['restoreSessions'], 'stopped sessions still saved')
            settings = invoke(page, 'load_app_preferences')['preferences']['uiSettings']
            settings['restoreOnStartup'] = False
            settings['terminalShortcuts']['copy'] = 'Ctrl+Shift+C'
            invoke(page, 'save_ui_settings', {'uiSettings': settings})
            print('PASS: fresh-process restoration, workspace ownership, directories, no refresh duplicates, stop cleanup')
            invoke(page, 'shutdown_application')
            return

        assert not invoke(page, 'list_active_terminals'), 'use an isolated empty test instance'
        page.get_by_role('button', name='Settings', exact=True).click()
        restore_toggle = page.get_by_role('checkbox', name='Restore CLI sessions on startup')
        if not restore_toggle.is_checked(): restore_toggle.click()
        wait_for(lambda: restore_toggle.is_checked(), 'restore setting did not save')
        field = page.get_by_role('textbox', name='Copy selection', exact=True)
        field.press('Control+Shift+X')
        wait_for(lambda: field.input_value() == 'Ctrl+Shift+X', 'shortcut remapping failed')
        page.get_by_role('textbox', name='Paste', exact=True).press('Control+Shift+X')
        assert page.get_by_text('This shortcut is already assigned to another action.', exact=True).is_visible()
        assert page.get_by_role('textbox', name='Paste', exact=True).input_value() == 'Ctrl+V'
        page.get_by_role('button', name='Reset shortcuts', exact=True).click()
        wait_for(lambda: field.input_value() == 'Ctrl+Shift+C', 'reset failed')
        page.get_by_role('button', name='Close settings dialog', exact=True).click()
        for label in ['CLI', 'All', 'Folder']:
            button = page.locator('.session-grouping-control').get_by_role('button', name=label)
            button.click()
            wait_for(lambda: button.get_attribute('aria-pressed') == 'true', 'group selection failed')
            assert '✓' in button.inner_text()
            assert button.evaluate('e => getComputedStyle(e).backgroundColor') == 'rgb(23, 98, 124)'
        launch(page, 'Restore A')
        terminal = page.locator('.terminal-container-active .xterm-helper-textarea')
        with preserved_clipboard(page):
            invoke(page, 'plugin:clipboard-manager|write_text', {'text': 'echo CLIPBOARD_PROBE_7841'})
            terminal.press('Control+V')
            wait_for(lambda: 'CLIPBOARD_PROBE_7841' in snapshot_text(page), 'clipboard paste did not reach CLI')
            terminal.press('Enter')
            wait_for(lambda: 'CLIPBOARD_PROBE_7841' in snapshot_text(page), 'clipboard paste failed')
            row = page.locator('.terminal-container-active .xterm-rows > div').filter(has_text='CLIPBOARD_PROBE_7841').last
            row.wait_for(state='visible')
            box = row.bounding_box()
            page.mouse.move(box['x'] + 1, box['y'] + box['height']/2)
            page.mouse.down()
            page.mouse.move(box['x'] + box['width'] - 2, box['y'] + box['height']/2, steps=10)
            page.mouse.up()
            invoke(page, 'plugin:clipboard-manager|write_text', {'text': 'copy-sentinel'})
            page.keyboard.press('Control+c')
            wait_for(lambda: 'CLIPBOARD_PROBE_7841' in invoke(page, 'plugin:clipboard-manager|read_text'), 'smart copy failed')
            invoke(page, 'plugin:clipboard-manager|write_text', {'text': 'copy-sentinel'})
            page.keyboard.press('Control+Shift+c')
            wait_for(lambda: 'CLIPBOARD_PROBE_7841' in invoke(page, 'plugin:clipboard-manager|read_text'), 'explicit copy failed')
            page.get_by_role('button', name='Settings', exact=True).click()
            field = page.get_by_role('textbox', name='Copy selection', exact=True)
            field.press('Control+Shift+X')
            wait_for(lambda: field.input_value() == 'Ctrl+Shift+X', 'live remapping failed')
            page.screenshot(path='artifacts/session-usability-settings.png')
            page.get_by_role('button', name='Close settings dialog', exact=True).click()
            box = row.bounding_box()
            page.mouse.move(box['x'] + 1, box['y'] + box['height']/2)
            page.mouse.down()
            page.mouse.move(box['x'] + box['width'] - 2, box['y'] + box['height']/2, steps=10)
            page.mouse.up()
            invoke(page, 'plugin:clipboard-manager|write_text', {'text': 'copy-sentinel'})
            page.keyboard.press('Control+Shift+x')
            wait_for(lambda: 'CLIPBOARD_PROBE_7841' in invoke(page, 'plugin:clipboard-manager|read_text'), 'remapped copy failed')
            page.mouse.click(box['x'] + 4, box['y'] + 8)
            invoke(page, 'write_terminal', {'sessionId': invoke(page, 'list_active_terminals')[0]['sessionId'], 'data': 'ping -t 127.0.0.1\r'})
            wait_for(lambda: 'TTL=' in snapshot_text(page), 'interrupt probe did not start')
            prompts = snapshot_text(page).count(PROJECT_DIR + '>')
            page.keyboard.press('Control+c')
            wait_for(lambda: snapshot_text(page).count(PROJECT_DIR + '>') > prompts, 'Ctrl+C did not interrupt running CLI command')
            invoke(page, 'plugin:clipboard-manager|write_text', {'text': 'echo ALTERNATE_PASTE_7841'})
            terminal.press('Control+Shift+V')
            wait_for(lambda: 'ALTERNATE_PASTE_7841' in snapshot_text(page), 'alternate paste did not reach CLI')
            terminal.press('Enter')
            wait_for(lambda: 'ALTERNATE_PASTE_7841' in snapshot_text(page), 'alternate paste failed')
        page.get_by_role('button', name='Add workspace', exact=True).click()
        launch(page, 'Restore B')
        assert len(invoke(page, 'load_app_preferences')['preferences']['restoreSessions']) == 2
        Path('artifacts').mkdir(exist_ok=True)
        page.screenshot(path='artifacts/session-usability-real.png')
        print('PASS: restore toggle, live remapping/conflicts/reset, dark grouping, smart copy, explicit copy, interrupt, paste and alternate paste')
        invoke(page, 'shutdown_application')


if __name__ == '__main__':
    main()
