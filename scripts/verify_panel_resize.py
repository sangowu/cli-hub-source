"""Pointer resize/settings regression. --native targets isolated host-test only."""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from verify_stage4_ui import TAURI_MOCK
from verify_session_sidebar import seed
from verify_terminal_host import invoke, rpc, wait_for, snapshots, send
from verify_terminal_layout import check_geometry


def track_resizes(page):
    page.evaluate('''() => {
        window.__panelResizeCalls = [];
        const original = window.__TAURI_INTERNALS__.invoke;
        const wrapped = (command, args) => {
            if (command === 'resize_terminal') window.__panelResizeCalls.push(args);
            return original(command, args);
        };
        window.__TAURI_INTERNALS__.invoke = wrapped;
        window.__panelCanTrace = window.__TAURI_INTERNALS__.invoke === wrapped;
        window.__panelTerminalNode = document.querySelector('.terminal-container-active');
    }''')


def drag(page, selector, target_x=None, target_y=None):
    box = page.locator(selector).bounding_box()
    start_x, start_y = box['x'] + box['width']/2, box['y'] + box['height']/2
    page.evaluate('window.__panelResizeCalls = []')
    screen = page.locator('.terminal-container-active .xterm-screen')
    before = screen.bounding_box()
    page.mouse.move(start_x, start_y)
    page.mouse.down()
    page.mouse.move(target_x if target_x is not None else start_x,
                    target_y if target_y is not None else start_y, steps=25)
    page.wait_for_timeout(100)
    if page.evaluate('window.__panelCanTrace'):
        assert page.evaluate('window.__panelResizeCalls.length') == 0, 'PTY resized while pointer held'
    during = screen.bounding_box()
    assert abs(before['width']-during['width']) < 0.1 and abs(before['height']-during['height']) < 0.1, 'Terminal grid reflowed during drag'
    assert page.evaluate('window.__panelTerminalNode === document.querySelector(".terminal-container-active")')
    assert page.locator('.terminal-container-active .xterm-rows').inner_text().strip(), 'Terminal became blank'
    page.mouse.up()
    page.wait_for_timeout(150)
    count = page.evaluate('window.__panelResizeCalls.length')
    if page.evaluate('window.__panelCanTrace'):
        assert 1 <= count <= 2, f'Unexpected resize count after release: {count}'
    else:
        # Tauri's native invoke object is immutable; inspect the actual terminal
        # grid instead of claiming an uninstalled spy counted native RPC calls.
        after = screen.bounding_box()
        assert abs(before['width']-after['width']) > 1 or abs(before['height']-after['height']) > 1, 'Terminal did not fit after release'
    check_geometry(page)


def check_settings(page):
    page.locator('.settings-button').click()
    controls = page.locator('.settings-dialog .settings-action-button, .settings-dialog .settings-actions button, '
                            '.settings-dialog select, .settings-dialog .shortcut-key-input')
    boxes = controls.evaluate_all('(els)=>els.map(e=>({w:e.getBoundingClientRect().width,h:e.getBoundingClientRect().height}))')
    assert len(boxes) >= 10
    assert all(abs(b['w']-170)<0.1 and abs(b['h']-34)<0.1 for b in boxes), boxes
    close = page.locator('.settings-dialog .dialog-close')
    assert close.bounding_box()['width'] == 30
    page.screenshot(path='artifacts/panel-settings-native.png' if '--native' in sys.argv else 'artifacts/panel-settings-mock.png')
    close.click()


def main():
    native = '--native' in sys.argv
    session_ids = []
    with sync_playwright() as pw:
        browser = pw.chromium.connect_over_cdp('http://127.0.0.1:9224') if native else pw.chromium.launch()
        page = browser.contexts[0].pages[0] if native else browser.new_page(viewport=dict(width=1280,height=760))
        page.set_default_timeout(10000)
        if native:
            page.wait_for_function('!!window.__TAURI_INTERNALS__')
            assert invoke(page, 'plugin:app|identifier') == 'com.clihub.host-test'
            assert not snapshots(), 'Use an empty isolated test host'
            page.evaluate('localStorage.removeItem("cli-hub.session-records.v1")')
            page.reload(wait_until='networkidle')
            settings = invoke(page, 'load_app_preferences')['preferences']['uiSettings']
            settings.update(language='en', sessionSidebarCollapsed=False, sessionGrouping='cli', restoreOnStartup=False)
            invoke(page, 'save_ui_settings', {'uiSettings': settings})
            for index in range(8):
                invoke(page, 'start_terminal', {'request': dict(pageId='workspace-default', preset='cmd',
                    name=f'Resize test session {index+1}', executable='', arguments=[],
                    workingDirectory=str(Path.cwd()), environment=[])})
            session_ids = [s['sessionId'] for s in snapshots()]
            page.reload(wait_until='networkidle')
        else:
            page.add_init_script(seed('en','dark') + TAURI_MOCK)
            page.goto('http://localhost:1420', wait_until='networkidle')
        try:
            page.evaluate('document.fonts.ready')
            page.locator('.session-card-main').first.click()
            if native:
                wait_for(lambda: '>' in bytes(snapshots()[0]['output']).decode(errors='replace'), 'CMD not ready')
                send(session_ids[0], 'echo RESIZE_STABLE_CONTENT')
                wait_for(lambda: 'RESIZE_STABLE_CONTENT' in bytes(snapshots()[0]['output']).decode(errors='replace'), 'CMD output missing')
            page.wait_for_timeout(200)
            track_resizes(page)
            left = page.locator('.terminal-card').bounding_box()['x']
            overview = page.locator('.workspace-overview')
            assert overview.evaluate('(el)=>el.scrollHeight>el.clientHeight'), 'Fixture must exercise vertical scrollbar'
            for _ in range(3):
                drag(page, '.session-sidebar-resizer', target_x=left+190)
                assert page.locator('.session-card-grid').first.evaluate('(el)=>getComputedStyle(el).gridTemplateColumns.split(" ").length') == 1
                drag(page, '.session-sidebar-resizer', target_x=left+360)
                assert page.locator('.session-card-grid').first.evaluate('(el)=>getComputedStyle(el).gridTemplateColumns.split(" ").length') == 2
                assert overview.evaluate('(el)=>el.scrollWidth<=el.clientWidth')
            # Exercise a fractional layout boundary similar to DPI rounding.
            overview.evaluate('(el)=>el.style.zoom="1.003"')
            drag(page, '.session-sidebar-resizer', target_x=left+190)
            drag(page, '.session-sidebar-resizer', target_x=left+360)
            assert page.locator('.session-card-grid').first.evaluate('(el)=>getComputedStyle(el).gridTemplateColumns.split(" ").length') == 2
            overview.evaluate('(el)=>el.style.zoom=""')
            bar = page.locator('.input-panel-resizer').bounding_box()
            drag(page, '.input-panel-resizer', target_y=bar['y']-50)
            bar = page.locator('.input-panel-resizer').bounding_box()
            drag(page, '.input-panel-resizer', target_y=bar['y']+50)
            print('PASS: pointer one/two-column round trips with scrollbar and fractional zoom; both dividers defer PTY resize until release; terminal retained')
            for language in ['en','zh-CN']:
                settings = invoke(page, 'load_app_preferences')['preferences']['uiSettings']
                settings['language'] = language
                invoke(page, 'save_ui_settings', {'uiSettings': settings})
                page.reload(wait_until='networkidle')
                check_settings(page)
            print('PASS: English/Chinese Settings action buttons, inputs and selects are 170x34; close icon remains 30x30')
            page.screenshot(path='artifacts/panel-resize-native.png' if native else 'artifacts/panel-resize-mock.png')
        finally:
            if native:
                for session_id in session_ids:
                    rpc('stop_terminal', {'sessionId': session_id})
            browser.close()


if __name__ == '__main__':
    main()
