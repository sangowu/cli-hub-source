"""Mock UI regression; --native uses only the isolated host-test WebView on 9224."""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from verify_session_sidebar import seed
from verify_stage4_ui import TAURI_MOCK, emit_output

ARTIFACTS = Path('artifacts/terminal-layout')


def invoke(page, command, args=None):
    return page.evaluate('([c,a])=>window.__TAURI_INTERNALS__.invoke(c,a)', [command, args or {}])


def check_geometry(page):
    page.wait_for_function('''() => {
        const container=document.querySelector('.terminal-container-active');
        const screen=container?.querySelector('.xterm-screen');
        if (!screen) return false;
        const c=container.getBoundingClientRect(), s=screen.getBoundingClientRect();
        return s.height>0 && s.bottom<=c.bottom-9 && s.right<=c.right-11;
    }''')
    assert page.evaluate('document.documentElement.scrollWidth<=innerWidth')
    backgrounds = page.locator('.terminal-container-active').evaluate('''el => {
        const color = selector => getComputedStyle(el.querySelector(selector)).backgroundColor;
        return [color('.xterm'), color('.xterm-viewport'), color('.xterm-scrollable-element')];
    }''')
    assert backgrounds == ['rgb(16, 20, 24)'] * 3, backgrounds


def check_sidebar(page):
    resizer = page.locator('.session-sidebar-resizer')
    for key, expected_width, columns in [('Home', 190, 1), ('End', 360, 2)]:
        resizer.press(key)
        page.wait_for_timeout(100)
        actual = page.locator('.workspace-sidebar').bounding_box()['width']
        assert abs(actual - expected_width) <= 1, (actual, expected_width)
        assert page.locator('.session-card-grid').first.evaluate(
            '(el)=>getComputedStyle(el).gridTemplateColumns.split(" ").length') == columns
        check_geometry(page)


def main():
    ARTIFACTS.mkdir(parents=True, exist_ok=True)
    native = '--native' in sys.argv
    with sync_playwright() as pw:
        browser = pw.chromium.connect_over_cdp('http://127.0.0.1:9224') if native else pw.chromium.launch()
        page = browser.contexts[0].pages[0] if native else browser.new_page(viewport=dict(width=1280, height=900))
        if native:
            page.wait_for_function('!!window.__TAURI_INTERNALS__')
            assert invoke(page, 'plugin:app|identifier') == 'com.clihub.host-test'
            assert not invoke(page, 'list_active_terminals'), 'Isolated instance must have no live sessions'
            settings = invoke(page, 'load_app_preferences')['preferences']['uiSettings']
            settings.update(language='en', sessionSidebarCollapsed=False, restoreOnStartup=False)
            invoke(page, 'save_ui_settings', {'uiSettings': settings})
            page.reload(wait_until='networkidle')
            page.get_by_role('button', name='New session', exact=True).click()
            page.locator('#launch-preset').select_option('powerShell')
            page.locator('#session-name').fill('Terminal layout smoke')
            page.locator('#working-directory').fill(str(Path.cwd()))
            page.get_by_role('button', name='Start Terminal layout smoke', exact=True).click()
            session_id = invoke(page, 'list_active_terminals')[0]['sessionId']
            invoke(page, 'write_terminal', {'sessionId': session_id,
                'data': '1..80 | ForEach-Object { "LAYOUT_LINE_$_" }; "LAYOUT_BOTTOM_VISIBLE"\r'})
            expect(page.locator('.session-card-selected .session-preview')).to_contain_text('LAYOUT_BOTTOM_VISIBLE')
        else:
            page.add_init_script(seed('en', 'dark') + TAURI_MOCK)
            page.goto('http://localhost:1420', wait_until='networkidle')
            emit_output(page, 1, '\x1b[2J\x1b[H● Latest assistant reply\r\n  Verified result.\r\n' +
                '─' * 30 + '\r\n❯ Next draft\r\n⏵⏵ bypass permissions on · shift+tab to cycle')
            expect(page.locator('.session-preview').first).to_have_text('Latest assistant reply\nVerified result.')
        try:
            assert page.locator('.app-header .eyebrow, .app-header .subtitle').count() == 0
            page.evaluate('document.fonts.ready')
            if native:
                # Real WebView / PTY, including resize and focus transitions.
                check_sidebar(page)
                page.locator('.terminal-container-active .xterm-screen').dblclick(position=dict(x=20, y=20))
                expect(page.locator('.terminal-stage')).to_have_class('terminal-stage terminal-stage-focus')
                check_geometry(page)
                page.locator('.terminal-focus-exit').click()
                check_geometry(page)
            else:
                check_sidebar(page)
                for width, height in [(720,600), (1892,1144), (1280,900)]:
                    page.set_viewport_size(dict(width=width, height=height))
                    check_geometry(page)
                    assert page.locator('.workspace-sidebar').bounding_box()['width'] <= 360
            page.screenshot(path=str(ARTIFACTS / ('native.png' if native else 'mock.png')))
            print('PASS: ' + ('native WebView2/PTY output and resize' if native else 'reply preview, one/two columns, minimum/large viewport') + '; terminal bottom/right visible')
        finally:
            if native:
                invoke(page, 'stop_terminal', {'sessionId': session_id})
                invoke(page, 'shutdown_application')
            else:
                browser.close()


if __name__ == '__main__':
    main()
