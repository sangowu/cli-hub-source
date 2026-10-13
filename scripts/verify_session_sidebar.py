"""Mocked regression for the collapsible session sidebar (run with Vite on port 1420)."""
from pathlib import Path
import json
from playwright.sync_api import sync_playwright, expect
from verify_stage4_ui import TAURI_MOCK

PRESETS = ['powerShell', 'cmd', 'codex', 'claude', 'custom', 'windowsPowerShell', 'cmd', 'powerShell']
SESSIONS = []
for index, preset in enumerate(PRESETS + ['cmd'], 1):
    directory = 'D:\\sidebar-test\\' + ('a' if index % 2 else 'b')
    output = list(f'SIDEBAR_OUTPUT_{index}\r\n'.encode())
    SESSIONS.append(dict(sessionId=index, processId=1000+index,
        pageId='workspace-default' if index <= 8 else 'sidebar-second',
        preset=preset, name=f'Sidebar {index}', workingDirectory=directory,
        workingDirectoryKey=directory.lower(), output=output, outputStartOffset=0,
        nextOutputOffset=len(output), outputTruncated=False))


def seed(language, mode):
    preferences = dict(version=1, launchProfiles=[], recentDirectories=[],
        uiSettings=dict(language=language, colorMode=mode, startMinimized=False, sessionGrouping='directory'),
        workspacePages=[dict(id='workspace-default',name='Sidebar A'),dict(id='sidebar-second',name='Sidebar B')],
        activeWorkspaceId='workspace-default')
    return f'''if (!sessionStorage.getItem('cli-hub-mock-sessions')) {{
        sessionStorage.setItem('cli-hub-mock-sessions', {json.dumps(json.dumps(SESSIONS))});
        sessionStorage.setItem('cli-hub-mock-preferences', {json.dumps(json.dumps(preferences))});
    }}'''


def main():
    output = Path('artifacts/session-sidebar'); output.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as pw:
        browser = pw.chromium.launch()
        for language in ['en', 'zh-CN']:
            for mode in ['light', 'dark']:
                for width, height in [(1280, 900), (720, 600)]:
                    page = browser.new_page(viewport=dict(width=width,height=height))
                    errors = []
                    page.on('pageerror',lambda error: errors.append(str(error)))
                    page.add_init_script(seed(language,mode) + TAURI_MOCK)
                    page.goto('http://localhost:1420',wait_until='networkidle')
                    toggle = page.locator('.session-sidebar-toggle')
                    expect(toggle).to_be_enabled()
                    expect(toggle).to_have_attribute('aria-expanded','true')
                    brand = page.locator('.app-brand')
                    expect(brand).to_have_attribute('aria-label', 'CLI Hub')
                    assert brand.evaluate('(el)=>getComputedStyle(el).fontSize') == '24px'
                    assert brand.locator('img').evaluate('(el)=>el.complete && el.naturalWidth>0')
                    assert brand.locator('img').bounding_box()['width'] == 32
                    brand_box = brand.bounding_box()
                    actions_box = page.locator('.header-actions').bounding_box()
                    assert abs(brand_box['y'] + brand_box['height']/2 - actions_box['y'] - actions_box['height']/2) <= 1
                    assert brand_box['x'] + brand_box['width'] < actions_box['x']
                    expect(page.locator('.terminal-toolbar')).to_contain_text(
                        '双击CLI界面可进入全屏模式' if language == 'zh-CN'
                        else 'Double-click the CLI view to enter fullscreen mode')
                    page.locator('.session-sidebar-resizer').press('Home')
                    page.locator('.session-sidebar-resizer').press('ArrowRight')
                    expanded_width = page.locator('.workspace-sidebar').bounding_box()['width']
                    overview = page.locator('.workspace-overview')
                    overview.evaluate('(el)=>el.scrollTop=80')
                    page.wait_for_timeout(100)
                    scroll_top = overview.evaluate('(el)=>el.scrollTop')
                    assert scroll_top > 0
                    page.evaluate('window.__sidebarTerminalNodes=[...document.querySelectorAll(".terminal-container")]')
                    toggle.click()
                    expect(toggle).to_have_attribute('aria-expanded','false')
                    assert page.locator('.workspace-sidebar').bounding_box()['width'] == 52
                    expect(page.locator('.session-sidebar-resizer')).to_be_hidden()
                    expect(page.locator('.workspace-sidebar-footer')).to_be_hidden()
                    icons = page.locator('.session-icon-button')
                    expect(icons).to_have_count(8)
                    assert 'D:\\sidebar-test' in icons.first.get_attribute('title')
                    assert len(set(icons.all_text_contents())) == 8
                    assert page.evaluate('window.__sidebarTerminalNodes.every((el,i)=>el===document.querySelectorAll(".terminal-container")[i])')
                    assert page.evaluate('window.__mockPreferences.uiSettings.sessionSidebarCollapsed')
                    icon = icons.filter(has=page.locator('small',has_text='3'))
                    icon.click()
                    expect(icon).to_have_attribute('aria-pressed','true')
                    assert 'Sidebar 3' in page.locator('.terminal-toolbar').inner_text()
                    expect(toggle).to_have_attribute('aria-expanded','false')
                    page.evaluate('window.__emitTauriEvent("terminal-exit", {sessionId:7,exitCode:0,error:null})')
                    page.evaluate('window.__emitTauriEvent("terminal-exit", {sessionId:8,exitCode:null,error:"Test launch failure"})')
                    expect(page.locator('.session-icon-button .session-dot-exited')).to_have_count(1)
                    expect(page.locator('.session-icon-button .session-dot-failed')).to_have_count(1)
                    icons.last.scroll_into_view_if_needed()
                    page.screenshot(path=str(output/f'{language}-{mode}-{width}-collapsed.png'))
                    assert page.evaluate('document.documentElement.scrollWidth<=innerWidth')
                    assert page.locator('.session-icon-list').evaluate('(el)=>el.scrollWidth<=el.clientWidth')
                    toggle.click()
                    expect(toggle).to_have_attribute('aria-expanded','true')
                    assert abs(page.locator('.workspace-sidebar').bounding_box()['width']-expanded_width)<1
                    assert abs(overview.evaluate('(el)=>el.scrollTop')-scroll_top)<1
                    expect(page.locator('.session-card-selected .session-card-main')).to_have_attribute('aria-pressed','true')
                    toggle.click()
                    expect(toggle).to_have_attribute('aria-expanded','false')
                    page.get_by_role('tab',name='Sidebar B',exact=False).click()
                    expect(icons).to_have_count(1)
                    expect(toggle).to_have_attribute('aria-expanded','false')
                    page.get_by_role('tab',name='Sidebar A',exact=False).click()
                    expect(icons).to_have_count(8)
                    page.reload(wait_until='networkidle')
                    expect(toggle).to_have_attribute('aria-expanded','false')
                    expect(icons).to_have_count(8)
                    expect(page.locator('.session-icon-button .session-dot-exited')).to_have_count(1)
                    expect(page.locator('.session-icon-button .session-dot-failed')).to_have_count(1)
                    # A failed preference write must leave the current layout intact and report the error.
                    page.evaluate('''() => { window.__sidebarOriginalInvoke=window.__TAURI_INTERNALS__.invoke;
                        window.__TAURI_INTERNALS__.invoke=(command,args)=>command==='save_ui_settings'
                            ? Promise.reject(new Error('sidebar test save failure'))
                            : window.__sidebarOriginalInvoke(command,args); }''')
                    toggle.click()
                    expect(page.locator('.header-error')).to_contain_text('sidebar test save failure')
                    expect(toggle).to_have_attribute('aria-expanded','false')
                    expect(toggle).to_be_enabled()
                    assert not errors,errors
                    print(f'PASS {language} {mode} {width}x{height}')
                    page.close()
        browser.close()


if __name__ == '__main__':
    main()
