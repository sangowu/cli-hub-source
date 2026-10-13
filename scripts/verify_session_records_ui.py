"""Session record persistence/update and background-settings design regression."""
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from verify_stage4_ui import TAURI_MOCK
from verify_host_update_ui import UPDATE_MOCK


def main():
    output = Path('artifacts/session-records')
    output.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as pw:
        browser = pw.chromium.launch()
        page = browser.new_page(locale='en-US')
        page.add_init_script(TAURI_MOCK + UPDATE_MOCK)
        page.goto('http://localhost:1420', wait_until='networkidle')
        for name in ['Record live', 'Record ended']:
            page.get_by_role('button', name='New session', exact=True).click()
            page.locator('#launch-preset').select_option('cmd')
            page.locator('#session-name').fill(name)
            page.get_by_role('button', name='Start ' + name, exact=True).click()
        page.evaluate("window.__TAURI_INTERNALS__.invoke('stop_terminal', {sessionId:2})")
        expect(page.locator('.session-card-exited')).to_have_count(1)
        page.evaluate("window.__emitTauriEvent('terminal-output', {sessionId:1,bytes:Array.from(new TextEncoder().encode('SECRET_OUTPUT_PROBE'))})")
        page.wait_for_function("JSON.parse(localStorage.getItem('cli-hub.session-records.v1')).records.length === 2")
        history = page.evaluate("localStorage.getItem('cli-hub.session-records.v1')")
        assert 'SECRET_OUTPUT_PROBE' not in history and 'preview' not in history and 'environment' not in history
        page.get_by_role('button', name='Settings', exact=True).click()
        page.get_by_role('button', name='Check for updates', exact=True).click()
        page.get_by_role('button', name='Install 0.1.9', exact=True).click()
        page.wait_for_function("window.__hostUpdateCalls.includes('plugin:process|restart')")
        page.reload(wait_until='networkidle')
        expect(page.locator('.session-card')).to_have_count(2)
        expect(page.locator('.session-card-running')).to_have_count(1)
        expect(page.locator('.session-card-exited')).to_have_count(1)
        expect(page.locator('.session-card-exited .session-preview')).to_contain_text('Saved session record')
        page.get_by_role('button', name='Collapse session sidebar', exact=True).click()
        ended_icon = page.locator('.session-icon-button').filter(has=page.locator('.session-dot-exited'))
        expect(ended_icon.locator('small')).to_have_text('2')
        expect(ended_icon).to_have_attribute('aria-label', 'Record ended · Session 2 · Exited')
        ended_icon.click()
        expect(page.locator('.terminal-toolbar')).to_contain_text('Record ended · Session 2')
        page.get_by_role('button', name='Expand session sidebar', exact=True).click()
        page.locator('.session-card-exited .session-card-remove').click()
        page.wait_for_function("JSON.parse(localStorage.getItem('cli-hub.session-records.v1')).records.length === 1")
        page.reload(wait_until='networkidle')
        expect(page.locator('.session-card')).to_have_count(1)
        page.evaluate("localStorage.setItem('cli-hub.session-records.v1', 'INVALID_HISTORY')")
        page.reload(wait_until='networkidle')
        expect(page.locator('.header-error')).to_contain_text('Unable to load session records')
        assert page.evaluate("localStorage.getItem('cli-hub.session-records.v1')") == 'INVALID_HISTORY'
        page.get_by_role('button', name='Settings', exact=True).click()
        page.get_by_role('button', name='Check for updates', exact=True).click()
        page.get_by_role('button', name='Install 0.1.9', exact=True).click()
        expect(page.locator('.settings-body .error-message')).to_contain_text('Session records are unavailable')
        assert 'plugin:updater|install' not in page.evaluate('window.__hostUpdateCalls')
        page.close()
        print('PASS: update/reload keeps live and ended cards; metadata-only storage; removal; corrupt-history protection')
        page = browser.new_page(locale='en-US')
        page.add_init_script(TAURI_MOCK + UPDATE_MOCK + """
          const recordInvoke = window.__TAURI_INTERNALS__.invoke;
          window.__TAURI_INTERNALS__.invoke = async (command, args) => {
            if (command === 'plugin:updater|download')
              await new Promise(resolve => window.__releaseRecordDownload = resolve);
            return recordInvoke(command, args);
          };
        """)
        page.goto('http://localhost:1420', wait_until='networkidle')
        page.get_by_role('button', name='New session', exact=True).click()
        page.locator('#launch-preset').select_option('cmd')
        page.locator('#session-name').fill('Ends during download')
        page.get_by_role('button', name='Start Ends during download', exact=True).click()
        page.get_by_role('button', name='Settings', exact=True).click()
        page.get_by_role('button', name='Check for updates', exact=True).click()
        page.get_by_role('button', name='Install 0.1.9', exact=True).click()
        page.wait_for_function('typeof window.__releaseRecordDownload === "function"')
        page.evaluate("window.__TAURI_INTERNALS__.invoke('stop_terminal', {sessionId:1})")
        page.wait_for_function("JSON.parse(localStorage.getItem('cli-hub.session-records.v1')).records[0].status === 'exited'")
        page.evaluate('window.__releaseRecordDownload()')
        page.wait_for_function("window.__hostUpdateCalls.includes('plugin:process|restart')")
        latest = page.evaluate("JSON.parse(localStorage.getItem('cli-hub.session-records.v1')).records[0]")
        assert latest['status'] == 'exited' and latest['exitCode'] == 0
        page.close()
        print('PASS: update preflight saves the latest status after a CLI exits during download')
        for language in ['en', 'zh-CN']:
            for mode in ['light', 'dark']:
                for width, height in [(1280, 900), (720, 600)]:
                    page = browser.new_page(viewport={'width': width, 'height': height}, locale='en-US')
                    page.add_init_script(TAURI_MOCK)
                    page.goto('http://localhost:1420', wait_until='networkidle')
                    page.get_by_role('button', name='Settings', exact=True).click()
                    page.locator('#interface-language').select_option(language)
                    page.locator('#interface-color-mode').select_option(mode)
                    card = page.locator('.background-settings')
                    title = card.locator('.settings-section-title')
                    description = card.locator('.settings-section-description')
                    button = card.locator('.settings-action-button')
                    style = '(el) => { const s=getComputedStyle(el); return [s.fontSize,s.fontWeight,s.height,s.borderRadius,s.backgroundColor,s.color] }'
                    assert title.evaluate(style)[:2] == ['12px', '700']
                    assert description.evaluate(style)[0] == '10px'
                    assert description.evaluate('(el)=>getComputedStyle(el).lineHeight') == '14px'
                    assert button.evaluate(style)[:4] == ['12px', '700', '34px', '8px']
                    assert card.evaluate('(el)=>getComputedStyle(el).padding') == '11px 12px'
                    assert card.locator('.settings-section-actions').evaluate('(el)=>getComputedStyle(el).justifyContent') == 'flex-end'
                    assert page.locator('.settings-body').evaluate('(el)=>el.scrollWidth<=el.clientWidth')
                    card.scroll_into_view_if_needed()
                    page.screenshot(path=str(output/f'background-{language}-{mode}-{width}.png'))
                    button.focus()
                    page.keyboard.press('Tab')
                    page.keyboard.press('Shift+Tab')
                    assert button.evaluate('(el)=>getComputedStyle(el).outlineWidth') == '2px'
                    page.close()
                    print(f'PASS: background settings {language} {mode} {width}x{height}')
        browser.close()


if __name__ == '__main__':
    main()
