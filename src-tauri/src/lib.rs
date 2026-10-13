mod preferences;
mod private_storage;
mod terminal;
mod terminal_host;
mod updater_cleanup;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use terminal_host::HostClient;

#[tauri::command]
fn is_autostart_launch() -> bool {
    std::env::args_os().any(|argument| argument == "--autostart")
}

#[tauri::command]
fn update_tray_status(
    app: AppHandle,
    status: State<'_, MenuItem<tauri::Wry>>,
    active_count: usize,
) -> Result<(), String> {
    status
        .set_text(format!("CLI Hub · {active_count} active"))
        .map_err(|error| error.to_string())?;
    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_tooltip(Some(format!("CLI Hub · {active_count} active")))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn request_application_exit(app: &AppHandle) {
    let active_count = app
        .state::<HostClient>()
        .call("list_active_terminals", serde_json::json!({}))
        .ok()
        .and_then(|value| value.as_array().map(Vec::len))
        .unwrap_or(1);
    if active_count == 0 {
        if let Err(error) = app
            .state::<HostClient>()
            .call("shutdown_application", serde_json::json!({}))
        {
            show_main_window(app);
            let _ = app.emit("session-persistence-error", error);
            return;
        }
        app.exit(0);
        return;
    }

    show_main_window(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("application-close-requested", active_count);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut arguments = std::env::args_os();
    arguments.next();
    if arguments.next().as_deref() == Some(std::ffi::OsStr::new("--terminal-host")) {
        if let Some(root) = arguments.next() {
            terminal_host::run(root.into());
        }
        return;
    }
    let builder = tauri::Builder::default();
    #[cfg(desktop)]
    let builder =
        builder
            .plugin(tauri_plugin_process::init())
            .plugin(tauri_plugin_single_instance::init(|app, _, _| {
                show_main_window(app);
            }));

    builder
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec!["--autostart"]),
            ))?;
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            updater_cleanup::clean_stale_artifacts();
            let root = app.path().app_config_dir()?;
            app.manage(HostClient::ensure(&root).map_err(std::io::Error::other)?);

            #[cfg(desktop)]
            {
                let status =
                    MenuItem::with_id(app, "status", "CLI Hub · 0 active", false, None::<&str>)?;
                let open = MenuItem::with_id(app, "open", "Open CLI Hub", true, None::<&str>)?;
                let separator = PredefinedMenuItem::separator(app)?;
                let quit = MenuItem::with_id(app, "quit", "Stop all and quit", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&status, &open, &separator, &quit])?;
                let mut tray = TrayIconBuilder::with_id("main-tray")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .tooltip("CLI Hub · 0 active")
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "open" => show_main_window(app),
                        "quit" => request_application_exit(app),
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            show_main_window(tray.app_handle());
                        }
                    });
                if let Some(icon) = app.default_window_icon() {
                    tray = tray.icon(icon.clone());
                }
                tray.build(app)?;
                app.manage(status);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // The host owns the terminals; closing this interface cannot end their PTYs.
                api.prevent_close();
                window.app_handle().exit(0);
            }
        })
        .invoke_handler(tauri::generate_handler![
            is_autostart_launch,
            update_tray_status,
            terminal_host::load_app_preferences,
            terminal_host::migrate_legacy_preferences,
            terminal_host::save_launch_profiles,
            terminal_host::save_recent_directories,
            terminal_host::save_ui_settings,
            terminal_host::save_workspace_pages,
            terminal_host::start_terminal,
            terminal_host::list_active_terminals,
            terminal_host::write_terminal,
            terminal_host::resize_terminal,
            terminal_host::stop_terminal,
            terminal_host::stop_all_terminals,
            terminal_host::stop_workspace_terminals,
            terminal_host::shutdown_application,
            terminal_host::restore_terminals,
            terminal_host::prepare_interface_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running CLI Hub");
}
