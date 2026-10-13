"""Reply previews wait for output to settle; --native uses isolated host-test only."""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from verify_stage4_ui import TAURI_MOCK
from verify_session_sidebar import seed
from verify_terminal_host import invoke, snapshots, rpc


def main():
    native = '--native' in sys.argv
    session_id = None
    with sync_playwright() as pw:
        browser = pw.chromium.connect_over_cdp('http://127.0.0.1:9224') if native else pw.chromium.launch()
        page = browser.contexts[0].pages[0] if native else browser.new_page(viewport=dict(width=1280, height=760))
        page.set_default_timeout(15000)
        try:
            if native:
                page.wait_for_function('!!window.__TAURI_INTERNALS__')
                assert invoke(page, 'plugin:app|identifier') == 'com.clihub.host-test'
                assert not snapshots(), 'Use an empty isolated host'
                page.evaluate('localStorage.removeItem("cli-hub.session-records.v1")')
                settings = invoke(page, 'load_app_preferences')['preferences']['uiSettings']
                settings.update(language='en', sessionSidebarCollapsed=False, restoreOnStartup=False)
                invoke(page, 'save_ui_settings', {'uiSettings': settings})
                page.reload(wait_until='networkidle')
                fixture = Path('artifacts/preview-animation.ps1').resolve()
                fixture.write_text('''$esc = [char]27
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
[Console]::Write("$esc[2J$esc[H● Stable answer`r`n")
Start-Sleep -Seconds 3
[Console]::Write("$esc[2J$esc[H")
Start-Sleep -Seconds 1
for ($i = 0; $i -lt 75; $i++) {
  [Console]::Write("$esc[2J$esc[H⠋ Thinking…`r`n$($i)%`r`n")
  Start-Sleep -Milliseconds 40
}
for ($i = 0; $i -lt 90; $i++) {
  [Console]::Write("$esc[2J$esc[H● New answer $i`r`n")
  Start-Sleep -Milliseconds 40
}
[Console]::Write("$esc[2J$esc[H● New answer complete`r`n")
Start-Sleep -Seconds 30
''', encoding='utf-8-sig')
                result = invoke(page, 'start_terminal', {'request': dict(pageId='workspace-default', preset='custom',
                    name='Preview stability', executable='C:/Program Files/PowerShell/7/pwsh.exe',
                    arguments=['-NoLogo', '-NoProfile', '-File', str(fixture)], workingDirectory=str(Path.cwd()), environment=[])})
                session_id = result['sessionId']
                page.reload(wait_until='networkidle')
                card = page.locator('.session-card').filter(has_text='Preview stability')
            else:
                page.add_init_script(seed('en', 'dark') + TAURI_MOCK)
                page.goto('http://localhost:1420', wait_until='networkidle')
                session_id = 1
                card = page.locator('.session-card').filter(has_text='Sidebar 1').first
            card.locator('.session-card-main').click()
            preview = card.locator('.session-preview')
            page.evaluate('document.fonts.ready')
            if not native:
                emit(page, session_id, '● Stable answer')
                page.wait_for_timeout(1000)
                expect(preview).not_to_have_text('Stable answer')
            expect(preview).to_have_text('Stable answer')
            original_height = card.bounding_box()['height']
            preview.evaluate('''el => {
                window.__previewFrames = [];
                window.__previewObserver = new MutationObserver(() => {
                    window.__previewFrames.push({text: el.textContent, time: performance.now()});
                });
                window.__previewObserver.observe(el, {childList:true, characterData:true, subtree:true});
            }''')
            if native:
                expect(preview).to_have_text('New answer complete', timeout=15000)
            else:
                for text in ['', '⠋ Thinking…', '50%', 'temporary tool output', '❯ draft']:
                    emit(page, session_id, text)
                    page.wait_for_timeout(650)
                    expect(preview).to_have_text('Stable answer')
                    assert abs(card.bounding_box()['height'] - original_height) < 0.1
                for index in range(90):
                    emit(page, session_id, f'● New answer {index}\r\nwith a second line')
                    page.wait_for_timeout(40)
                    expect(preview).to_have_text('Stable answer')
                # Stream longer than the quiet interval: a periodic update must
                # not publish partial replies, including after 360-char clipping.
                for index in range(60):
                    emit(page, session_id, '● ' + 'x'*400 + str(index))
                    page.wait_for_timeout(40)
                    expect(preview).to_have_text('Stable answer')
                expect(page.locator('.session-status-activity')).to_have_text('Output just now')
                emit(page, session_id, '● New answer complete')
                page.wait_for_timeout(1000)
                expect(preview).to_have_text('Stable answer')
                expect(preview).to_have_text('New answer complete')
            frames = page.evaluate('window.__previewFrames')
            assert [f['text'] for f in frames] == ['New answer complete'], frames
            assert abs(card.bounding_box()['height'] - original_height) < 0.1
            assert abs(preview.bounding_box()['height'] - 31) < 0.1
            page.screenshot(path='artifacts/preview-stability-native.png' if native else 'artifacts/preview-stability-mock.png')
            print(f'PASS: {"native PTY" if native else "mock"} previous reply retained throughout generation; one complete preview after idle; card height stable')
        finally:
            if native and session_id is not None:
                rpc('stop_terminal', {'sessionId': session_id})
            browser.close()


def emit(page, session_id, text):
    page.evaluate('''([sessionId, text]) => window.__emitTauriEvent('terminal-output', {
        sessionId, bytes: [...new TextEncoder().encode('\x1b[2J\x1b[H' + text)]
    })''', [session_id, text])


if __name__ == '__main__':
    main()
