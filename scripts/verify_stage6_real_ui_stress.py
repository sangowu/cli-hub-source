from __future__ import annotations

"""Real Tauri/WebView2 release-gate test for the 24-session workspace limit.

Close other CLI Hub instances, then start the normal development app with:

    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9222'
    npm run tauri -- dev --no-watch

Run this script from a second terminal. It drives the actual Tauri IPC surface,
starts 24 real CMD/ConPTY sessions, and always attempts to stop them on exit.
"""

from collections import Counter
from pathlib import Path
import time

from playwright.sync_api import Error as PlaywrightError
from playwright.sync_api import Page, sync_playwright


CDP_URL = "http://127.0.0.1:9222"
PROJECT_DIR = str(Path(__file__).resolve().parents[1])
SCREENSHOT = Path("artifacts/stage6-real-24.png")
WORKSPACES = [
    {"id": "e2e-workspace-1", "name": "E2E Page 1"},
    {"id": "e2e-workspace-2", "name": "E2E Page 2"},
    {"id": "e2e-workspace-3", "name": "E2E Page 3"},
]


def invoke(page: Page, command: str, args: dict | None = None):
    return page.evaluate(
        """async ({ command, args }) =>
          await window.__TAURI_INTERNALS__.invoke(command, args || {})""",
        {"command": command, "args": args or {}},
    )


def expect_invoke_error(page: Page, command: str, args: dict, expected: str) -> str:
    try:
        invoke(page, command, args)
    except PlaywrightError as error:
        message = str(error)
        assert expected.lower() in message.lower(), message
        return message
    raise AssertionError(f"{command} unexpectedly succeeded")


def wait_for_active_count(page: Page, expected: int, timeout: float = 10.0) -> list[dict]:
    deadline = time.monotonic() + timeout
    snapshots: list[dict] = []
    while time.monotonic() < deadline:
        snapshots = invoke(page, "list_active_terminals")
        if len(snapshots) == expected:
            return snapshots
        time.sleep(0.05)
    raise AssertionError(
        f"expected {expected} active terminals, found {len(snapshots)}"
    )


def output_text(snapshot: dict) -> str:
    return bytes(snapshot["output"]).decode("utf-8", errors="replace")


def wait_for_isolated_markers(page: Page, expected_markers: dict[int, str]) -> None:
    deadline = time.monotonic() + 12.0
    snapshots: list[dict] = []
    while time.monotonic() < deadline:
        snapshots = invoke(page, "list_active_terminals")
        by_id = {snapshot["sessionId"]: snapshot for snapshot in snapshots}
        if all(
            marker in output_text(by_id[session_id])
            for session_id, marker in expected_markers.items()
            if session_id in by_id
        ) and len(by_id) == len(expected_markers):
            break
        time.sleep(0.05)
    else:
        missing = [
            marker
            for session_id, marker in expected_markers.items()
            if session_id not in {item["sessionId"] for item in snapshots}
            or marker
            not in output_text(
                next(item for item in snapshots if item["sessionId"] == session_id)
            )
        ]
        raise AssertionError(f"markers missing from terminal histories: {missing}")

    for snapshot in snapshots:
        own_marker = expected_markers[snapshot["sessionId"]]
        output = output_text(snapshot)
        assert own_marker in output
        for other_marker in expected_markers.values():
            if other_marker != own_marker:
                assert other_marker not in output, (
                    f"session {snapshot['sessionId']} received another session marker"
                )


def performance_metrics(context, page: Page) -> dict[str, float]:
    session = context.new_cdp_session(page)
    session.send("Performance.enable")
    metrics = session.send("Performance.getMetrics")["metrics"]
    session.detach()
    return {item["name"]: item["value"] for item in metrics}


def main() -> None:
    SCREENSHOT.parent.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as playwright:
        browser = playwright.chromium.connect_over_cdp(CDP_URL)
        context = browser.contexts[0]
        pages = [page for item in browser.contexts for page in item.pages]
        page = next(page for page in pages if page.url.startswith("http://localhost:1420"))
        page.set_default_timeout(15_000)
        page.wait_for_selector(".app-shell")

        try:
            invoke(page, "stop_all_terminals")
            wait_for_active_count(page, 0)
            invoke(
                page,
                "save_ui_settings",
                {
                    "uiSettings": {
                        "language": "en",
                        "colorMode": "dark",
                        "startMinimized": False,
                        "sessionGrouping": "directory",
                    }
                },
            )
            invoke(
                page,
                "save_workspace_pages",
                {
                    "workspacePages": WORKSPACES,
                    "activeWorkspaceId": WORKSPACES[0]["id"],
                },
            )
            page.reload(wait_until="domcontentloaded")
            page.wait_for_selector(".app-shell")
            assert page.locator(".workspace-tab-main").count() == 3

            baseline = performance_metrics(context, page)
            responses: list[dict] = []
            expected_markers: dict[int, str] = {}
            start_time = time.perf_counter()

            for workspace_index, workspace in enumerate(WORKSPACES, start=1):
                for session_index in range(1, 9):
                    marker = f"OUTPUT:__E2E_P{workspace_index}_S{session_index}__"
                    response = invoke(
                        page,
                        "start_terminal",
                        {
                            "request": {
                                "pageId": workspace["id"],
                                "preset": "cmd",
                                "name": f"E2E P{workspace_index} S{session_index}",
                                "executable": "",
                                "arguments": [
                                    "/D",
                                    "/Q",
                                    "/K",
                                    "echo OUTPUT:%CLI_HUB_E2E_MARKER%",
                                ],
                                "workingDirectory": PROJECT_DIR,
                                "environment": [
                                    {
                                        "name": "CLI_HUB_E2E_MARKER",
                                        "value": marker.removeprefix("OUTPUT:"),
                                    },
                                    {
                                        "name": "CLI_HUB_UI_MARKER",
                                        "value": "UI_INTERACTION_OK",
                                    },
                                ],
                            },
                            "cols": 100,
                            "rows": 30,
                        },
                    )
                    responses.append(response)
                    expected_markers[response["sessionId"]] = marker

                if workspace_index == 1:
                    expect_invoke_error(
                        page,
                        "start_terminal",
                        {
                            "request": {
                                "pageId": workspace["id"],
                                "preset": "cmd",
                                "name": "Rejected ninth session",
                                "executable": "",
                                "arguments": ["/D", "/Q"],
                                "workingDirectory": PROJECT_DIR,
                                "environment": [],
                            },
                            "cols": 100,
                            "rows": 30,
                        },
                        "maximum of 8",
                    )

            active = wait_for_active_count(page, 24)
            assert Counter(item["pageId"] for item in active) == Counter(
                {workspace["id"]: 8 for workspace in WORKSPACES}
            )
            expect_invoke_error(
                page,
                "start_terminal",
                {
                    "request": {
                        "pageId": "e2e-overflow-workspace",
                        "preset": "cmd",
                        "name": "Rejected twenty-fifth session",
                        "executable": "",
                        "arguments": ["/D", "/Q"],
                        "workingDirectory": PROJECT_DIR,
                        "environment": [],
                    },
                    "cols": 100,
                    "rows": 30,
                },
                "global safety limit of 24",
            )

            for response in responses:
                invoke(
                    page,
                    "resize_terminal",
                    {
                        "sessionId": response["sessionId"],
                        "cols": 90 + response["sessionId"] % 20,
                        "rows": 24 + response["sessionId"] % 8,
                    },
                )
                invoke(
                    page,
                    "write_terminal",
                    {"sessionId": response["sessionId"], "data": "\u001b[1;1R"},
                )
            wait_for_isolated_markers(page, expected_markers)
            backend_start_seconds = time.perf_counter() - start_time

            reload_started = time.perf_counter()
            page.reload(wait_until="domcontentloaded")
            page.wait_for_selector(".session-card")
            page.wait_for_function(
                "document.querySelectorAll('.session-card-running').length === 8"
            )
            reload_seconds = time.perf_counter() - reload_started
            assert "8/8 active" in page.locator(".status").inner_text()
            assert "24/24 total" in page.locator(".status").inner_text()
            assert page.locator(".terminal-container").count() == 8

            switch_seconds: list[float] = []
            for workspace in WORKSPACES:
                switched = time.perf_counter()
                page.locator(".workspace-tab-main", has_text=workspace["name"]).click()
                page.wait_for_function(
                    "document.querySelectorAll('.session-card-running').length === 8"
                )
                assert page.locator(".session-card").count() == 8
                assert page.locator(".terminal-container").count() == 8
                switch_seconds.append(time.perf_counter() - switched)

            page.locator(".session-card-main").first.click()
            composer = page.locator("#terminal-input")
            composer.fill("echo %CLI_HUB_UI_MARKER%")
            page.get_by_role("button", name="Send", exact=True).click()
            ui_deadline = time.monotonic() + 5.0
            ui_matches: list[dict] = []
            while time.monotonic() < ui_deadline:
                ui_matches = [
                    snapshot
                    for snapshot in invoke(page, "list_active_terminals")
                    if output_text(snapshot).count("UI_INTERACTION_OK") == 1
                    and "echo UI_INTERACTION_OK" not in output_text(snapshot)
                ]
                if len(ui_matches) == 1:
                    break
                time.sleep(0.05)
            if len(ui_matches) != 1:
                output_tails = {
                    snapshot["name"]: repr(output_text(snapshot)[-500:])
                    for snapshot in invoke(page, "list_active_terminals")
                }
                raise AssertionError(
                    f"Send did not execute exactly one CLI command: {output_tails}"
                )

            selected_session_id = ui_matches[0]["sessionId"]
            composer.fill("echo CWD:%CD%")
            page.get_by_role("button", name="Send", exact=True).click()
            cwd_marker = f"CWD:{PROJECT_DIR}"
            cwd_deadline = time.monotonic() + 5.0
            selected_output = ""
            while time.monotonic() < cwd_deadline:
                selected = next(
                    snapshot
                    for snapshot in invoke(page, "list_active_terminals")
                    if snapshot["sessionId"] == selected_session_id
                )
                selected_output = output_text(selected)
                if cwd_marker.lower() in selected_output.lower():
                    break
                time.sleep(0.05)
            else:
                raise AssertionError(
                    "CMD did not retain the requested working directory: "
                    f"{repr(selected_output[-500:])}"
                )

            after_24 = performance_metrics(context, page)
            page.screenshot(path=str(SCREENSHOT), full_page=True)

            page.get_by_role("button", name="Stop page", exact=True).click()
            page.get_by_role("dialog").wait_for(state="visible")
            page.get_by_role("button", name="Stop this page", exact=True).click()
            wait_for_active_count(page, 16)
            page.wait_for_function(
                "document.querySelectorAll('.session-card-running').length === 0"
            )
            page.locator(".workspace-tab-main", has_text="E2E Page 1").click()
            page.wait_for_function(
                "document.querySelectorAll('.session-card-running').length === 8"
            )

            invoke(page, "stop_all_terminals")
            wait_for_active_count(page, 0)

            print(
                {
                    "active_sessions": 24,
                    "workspace_counts": {workspace["id"]: 8 for workspace in WORKSPACES},
                    "backend_start_and_io_seconds": round(backend_start_seconds, 3),
                    "recovery_reload_seconds": round(reload_seconds, 3),
                    "tab_switch_seconds": [round(value, 3) for value in switch_seconds],
                    "js_heap_before_mb": round(
                        baseline.get("JSHeapUsedSize", 0) / 1024 / 1024, 2
                    ),
                    "js_heap_with_24_mb": round(
                        after_24.get("JSHeapUsedSize", 0) / 1024 / 1024, 2
                    ),
                    "screenshot": str(SCREENSHOT.resolve()),
                    "remaining_active_sessions": 0,
                }
            )
        finally:
            try:
                invoke(page, "stop_all_terminals")
            except Exception:
                pass
            browser.close()


if __name__ == "__main__":
    main()
