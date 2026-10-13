"""Launch template UI/save contract; --native smoke uses isolated host-test only."""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from verify_stage4_ui import TAURI_MOCK

TEMPLATES = [('pi', 'Pi'), ('hermes', 'Hermes Agent'), ('opencode', 'OpenCode')]


def invoke(page, command, args=None):
    return page.evaluate('([c,a])=>window.__TAURI_INTERNALS__.invoke(c,a)', [command, args or {}])


def main():
    native = '--native' in sys.argv
    with sync_playwright() as pw:
        browser = pw.chromium.connect_over_cdp('http://127.0.0.1:9224') if native else pw.chromium.launch()
        page = browser.contexts[0].pages[0] if native else browser.new_page(viewport=dict(width=1280, height=900), locale='en-US')
        page.set_default_timeout(5000)
        if native:
            assert invoke(page, 'plugin:app|identifier') == 'com.clihub.host-test'
            assert not invoke(page, 'list_active_terminals'), 'Use an empty isolated test instance'
            settings = invoke(page, 'load_app_preferences')['preferences']['uiSettings']
            settings.update(language='en', restoreOnStartup=False, sessionSidebarCollapsed=False)
            invoke(page, 'save_ui_settings', {'uiSettings': settings})
            page.reload(wait_until='networkidle')
        else:
            page.add_init_script(TAURI_MOCK)
            page.goto('http://localhost:1420', wait_until='networkidle')
        try:
            for command, name in TEMPLATES:
                page.get_by_role('button', name='New session', exact=True).click()
                page.locator('#saved-profile').select_option('')
                page.locator('#launch-preset').select_option(command)
                expect(page.locator(f'#launch-preset option[value="{command}"]')).to_have_text(name)
                expect(page.locator('#session-name')).to_have_value(name)
                expect(page.locator('#executable')).to_have_value(command)
                expect(page.locator('#executable')).to_be_enabled()
                expect(page.locator('#arguments')).to_have_value('')
                page.locator('#arguments').fill('--version')
                page.locator('#working-directory').fill(str(Path.cwd()))
                if not native:
                    page.get_by_role('button', name='Save profile', exact=True).click()
                    page.wait_for_function('(name)=>window.__mockPreferences.launchProfiles.some(p=>p.label===name)', arg=name)
                    profile = page.evaluate('(name)=>window.__mockPreferences.launchProfiles.find(p=>p.label===name)', name)
                    assert profile['preset'] == 'custom' and profile['executable'] == command
                    assert profile['argumentsText'] == '--version'
                    page.reload(wait_until='networkidle')
                    page.get_by_role('button', name='New session', exact=True).click()
                    page.locator('#saved-profile').select_option(label=name)
                    expect(page.locator('#launch-preset')).to_have_value(command)
                    expect(page.locator('#arguments')).to_have_value('--version')
                    page.get_by_role('button', name='Start ' + name, exact=True).click()
                    assert invoke(page, 'list_active_terminals')[-1]['preset'] == 'custom'
                else:
                    # Pi/Hermes need not be installed: verify all templates with
                    # a local .cmd fixture, preserving the structured argument.
                    fixture = Path('artifacts/agent-template-smoke.cmd').resolve()
                    fixture.write_text(f'@echo off\necho TEMPLATE_SMOKE_{command}_%~1\n', encoding='ascii')
                    page.locator('#executable').fill(str(fixture))
                    expect(page.locator('#launch-preset')).to_have_value('custom')
                    page.get_by_role('button', name='Start ' + name, exact=True).click()
                    expect(page.locator('.session-preview', has_text=f'TEMPLATE_SMOKE_{command}_--version')).to_be_visible()
                print(f'PASS: {name} ' + ('native custom-script launch' if native else 'template, profile persistence and compatible launch'))
            if not native:
                page.get_by_role('button', name='New session', exact=True).click()
                page.locator('#launch-preset').select_option('pi')
                page.locator('#launch-preset').select_option('custom')
                expect(page.locator('#launch-preset')).to_have_value('custom')
                expect(page.locator('#executable')).to_have_value('')
                expect(page.locator('#arguments')).to_have_value('')
                page.locator('#launch-preset').select_option('claude')
                expect(page.locator('#executable')).to_be_disabled()
                print('PASS: template/custom/built-in transitions')
        finally:
            if native:
                invoke(page, 'shutdown_application')
            else:
                browser.close()


if __name__ == '__main__':
    main()
