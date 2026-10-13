from __future__ import annotations

import argparse
import os
from pathlib import Path

from playwright.sync_api import Page, sync_playwright


CDP_URL = os.environ.get("CLI_HUB_RELEASE_CDP_URL", "http://127.0.0.1:9230")
MARKER_ID = "release-acceptance-v0-1-6"
SCREENSHOT = Path("artifacts/installed-release.png")


def invoke(page: Page, command: str, args: dict | None = None):
    return page.evaluate(
        """async ({ command, args }) =>
          await window.__TAURI_INTERNALS__.invoke(command, args || {})""",
        {"command": command, "args": args or {}},
    )


def assert_release_copy(page: Page) -> tuple[str, str]:
    page.wait_for_selector(".app-shell")
    eyebrow = page.locator(".app-header .eyebrow").inner_text().strip()
    subtitle = page.locator(".app-header .subtitle").inner_text().strip()
    expected = {
        (
            "DESKTOP CLI WORKSPACE",
            "Launch, monitor, and interact with multiple CLI sessions in one place.",
        ),
        ("桌面 CLI 工作台", "在一个工作台中启动、监控并交互多个 CLI 会话。"),
    }
    assert (eyebrow, subtitle) in expected, (eyebrow, subtitle)
    return eyebrow, subtitle


def acceptance_profile() -> dict[str, str]:
    return {
        "id": MARKER_ID,
        "label": "Release Acceptance",
        "preset": "cmd",
        "sessionName": "Release Acceptance",
        "executable": "",
        "argumentsText": "",
        "workingDirectory": "",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "mode",
        choices=("prepare", "verify", "cleanup", "autostart", "hide", "visible"),
    )
    args = parser.parse_args()
    SCREENSHOT.parent.mkdir(parents=True, exist_ok=True)

    with sync_playwright() as playwright:
        browser = playwright.chromium.connect_over_cdp(CDP_URL)
        page = browser.contexts[0].pages[0]
        page.set_default_timeout(15_000)
        eyebrow, subtitle = assert_release_copy(page)

        loaded = invoke(page, "load_app_preferences")
        profiles = loaded["preferences"]["launchProfiles"]
        marker_profiles = [profile for profile in profiles if profile["id"] == MARKER_ID]

        if args.mode == "prepare":
            next_profiles = [profile for profile in profiles if profile["id"] != MARKER_ID]
            next_profiles.append(acceptance_profile())
            invoke(page, "save_launch_profiles", {"launchProfiles": next_profiles})
            reloaded = invoke(page, "load_app_preferences")
            assert any(
                profile["id"] == MARKER_ID
                for profile in reloaded["preferences"]["launchProfiles"]
            )
        elif args.mode == "verify":
            assert len(marker_profiles) == 1, "release acceptance profile was not preserved"
        elif args.mode == "cleanup":
            invoke(
                page,
                "save_launch_profiles",
                {
                    "launchProfiles": [
                        profile for profile in profiles if profile["id"] != MARKER_ID
                    ]
                },
            )
            reloaded = invoke(page, "load_app_preferences")
            assert not any(
                profile["id"] == MARKER_ID
                for profile in reloaded["preferences"]["launchProfiles"]
            )
        elif args.mode == "autostart":
            initially_enabled = invoke(page, "plugin:autostart|is_enabled")
            toggle_command = (
                "plugin:autostart|disable"
                if initially_enabled
                else "plugin:autostart|enable"
            )
            restore_command = (
                "plugin:autostart|enable"
                if initially_enabled
                else "plugin:autostart|disable"
            )
            invoke(page, toggle_command)
            assert invoke(page, "plugin:autostart|is_enabled") is not initially_enabled
            invoke(page, restore_command)
            assert invoke(page, "plugin:autostart|is_enabled") is initially_enabled
        elif args.mode == "hide":
            assert invoke(page, "plugin:window|is_visible", {"label": "main"}) is True
            invoke(page, "plugin:window|hide", {"label": "main"})
            assert invoke(page, "plugin:window|is_visible", {"label": "main"}) is False
        else:
            assert invoke(page, "plugin:window|is_visible", {"label": "main"}) is True

        final_preferences = invoke(page, "load_app_preferences")["preferences"]
        acceptance_profile_present = any(
            profile["id"] == MARKER_ID
            for profile in final_preferences["launchProfiles"]
        )
        page.screenshot(path=str(SCREENSHOT), full_page=True)
        print(
            {
                "mode": args.mode,
                "eyebrow": eyebrow,
                "subtitle": subtitle,
                "existing_profile_count": len(profiles),
                "acceptance_profile_present": acceptance_profile_present,
                "screenshot": str(SCREENSHOT.resolve()),
            }
        )
        browser.close()


if __name__ == "__main__":
    main()
