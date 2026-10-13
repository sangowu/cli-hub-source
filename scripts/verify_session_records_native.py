"""Native prepare/restarted regression, isolated newline-test instance on CDP 9226."""
import argparse
import json
from pathlib import Path
from playwright.sync_api import sync_playwright
from verify_terminal_newline import invoke, wait_for

STATE = Path('artifacts/session-records-native.json')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('phase', choices=['prepare', 'restarted', 'cleanup'])
    phase = parser.parse_args().phase
    with sync_playwright() as pw:
        browser = pw.chromium.connect_over_cdp('http://127.0.0.1:9226')
        page = browser.contexts[0].pages[0]
        page.wait_for_url('**:1420/**')
        assert invoke(page, 'plugin:app|identifier') == 'com.clihub.newline-test'
        if phase == 'prepare':
            assert not invoke(page, 'list_active_terminals'), 'Use an empty isolated test instance'
            settings = invoke(page, 'load_app_preferences')['preferences']['uiSettings']
            settings.update(language='en', restoreOnStartup=False, sessionSidebarCollapsed=False)
            invoke(page, 'save_ui_settings', {'uiSettings': settings})
            old_records = page.evaluate("JSON.parse(localStorage.getItem('cli-hub.session-records.v1') || '{\"records\":[]}').records.length")
            page.reload(wait_until='networkidle')
            wait_for(lambda: page.locator('.session-card').count() == old_records, 'Existing test records were not loaded')
            for _ in range(old_records):
                page.locator('.session-card-remove').first.click()
            page.wait_for_function("JSON.parse(localStorage.getItem('cli-hub.session-records.v1') || '{\"records\":[]}').records.length === 0")
            for name in ['Native record live', 'Native record ended']:
                page.get_by_role('button', name='New session', exact=True).click()
                page.locator('#launch-preset').select_option('cmd')
                page.locator('#session-name').fill(name)
                page.locator('#working-directory').fill(str(Path.cwd()))
                page.get_by_role('button', name='Start ' + name, exact=True).click()
                wait_for(lambda: any(s['name'] == name for s in invoke(page, 'list_active_terminals')), 'CLI launch failed')
            sessions = invoke(page, 'list_active_terminals')
            ended = next(s for s in sessions if s['name'] == 'Native record ended')
            invoke(page, 'stop_terminal', {'sessionId': ended['sessionId']})
            wait_for(lambda: page.locator('.session-card-exited').count() == 1, 'Stopped card missing')
            live = next(s for s in invoke(page, 'list_active_terminals') if s['name'] == 'Native record live')
            host = invoke(page, 'prepare_interface_update')
            page.wait_for_function("JSON.parse(localStorage.getItem('cli-hub.session-records.v1')).records.length === 2")
            STATE.write_text(json.dumps({'sessionId': live['sessionId'], 'processId': live['processId'], 'hostPid': host['pid']}), encoding='utf-8')
            try:
                invoke(page, 'plugin:process|restart')
            except Exception as error:
                if not any(part in str(error) for part in ['closed', 'destroyed', 'disconnected']):
                    raise
            print('PREPARED: real live+ended cards saved; interface restart requested without stopping host')
        else:
            if phase == 'restarted':
                expected = json.loads(STATE.read_text(encoding='utf-8'))
                wait_for(lambda: page.locator('.session-card').count() == 2, 'Cards lost after native restart')
                live = invoke(page, 'list_active_terminals')
                assert len(live) == 1 and live[0]['sessionId'] == expected['sessionId'] and live[0]['processId'] == expected['processId']
                assert invoke(page, 'prepare_interface_update')['pid'] == expected['hostPid']
                assert page.locator('.session-card-exited').count() == 1
                assert page.locator('.session-card-running').count() == 1
                print('PASS: native restart preserved host and CLI PIDs, live and stopped cards')
            invoke(page, 'stop_all_terminals')
            invoke(page, 'shutdown_application')


if __name__ == '__main__':
    main()
