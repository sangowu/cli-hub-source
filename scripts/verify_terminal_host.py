"""Real Windows host smoke test; requires host-test.conf.json and CDP 9224.

prepare: create two live PTYs, a running task and in-memory shell state.
detached: verify the original task continued after the test GUI was killed/closed.
reconnected: verify the restarted GUI attached to the same PIDs; then clean up.
Only com.clihub.host-test is accessed. No production sessions or credentials.
"""
import argparse
import json
import os
import socket
import time
from pathlib import Path
from playwright.sync_api import sync_playwright

ROOT = Path(os.environ['APPDATA']) / 'com.clihub.host-test'
STATE = Path('artifacts/terminal-host-test-state.json')


def rpc(command, args=None, token=None, protocol=1):
    endpoint = json.loads((ROOT / 'terminal-host.json').read_text())
    with socket.create_connection(('127.0.0.1', endpoint['port']), timeout=10) as stream:
        stream.sendall((json.dumps(dict(protocol=protocol, token=token if token is not None else endpoint['token'],
                                       command=command, args=args or {})) + '\n').encode())
        result = json.loads(stream.makefile('rb').readline())
    if result['error']:
        raise RuntimeError(result['error'])
    return result['value']


def invoke(page, command, args=None):
    return page.evaluate('async ([c,a]) => window.__TAURI_INTERNALS__.invoke(c,a)', [command, args or {}])


def wait_for(predicate, message):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        if predicate(): return
        time.sleep(.1)
    raise AssertionError(message)


def snapshots(): return rpc('list_active_terminals')
def text(): return '\n'.join(bytes(s['output']).decode('utf-8', errors='replace') for s in snapshots())
def send(session_id, command):
    rpc('write_terminal', {'sessionId': session_id, 'data': command})
    rpc('write_terminal', {'sessionId': session_id, 'data': '\r'})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('phase', choices=['prepare', 'detached', 'reconnected', 'restarted', 'cleanup'])
    parser.add_argument('--keep-running', action='store_true', help='Keep isolated tasks for another installer test')
    arguments = parser.parse_args()
    phase = arguments.phase
    if phase == 'cleanup':
        rpc('shutdown_application')
        return
    if phase == 'detached':
        saved = json.loads(STATE.read_text())
        assert rpc('ping')['pid'] == saved['hostPid']
        assert [[s['sessionId'], s['processId']] for s in snapshots()] == saved['sessions']
        assert len(text()) > saved['outputLength'], 'Task did not continue while GUI was absent'
        send(saved['sessions'][0][0], 'echo STATE_%CLI_HUB_HOST_STATE%')
        wait_for(lambda: 'STATE_preserved-in-memory' in text(), 'Shell memory was lost')
        print('Original host, both PTY PIDs, running task and shell memory survived GUI exit')
        return
    with sync_playwright() as pw:
        browser = pw.chromium.connect_over_cdp('http://127.0.0.1:9224')
        is_test_page = lambda p: ':1420' in p.url or 'tauri.localhost' in p.url or p.url.startswith('tauri://')
        wait_for(lambda: any(is_test_page(p) for c in browser.contexts for p in c.pages), 'Test GUI did not load')
        page = next(p for c in browser.contexts for p in c.pages if is_test_page(p))
        page.wait_for_load_state('networkidle')
        if phase == 'prepare':
            assert not snapshots(), 'Use a clean isolated test host'
            settings = rpc('load_app_preferences')['preferences']['uiSettings']
            settings.update(language='en', restoreOnStartup=False)
            invoke(page, 'save_ui_settings', {'uiSettings': settings})
            page.reload()
            page.wait_for_load_state('networkidle')
            for command, args in [('ping', {'token':'wrong'}), ('ping', {'protocol':2})]:
                try: rpc(command, **args)
                except RuntimeError as error: assert 'rejected' in str(error)
                else: raise AssertionError('Invalid IPC credentials accepted')
            for preset, name in [('cmd', 'Host CMD'), ('windowsPowerShell', 'Host task')]:
                invoke(page, 'start_terminal', {'request': dict(pageId='workspace-default', preset=preset, name=name,
                    executable='', arguments=[], workingDirectory=str(Path.cwd()),
                    environment=[dict(name='CLI_HUB_EPHEMERAL', value='not-on-disk')])})
            sessions = snapshots()
            # Windows ConPTY queries the cursor on startup; mount xterm before submitting input.
            page.reload()
            page.wait_for_load_state('networkidle')
            wait_for(lambda: page.get_by_role('button', name='New session', exact=True).is_enabled(), 'UI did not attach PTYs')
            page.locator('.session-card-main').filter(has_text='Host CMD').click()
            wait_for(lambda: 'cli-hub>' in text(), 'CMD cursor negotiation did not finish')
            send(sessions[0]['sessionId'], 'set CLI_HUB_HOST_STATE=preserved-in-memory')
            send(sessions[0]['sessionId'], 'echo PREPARED_%CLI_HUB_HOST_STATE%')
            wait_for(lambda: 'PREPARED_preserved-in-memory' in text(), 'Initial shell state did not persist')
            page.locator('.session-card-main').filter(has_text='Host task').click()
            send(sessions[1]['sessionId'], "1..3600 | ForEach-Object { Write-Output ('HOST_TICK_' + $_); Start-Sleep -Milliseconds 500 }")
            wait_for(lambda: 'HOST_TICK_2' in text(), 'Running task did not start')
            page.reload()
            page.wait_for_load_state('networkidle')
            wait_for(lambda: page.get_by_role('button', name='New session', exact=True).is_enabled(), 'UI did not reconnect')
            assert invoke(page, 'prepare_interface_update')['pid'] == rpc('ping')['pid']
            assert len(invoke(page, 'list_active_terminals')) == 2
            assert 'not-on-disk' not in (ROOT/'preferences.json').read_text()
            STATE.parent.mkdir(exist_ok=True)
            STATE.write_text(json.dumps(dict(hostPid=rpc('ping')['pid'], sessions=[[s['sessionId'],s['processId']] for s in sessions], outputLength=len(text()))))
            page.screenshot(path='artifacts/terminal-host-before.png')
            print('Live CMD state, running PowerShell task, authenticated IPC and update preflight verified')
        elif phase == 'restarted':
            saved = json.loads(STATE.read_text())
            wait_for(lambda: page.get_by_role('button', name='New session', exact=True).is_enabled(), 'Fresh host not ready')
            status = invoke(page, 'prepare_interface_update')
            assert status['pid'] != saved['hostPid']
            current_version = json.loads((Path(__file__).resolve().parents[1] / 'package.json').read_text(encoding='utf-8'))['version']
            assert status['hostVersion'] == current_version
            assert not snapshots()
            invoke(page, 'shutdown_application')
            print(f'Full stop followed by reopen started the current v{current_version} host without stale-PID blockage')
        else:
            saved = json.loads(STATE.read_text())
            wait_for(lambda: page.get_by_role('button', name='New session', exact=True).is_enabled(), 'Reopened UI not ready')
            assert invoke(page, 'prepare_interface_update')['pid'] == saved['hostPid']
            assert [[s['sessionId'],s['processId']] for s in invoke(page, 'list_active_terminals')] == saved['sessions']
            assert page.get_by_role('button', name='Host CMD', exact=False).count() > 0
            assert 'HOST_TICK_' in text() and 'STATE_preserved-in-memory' in text()
            # Repeated reconnects exceed the connection cap if idle subscriber cleanup leaks.
            for _ in range(6):
                page.reload(); page.wait_for_load_state('networkidle')
                wait_for(lambda: page.get_by_role('button', name='New session', exact=True).is_enabled(), 'Repeated reconnect failed')
                time.sleep(1.2)
            assert len(snapshots()) == 2
            invoke(page, 'resize_terminal', {'sessionId':saved['sessions'][0][0], 'cols':120, 'rows':40})
            send(saved['sessions'][0][0], 'echo RECONNECTED_INPUT_OK')
            wait_for(lambda: 'RECONNECTED_INPUT_OK' in text(), 'Reconnected input failed')
            page.screenshot(path='artifacts/terminal-host-after.png')
            # Stop just this test's sessions and host through the normal explicit shutdown path.
            if not arguments.keep_running:
                invoke(page, 'shutdown_application')
            print('Real reopened Tauri UI attached to original PIDs; output, input, resize and repeated reconnect passed' + ('; explicit shutdown passed' if not arguments.keep_running else ''))
        browser.close()


if __name__ == '__main__': main()
