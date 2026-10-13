"""Packaged frontend CSP and authenticated terminal smoke; isolated host-test only.

Build with tauri build --debug --no-bundle --config scripts/host-test.conf.json,
then launch that executable with CDP port 9224. Never target production sessions.
"""
from pathlib import Path
from playwright.sync_api import sync_playwright
from verify_terminal_host import rpc, invoke, wait_for, snapshots, send


def main():
    with sync_playwright() as pw:
        browser = pw.chromium.connect_over_cdp('http://127.0.0.1:9224')
        page = browser.contexts[0].pages[0]
        assert invoke(page, 'plugin:app|identifier') == 'com.clihub.host-test'
        assert page.url.startswith(('http://tauri.localhost', 'https://tauri.localhost')), 'Use bundled frontend, not Vite'
        assert not snapshots(), 'Use an empty isolated test host'
        page.reload(wait_until='networkidle')
        page.wait_for_function('document.fonts.status === "loaded"')
        assert page.locator('.app-shell').count() == 1
        result = page.evaluate('''async () => {
            const violations = [];
            const policies = [];
            const handler = e => { violations.push(e.effectiveDirective); policies.push(e.originalPolicy); };
            document.addEventListener('securitypolicyviolation', handler);
            const script = document.createElement('script');
            script.textContent = 'window.__cliHubInlineProbe = true';
            document.head.appendChild(script);
            let blocked = false;
            try { await fetch('https://example.com/cli-hub-csp-probe'); } catch { blocked = true; }
            await new Promise(resolve => setTimeout(resolve, 100));
            script.remove();
            document.removeEventListener('securitypolicyviolation', handler);
            return {inlineRan: !!window.__cliHubInlineProbe, blocked, violations, policies};
        }''')
        assert not result['inlineRan'] and result['blocked'], result
        assert 'script-src-elem' in result['violations'] and 'connect-src' in result['violations'], result
        policy = result['policies'][0]
        assert "'unsafe-eval'" not in policy
        script_policy = next(part for part in policy.split(';') if part.strip().startswith('script-src'))
        assert "'unsafe-inline'" not in script_policy
        print('PASS: bundled CSP blocks unapproved inline script and remote fetch; assets/fonts/UI load')
        for kwargs in [dict(token='wrong'), dict(protocol=2)]:
            try:
                rpc('ping', **kwargs)
            except RuntimeError as error:
                assert 'rejected' in str(error)
            else:
                raise AssertionError('Invalid IPC credentials accepted')
        print('PASS: invalid host token and protocol rejected')
        try:
            invoke(page, 'start_terminal', {'request': dict(pageId='workspace-default', preset='cmd',
                name='Security smoke', executable='', arguments=[], workingDirectory=str(Path.cwd()),
                environment=[dict(name='CLI_HUB_SECURITY_PROBE', value='ephemeral-test-only')])})
            page.reload(wait_until='networkidle')
            wait_for(lambda: snapshots() and '>' in bytes(snapshots()[0]['output']).decode(errors='replace'),
                     'CMD cursor negotiation failed')
            session = snapshots()[0]
            send(session['sessionId'], 'echo SECURITY_SMOKE_OK')
            wait_for(lambda: 'SECURITY_SMOKE_OK' in bytes(snapshots()[0]['output']).decode(errors='replace'),
                     'Terminal input/output failed')
            invoke(page, 'resize_terminal', {'sessionId': session['sessionId'], 'cols': 90, 'rows': 25})
            settings = invoke(page, 'load_app_preferences')['preferences']['uiSettings']
            invoke(page, 'save_ui_settings', {'uiSettings': settings})
            print('PASS: real CMD launch, reconnect, input/output, resize and preference save under CSP')
            page.screenshot(path='artifacts/security-native.png')
        finally:
            rpc('stop_all_terminals')
        browser.close()


if __name__ == '__main__':
    main()
