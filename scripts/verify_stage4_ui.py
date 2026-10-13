from pathlib import Path
import re

from playwright.sync_api import sync_playwright


APP_URL = "http://localhost:1420"
SCREENSHOT = Path("artifacts/stage4-ui.png")
LAUNCHER_SCREENSHOT = Path("artifacts/stage4-launcher.png")
SETTINGS_SCREENSHOT = Path("artifacts/stage6-settings-zh.png")
MINIMUM_SCREENSHOT = Path("artifacts/stage6-minimum-window.png")
STOP_WARNING_SCREENSHOT = Path("artifacts/stage7-stop-warning.png")
TERMINAL_FOCUS_SCREENSHOT = Path("artifacts/stage8-terminal-focus.png")

TAURI_MOCK = r"""
(() => {
  let callbackId = 1;
  const callbacks = new Map();
  const listeners = new Map();
  const storageKey = "cli-hub-mock-sessions";
  const preferencesKey = "cli-hub-mock-preferences";
  const sessions = JSON.parse(sessionStorage.getItem(storageKey) || "[]");
  let preferences = JSON.parse(sessionStorage.getItem(preferencesKey) || JSON.stringify({
    version: 1,
    launchProfiles: [],
    recentDirectories: [],
    uiSettings: {
      language: "system",
      colorMode: "system",
      startMinimized: true,
      sessionGrouping: "directory"
    },
    workspacePages: [{ id: "workspace-default", name: "Workspace 1" }],
    activeWorkspaceId: "workspace-default",
  }));
  let autostartEnabled = sessionStorage.getItem("cli-hub-mock-autostart") === "true";
  window.__terminalWrites = [];
  window.__commands = [];
  let sessionId = sessions.reduce((largest, session) => Math.max(largest, session.sessionId), 0) + 1;

  function saveSessions() {
    sessionStorage.setItem(storageKey, JSON.stringify(sessions));
  }

  function savePreferences() {
    sessionStorage.setItem(preferencesKey, JSON.stringify(preferences));
    window.__mockPreferences = preferences;
  }
  savePreferences();

  function transformCallback(callback, once = false) {
    const id = callbackId++;
    callbacks.set(id, (value) => {
      if (once) callbacks.delete(id);
      if (callback) callback(value);
    });
    return id;
  }

  function emit(event, payload) {
    if (event === "terminal-exit") {
      const index = sessions.findIndex((session) => session.sessionId === payload.sessionId);
      if (index >= 0) sessions.splice(index, 1);
      saveSessions();
    }
    if (event === "terminal-output") {
      const session = sessions.find((candidate) => candidate.sessionId === payload.sessionId);
      if (session) {
        payload.offset = session.nextOutputOffset;
        session.output.push(...payload.bytes);
        session.nextOutputOffset += payload.bytes.length;
        saveSessions();
      }
    }
    for (const id of listeners.get(event) || []) {
      callbacks.get(id)?.({ event, id, payload });
    }
  }

  window.__emitTauriEvent = emit;
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener(_event, id) {
      callbacks.delete(id);
    },
  };
  window.__TAURI_INTERNALS__ = {
    callbacks,
    transformCallback,
    unregisterCallback(id) {
      callbacks.delete(id);
    },
    runCallback(id, value) {
      callbacks.get(id)?.(value);
    },
    convertFileSrc(path) {
      return path;
    },
    async invoke(command, args = {}) {
      window.__commands.push(command);
      if (command === "plugin:event|listen") {
        const eventListeners = listeners.get(args.event) || [];
        eventListeners.push(args.handler);
        listeners.set(args.event, eventListeners);
        return args.handler;
      }
      if (command === "plugin:event|unlisten") {
        const eventListeners = listeners.get(args.event) || [];
        listeners.set(args.event, eventListeners.filter((id) => id !== args.eventId));
        callbacks.delete(args.eventId);
        return null;
      }
      if (command === "plugin:dialog|open") {
        return ["D:\\\\Demo\\\\notes.txt"];
      }
      if (command === "plugin:app|version") {
        return "0.1.6";
      }
      if (command === "plugin:updater|check") {
        return null;
      }
      if (command === "plugin:autostart|is_enabled") {
        return autostartEnabled;
      }
      if (command === "plugin:autostart|enable") {
        autostartEnabled = true;
        sessionStorage.setItem("cli-hub-mock-autostart", "true");
        return null;
      }
      if (command === "plugin:autostart|disable") {
        autostartEnabled = false;
        sessionStorage.setItem("cli-hub-mock-autostart", "false");
        return null;
      }
      if (command === "is_autostart_launch") {
        return false;
      }
      if (command === "load_app_preferences") {
        return { exists: true, preferences };
      }
      if (command === "migrate_legacy_preferences") {
        return preferences;
      }
      if (command === "save_launch_profiles") {
        preferences = { ...preferences, launchProfiles: args.launchProfiles };
        savePreferences();
        return preferences;
      }
      if (command === "save_recent_directories") {
        preferences = { ...preferences, recentDirectories: args.recentDirectories };
        savePreferences();
        return preferences;
      }
      if (command === "save_ui_settings") {
        preferences = { ...preferences, uiSettings: args.uiSettings };
        savePreferences();
        return preferences;
      }
      if (command === "save_workspace_pages") {
        preferences = {
          ...preferences,
          workspacePages: args.workspacePages,
          activeWorkspaceId: args.activeWorkspaceId,
        };
        savePreferences();
        return preferences;
      }
      if (command === "update_tray_status") {
        window.__mockTrayActiveCount = args.activeCount;
        return null;
      }
      if (command === "start_terminal") {
        const pageSessions = sessions.filter(
          (session) => session.pageId === args.request.pageId
        );
        if (pageSessions.length >= 8) throw new Error("workspace limit reached");
        if (sessions.length >= 24) throw new Error("global limit reached");
        const id = sessionId++;
        const directory = args.request.workingDirectory || "D:\\Demo\\cli-hub\\src-tauri";
        const session = {
          sessionId: id,
          processId: 1000 + id,
          pageId: args.request.pageId,
          preset: args.request.preset,
          name: args.request.name || "CLI",
          workingDirectory: directory,
          workingDirectoryKey: directory.replaceAll("/", "\\").toLowerCase(),
          output: [],
          outputStartOffset: 0,
          nextOutputOffset: 0,
          outputTruncated: false,
        };
        sessions.push(session);
        saveSessions();
        return session;
      }
      if (command === "list_active_terminals") {
        return sessions;
      }
      if (command === "restore_terminals") return [];
      if (command === "stop_terminal") {
        const index = sessions.findIndex((session) => session.sessionId === args.sessionId);
        if (index >= 0) sessions.splice(index, 1);
        saveSessions();
        queueMicrotask(() => emit("terminal-exit", {
          sessionId: args.sessionId,
          exitCode: 0,
          error: null,
        }));
        return null;
      }
      if (command === "stop_all_terminals") {
        const active = sessions.splice(0, sessions.length);
        saveSessions();
        active.forEach((session) => queueMicrotask(() => emit("terminal-exit", {
          sessionId: session.sessionId,
          exitCode: 0,
          error: null,
        })));
        return active.length;
      }
      if (command === "stop_workspace_terminals") {
        const active = sessions.filter((session) => session.pageId === args.pageId);
        for (let index = sessions.length - 1; index >= 0; index -= 1) {
          if (sessions[index].pageId === args.pageId) sessions.splice(index, 1);
        }
        saveSessions();
        active.forEach((session) => queueMicrotask(() => emit("terminal-exit", {
          sessionId: session.sessionId,
          exitCode: 0,
          error: null,
        })));
        return active.length;
      }
      if (command === "shutdown_application") {
        window.__shutdownRequested = true;
        sessions.splice(0, sessions.length);
        saveSessions();
        return null;
      }
      if (command === "write_terminal") {
        window.__terminalWrites.push(args);
        return null;
      }
      if (command === "resize_terminal") {
        return null;
      }
      throw new Error(`Unexpected mock command: ${command}`);
    },
  };
})();
"""


def emit_output(page, session_id: int, text: str) -> None:
    page.evaluate(
        """([sessionId, text]) => {
          const bytes = Array.from(new TextEncoder().encode(text));
          window.__emitTauriEvent("terminal-output", { sessionId, bytes });
        }""",
        [session_id, text],
    )


def main() -> None:
    SCREENSHOT.parent.mkdir(parents=True, exist_ok=True)
    console_errors: list[str] = []

    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True)
        page = browser.new_page(viewport={"width": 1100, "height": 760}, locale="en-US")
        page.set_default_timeout(5_000)
        page.add_init_script(TAURI_MOCK)
        page.on(
            "console",
            lambda message: console_errors.append(message.text)
            if message.type == "error"
            else None,
        )
        page.goto(APP_URL)
        page.wait_for_load_state("networkidle")

        interface_font = page.locator("html").evaluate(
            "element => getComputedStyle(element).fontFamily"
        )
        assert "CLI Hub JetBrainsMono NF" in interface_font
        assert page.evaluate(
            'document.fonts.check(\'400 13px "CLI Hub JetBrainsMono NF"\')'
        )
        assert page.evaluate(
            'document.fonts.check(\'700 13px "CLI Hub JetBrainsMono NF"\')'
        )
        font_resources = page.evaluate(
            "performance.getEntriesByType('resource')"
            ".map(entry => entry.name).filter(name => name.endsWith('.woff2'))"
        )
        assert any("JetBrainsMonoNerdFont-Regular" in name for name in font_resources)
        assert any("JetBrainsMonoNerdFont-Bold" in name for name in font_resources)
        license_text = page.evaluate(
            "fetch('/licenses/NerdFonts-LICENSE.txt').then(response => response.text())"
        )
        assert "SIL OPEN FONT LICENSE Version 1.1" in license_text
        assert page.locator("#launch-preset").count() == 0
        new_button = page.get_by_role("button", name="New session")
        assert new_button.is_enabled()
        header_control_boxes = [
            page.locator(".status").bounding_box(),
            page.get_by_role("button", name="Settings").bounding_box(),
            new_button.bounding_box(),
        ]
        assert all(box for box in header_control_boxes)
        header_control_heights = [box["height"] for box in header_control_boxes if box]
        assert max(header_control_heights) - min(header_control_heights) <= 1
        assert all(33 <= height <= 35 for height in header_control_heights)
        new_button.click()

        start_button = page.get_by_role("button", name="Start PowerShell")
        start_button.wait_for(state="visible")
        assert start_button.is_enabled(), "Start button should enable after event listeners register"
        launcher_dialog = page.get_by_role("dialog", name="New CLI session")
        assert "new-session-dialog" in (launcher_dialog.get_attribute("class") or "")
        assert page.locator("#launcher-title").evaluate(
            "element => getComputedStyle(element).fontSize"
        ) == "19px"
        assert page.locator("label[for='session-name']").evaluate(
            "element => getComputedStyle(element).fontSize"
        ) == "11px"
        assert page.locator("#session-name").evaluate(
            "element => getComputedStyle(element).fontSize"
        ) == "13px"
        start_button_box = start_button.bounding_box()
        preset_box = page.locator("#launch-preset").bounding_box()
        launcher_box = launcher_dialog.bounding_box()
        assert start_button_box and 29 <= start_button_box["height"] <= 31
        assert preset_box and 35 <= preset_box["height"] <= 37
        assert launcher_box and launcher_box["height"] < 640

        preset_select = page.locator("#launch-preset")
        assert preset_select.locator("option[value='claude']").inner_text() == "Claude Code"
        preset_select.select_option("claude")
        assert page.locator("#session-name").input_value() == "Claude Code"
        assert "claude.exe" in page.locator("#executable").input_value()
        assert page.get_by_role("button", name="Start Claude Code").is_visible()
        preset_select.select_option("powerShell")

        saved_profiles = page.locator("#saved-profile")
        assert saved_profiles.locator("option").count() == 1
        page.locator("#environment").fill("SECRET_TOKEN=not-persisted")
        save_profile_button = page.get_by_role("button", name="Save profile")
        assert save_profile_button.is_enabled(), {
            "errors": page.locator(".header-error, .error-message").all_inner_texts(),
            "commands": page.evaluate("window.__commands"),
        }
        save_profile_button.click()
        assert saved_profiles.locator("option").count() == 2
        assert saved_profiles.locator("option").nth(1).inner_text() == "PowerShell"
        stored_profiles = page.evaluate(
            "JSON.stringify(window.__mockPreferences.launchProfiles)"
        )
        assert "SECRET_TOKEN" not in stored_profiles
        page.locator("#environment").fill("")

        directory = page.locator("#working-directory")
        directory.fill(r"D:\Demo\cli-hub")
        start_button.click()
        page.locator(".session-card").wait_for(state="visible")
        assert page.get_by_role("dialog").count() == 0
        emit_output(page, 1, "__SESSION_ONE__\r\nPS> ")

        new_button.click()
        recent_directories = page.locator("#recent-working-directories option")
        assert recent_directories.count() == 1
        assert recent_directories.first.get_attribute("value") == r"D:\Demo\cli-hub"
        page.locator("#launch-preset").select_option("cmd")
        page.get_by_role("button", name="Start Command Prompt").click()
        emit_output(page, 2, "__SESSION_TWO__\r\nC:\\> ")
        page.locator(".session-preview", has_text="__SESSION_TWO__").wait_for(
            state="visible"
        )

        assert page.locator(".workspace-group").count() == 1
        assert page.locator(".workspace-group-header").count() == 1
        assert page.locator(".session-card").count() == 2
        assert page.locator(".session-preview", has_text="__SESSION_ONE__").count() == 1
        assert page.locator(".session-preview", has_text="__SESSION_TWO__").count() == 1
        preview_font = page.locator(".session-preview").nth(0).evaluate(
            "element => getComputedStyle(element).fontFamily"
        )
        terminal_font = page.locator(".xterm-rows").nth(0).evaluate(
            "element => getComputedStyle(element).fontFamily"
        )
        assert "CLI Hub JetBrainsMono NF" in preview_font
        assert "CLI Hub JetBrainsMono NF" in terminal_font

        grouping = page.get_by_role("group", name="Session grouping")
        grouping.get_by_role("button", name="CLI", exact=True).click()
        assert page.locator(".workspace-group-header").count() == 2
        assert "PowerShell" in page.locator(".workspace-group-header").nth(0).inner_text()
        assert "Command Prompt" in page.locator(".workspace-group-header").nth(1).inner_text()
        grouping.get_by_role("button", name="All", exact=True).click()
        assert page.locator(".workspace-group").count() == 1
        assert page.locator(".workspace-group-header").count() == 0
        assert page.locator(".session-card").count() == 2
        grouping.get_by_role("button", name="Folder", exact=True).click()
        assert page.locator(".workspace-group-header").count() == 1

        new_button.click()
        directory.fill(r"D:\Demo")
        page.locator("#launch-preset").select_option("powerShell")
        page.get_by_role("button", name="Start PowerShell").click()
        emit_output(page, 3, "PS> __OTHER_DIRECTORY__\r\n")
        page.wait_for_timeout(150)

        assert page.locator(".workspace-group").count() == 2
        assert page.locator(".session-card").count() == 3

        grouping.get_by_role("button", name="CLI", exact=True).click()
        assert page.locator(".workspace-group-header").count() == 2

        page.reload()
        page.wait_for_load_state("networkidle")
        page.locator(".session-card").nth(2).wait_for(state="visible")
        page.locator(".session-preview", has_text="__SESSION_ONE__").wait_for(state="visible")
        page.locator(".session-preview", has_text="__SESSION_TWO__").wait_for(state="visible")
        assert page.locator(".session-card").count() == 3
        assert page.locator(".session-card-footer", has_text="Recovered").count() == 3
        assert page.get_by_role("button", name="CLI", exact=True).get_attribute(
            "aria-pressed"
        ) == "true"
        assert page.locator(".session-preview", has_text="__SESSION_ONE__").count() == 1
        assert page.locator(".session-preview", has_text="__SESSION_TWO__").count() == 1
        assert "3/8 active" in page.locator(".status").inner_text()

        first_card = page.locator(".session-card").nth(0)
        first_card.locator(".session-card-main").click()
        assert "PowerShell · Session 1" in page.locator(".terminal-toolbar").inner_text()
        assert page.get_by_role("button", name="Remove PowerShell session").count() == 0

        terminal_stage = page.get_by_role("region", name="Interactive CLI terminal")
        normal_terminal_box = terminal_stage.bounding_box()
        assert normal_terminal_box
        terminal_stage.dblclick()
        page.locator(".terminal-stage-focus").wait_for(state="visible")
        focused_terminal_box = terminal_stage.bounding_box()
        assert focused_terminal_box
        assert focused_terminal_box["x"] == 0
        assert focused_terminal_box["y"] == 0
        assert focused_terminal_box["width"] == 1100
        assert focused_terminal_box["height"] == 760
        assert page.get_by_role("button", name="Exit focus").is_visible()
        page.screenshot(path=str(TERMINAL_FOCUS_SCREENSHOT), full_page=True)
        terminal_stage.dblclick(position={"x": 320, "y": 240})
        page.locator(".terminal-stage-focus").wait_for(state="detached")
        restored_terminal_box = terminal_stage.bounding_box()
        assert restored_terminal_box
        assert abs(restored_terminal_box["width"] - normal_terminal_box["width"]) <= 1

        composer = page.locator("#terminal-input")
        assert composer.evaluate("element => element.tagName") == "TEXTAREA"
        composer.fill("1. Inspect the logs")
        composer.press("Shift+Enter")
        assert composer.input_value() == "1. Inspect the logs\n2. "
        composer.press("Shift+Enter")
        assert composer.input_value() == "1. Inspect the logs\n"
        composer.fill("1. Inspect the logs")
        composer.press("Shift+Enter")
        composer.type("Summarize the failure")
        page.get_by_role("button", name="Attach").click()
        attachment = page.locator(".attachment-chip")
        attachment.wait_for(state="visible")
        assert attachment.inner_text().startswith("notes.txt")
        page.get_by_role("button", name="Send", exact=True).click()
        page.wait_for_function("window.__terminalWrites.length >= 2")
        sent_data = page.evaluate("window.__terminalWrites.at(-2).data")
        submit_data = page.evaluate("window.__terminalWrites.at(-1).data")
        expected_sent_data = (
            "1. Inspect the logs\n2. Summarize the failure\n\n"
            "Attached local files (path references only):\n"
            "- D:\\\\Demo\\\\notes.txt"
        )
        assert sent_data == expected_sent_data, repr(sent_data)
        assert submit_data == "\r", repr(submit_data)
        assert composer.input_value() == ""
        assert attachment.count() == 0

        composer_box = page.locator(".composer-box").bounding_box()
        send_box = page.get_by_role("button", name="Send", exact=True).bounding_box()
        assert composer_box and send_box
        assert composer_box["y"] == send_box["y"]
        assert composer_box["height"] == send_box["height"]

        control_button_boxes = [
            button.bounding_box()
            for button in page.locator(".control-row button").all()
        ]
        assert all(box and 26 <= box["height"] <= 29 for box in control_button_boxes)

        assert page.locator(".workspace-tab-close").count() == 0
        page.get_by_role("button", name="Add workspace").click()
        assert page.get_by_role("tab").count() == 2
        assert page.locator(".workspace-tab-close").count() == 2
        tab_alignment = page.get_by_role("tab").first.evaluate(
            """element => {
              const content = element.querySelector('.workspace-tab-content');
              const tabBox = element.getBoundingClientRect();
              const contentBox = content.getBoundingClientRect();
              return {
                outer: getComputedStyle(element).alignItems,
                inner: getComputedStyle(content).alignItems,
                centerDelta: Math.abs(
                  (tabBox.top + tabBox.height / 2) -
                  (contentBox.top + contentBox.height / 2)
                ),
              };
            }"""
        )
        assert tab_alignment["outer"] == "center"
        assert tab_alignment["inner"] == "baseline"
        assert tab_alignment["centerDelta"] <= 1
        assert page.locator(".session-card").count() == 0
        new_button.click()
        page.get_by_role("button", name="Start PowerShell").click()
        emit_output(page, 4, "PS> __WORKSPACE_TWO__\r\n")
        page.locator(".session-preview", has_text="__WORKSPACE_TWO__").wait_for(state="visible")
        assert page.locator(".session-card").count() == 1
        assert "__WORKSPACE_TWO__" in page.locator(".session-preview").inner_text()
        workspace_two_tab = page.get_by_role("tab", name=re.compile("Workspace 2"))
        workspace_two_tab.dblclick()
        workspace_name_input = page.get_by_role("textbox", name="Rename workspace")
        workspace_name_input.fill("Agents")
        workspace_name_input.press("Enter")
        agents_tab = page.get_by_role("tab", name=re.compile("Agents"))
        agents_tab.wait_for(state="visible")
        page.get_by_role("button", name="Close Agents").click()
        close_workspace_dialog = page.get_by_role(
            "dialog", name="Stop sessions and close Agents?"
        )
        close_workspace_dialog.wait_for(state="visible")
        assert "only the CLI sessions assigned to it" in close_workspace_dialog.inner_text()
        close_workspace_dialog.get_by_role("button", name="Cancel").click()
        assert page.locator(".session-card-running").count() == 1
        page.get_by_role("tab", name=re.compile("Workspace 1")).click()
        assert page.locator(".session-card").count() == 3
        first_card.locator(".session-card-main").click()
        assert "3/8 active" in page.locator(".status").inner_text()
        assert "4/24 total" in page.locator(".status").inner_text()

        page.get_by_role("button", name="Stop page", exact=True).click()
        stop_all_dialog = page.get_by_role(
            "dialog", name="Stop 3 running sessions in Workspace 1?"
        )
        stop_all_dialog.wait_for(state="visible")
        assert "Other workspaces will keep running" in stop_all_dialog.inner_text()
        stop_all_dialog.get_by_role("button", name="Cancel").click()
        assert stop_all_dialog.count() == 0
        assert page.locator(".session-card-running").count() == 3

        running_background = first_card.evaluate("element => getComputedStyle(element).backgroundColor")
        page.get_by_role("button", name="Stop selected").click()
        stop_selected_dialog = page.get_by_role("dialog", name="Stop PowerShell?")
        stop_selected_dialog.wait_for(state="visible")
        assert "Unsaved work" in stop_selected_dialog.inner_text()
        assert "stop-confirmation-dialog" in (
            stop_selected_dialog.get_attribute("class") or ""
        )
        assert stop_selected_dialog.locator("h2").evaluate(
            "element => getComputedStyle(element).fontSize"
        ) == "19px"
        assert stop_selected_dialog.locator(
            ".launcher-dialog-header p:not(.eyebrow)"
        ).evaluate("element => getComputedStyle(element).fontSize") == "11px"
        stop_selected_box = stop_selected_dialog.bounding_box()
        assert stop_selected_box and stop_selected_box["height"] < 220
        stop_dialog_button_boxes = [
            button.bounding_box()
            for button in stop_selected_dialog.locator(
                ".stop-confirmation-actions button"
            ).all()
        ]
        assert len(stop_dialog_button_boxes) == 2
        assert all(box and 29 <= box["height"] <= 31 for box in stop_dialog_button_boxes)
        page.screenshot(path=str(STOP_WARNING_SCREENSHOT), full_page=True)
        stop_selected_dialog.get_by_role("button", name="Cancel").click()
        assert stop_selected_dialog.count() == 0
        assert first_card.locator(".session-card-status").inner_text() == "Running"

        page.get_by_role("button", name="Stop selected").click()
        page.get_by_role("dialog", name="Stop PowerShell?").get_by_role(
            "button", name="Stop session"
        ).click()
        page.locator(".session-card-exited").wait_for(state="visible")
        exited_background = first_card.evaluate("element => getComputedStyle(element).backgroundColor")
        assert running_background != exited_background, "Stopped session should use yellow state background"
        assert "Exited" in first_card.inner_text()
        remove_button = page.get_by_role("button", name="Remove PowerShell session")
        remove_button.wait_for(state="visible")
        card_box = first_card.bounding_box()
        remove_box = remove_button.bounding_box()
        assert card_box and remove_box
        remove_center_x = remove_box["x"] + remove_box["width"] / 2
        remove_center_y = remove_box["y"] + remove_box["height"] / 2
        card_right = card_box["x"] + card_box["width"]
        card_top = card_box["y"]
        assert abs(remove_center_x - card_right) <= 4
        assert abs(remove_center_y - card_top) <= 4
        assert remove_button.evaluate("element => getComputedStyle(element).backgroundColor") == "rgb(189, 62, 56)"

        toolbar_box = page.locator(".terminal-toolbar").bounding_box()
        toolbar_hint_box = page.locator(".terminal-toolbar > span").nth(1).bounding_box()
        overview_box = page.locator(".workspace-overview").bounding_box()
        terminal_stage_box = page.locator(".terminal-stage").bounding_box()
        assert toolbar_box and toolbar_box["width"] > 900
        assert toolbar_hint_box and toolbar_hint_box["x"] > 700
        assert overview_box and 280 <= overview_box["width"] <= 360
        assert terminal_stage_box and terminal_stage_box["width"] > overview_box["width"]

        sidebar_resizer = page.get_by_role(
            "separator", name="Resize session overview"
        )
        sidebar_resizer.press('Home')
        overview_box = page.locator('.workspace-overview').bounding_box()
        sidebar_resizer_box = sidebar_resizer.bounding_box()
        assert sidebar_resizer_box
        page.mouse.move(
            sidebar_resizer_box["x"] + sidebar_resizer_box["width"] / 2,
            sidebar_resizer_box["y"] + sidebar_resizer_box["height"] / 2,
        )
        page.mouse.down()
        page.mouse.move(
            sidebar_resizer_box["x"] + 200,
            sidebar_resizer_box["y"] + sidebar_resizer_box["height"] / 2,
        )
        page.mouse.up()
        resized_overview_box = page.locator(".workspace-overview").bounding_box()
        resized_terminal_box = page.locator(".terminal-stage").bounding_box()
        assert resized_overview_box and overview_box["width"] < resized_overview_box["width"] <= 360
        assert resized_terminal_box and resized_terminal_box["width"] >= 320
        sidebar_resizer.dblclick()
        reset_overview_box = page.locator(".workspace-overview").bounding_box()
        assert reset_overview_box and abs(reset_overview_box["width"] - 360) <= 1

        input_resizer = page.get_by_role("separator", name="Resize input panel")
        input_panel_box = page.locator(".input-panel").bounding_box()
        input_resizer_box = input_resizer.bounding_box()
        assert input_panel_box and input_resizer_box
        page.mouse.move(
            input_resizer_box["x"] + input_resizer_box["width"] / 2,
            input_resizer_box["y"] + input_resizer_box["height"] / 2,
        )
        page.mouse.down()
        page.mouse.move(
            input_resizer_box["x"] + input_resizer_box["width"] / 2,
            input_resizer_box["y"] - 80,
        )
        page.mouse.up()
        resized_input_panel_box = page.locator(".input-panel").bounding_box()
        assert resized_input_panel_box
        assert input_panel_box["height"] + 70 <= resized_input_panel_box["height"] <= 360
        input_resizer.dblclick()
        reset_input_panel_box = page.locator(".input-panel").bounding_box()
        assert reset_input_panel_box
        assert abs(reset_input_panel_box["height"] - input_panel_box["height"]) <= 1

        default_card_box = first_card.bounding_box()
        assert default_card_box
        page.set_viewport_size({"width": 720, "height": 600})
        page.wait_for_timeout(100)
        minimum_card_box = first_card.bounding_box()
        minimum_overview_box = page.locator(".workspace-overview").bounding_box()
        assert minimum_card_box and abs(minimum_card_box["width"] - default_card_box["width"]) <= 1
        assert minimum_overview_box and 189 <= minimum_overview_box["width"] <= 191
        minimum_card_right = minimum_card_box["x"] + minimum_card_box["width"]
        minimum_overview_right = minimum_overview_box["x"] + minimum_overview_box["width"]
        assert 0 < minimum_overview_right - minimum_card_right <= 20
        page.screenshot(path=str(MINIMUM_SCREENSHOT), full_page=True)
        page.set_viewport_size({"width": 1100, "height": 760})
        page.wait_for_timeout(100)

        overview_scrollbar = page.locator(".workspace-overview").evaluate(
            "element => getComputedStyle(element).scrollbarColor"
        )
        terminal_scrollbar = page.locator(".xterm-viewport").nth(0).evaluate(
            "element => getComputedStyle(element).scrollbarColor"
        )
        assert overview_scrollbar == terminal_scrollbar

        page.screenshot(path=str(SCREENSHOT), full_page=True)
        remove_button.click()
        assert page.locator(".session-card").count() == 2
        assert page.get_by_role("button", name="Remove PowerShell session").count() == 0
        new_button.click()
        dialog = page.get_by_role("dialog")
        dialog.wait_for(state="visible")
        dialog_box = dialog.bounding_box()
        assert dialog_box and dialog_box["y"] >= 0
        assert dialog_box["y"] + dialog_box["height"] <= 760
        saved_profiles.select_option(label="PowerShell")
        assert page.locator("#session-name").input_value() == "PowerShell"
        assert page.locator("#working-directory").input_value() == ""
        assert page.locator("#environment").input_value() == ""
        page.locator("#session-name").fill("PowerShell Updated")
        page.get_by_role("button", name="Update profile").click()
        assert saved_profiles.locator("option").nth(1).inner_text() == "PowerShell Updated"
        page.screenshot(path=str(LAUNCHER_SCREENSHOT), full_page=True)
        page.keyboard.press("Escape")
        assert dialog.count() == 0

        page.reload()
        page.wait_for_load_state("networkidle")
        assert page.get_by_role("tab", name=re.compile("Agents")).count() == 1
        new_button.click()
        saved_profiles.select_option(label="PowerShell Updated")
        assert page.locator("#session-name").input_value() == "PowerShell Updated"
        page.get_by_role("button", name="Delete").click()
        assert saved_profiles.locator("option").count() == 1
        page.keyboard.press("Escape")

        settings_button = page.get_by_role("button", name="Settings")
        settings_button.click()
        settings_dialog = page.get_by_role("dialog", name="Settings")
        settings_dialog.wait_for(state="visible")
        assert settings_dialog.locator("h2").evaluate(
            "element => getComputedStyle(element).fontSize"
        ) == "19px"
        assert settings_dialog.locator(".settings-option strong").first.evaluate(
            "element => getComputedStyle(element).fontSize"
        ) == "12px"
        settings_dialog_box = settings_dialog.bounding_box()
        assert settings_dialog_box and settings_dialog_box["height"] < 650
        autostart_toggle = page.locator("#autostart-enabled")
        assert not autostart_toggle.is_checked()
        autostart_box = autostart_toggle.bounding_box()
        assert autostart_box and 21 <= autostart_box["height"] <= 23
        autostart_toggle.check()
        assert autostart_toggle.is_checked()
        minimized_toggle = page.locator("#start-minimized")
        assert minimized_toggle.is_enabled()
        assert minimized_toggle.is_checked()

        color_mode = page.locator("#interface-color-mode")
        assert color_mode.input_value() == "system"
        color_mode_box = color_mode.bounding_box()
        assert color_mode_box and 33 <= color_mode_box["height"] <= 35
        done_button_box = page.get_by_role("button", name="Done").bounding_box()
        assert done_button_box and 33 <= done_button_box["height"] <= 35
        assert abs(done_button_box["width"] - color_mode_box["width"]) < 1
        assert "Current version 0.1.6" in settings_dialog.inner_text()
        update_button = page.get_by_role("button", name="Check for updates")
        update_button_box = update_button.bounding_box()
        assert update_button_box and 33 <= update_button_box["height"] <= 35
        update_button.click()
        assert "CLI Hub is up to date." in settings_dialog.inner_text()
        page.emulate_media(color_scheme="dark")
        page.wait_for_function(
            'document.documentElement.dataset.colorMode === "dark"'
        )
        color_mode.select_option("light")
        page.wait_for_function(
            'document.documentElement.dataset.colorMode === "light"'
        )
        color_mode.select_option("dark")
        page.wait_for_function(
            'document.documentElement.dataset.colorMode === "dark"'
        )

        page.locator("#interface-language").select_option("zh-CN")
        assert page.get_by_role("button", name="完成").is_visible()
        assert page.get_by_role("button", name="新建会话").is_visible()
        page.get_by_role("button", name="完成").click()
        assert settings_dialog.count() == 0
        stored_settings = page.evaluate(
            "JSON.stringify(window.__mockPreferences.uiSettings)"
        )
        assert '\"language\":\"zh-CN\"' in stored_settings
        assert '\"colorMode\":\"dark\"' in stored_settings
        assert '\"sessionGrouping\":\"cli\"' in stored_settings
        page.get_by_role("button", name="新建会话").click()
        chinese_launcher = page.get_by_role("dialog", name="新建 CLI 会话")
        chinese_launcher.wait_for(state="visible")
        chinese_launcher_box = chinese_launcher.bounding_box()
        assert chinese_launcher_box and chinese_launcher_box["height"] < 640
        assert page.locator("label[for='session-name']").evaluate(
            "element => getComputedStyle(element).fontSize"
        ) == "11px"
        page.keyboard.press("Escape")
        assert chinese_launcher.count() == 0

        page.reload()
        page.wait_for_load_state("networkidle")
        page.get_by_role("button", name="设置").click()
        chinese_settings = page.get_by_role("dialog", name="设置")
        chinese_settings.wait_for(state="visible")
        assert page.locator("#interface-language").input_value() == "zh-CN"
        assert page.locator("#interface-color-mode").input_value() == "dark"
        assert page.locator("html").get_attribute("data-color-mode") == "dark"
        assert page.locator("#autostart-enabled").is_checked()
        page.screenshot(path=str(SETTINGS_SCREENSHOT), full_page=True)
        page.locator("#interface-color-mode").select_option("light")
        page.locator("#interface-language").select_option("en")
        page.get_by_role("button", name="Done").click()

        page.evaluate('window.__emitTauriEvent("application-close-requested", 2)')
        shutdown_dialog = page.get_by_role("dialog", name="Stop all sessions and exit?")
        shutdown_dialog.wait_for(state="visible")
        assert "2 running sessions" in shutdown_dialog.inner_text()
        shutdown_dialog.get_by_role("button", name="Cancel").click()
        assert shutdown_dialog.count() == 0
        assert page.locator(".session-card-running").count() == 2

        page.evaluate('window.__emitTauriEvent("application-close-requested", 2)')
        shutdown_dialog.get_by_role("button", name="Stop all and exit").click()
        assert page.evaluate("window.__shutdownRequested === true")
        browser.close()

    assert not console_errors, f"Browser console errors: {console_errors}"
    print(
        "Stage 6 interaction UI verified: multiline composer, numbered lines, local path "
        "attachments, bundled JetBrainsMono Nerd Font interface and terminal typography, "
        "persisted language and system/light/dark color settings, "
        "OS-backed autostart controls, isolated workspace tabs and counts, "
        "guarded selected/workspace session stops, "
        "New-session dialog with Claude Code preset, refresh recovery, persisted directory/CLI/"
        "ungrouped views, bounded draggable panel dividers, double-click terminal focus mode, "
        "matched scrollbars, selection, "
        "removable stopped cards, safe exit, profiles, and recent directories."
    )


if __name__ == "__main__":
    main()
