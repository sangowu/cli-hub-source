use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::private_storage;
use crate::terminal::{LaunchRequest, TerminalManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

const PREFERENCES_SCHEMA_VERSION: u32 = 1;
const MAX_LAUNCH_PROFILES: usize = 20;
const MAX_RECENT_DIRECTORIES: usize = 10;
const MAX_WORKSPACE_PAGES: usize = 20;
const DEFAULT_WORKSPACE_ID: &str = "workspace-default";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPreferences {
    #[serde(default = "schema_version")]
    version: u32,
    #[serde(default)]
    launch_profiles: Vec<LaunchProfile>,
    #[serde(default)]
    recent_directories: Vec<String>,
    #[serde(default)]
    ui_settings: UiSettings,
    #[serde(default = "default_workspace_pages")]
    workspace_pages: Vec<WorkspacePage>,
    #[serde(default = "default_workspace_id")]
    active_workspace_id: String,
    #[serde(default)]
    migrated_origins: Vec<String>,
    #[serde(default)]
    restore_sessions: Vec<LaunchRequest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacePage {
    id: String,
    name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchProfile {
    id: String,
    label: String,
    preset: LaunchPreset,
    session_name: String,
    executable: String,
    arguments_text: String,
    working_directory: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum LaunchPreset {
    PowerShell,
    WindowsPowerShell,
    Cmd,
    Codex,
    Claude,
    Custom,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSettings {
    language: LanguagePreference,
    color_mode: ColorModePreference,
    start_minimized: bool,
    #[serde(default)]
    session_grouping: SessionGrouping,
    #[serde(default)]
    session_sidebar_collapsed: bool,
    #[serde(default)]
    restore_on_startup: bool,
    #[serde(default)]
    terminal_shortcuts: TerminalShortcuts,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TerminalShortcuts {
    copy: String,
    paste: String,
    paste_alternate: String,
    interrupt: String,
    newline: String,
    smart_copy: bool,
}

impl Default for TerminalShortcuts {
    fn default() -> Self {
        Self {
            copy: "Ctrl+Shift+C".into(),
            paste: "Ctrl+V".into(),
            paste_alternate: "Ctrl+Shift+V".into(),
            interrupt: "Ctrl+C".into(),
            newline: "Shift+Enter".into(),
            smart_copy: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum LanguagePreference {
    System,
    En,
    #[serde(rename = "zh-CN")]
    ZhCn,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum ColorModePreference {
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum SessionGrouping {
    #[default]
    Directory,
    Cli,
    None,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            version: PREFERENCES_SCHEMA_VERSION,
            launch_profiles: Vec::new(),
            recent_directories: Vec::new(),
            ui_settings: UiSettings::default(),
            workspace_pages: default_workspace_pages(),
            active_workspace_id: default_workspace_id(),
            migrated_origins: Vec::new(),
            restore_sessions: Vec::new(),
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            language: LanguagePreference::System,
            color_mode: ColorModePreference::System,
            start_minimized: false,
            session_grouping: SessionGrouping::Directory,
            session_sidebar_collapsed: false,
            restore_on_startup: false,
            terminal_shortcuts: TerminalShortcuts::default(),
        }
    }
}

impl AppPreferences {
    fn validate(mut self) -> Result<Self, String> {
        if self.version != PREFERENCES_SCHEMA_VERSION {
            return Err(format!(
                "unsupported preferences version {}; expected {}",
                self.version, PREFERENCES_SCHEMA_VERSION
            ));
        }
        if self.launch_profiles.len() > MAX_LAUNCH_PROFILES {
            return Err(format!(
                "too many launch profiles; maximum is {MAX_LAUNCH_PROFILES}"
            ));
        }
        if self.recent_directories.len() > MAX_RECENT_DIRECTORIES {
            return Err(format!(
                "too many recent directories; maximum is {MAX_RECENT_DIRECTORIES}"
            ));
        }
        if self.restore_sessions.len() > 24 {
            return Err("too many saved terminal sessions; maximum is 24".into());
        }
        if self.workspace_pages.is_empty() || self.workspace_pages.len() > MAX_WORKSPACE_PAGES {
            return Err(format!(
                "workspace page count must be between 1 and {MAX_WORKSPACE_PAGES}"
            ));
        }
        let mut workspace_ids = std::collections::HashSet::new();
        for workspace in &mut self.workspace_pages {
            workspace.id = workspace.id.trim().to_string();
            workspace.name = workspace.name.trim().to_string();
            if workspace.id.is_empty()
                || workspace.id.len() > 128
                || workspace.name.is_empty()
                || workspace.name.len() > 40
                || !workspace_ids.insert(workspace.id.clone())
            {
                return Err("workspace pages contain an invalid or duplicate entry".to_string());
            }
        }
        if !workspace_ids.contains(self.active_workspace_id.trim()) {
            self.active_workspace_id = self.workspace_pages[0].id.clone();
        }
        self.restore_sessions
            .retain(|request| workspace_ids.contains(&request.page_id));

        self.version = PREFERENCES_SCHEMA_VERSION;
        Ok(self)
    }
}

pub struct PreferencesStore {
    path: PathBuf,
    stored: Mutex<StoredPreferences>,
}

struct StoredPreferences {
    exists: bool,
    preferences: AppPreferences,
    load_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedPreferences {
    exists: bool,
    preferences: AppPreferences,
}

impl PreferencesStore {
    pub(crate) fn restore_sessions(&self) -> Result<Vec<LaunchRequest>, String> {
        let stored = self
            .stored
            .lock()
            .map_err(|_| "preferences store lock is poisoned")?;
        ensure_loaded(&stored)?;
        Ok(if stored.preferences.ui_settings.restore_on_startup {
            stored.preferences.restore_sessions.clone()
        } else {
            vec![]
        })
    }

    pub(crate) fn save_restore_sessions(&self, sessions: Vec<LaunchRequest>) -> Result<(), String> {
        {
            let stored = self
                .stored
                .lock()
                .map_err(|_| "preferences store lock is poisoned")?;
            ensure_loaded(&stored)?;
            if !stored.preferences.ui_settings.restore_on_startup
                && stored.preferences.restore_sessions.is_empty()
            {
                return Ok(());
            }
        }
        self.update(|preferences| {
            preferences.restore_sessions = if preferences.ui_settings.restore_on_startup {
                sessions
            } else {
                vec![]
            }
        })?;
        Ok(())
    }
    pub fn open(path: PathBuf) -> Self {
        let stored = match load_preferences(&path) {
            Ok(Some(preferences)) => StoredPreferences {
                exists: true,
                preferences,
                load_error: None,
            },
            Ok(None) => StoredPreferences {
                exists: false,
                preferences: AppPreferences::default(),
                load_error: None,
            },
            Err(error) => StoredPreferences {
                exists: path.exists(),
                preferences: AppPreferences::default(),
                load_error: Some(error),
            },
        };

        Self {
            path,
            stored: Mutex::new(stored),
        }
    }

    fn update(
        &self,
        transform: impl FnOnce(&mut AppPreferences),
    ) -> Result<AppPreferences, String> {
        let mut stored = self
            .stored
            .lock()
            .map_err(|_| "preferences store lock is poisoned".to_string())?;
        ensure_loaded(&stored)?;

        let mut next = stored.preferences.clone();
        transform(&mut next);
        let next = next.validate()?;
        write_preferences(&self.path, &next)?;

        stored.exists = true;
        stored.preferences = next.clone();
        Ok(next)
    }
}

pub fn load_app_preferences(
    store: State<'_, PreferencesStore>,
) -> Result<LoadedPreferences, String> {
    let stored = store
        .stored
        .lock()
        .map_err(|_| "preferences store lock is poisoned".to_string())?;
    ensure_loaded(&stored)?;

    Ok(LoadedPreferences {
        exists: stored.exists,
        preferences: stored.preferences.clone(),
    })
}

pub fn migrate_legacy_preferences(
    store: State<'_, PreferencesStore>,
    origin: String,
    preferences: AppPreferences,
) -> Result<AppPreferences, String> {
    let mut stored = store
        .stored
        .lock()
        .map_err(|_| "preferences store lock is poisoned".to_string())?;
    ensure_loaded(&stored)?;

    if stored.preferences.migrated_origins.contains(&origin) {
        return Ok(stored.preferences.clone());
    }

    let preferences = merge_legacy_preferences(
        stored.exists,
        &stored.preferences,
        preferences.validate()?,
        origin,
    )?;
    write_preferences(&store.path, &preferences)?;
    stored.exists = true;
    stored.preferences = preferences.clone();
    Ok(preferences)
}

pub fn save_launch_profiles(
    store: State<'_, PreferencesStore>,
    launch_profiles: Vec<LaunchProfile>,
) -> Result<AppPreferences, String> {
    store.update(|preferences| preferences.launch_profiles = launch_profiles)
}

pub fn save_recent_directories(
    store: State<'_, PreferencesStore>,
    recent_directories: Vec<String>,
) -> Result<AppPreferences, String> {
    store.update(|preferences| preferences.recent_directories = recent_directories)
}

pub fn save_ui_settings(
    store: State<'_, PreferencesStore>,
    manager: State<'_, Arc<TerminalManager>>,
    ui_settings: UiSettings,
) -> Result<AppPreferences, String> {
    let shortcuts = &ui_settings.terminal_shortcuts;
    let keys = [
        &shortcuts.copy,
        &shortcuts.paste,
        &shortcuts.paste_alternate,
        &shortcuts.interrupt,
        &shortcuts.newline,
    ];
    if keys.iter().collect::<std::collections::HashSet<_>>().len() != keys.len() {
        return Err("Terminal shortcuts must be distinct".into());
    }
    store.update(|preferences| preferences.ui_settings = ui_settings)?;
    manager.persist_sessions(&store)?;
    let stored = store
        .stored
        .lock()
        .map_err(|_| "preferences store lock is poisoned")?;
    Ok(stored.preferences.clone())
}

pub fn save_workspace_pages(
    store: State<'_, PreferencesStore>,
    workspace_pages: Vec<WorkspacePage>,
    active_workspace_id: String,
) -> Result<AppPreferences, String> {
    store.update(|preferences| {
        preferences.workspace_pages = workspace_pages;
        preferences.active_workspace_id = active_workspace_id;
        preferences.restore_sessions.retain(|request| {
            preferences
                .workspace_pages
                .iter()
                .any(|page| page.id == request.page_id)
        });
    })
}

fn schema_version() -> u32 {
    PREFERENCES_SCHEMA_VERSION
}

fn default_workspace_id() -> String {
    DEFAULT_WORKSPACE_ID.to_string()
}

fn default_workspace_pages() -> Vec<WorkspacePage> {
    vec![WorkspacePage {
        id: default_workspace_id(),
        name: "Workspace 1".to_string(),
    }]
}

fn ensure_loaded(stored: &StoredPreferences) -> Result<(), String> {
    match &stored.load_error {
        Some(error) => Err(error.clone()),
        None => Ok(()),
    }
}

fn merge_legacy_preferences(
    stored_exists: bool,
    current: &AppPreferences,
    legacy: AppPreferences,
    origin: String,
) -> Result<AppPreferences, String> {
    if current.migrated_origins.contains(&origin) {
        return Ok(current.clone());
    }

    let mut preferences = if stored_exists {
        let mut merged = current.clone();
        for profile in legacy.launch_profiles {
            if merged.launch_profiles.len() >= MAX_LAUNCH_PROFILES {
                break;
            }
            if !merged
                .launch_profiles
                .iter()
                .any(|candidate| candidate.id == profile.id)
            {
                merged.launch_profiles.push(profile);
            }
        }
        for directory in legacy.recent_directories {
            if merged.recent_directories.len() >= MAX_RECENT_DIRECTORIES {
                break;
            }
            if !merged
                .recent_directories
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(&directory))
            {
                merged.recent_directories.push(directory);
            }
        }
        merged
    } else {
        legacy
    };
    preferences.migrated_origins.push(origin);
    preferences.validate()
}

fn load_preferences(path: &Path) -> Result<Option<AppPreferences>, String> {
    private_storage::restrict_existing_file(path)
        .map_err(|error| format!("failed to protect {}: {error}", path.display()))?;
    if !path.exists() {
        return Ok(None);
    }

    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let preferences = serde_json::from_slice::<AppPreferences>(&bytes)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?
        .validate()?;
    Ok(Some(preferences))
}

fn write_preferences(path: &Path, preferences: &AppPreferences) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("preferences path has no parent: {}", path.display()))?;
    let temporary_path = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(preferences)
        .map_err(|error| format!("failed to serialize preferences: {error}"))?;
    let mut file = private_storage::open_for_write(&temporary_path)
        .map_err(|error| format!("failed to open {}: {error}", temporary_path.display()))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("failed to write {}: {error}", temporary_path.display()))?;
    drop(file);

    replace_file(&temporary_path, path)
        .map_err(|error| format!("failed to replace {}: {error}", path.display()))?;
    sync_parent_directory(parent)
        .map_err(|error| format!("failed to sync {}: {error}", parent.display()))?;
    Ok(())
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source_wide: Vec<u16> = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let destination_wide: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let result = unsafe {
        MoveFileExW(
            source_wide.as_ptr(),
            destination_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) -> io::Result<()> {
    fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_preferences_path(test_name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("cli-hub-preferences-tests-{}", std::process::id()))
            .join(format!("{test_name}-preferences.json"))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(path.with_extension("json.tmp"));
    }

    #[test]
    fn missing_preferences_use_defaults_without_creating_a_file() {
        let path = temporary_preferences_path("missing");
        cleanup(&path);

        let store = PreferencesStore::open(path.clone());
        let stored = store.stored.lock().expect("preferences lock");

        assert!(!stored.exists);
        assert_eq!(stored.preferences.version, PREFERENCES_SCHEMA_VERSION);
        assert!(stored.preferences.launch_profiles.is_empty());
        assert_eq!(stored.preferences.workspace_pages.len(), 1);
        assert_eq!(stored.preferences.active_workspace_id, DEFAULT_WORKSPACE_ID);
        assert!(!path.exists());
    }

    #[test]
    fn existing_ui_settings_default_to_directory_grouping() {
        let preferences: AppPreferences = serde_json::from_value(serde_json::json!({
            "version": 1,
            "uiSettings": {
                "language": "system",
                "colorMode": "system",
                "startMinimized": true
            }
        }))
        .expect("existing preferences should remain compatible");

        assert!(matches!(
            preferences.ui_settings.session_grouping,
            SessionGrouping::Directory
        ));
        assert!(!preferences.ui_settings.restore_on_startup);
        assert!(!preferences.ui_settings.session_sidebar_collapsed);
        assert_eq!(
            preferences.ui_settings.terminal_shortcuts.interrupt,
            "Ctrl+C"
        );
        assert!(preferences.restore_sessions.is_empty());
    }

    #[test]
    fn collapsed_sidebar_preference_survives_a_store_reopen() {
        let path = temporary_preferences_path("collapsed-sidebar");
        cleanup(&path);
        let store = PreferencesStore::open(path.clone());
        store
            .update(|preferences| preferences.ui_settings.session_sidebar_collapsed = true)
            .expect("save collapsed sidebar");
        drop(store);

        let reopened = PreferencesStore::open(path.clone());
        let stored = reopened.stored.lock().expect("preferences lock");
        assert!(stored.preferences.ui_settings.session_sidebar_collapsed);
        assert!(stored.exists);
        drop(stored);
        drop(reopened);
        cleanup(&path);
    }

    #[test]
    fn legacy_shortcuts_keep_custom_bindings_and_default_newline() {
        let shortcuts: TerminalShortcuts = serde_json::from_value(serde_json::json!({
            "copy": "Ctrl+Shift+X", "interrupt": "Ctrl+Q", "smartCopy": false
        }))
        .expect("old shortcut preferences remain compatible");
        assert_eq!(shortcuts.newline, "Shift+Enter");
        assert_eq!(shortcuts.copy, "Ctrl+Shift+X");
        assert_eq!(shortcuts.interrupt, "Ctrl+Q");
        assert!(!shortcuts.smart_copy);
    }

    #[test]
    fn custom_newline_shortcut_survives_a_store_reopen() {
        let path = temporary_preferences_path("newline-shortcut");
        cleanup(&path);
        let store = PreferencesStore::open(path.clone());
        store
            .update(|preferences| {
                preferences.ui_settings.terminal_shortcuts.newline = "Ctrl+Enter".into()
            })
            .expect("save newline shortcut");
        drop(store);
        let reopened = PreferencesStore::open(path.clone());
        let stored = reopened.stored.lock().expect("preferences lock");
        assert_eq!(
            stored.preferences.ui_settings.terminal_shortcuts.newline,
            "Ctrl+Enter"
        );
        drop(stored);
        drop(reopened);
        cleanup(&path);
    }

    #[test]
    fn restore_requests_round_trip_without_temporary_environment() {
        let path = temporary_preferences_path("restore");
        cleanup(&path);
        let store = PreferencesStore::open(path.clone());
        store
            .update(|preferences| preferences.ui_settings.restore_on_startup = true)
            .unwrap();
        let request: LaunchRequest = serde_json::from_value(serde_json::json!({
            "pageId": "workspace-default", "preset": "cmd", "name": "Saved CMD", "executable": "",
            "arguments": ["/Q"], "workingDirectory": "D:\\Demo",
            "environment": [{"name": "SECRET_TOKEN", "value": "private-test-value"}]
        }))
        .unwrap();
        store.save_restore_sessions(vec![request]).unwrap();
        let bytes = fs::read_to_string(&path).unwrap();
        assert!(!bytes.contains("SECRET_TOKEN"));
        assert!(!bytes.contains("private-test-value"));
        let reopened = PreferencesStore::open(path.clone());
        let requests = reopened.restore_sessions().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            serde_json::to_value(&requests[0]).unwrap()["arguments"],
            serde_json::json!(["/Q"])
        );
        reopened
            .update(|preferences| preferences.ui_settings.restore_on_startup = false)
            .unwrap();
        reopened.save_restore_sessions(vec![]).unwrap();
        assert!(reopened.restore_sessions().unwrap().is_empty());
        assert!(load_preferences(&path)
            .unwrap()
            .unwrap()
            .restore_sessions
            .is_empty());
        cleanup(&path);
    }

    #[test]
    fn preferences_round_trip_through_atomic_file() {
        let path = temporary_preferences_path("round-trip");
        cleanup(&path);
        let profile = LaunchProfile {
            id: "profile-1".to_string(),
            label: "Codex".to_string(),
            preset: LaunchPreset::Codex,
            session_name: "Codex".to_string(),
            executable: "codex.exe".to_string(),
            arguments_text: "--no-daemon".to_string(),
            working_directory: "D:\\Demo".to_string(),
        };

        let store = PreferencesStore::open(path.clone());
        store
            .update(|preferences| {
                preferences.launch_profiles = vec![profile];
                preferences.recent_directories = vec!["D:\\Demo".to_string()];
            })
            .expect("save preferences");
        drop(store);

        let reopened = PreferencesStore::open(path.clone());
        let loaded = reopened.stored.lock().expect("preferences lock");

        assert!(loaded.exists);
        assert_eq!(loaded.preferences.launch_profiles.len(), 1);
        assert_eq!(loaded.preferences.launch_profiles[0].label, "Codex");
        assert_eq!(loaded.preferences.recent_directories, vec!["D:\\Demo"]);
        drop(loaded);
        cleanup(&path);
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let preferences = AppPreferences {
            version: PREFERENCES_SCHEMA_VERSION + 1,
            ..AppPreferences::default()
        };

        assert!(preferences.validate().is_err());
    }

    #[test]
    fn workspace_pages_require_unique_ids_and_a_valid_active_page() {
        let preferences = AppPreferences {
            workspace_pages: vec![
                WorkspacePage {
                    id: "duplicate".to_string(),
                    name: "One".to_string(),
                },
                WorkspacePage {
                    id: "duplicate".to_string(),
                    name: "Two".to_string(),
                },
            ],
            ..AppPreferences::default()
        };
        assert!(preferences.validate().is_err());

        let preferences = AppPreferences {
            workspace_pages: vec![WorkspacePage {
                id: "workspace-a".to_string(),
                name: "Workspace A".to_string(),
            }],
            active_workspace_id: "missing".to_string(),
            ..AppPreferences::default()
        }
        .validate()
        .expect("valid workspace list");
        assert_eq!(preferences.active_workspace_id, "workspace-a");
    }

    #[test]
    fn legacy_profiles_merge_once_per_webview_origin() {
        let current = AppPreferences {
            launch_profiles: vec![LaunchProfile {
                id: "installed".to_string(),
                label: "Installed".to_string(),
                preset: LaunchPreset::Cmd,
                session_name: "Installed".to_string(),
                executable: "cmd.exe".to_string(),
                arguments_text: String::new(),
                working_directory: "D:\\installed".to_string(),
            }],
            migrated_origins: vec!["http://tauri.localhost".to_string()],
            ..AppPreferences::default()
        };
        let legacy = AppPreferences {
            launch_profiles: vec![LaunchProfile {
                id: "development".to_string(),
                label: "Development".to_string(),
                preset: LaunchPreset::PowerShell,
                session_name: "Development".to_string(),
                executable: "pwsh.exe".to_string(),
                arguments_text: String::new(),
                working_directory: "D:\\development".to_string(),
            }],
            ..AppPreferences::default()
        };

        let merged = merge_legacy_preferences(
            true,
            &current,
            legacy.clone(),
            "http://localhost:1420".to_string(),
        )
        .expect("merge legacy preferences");
        let repeated =
            merge_legacy_preferences(true, &merged, legacy, "http://localhost:1420".to_string())
                .expect("skip repeated migration");

        assert_eq!(merged.launch_profiles.len(), 2);
        assert_eq!(repeated.launch_profiles.len(), 2);
        assert_eq!(repeated.migrated_origins.len(), 2);
    }
}
