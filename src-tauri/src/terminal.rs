use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::{OsStr, OsString},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{sync_channel, Receiver},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use crate::preferences::PreferencesStore;
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State, Window};

const DEFAULT_COLS: u16 = 100;
const DEFAULT_ROWS: u16 = 30;
const MAX_SESSIONS_PER_WORKSPACE: usize = 8;
const MAX_TOTAL_SESSIONS: usize = 24;
const OUTPUT_QUEUE_CAPACITY: usize = 64;
const OUTPUT_BATCH_MAX_BYTES: usize = 32 * 1024;
const OUTPUT_BATCH_WAIT: Duration = Duration::from_millis(8);
const OUTPUT_HISTORY_MAX_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LaunchPreset {
    PowerShell,
    WindowsPowerShell,
    Cmd,
    Codex,
    Claude,
    Custom,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentVariable {
    name: String,
    value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    pub(crate) page_id: String,
    preset: LaunchPreset,
    name: String,
    executable: String,
    #[serde(default)]
    arguments: Vec<String>,
    working_directory: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing)]
    environment: Vec<EnvironmentVariable>,
}

#[derive(Debug)]
struct ResolvedLaunch {
    name: String,
    executable: PathBuf,
    arguments: Vec<OsString>,
    working_directory: PathBuf,
    environment: Vec<(OsString, OsString)>,
}

struct ActiveTerminal {
    launch_request: LaunchRequest,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
    process_id: Option<u32>,
    page_id: String,
    preset: LaunchPreset,
    name: String,
    working_directory: String,
    working_directory_key: String,
    output_history: OutputHistory,
    child_exited: Arc<AtomicBool>,
}

#[derive(Default)]
struct OutputHistory {
    bytes: VecDeque<u8>,
    start_offset: u64,
    next_offset: u64,
    truncated: bool,
}

impl OutputHistory {
    fn append(&mut self, bytes: &[u8]) -> u64 {
        let offset = self.next_offset;
        self.next_offset = self.next_offset.saturating_add(bytes.len() as u64);
        self.bytes.extend(bytes);

        let overflow = self.bytes.len().saturating_sub(OUTPUT_HISTORY_MAX_BYTES);
        if overflow > 0 {
            self.bytes.drain(..overflow);
            self.start_offset = self.start_offset.saturating_add(overflow as u64);
            self.truncated = true;
        }

        offset
    }

    fn bytes(&self) -> Vec<u8> {
        self.bytes.iter().copied().collect()
    }
}

pub struct TerminalManager {
    active: Mutex<HashMap<u64, ActiveTerminal>>,
    next_id: AtomicU64,
    restore_started: AtomicBool,
    preserving_sessions: AtomicBool,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            active: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            restore_started: AtomicBool::new(false),
            preserving_sessions: AtomicBool::new(false),
        }
    }

    fn remove(&self, session_id: u64) {
        let mut active = self.active.lock().expect("terminal state lock poisoned");
        active.remove(&session_id);
    }

    fn record_output(&self, session_id: u64, bytes: &[u8]) -> Option<u64> {
        let mut active = self.active.lock().ok()?;
        Some(active.get_mut(&session_id)?.output_history.append(bytes))
    }

    pub fn active_count(&self) -> usize {
        // Fail closed: an unavailable session map must not allow a potentially destructive exit.
        self.active.lock().map(|active| active.len()).unwrap_or(1)
    }

    pub(crate) fn persist_sessions(&self, store: &PreferencesStore) -> Result<(), String> {
        let active = self
            .active
            .lock()
            .map_err(|_| "Terminal state is unavailable")?;
        if self.preserving_sessions.load(Ordering::Acquire) {
            return Ok(());
        }
        let mut sessions: Vec<_> = active.iter().collect();
        sessions.sort_by_key(|(id, _)| **id);
        store.save_restore_sessions(
            sessions
                .into_iter()
                .map(|(_, session)| session.launch_request.clone())
                .collect(),
        )
    }
}

pub fn restore_terminals(
    window: Window,
    manager: State<'_, Arc<TerminalManager>>,
    store: State<'_, PreferencesStore>,
) -> Result<Vec<String>, String> {
    if manager.restore_started.swap(true, Ordering::AcqRel) || manager.active_count() > 0 {
        return Ok(vec![]);
    }
    let requests = store.restore_sessions()?;
    manager.preserving_sessions.store(true, Ordering::Release);
    let mut errors = Vec::new();
    for request in requests {
        let name = request.name.clone();
        if let Err(error) = start_terminal(window.clone(), manager.clone(), request, None, None) {
            errors.push(format!("{name}: {error}"));
        }
    }
    manager.preserving_sessions.store(false, Ordering::Release);
    manager.persist_sessions(&store)?;
    Ok(errors)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTerminalResponse {
    session_id: u64,
    process_id: Option<u32>,
    page_id: String,
    preset: LaunchPreset,
    name: String,
    working_directory: String,
    working_directory_key: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTerminalSnapshot {
    session_id: u64,
    process_id: Option<u32>,
    page_id: String,
    preset: LaunchPreset,
    name: String,
    working_directory: String,
    working_directory_key: String,
    output: Vec<u8>,
    output_start_offset: u64,
    next_output_offset: u64,
    output_truncated: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalOutput {
    session_id: u64,
    offset: u64,
    bytes: Vec<u8>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalExit {
    session_id: u64,
    exit_code: Option<u32>,
    error: Option<String>,
}

pub fn start_terminal(
    window: Window,
    manager: State<'_, Arc<TerminalManager>>,
    request: LaunchRequest,
    cols: Option<u16>,
    rows: Option<u16>,
) -> Result<StartTerminalResponse, String> {
    let mut active = manager
        .active
        .lock()
        .map_err(|_| "Terminal state is unavailable".to_string())?;

    let page_id = validate_page_id(&request.page_id)?;
    let page_active_count = active
        .values()
        .filter(|session| session.page_id == page_id)
        .count();
    ensure_session_capacity(active.len(), page_active_count)?;

    let size = sanitize_size(cols, rows);
    let pty_pair = native_pty_system()
        .openpty(size)
        .map_err(|error| format!("Unable to create terminal: {error}"))?;

    let mut launch_request = request.clone();
    launch_request.environment.clear();
    let preset = request.preset;
    let resolved = resolve_launch(request)?;
    let name = resolved.name.clone();
    let working_directory = resolved.working_directory.to_string_lossy().into_owned();
    let working_directory_key = directory_group_key(&resolved.working_directory);
    launch_request.working_directory = Some(working_directory.clone());
    let command = build_command(resolved);

    let mut child = pty_pair
        .slave
        .spawn_command(command)
        .map_err(|error| format!("Unable to start {name}: {error}"))?;
    let process_id = child.process_id();
    let killer = child.clone_killer();
    let reader = pty_pair
        .master
        .try_clone_reader()
        .map_err(|error| format!("Unable to read terminal output: {error}"))?;
    let writer = pty_pair
        .master
        .take_writer()
        .map_err(|error| format!("Unable to open terminal input: {error}"))?;

    let session_id = manager.next_id.fetch_add(1, Ordering::Relaxed);
    let child_exited = Arc::new(AtomicBool::new(false));
    active.insert(
        session_id,
        ActiveTerminal {
            launch_request,
            master: pty_pair.master,
            writer,
            killer,
            process_id,
            page_id: page_id.clone(),
            preset,
            name: name.clone(),
            working_directory: working_directory.clone(),
            working_directory_key: working_directory_key.clone(),
            output_history: OutputHistory::default(),
            child_exited: Arc::clone(&child_exited),
        },
    );
    drop(active);

    if let Err(error) = manager.persist_sessions(&window.state::<PreferencesStore>()) {
        let _ = window.emit("session-persistence-error", error);
    }

    let output_pipeline = spawn_output_pipeline(
        window.clone(),
        Arc::clone(manager.inner()),
        session_id,
        reader,
    );

    let manager = Arc::clone(manager.inner());
    thread::spawn(move || {
        let exit_event = match child.wait() {
            Ok(status) => TerminalExit {
                session_id,
                exit_code: Some(status.exit_code()),
                error: None,
            },
            Err(error) => TerminalExit {
                session_id,
                exit_code: None,
                error: Some(error.to_string()),
            },
        };

        child_exited.store(true, Ordering::Release);
        manager.remove(session_id);
        if let Err(error) = manager.persist_sessions(&window.state::<PreferencesStore>()) {
            let _ = window.emit("session-persistence-error", error);
        }
        let _ = output_pipeline.join();
        let _ = window.emit("terminal-exit", exit_event);
    });

    Ok(StartTerminalResponse {
        session_id,
        process_id,
        page_id,
        preset,
        name,
        working_directory,
        working_directory_key,
    })
}

pub fn list_active_terminals(
    manager: State<'_, Arc<TerminalManager>>,
) -> Result<Vec<ActiveTerminalSnapshot>, String> {
    let active = manager
        .active
        .lock()
        .map_err(|_| "Terminal state is unavailable".to_string())?;
    let mut snapshots = active
        .iter()
        .map(|(session_id, session)| ActiveTerminalSnapshot {
            session_id: *session_id,
            process_id: session.process_id,
            page_id: session.page_id.clone(),
            preset: session.preset,
            name: session.name.clone(),
            working_directory: session.working_directory.clone(),
            working_directory_key: session.working_directory_key.clone(),
            output: session.output_history.bytes(),
            output_start_offset: session.output_history.start_offset,
            next_output_offset: session.output_history.next_offset,
            output_truncated: session.output_history.truncated,
        })
        .collect::<Vec<_>>();
    snapshots.sort_by_key(|session| session.session_id);
    Ok(snapshots)
}

pub fn write_terminal(
    manager: State<'_, Arc<TerminalManager>>,
    session_id: u64,
    data: String,
) -> Result<(), String> {
    let mut active = manager
        .active
        .lock()
        .map_err(|_| "Terminal state is unavailable".to_string())?;
    let session = current_session(&mut active, session_id)?;

    session
        .writer
        .write_all(data.as_bytes())
        .and_then(|_| session.writer.flush())
        .map_err(|error| format!("Unable to write to terminal: {error}"))
}

pub fn resize_terminal(
    manager: State<'_, Arc<TerminalManager>>,
    session_id: u64,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut active = manager
        .active
        .lock()
        .map_err(|_| "Terminal state is unavailable".to_string())?;
    let session = current_session(&mut active, session_id)?;

    session
        .master
        .resize(sanitize_size(Some(cols), Some(rows)))
        .map_err(|error| format!("Unable to resize terminal: {error}"))
}

pub fn stop_terminal(
    manager: State<'_, Arc<TerminalManager>>,
    session_id: u64,
) -> Result<(), String> {
    let stop_result = {
        let mut active = manager
            .active
            .lock()
            .map_err(|_| "Terminal state is unavailable".to_string())?;
        let session = current_session(&mut active, session_id)?;

        session.killer.kill()
    };

    #[cfg(windows)]
    {
        confirm_windows_stop(stop_result, || {
            let active = manager
                .active
                .lock()
                .map_err(|_| "Terminal state is unavailable".to_string())?;
            Ok(active
                .get(&session_id)
                .map_or(true, |session| session.child_exited.load(Ordering::Acquire)))
        })
    }

    #[cfg(not(windows))]
    {
        stop_result.map_err(|error| format!("Unable to stop terminal: {error}"))
    }
}

pub fn stop_all_terminals(manager: State<'_, Arc<TerminalManager>>) -> Result<usize, String> {
    stop_all(&manager)
}

pub fn stop_workspace_terminals(
    manager: State<'_, Arc<TerminalManager>>,
    page_id: String,
) -> Result<usize, String> {
    let page_id = validate_page_id(&page_id)?;
    stop_matching(&manager, |session| session.page_id == page_id)
}

pub(crate) fn prepare_shutdown(
    manager: &Arc<TerminalManager>,
    store: &PreferencesStore,
) -> Result<(), String> {
    manager.persist_sessions(store)?;
    manager.preserving_sessions.store(true, Ordering::Release);
    if let Err(error) = stop_all(&manager) {
        manager.preserving_sessions.store(false, Ordering::Release);
        return Err(error);
    }
    Ok(())
}

fn stop_all(manager: &Arc<TerminalManager>) -> Result<usize, String> {
    stop_matching(manager, |_| true)
}

fn stop_matching(
    manager: &Arc<TerminalManager>,
    predicate: impl Fn(&ActiveTerminal) -> bool,
) -> Result<usize, String> {
    let (session_ids, stop_result) = {
        let mut active = manager
            .active
            .lock()
            .map_err(|_| "Terminal state is unavailable".to_string())?;
        let session_ids = active
            .iter()
            .filter_map(|(session_id, session)| predicate(session).then_some(*session_id))
            .collect::<Vec<_>>();
        let mut first_error = None;

        for session_id in &session_ids {
            if let Some(session) = active.get_mut(session_id) {
                if let Err(error) = session.killer.kill() {
                    first_error.get_or_insert(error);
                }
            }
        }

        let stop_result = first_error.map_or(Ok(()), Err);
        (session_ids, stop_result)
    };

    if session_ids.is_empty() {
        return Ok(0);
    }

    #[cfg(windows)]
    confirm_windows_stop(stop_result, || {
        let active = manager
            .active
            .lock()
            .map_err(|_| "Terminal state is unavailable".to_string())?;
        Ok(session_ids.iter().all(|session_id| {
            active
                .get(session_id)
                .map_or(true, |session| session.child_exited.load(Ordering::Acquire))
        }))
    })?;

    #[cfg(not(windows))]
    stop_result.map_err(|error| format!("Unable to stop terminals: {error}"))?;

    Ok(session_ids.len())
}

fn validate_page_id(page_id: &str) -> Result<String, String> {
    let page_id = page_id.trim();
    if page_id.is_empty() {
        return Err("Workspace identifier is required".to_string());
    }
    if page_id.len() > 128 || page_id.contains('\0') {
        return Err("Workspace identifier is invalid".to_string());
    }
    Ok(page_id.to_string())
}

fn ensure_session_capacity(total_active: usize, workspace_active: usize) -> Result<(), String> {
    if total_active >= MAX_TOTAL_SESSIONS {
        return Err(format!(
            "The global safety limit of {MAX_TOTAL_SESSIONS} active terminal sessions has been reached"
        ));
    }
    if workspace_active >= MAX_SESSIONS_PER_WORKSPACE {
        return Err(format!(
            "This workspace already has the maximum of {MAX_SESSIONS_PER_WORKSPACE} active terminal sessions"
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn confirm_windows_stop<F>(
    stop_result: std::io::Result<()>,
    mut has_exited: F,
) -> Result<(), String>
where
    F: FnMut() -> Result<bool, String>,
{
    let deadline = Instant::now() + Duration::from_secs(2);

    loop {
        if has_exited()? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return match stop_result {
                Ok(()) => Err(
                    "Unable to stop terminal: process did not exit within two seconds".to_string(),
                ),
                Err(error) => Err(format!("Unable to stop terminal: {error}")),
            };
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn current_session(
    active: &mut HashMap<u64, ActiveTerminal>,
    session_id: u64,
) -> Result<&mut ActiveTerminal, String> {
    active
        .get_mut(&session_id)
        .ok_or_else(|| "The terminal session is no longer active".to_string())
}

fn spawn_output_pipeline(
    window: Window,
    manager: Arc<TerminalManager>,
    session_id: u64,
    mut reader: Box<dyn Read + Send>,
) -> thread::JoinHandle<()> {
    let (sender, receiver) = sync_channel(OUTPUT_QUEUE_CAPACITY);

    thread::spawn(move || {
        let mut buffer = vec![0_u8; 8 * 1024];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    if sender.send(buffer[..count].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    thread::spawn(move || {
        let mut pending = None;
        let mut next_offset = 0_u64;

        while let Some(bytes) = receive_output_batch(&receiver, &mut pending) {
            let offset = next_offset;
            next_offset = next_offset.saturating_add(bytes.len() as u64);
            let _ = manager.record_output(session_id, &bytes);
            if window
                .emit(
                    "terminal-output",
                    TerminalOutput {
                        session_id,
                        offset,
                        bytes,
                    },
                )
                .is_err()
            {
                break;
            }
        }
    })
}

fn receive_output_batch(
    receiver: &Receiver<Vec<u8>>,
    pending: &mut Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    let mut batch = match pending.take() {
        Some(chunk) => chunk,
        None => receiver.recv().ok()?,
    };
    let deadline = Instant::now() + OUTPUT_BATCH_WAIT;

    while batch.len() < OUTPUT_BATCH_MAX_BYTES {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            break;
        };

        match receiver.recv_timeout(remaining) {
            Ok(chunk) if batch.len() + chunk.len() <= OUTPUT_BATCH_MAX_BYTES => {
                batch.extend_from_slice(&chunk);
            }
            Ok(chunk) => {
                *pending = Some(chunk);
                break;
            }
            Err(_) => break,
        }
    }

    Some(batch)
}

fn sanitize_size(cols: Option<u16>, rows: Option<u16>) -> PtySize {
    PtySize {
        cols: cols.unwrap_or(DEFAULT_COLS).max(1),
        rows: rows.unwrap_or(DEFAULT_ROWS).max(1),
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn resolve_launch(request: LaunchRequest) -> Result<ResolvedLaunch, String> {
    let LaunchRequest {
        page_id: _,
        preset,
        name,
        executable,
        arguments,
        working_directory,
        environment,
    } = request;

    if arguments.iter().any(|argument| argument.contains('\0')) {
        return Err("Arguments cannot contain null characters".to_string());
    }

    let name = match name.trim() {
        "" => preset_name(preset).to_string(),
        value => value.to_string(),
    };
    let user_arguments = arguments
        .into_iter()
        .map(OsString::from)
        .collect::<Vec<_>>();
    let (executable, arguments) = match preset {
        LaunchPreset::PowerShell => {
            let mut arguments = vec![OsString::from("-NoLogo"), OsString::from("-NoProfile")];
            arguments.extend(user_arguments);
            (resolve_powershell_program()?, arguments)
        }
        LaunchPreset::WindowsPowerShell => {
            #[cfg(windows)]
            {
                let mut arguments = vec![OsString::from("-NoLogo"), OsString::from("-NoProfile")];
                arguments.extend(user_arguments);
                (resolve_executable("powershell.exe")?, arguments)
            }
            #[cfg(not(windows))]
            {
                return Err("Windows PowerShell is only available on Windows".to_string());
            }
        }
        LaunchPreset::Cmd => {
            #[cfg(windows)]
            {
                (resolve_executable("cmd.exe")?, user_arguments)
            }
            #[cfg(not(windows))]
            {
                return Err("CMD is only available on Windows".to_string());
            }
        }
        LaunchPreset::Codex => resolve_codex_program(user_arguments)?,
        LaunchPreset::Claude => resolve_claude_program(user_arguments)?,
        LaunchPreset::Custom => resolve_custom_program(&executable, user_arguments)?,
    };

    Ok(ResolvedLaunch {
        name,
        executable,
        arguments,
        working_directory: normalize_working_directory(working_directory)?,
        environment: normalize_environment(environment)?,
    })
}

fn build_command(resolved: ResolvedLaunch) -> CommandBuilder {
    let mut command = CommandBuilder::new(resolved.executable);
    command.args(resolved.arguments);
    command.cwd(resolved.working_directory);
    apply_terminal_environment_defaults(&mut command);
    for (name, value) in resolved.environment {
        command.env(name, value);
    }
    command
}

fn apply_terminal_environment_defaults(command: &mut CommandBuilder) {
    if environment_is_missing_empty_or_dumb(command.get_env("TERM")) {
        command.env("TERM", "xterm-256color");
    }
    if environment_is_missing_or_empty(command.get_env("COLORTERM")) {
        command.env("COLORTERM", "truecolor");
    }
}

fn environment_is_missing_empty_or_dumb(value: Option<&OsStr>) -> bool {
    match value {
        None => true,
        Some(value) => value
            .to_str()
            .is_some_and(|value| value.trim().is_empty() || value.eq_ignore_ascii_case("dumb")),
    }
}

fn environment_is_missing_or_empty(value: Option<&OsStr>) -> bool {
    match value {
        None => true,
        Some(value) => value.to_str().is_some_and(|value| value.trim().is_empty()),
    }
}

fn preset_name(preset: LaunchPreset) -> &'static str {
    match preset {
        LaunchPreset::PowerShell => "PowerShell",
        LaunchPreset::WindowsPowerShell => "Windows PowerShell",
        LaunchPreset::Cmd => "Command Prompt",
        LaunchPreset::Codex => "Codex",
        LaunchPreset::Claude => "Claude Code",
        LaunchPreset::Custom => "Custom CLI",
    }
}

fn normalize_environment(
    environment: Vec<EnvironmentVariable>,
) -> Result<Vec<(OsString, OsString)>, String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(environment.len());

    for variable in environment {
        let name = variable.name.trim();
        if name.is_empty() {
            return Err("Environment variable names cannot be empty".to_string());
        }
        if name.contains(['=', '\0']) || variable.value.contains('\0') {
            return Err(format!("Invalid environment variable: {name}"));
        }

        #[cfg(windows)]
        let identity = name.to_uppercase();
        #[cfg(not(windows))]
        let identity = name.to_string();

        if !seen.insert(identity) {
            return Err(format!("Duplicate environment variable: {name}"));
        }
        normalized.push((OsString::from(name), OsString::from(variable.value)));
    }

    Ok(normalized)
}

fn resolve_executable(executable: &str) -> Result<PathBuf, String> {
    let executable = executable.trim();
    if executable.is_empty() {
        return Err("Executable is required for a custom CLI".to_string());
    }
    if executable.contains('\0') {
        return Err("Executable cannot contain null characters".to_string());
    }

    let requested = Path::new(executable);
    let has_path = requested.is_absolute() || requested.components().count() > 1;
    if has_path {
        return find_executable_candidate(requested)
            .ok_or_else(|| format!("Executable not found: {}", requested.display()));
    }

    let Some(path) = std::env::var_os("PATH") else {
        return Err(format!("Executable not found on PATH: {executable}"));
    };
    for directory in std::env::split_paths(&path) {
        if let Some(candidate) = find_executable_candidate(&directory.join(requested)) {
            return Ok(candidate);
        }
    }

    Err(format!("Executable not found on PATH: {executable}"))
}

fn find_executable_candidate(requested: &Path) -> Option<PathBuf> {
    executable_candidates(requested)
        .into_iter()
        .find(|candidate| candidate.is_file())
        .map(normalize_executable_path)
}

fn normalize_executable_path(candidate: PathBuf) -> PathBuf {
    normalize_canonical_path(candidate.canonicalize().unwrap_or(candidate))
}

fn normalize_canonical_path(canonical: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let value = canonical.to_string_lossy();
        if let Some(path) = value.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{path}"));
        }
        if let Some(path) = value.strip_prefix(r"\\?\") {
            return PathBuf::from(path);
        }
    }

    canonical
}

fn executable_candidates(requested: &Path) -> Vec<PathBuf> {
    if requested.extension().is_some() {
        return vec![requested.to_path_buf()];
    }

    #[cfg(windows)]
    {
        let path_extensions =
            std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
        let mut candidates = path_extensions
            .split(';')
            .filter(|extension| !extension.is_empty())
            .map(|extension| requested.with_extension(extension.trim_start_matches('.')))
            .collect::<Vec<_>>();
        candidates.push(requested.to_path_buf());
        candidates
    }

    #[cfg(not(windows))]
    {
        vec![requested.to_path_buf()]
    }
}

#[cfg(windows)]
fn resolve_powershell_program() -> Result<PathBuf, String> {
    resolve_executable("pwsh.exe")
        .or_else(|_| resolve_executable("powershell.exe"))
        .map_err(|_| "Neither PowerShell 7 nor Windows PowerShell was found on PATH".to_string())
}

#[cfg(not(windows))]
fn resolve_powershell_program() -> Result<PathBuf, String> {
    resolve_executable("pwsh")
}

#[cfg(windows)]
fn resolve_codex_program(
    user_arguments: Vec<OsString>,
) -> Result<(PathBuf, Vec<OsString>), String> {
    if let Ok(executable) = resolve_executable("codex.exe") {
        return Ok((executable, user_arguments));
    }

    if let Ok(script) = resolve_executable("codex.ps1") {
        let mut arguments = vec![
            OsString::from("-NoLogo"),
            OsString::from("-NoProfile"),
            OsString::from("-File"),
            script.into_os_string(),
        ];
        arguments.extend(user_arguments);
        return Ok((resolve_powershell_program()?, arguments));
    }

    if let Ok(script) = resolve_executable("codex.cmd") {
        let mut arguments = vec![
            OsString::from("/D"),
            OsString::from("/S"),
            OsString::from("/C"),
            script.into_os_string(),
        ];
        arguments.extend(user_arguments);
        return Ok((resolve_executable("cmd.exe")?, arguments));
    }

    Err("Codex was not found on PATH. Install Codex or use a custom executable path.".to_string())
}

#[cfg(windows)]
fn resolve_claude_program(
    user_arguments: Vec<OsString>,
) -> Result<(PathBuf, Vec<OsString>), String> {
    if let Ok(executable) = resolve_executable("claude.exe") {
        return Ok((executable, user_arguments));
    }

    if let Ok(script) = resolve_executable("claude.ps1") {
        let mut arguments = vec![
            OsString::from("-NoLogo"),
            OsString::from("-NoProfile"),
            OsString::from("-File"),
            script.into_os_string(),
        ];
        arguments.extend(user_arguments);
        return Ok((resolve_powershell_program()?, arguments));
    }

    if let Ok(script) = resolve_executable("claude.cmd") {
        let mut arguments = vec![
            OsString::from("/D"),
            OsString::from("/S"),
            OsString::from("/C"),
            script.into_os_string(),
        ];
        arguments.extend(user_arguments);
        return Ok((resolve_executable("cmd.exe")?, arguments));
    }

    Err(
        "Claude Code was not found on PATH. Install Claude Code or use a custom executable path."
            .to_string(),
    )
}

#[cfg(windows)]
fn resolve_custom_program(
    executable: &str,
    user_arguments: Vec<OsString>,
) -> Result<(PathBuf, Vec<OsString>), String> {
    let executable = resolve_executable(executable)?;
    let extension = executable
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "ps1" => {
            let mut arguments = vec![
                OsString::from("-NoLogo"),
                OsString::from("-NoProfile"),
                OsString::from("-File"),
                executable.into_os_string(),
            ];
            arguments.extend(user_arguments);
            Ok((resolve_powershell_program()?, arguments))
        }
        "cmd" | "bat" => {
            let mut arguments = vec![
                OsString::from("/D"),
                OsString::from("/S"),
                OsString::from("/C"),
                executable.into_os_string(),
            ];
            arguments.extend(user_arguments);
            Ok((resolve_executable("cmd.exe")?, arguments))
        }
        _ => Ok((executable, user_arguments)),
    }
}

#[cfg(not(windows))]
fn resolve_custom_program(
    executable: &str,
    user_arguments: Vec<OsString>,
) -> Result<(PathBuf, Vec<OsString>), String> {
    Ok((resolve_executable(executable)?, user_arguments))
}

#[cfg(not(windows))]
fn resolve_codex_program(
    user_arguments: Vec<OsString>,
) -> Result<(PathBuf, Vec<OsString>), String> {
    Ok((resolve_executable("codex")?, user_arguments))
}

#[cfg(not(windows))]
fn resolve_claude_program(
    user_arguments: Vec<OsString>,
) -> Result<(PathBuf, Vec<OsString>), String> {
    Ok((resolve_executable("claude")?, user_arguments))
}

fn normalize_working_directory(directory: Option<String>) -> Result<PathBuf, String> {
    let path = if let Some(directory) = directory.filter(|value| !value.trim().is_empty()) {
        PathBuf::from(directory.trim())
    } else {
        dirs::home_dir().ok_or_else(|| "Unable to resolve the user home directory".to_string())?
    };

    if !path.is_dir() {
        return Err(format!(
            "Working directory does not exist: {}",
            path.display()
        ));
    }

    path.canonicalize()
        .map(normalize_canonical_path)
        .map_err(|error| format!("Unable to resolve working directory: {error}"))
}

fn directory_group_key(directory: &Path) -> String {
    let key = directory.to_string_lossy().into_owned();

    #[cfg(windows)]
    {
        key.replace('/', "\\").to_lowercase()
    }

    #[cfg(not(windows))]
    {
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    use std::{
        io::Read,
        sync::mpsc::{self, RecvTimeoutError},
        time::{Duration, Instant},
    };

    fn launch_request(preset: LaunchPreset) -> LaunchRequest {
        LaunchRequest {
            page_id: "workspace-default".to_string(),
            preset,
            name: String::new(),
            executable: String::new(),
            arguments: Vec::new(),
            working_directory: None,
            environment: Vec::new(),
        }
    }

    #[test]
    fn output_history_tracks_offsets_and_keeps_only_the_bounded_tail() {
        let mut history = OutputHistory::default();

        assert_eq!(history.append(b"abc"), 0);
        assert_eq!(history.append(b"de"), 3);
        assert_eq!(history.bytes(), b"abcde");
        assert_eq!(history.start_offset, 0);
        assert_eq!(history.next_offset, 5);
        assert!(!history.truncated);

        let oversized = vec![7_u8; OUTPUT_HISTORY_MAX_BYTES + 3];
        assert_eq!(history.append(&oversized), 5);
        assert_eq!(history.bytes.len(), OUTPUT_HISTORY_MAX_BYTES);
        assert_eq!(history.start_offset, 8);
        assert_eq!(history.next_offset, OUTPUT_HISTORY_MAX_BYTES as u64 + 8);
        assert!(history.truncated);
        assert!(history.bytes().iter().all(|byte| *byte == 7));
    }

    #[test]
    fn workspace_identifier_is_required_and_bounded() {
        assert_eq!(
            validate_page_id(" workspace-1 ").expect("valid workspace"),
            "workspace-1"
        );
        assert!(validate_page_id("   ").is_err());
        assert!(validate_page_id(&"x".repeat(129)).is_err());
    }

    #[test]
    fn workspace_and_global_session_limits_are_independent() {
        assert!(ensure_session_capacity(8, 7).is_ok());
        assert!(ensure_session_capacity(8, 8)
            .unwrap_err()
            .contains("workspace"));
        assert!(ensure_session_capacity(24, 0)
            .unwrap_err()
            .contains("global"));
    }

    #[test]
    fn terminal_environment_defaults_replace_inherited_dumb_values() {
        let mut command = CommandBuilder::new("example");
        command.env("TERM", "dumb");
        command.env_remove("COLORTERM");

        apply_terminal_environment_defaults(&mut command);

        assert_eq!(command.get_env("TERM"), Some(OsStr::new("xterm-256color")));
        assert_eq!(command.get_env("COLORTERM"), Some(OsStr::new("truecolor")));
    }

    #[test]
    fn explicit_terminal_environment_overrides_defaults() {
        let resolved = ResolvedLaunch {
            name: "Example".to_string(),
            executable: PathBuf::from("example"),
            arguments: Vec::new(),
            working_directory: PathBuf::from("."),
            environment: vec![
                (OsString::from("TERM"), OsString::from("dumb")),
                (OsString::from("COLORTERM"), OsString::from("custom-color")),
            ],
        };

        let command = build_command(resolved);

        assert_eq!(command.get_env("TERM"), Some(OsStr::new("dumb")));
        assert_eq!(
            command.get_env("COLORTERM"),
            Some(OsStr::new("custom-color"))
        );
    }

    #[cfg(windows)]
    fn powershell_test_command() -> CommandBuilder {
        build_command(
            resolve_launch(launch_request(LaunchPreset::PowerShell))
                .expect("PowerShell should resolve"),
        )
    }

    #[cfg(windows)]
    fn run_one_shot(label: &str, command: CommandBuilder) -> (u32, String) {
        let pty_pair = native_pty_system()
            .openpty(sanitize_size(None, None))
            .expect("PTY should be available on Windows");
        let mut child = pty_pair
            .slave
            .spawn_command(command)
            .expect("one-shot command should start inside the PTY");
        let mut killer = child.clone_killer();
        let mut reader = pty_pair
            .master
            .try_clone_reader()
            .expect("PTY output should be readable");
        let mut writer = pty_pair
            .master
            .take_writer()
            .expect("PTY input should be writable");
        drop(pty_pair.slave);
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut buffer = [0_u8; 4 * 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(count) if sender.send(buffer[..count].to_vec()).is_err() => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        });

        writer
            .write_all(b"\x1b[1;1R")
            .and_then(|_| writer.flush())
            .expect("terminal status response should be writable");

        let deadline = Instant::now() + Duration::from_secs(10);
        let mut output = Vec::new();
        let status = loop {
            while let Ok(chunk) = receiver.try_recv() {
                output.extend_from_slice(&chunk);
            }
            if let Some(status) = child
                .try_wait()
                .expect("one-shot command status should be readable")
            {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = killer.kill();
                panic!(
                    "{label} did not exit within ten seconds. Output: {}",
                    String::from_utf8_lossy(&output)
                );
            }
            thread::sleep(Duration::from_millis(10));
        };

        while let Ok(chunk) = receiver.recv_timeout(Duration::from_millis(20)) {
            output.extend_from_slice(&chunk);
        }

        (
            status.exit_code(),
            String::from_utf8_lossy(&output).into_owned(),
        )
    }

    #[cfg(windows)]
    struct StressSessionResult {
        index: usize,
        marker: String,
        exit_code: u32,
        output: String,
    }

    #[cfg(windows)]
    fn run_interactive_stress_session(
        round: usize,
        index: usize,
        ready_sender: mpsc::Sender<usize>,
        release: Arc<AtomicBool>,
        abort: Arc<AtomicBool>,
    ) -> Result<StressSessionResult, String> {
        let marker = format!("__CLI_HUB_STRESS_{round}_{index}__");
        let expected_output = format!("OUTPUT:{marker}");
        let pty_pair = native_pty_system()
            .openpty(sanitize_size(None, None))
            .map_err(|error| format!("session {index} could not create a PTY: {error}"))?;
        let mut request = launch_request(LaunchPreset::Cmd);
        request.page_id = format!("workspace-{}", index / MAX_SESSIONS_PER_WORKSPACE + 1);
        request.arguments = vec!["/D".to_string(), "/Q".to_string()];
        request.environment = vec![EnvironmentVariable {
            name: "CLI_HUB_STRESS_MARKER".to_string(),
            value: marker.clone(),
        }];
        let command = build_command(
            resolve_launch(request)
                .map_err(|error| format!("session {index} could not resolve CMD: {error}"))?,
        );
        let master = pty_pair.master;
        let slave = pty_pair.slave;
        let mut child = slave
            .spawn_command(command)
            .map_err(|error| format!("session {index} could not start CMD: {error}"))?;
        let mut killer = child.clone_killer();
        let mut reader = match master.try_clone_reader() {
            Ok(reader) => reader,
            Err(error) => {
                let _ = killer.kill();
                return Err(format!(
                    "session {index} could not clone its reader: {error}"
                ));
            }
        };
        let mut writer = match master.take_writer() {
            Ok(writer) => writer,
            Err(error) => {
                let _ = killer.kill();
                return Err(format!(
                    "session {index} could not open its writer: {error}"
                ));
            }
        };
        drop(slave);

        let (output_sender, output_receiver) = mpsc::channel();
        let reader_handle = thread::spawn(move || {
            let mut buffer = [0_u8; 4 * 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(count) if output_sender.send(buffer[..count].to_vec()).is_err() => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        });

        if ready_sender.send(index).is_err() {
            let _ = killer.kill();
            return Err(format!("session {index} could not report readiness"));
        }

        let release_deadline = Instant::now() + Duration::from_secs(15);
        while !release.load(Ordering::Acquire) {
            if abort.load(Ordering::Acquire) || Instant::now() >= release_deadline {
                let _ = killer.kill();
                return Err(format!(
                    "session {index} timed out waiting for the release gate"
                ));
            }
            thread::sleep(Duration::from_millis(2));
        }
        if abort.load(Ordering::Acquire) {
            let _ = killer.kill();
            return Err(format!("session {index} was aborted before interaction"));
        }

        if let Err(error) = master.resize(PtySize {
            rows: 24 + (index % 8) as u16,
            cols: 90 + index as u16,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            let _ = killer.kill();
            return Err(format!("session {index} could not resize its PTY: {error}"));
        }

        if let Err(error) = writer
            .write_all(b"\x1b[1;1R")
            .and_then(|_| writer.write_all(b"echo OUTPUT:%CLI_HUB_STRESS_MARKER%\r\n"))
            .and_then(|_| writer.flush())
        {
            let _ = killer.kill();
            return Err(format!("session {index} could not accept input: {error}"));
        }

        let output_deadline = Instant::now() + Duration::from_secs(10);
        let mut output = Vec::new();
        while Instant::now() < output_deadline {
            match output_receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(chunk) => {
                    output.extend_from_slice(&chunk);
                    if String::from_utf8_lossy(&output).contains(&expected_output) {
                        break;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        if !String::from_utf8_lossy(&output).contains(&expected_output) {
            let _ = killer.kill();
            return Err(format!(
                "session {index} did not emit its marker: {}",
                String::from_utf8_lossy(&output)
            ));
        }

        if let Err(error) = writer.write_all(b"exit\r\n").and_then(|_| writer.flush()) {
            let _ = killer.kill();
            return Err(format!("session {index} could not receive exit: {error}"));
        }

        let exit_deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < exit_deadline => {
                    thread::sleep(Duration::from_millis(10));
                }
                Ok(None) => {
                    let _ = killer.kill();
                    return Err(format!("session {index} did not exit within ten seconds"));
                }
                Err(error) => {
                    let _ = killer.kill();
                    return Err(format!("session {index} exit status failed: {error}"));
                }
            }
        };

        drop(writer);
        drop(master);
        let _ = reader_handle.join();
        while let Ok(chunk) = output_receiver.try_recv() {
            output.extend_from_slice(&chunk);
        }

        Ok(StressSessionResult {
            index,
            marker,
            exit_code: status.exit_code(),
            output: String::from_utf8_lossy(&output).into_owned(),
        })
    }

    #[cfg(windows)]
    fn run_twenty_four_pty_stress_cycle(round: usize) {
        let release = Arc::new(AtomicBool::new(false));
        let abort = Arc::new(AtomicBool::new(false));
        let (ready_sender, ready_receiver) = mpsc::channel();
        let cycle_started = Instant::now();
        let handles = (0..MAX_TOTAL_SESSIONS)
            .map(|index| {
                let ready_sender = ready_sender.clone();
                let release = Arc::clone(&release);
                let abort = Arc::clone(&abort);
                thread::spawn(move || {
                    run_interactive_stress_session(round, index, ready_sender, release, abort)
                })
            })
            .collect::<Vec<_>>();
        drop(ready_sender);

        let ready_deadline = Instant::now() + Duration::from_secs(15);
        let mut ready_sessions = HashSet::new();
        while ready_sessions.len() < MAX_TOTAL_SESSIONS && Instant::now() < ready_deadline {
            match ready_receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(index) => {
                    ready_sessions.insert(index);
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        if ready_sessions.len() != MAX_TOTAL_SESSIONS {
            abort.store(true, Ordering::Release);
        }
        release.store(true, Ordering::Release);

        let outcomes = handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "stress worker panicked".to_string())
                    .and_then(|result| result)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            ready_sessions.len(),
            MAX_TOTAL_SESSIONS,
            "only {} of {} PTYs reached the simultaneous-ready gate",
            ready_sessions.len(),
            MAX_TOTAL_SESSIONS
        );

        let errors = outcomes
            .iter()
            .filter_map(|outcome| outcome.as_ref().err())
            .cloned()
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "stress cycle {round} failed: {errors:?}");
        let results = outcomes.into_iter().map(Result::unwrap).collect::<Vec<_>>();

        for result in &results {
            assert_eq!(
                result.exit_code, 0,
                "session {} exited with {}: {}",
                result.index, result.exit_code, result.output
            );
            assert!(
                result.output.contains(&format!("OUTPUT:{}", result.marker)),
                "session {} lost its marker: {}",
                result.index,
                result.output
            );
            for other in &results {
                if other.index != result.index {
                    assert!(
                        !result.output.contains(&format!("OUTPUT:{}", other.marker)),
                        "session {} received output from session {}: {}",
                        result.index,
                        other.index,
                        result.output
                    );
                }
            }
        }

        eprintln!(
            "stress cycle {round}: {} simultaneous PTYs passed in {:?}",
            results.len(),
            cycle_started.elapsed()
        );
    }

    #[test]
    fn terminal_size_uses_defaults() {
        let size = sanitize_size(None, None);

        assert_eq!(size.cols, DEFAULT_COLS);
        assert_eq!(size.rows, DEFAULT_ROWS);
    }

    #[test]
    fn terminal_size_never_uses_zero() {
        let size = sanitize_size(Some(0), Some(0));

        assert_eq!(size.cols, 1);
        assert_eq!(size.rows, 1);
    }

    #[test]
    fn output_batch_preserves_chunk_order() {
        let (sender, receiver) = sync_channel(4);
        sender.send(b"first".to_vec()).unwrap();
        sender.send(b"-second".to_vec()).unwrap();
        drop(sender);
        let mut pending = None;

        let batch = receive_output_batch(&receiver, &mut pending).unwrap();

        assert_eq!(batch, b"first-second");
        assert!(receive_output_batch(&receiver, &mut pending).is_none());
    }

    #[test]
    fn blank_working_directory_uses_the_user_home() {
        let directory = normalize_working_directory(Some("  ".into())).unwrap();
        let home = dirs::home_dir().expect("the test user should have a home directory");
        let expected = normalize_canonical_path(
            home.canonicalize()
                .expect("the test user home directory should canonicalize"),
        );

        assert_eq!(directory, expected);
    }

    #[cfg(windows)]
    #[test]
    fn windows_working_directory_does_not_keep_the_extended_path_prefix() {
        let current = std::env::current_dir().expect("the test should have a current directory");
        let directory = normalize_working_directory(Some(current.to_string_lossy().into_owned()))
            .expect("the current directory should normalize");

        assert!(!directory.to_string_lossy().starts_with(r"\\?\"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_directory_group_key_is_case_insensitive() {
        assert_eq!(
            directory_group_key(Path::new(r"C:/Projects/CLI-Hub")),
            directory_group_key(Path::new(r"c:\projects\cli-hub"))
        );
    }

    #[test]
    fn custom_launch_preserves_structured_arguments_and_environment() {
        let mut request = launch_request(LaunchPreset::Custom);
        request.name = "Example CLI".to_string();
        request.executable = std::env::current_exe()
            .expect("test executable should have a path")
            .to_string_lossy()
            .into_owned();
        request.arguments = vec![
            "argument with spaces".to_string(),
            "--flag=value".to_string(),
        ];
        request.environment = vec![EnvironmentVariable {
            name: "CLI_HUB_TEST".to_string(),
            value: "value with spaces".to_string(),
        }];

        let resolved = resolve_launch(request).expect("custom launch should resolve");

        assert_eq!(resolved.name, "Example CLI");
        assert_eq!(
            resolved.arguments,
            vec![
                OsString::from("argument with spaces"),
                OsString::from("--flag=value")
            ]
        );
        assert_eq!(
            resolved.environment,
            vec![(
                OsString::from("CLI_HUB_TEST"),
                OsString::from("value with spaces")
            )]
        );
    }

    #[test]
    fn custom_launch_rejects_missing_executable() {
        let request = launch_request(LaunchPreset::Custom);

        assert_eq!(
            resolve_launch(request).unwrap_err(),
            "Executable is required for a custom CLI"
        );
    }

    #[test]
    fn duplicate_environment_variables_are_rejected() {
        let mut request = launch_request(LaunchPreset::Custom);
        request.executable = std::env::current_exe()
            .expect("test executable should have a path")
            .to_string_lossy()
            .into_owned();
        request.environment = vec![
            EnvironmentVariable {
                name: "CLI_HUB_TEST".to_string(),
                value: "first".to_string(),
            },
            EnvironmentVariable {
                name: "CLI_HUB_TEST".to_string(),
                value: "second".to_string(),
            },
        ];

        assert_eq!(
            resolve_launch(request).unwrap_err(),
            "Duplicate environment variable: CLI_HUB_TEST"
        );
    }

    #[test]
    fn missing_working_directory_is_rejected() {
        let mut request = launch_request(LaunchPreset::Custom);
        request.executable = std::env::current_exe()
            .expect("test executable should have a path")
            .to_string_lossy()
            .into_owned();
        request.working_directory = Some("__cli_hub_missing_directory__".to_string());

        let error = resolve_launch(request).unwrap_err();

        assert!(error.starts_with("Working directory does not exist:"));
    }

    #[test]
    fn launch_request_uses_frontend_camel_case_contract() {
        let request: LaunchRequest = serde_json::from_value(serde_json::json!({
            "pageId": "workspace-default",
            "preset": "powerShell",
            "name": "PowerShell",
            "executable": "",
            "arguments": [],
            "workingDirectory": null,
            "environment": []
        }))
        .expect("frontend launch request should deserialize");

        assert_eq!(request.preset, LaunchPreset::PowerShell);
        assert_eq!(request.page_id, "workspace-default");
    }

    #[cfg(windows)]
    #[test]
    fn windows_presets_resolve_installed_programs() {
        for preset in [
            LaunchPreset::PowerShell,
            LaunchPreset::WindowsPowerShell,
            LaunchPreset::Cmd,
            LaunchPreset::Codex,
            LaunchPreset::Claude,
        ] {
            let resolved = resolve_launch(launch_request(preset))
                .unwrap_or_else(|error| panic!("{preset:?} should resolve: {error}"));
            assert!(resolved.executable.is_file());
        }
    }

    #[cfg(windows)]
    #[test]
    fn cmd_codex_and_claude_run_through_real_ptys() {
        let mut cmd_request = launch_request(LaunchPreset::Cmd);
        cmd_request.arguments = vec![
            "/D".to_string(),
            "/C".to_string(),
            "echo".to_string(),
            "__CLI_HUB_CMD__".to_string(),
        ];
        let (cmd_status, cmd_output) = run_one_shot(
            "CMD",
            build_command(resolve_launch(cmd_request).expect("CMD should resolve")),
        );
        assert_eq!(cmd_status, 0, "CMD failed: {cmd_output}");
        assert!(cmd_output.contains("__CLI_HUB_CMD__"));

        let mut codex_request = launch_request(LaunchPreset::Codex);
        codex_request.arguments = vec!["--version".to_string()];
        let (codex_status, codex_output) = run_one_shot(
            "Codex",
            build_command(resolve_launch(codex_request).expect("Codex should resolve")),
        );
        assert_eq!(codex_status, 0, "Codex failed: {codex_output}");
        assert!(
            codex_output.to_lowercase().contains("codex"),
            "Codex version output was unexpected: {codex_output}"
        );

        let mut claude_request = launch_request(LaunchPreset::Claude);
        claude_request.arguments = vec!["--version".to_string()];
        let (claude_status, claude_output) = run_one_shot(
            "Claude Code",
            build_command(resolve_launch(claude_request).expect("Claude Code should resolve")),
        );
        assert_eq!(claude_status, 0, "Claude Code failed: {claude_output}");
        assert!(
            claude_output.to_lowercase().contains("claude"),
            "Claude Code version output was unexpected: {claude_output}"
        );

        let mut custom_request = launch_request(LaunchPreset::Custom);
        custom_request.executable = resolve_executable("codex.cmd")
            .expect("npm Codex CMD launcher should resolve")
            .to_string_lossy()
            .into_owned();
        custom_request.arguments = vec!["--version".to_string()];
        let (custom_status, custom_output) = run_one_shot(
            "custom CMD launcher",
            build_command(resolve_launch(custom_request).expect("custom CMD should resolve")),
        );
        assert_eq!(custom_status, 0, "custom CMD failed: {custom_output}");
        assert!(
            custom_output.to_lowercase().contains("codex"),
            "custom CMD output was unexpected: {custom_output}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn managed_pty_exposes_interactive_terminal_environment() {
        let mut request = launch_request(LaunchPreset::PowerShell);
        request.arguments = vec![
            "-Command".to_string(),
            "Write-Output \"$env:TERM|$env:COLORTERM\"".to_string(),
        ];
        let (status, output) = run_one_shot(
            "terminal environment",
            build_command(resolve_launch(request).expect("PowerShell should resolve")),
        );

        assert_eq!(status, 0, "environment probe failed: {output}");
        assert!(
            output.contains("xterm-256color|truecolor"),
            "managed PTY environment was unexpected: {output}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn eight_concurrent_ptys_keep_output_isolated() {
        let handles = (0..MAX_SESSIONS_PER_WORKSPACE)
            .map(|index| {
                thread::spawn(move || {
                    let marker = format!("__CLI_HUB_SESSION_{index}__");
                    let mut request = launch_request(LaunchPreset::Cmd);
                    request.arguments = vec![
                        "/D".to_string(),
                        "/C".to_string(),
                        "echo".to_string(),
                        marker.clone(),
                    ];
                    let command = build_command(
                        resolve_launch(request).expect("concurrent CMD should resolve"),
                    );
                    let result = run_one_shot(&marker, command);
                    (index, marker, result)
                })
            })
            .collect::<Vec<_>>();

        let results = handles
            .into_iter()
            .map(|handle| handle.join().expect("concurrent PTY test should join"))
            .collect::<Vec<_>>();

        for (index, marker, (status, output)) in &results {
            assert_eq!(*status, 0, "session {index} failed: {output}");
            assert!(
                output.contains(marker),
                "session {index} did not contain its marker: {output}"
            );
            for (other_index, other_marker, _) in &results {
                if other_index != index {
                    assert!(
                        !output.contains(other_marker),
                        "session {index} received output from session {other_index}: {output}"
                    );
                }
            }
        }
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "manual release-gate stress test that starts 240 CMD processes over ten cycles"]
    fn twenty_four_concurrent_ptys_survive_ten_cycles() {
        for round in 1..=10 {
            run_twenty_four_pty_stress_cycle(round);
        }
    }

    #[cfg(windows)]
    #[test]
    fn powershell_round_trip_through_real_pty() {
        let pty_pair = native_pty_system()
            .openpty(sanitize_size(None, None))
            .expect("PTY should be available on Windows");
        let mut child = pty_pair
            .slave
            .spawn_command(powershell_test_command())
            .expect("PowerShell should start inside the PTY");
        let mut reader = pty_pair
            .master
            .try_clone_reader()
            .expect("PTY output should be readable");
        let mut writer = pty_pair
            .master
            .take_writer()
            .expect("PTY input should be writable");
        drop(pty_pair.slave);
        let mut killer = child.clone_killer();
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut buffer = [0_u8; 4 * 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(count) if sender.send(buffer[..count].to_vec()).is_err() => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        });

        writer
            .write_all(b"\x1b[1;1RWrite-Output ('__CLI_' + 'HUB_PTY_RESULT__')")
            .and_then(|_| writer.flush())
            .expect("PowerShell command text should be written to the PTY");
        writer
            .write_all(b"\r")
            .and_then(|_| writer.flush())
            .expect("separate Enter should be written to the PTY");
        writer
            .write_all(
                b"Write-Output (([char]0x4E2D).ToString() + ([char]0x6587).ToString())\r\nexit 0\r\n",
            )
            .and_then(|_| writer.flush())
            .expect("remaining PowerShell commands should be written to the PTY");

        let marker = b"__CLI_HUB_PTY_RESULT__";
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut output = Vec::new();
        let mut exit_status = None;

        while Instant::now() < deadline {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(chunk) => output.extend_from_slice(&chunk),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }

            if exit_status.is_none() {
                exit_status = child
                    .try_wait()
                    .expect("PowerShell status should be readable");
            }
            if exit_status.is_some() && output.windows(marker.len()).any(|part| part == marker) {
                break;
            }
        }

        if exit_status.is_none() {
            let _ = killer.kill();
            exit_status = Some(child.wait().expect("PowerShell should stop after timeout"));
        }
        let output = String::from_utf8_lossy(&output);

        assert_eq!(
            exit_status.expect("PowerShell should exit").exit_code(),
            0,
            "PowerShell did not exit cleanly. Output: {output}"
        );
        assert!(
            output.contains("__CLI_HUB_PTY_RESULT__"),
            "PowerShell output did not include the smoke marker: {output}"
        );
        assert!(
            output.contains("中文"),
            "PowerShell output did not preserve Unicode text: {output}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn cmd_preserves_the_requested_working_directory() {
        let project_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("the Rust crate should be inside the project directory")
            .to_path_buf();
        let mut request = launch_request(LaunchPreset::Cmd);
        request.working_directory = Some(project_directory.to_string_lossy().into_owned());
        request.arguments = vec![
            "/D".to_string(),
            "/Q".to_string(),
            "/C".to_string(),
            "echo __CLI_HUB_CWD__%CD%".to_string(),
        ];
        let resolved = resolve_launch(request).expect("CMD launch should resolve");
        let expected = format!(
            "__CLI_HUB_CWD__{}",
            resolved.working_directory.to_string_lossy()
        );

        let (exit_code, output) =
            run_one_shot("CMD working-directory smoke test", build_command(resolved));

        assert_eq!(exit_code, 0, "CMD did not exit cleanly: {output}");
        assert!(
            output.to_lowercase().contains(&expected.to_lowercase()),
            "CMD did not retain the requested working directory. Expected {expected:?}, output: {output}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn cmd_without_a_working_directory_starts_in_the_user_home() {
        let mut request = launch_request(LaunchPreset::Cmd);
        request.arguments = vec![
            "/D".to_string(),
            "/Q".to_string(),
            "/C".to_string(),
            "echo __CLI_HUB_HOME__%CD%".to_string(),
        ];
        let resolved = resolve_launch(request).expect("CMD launch should resolve");
        let expected = format!(
            "__CLI_HUB_HOME__{}",
            resolved.working_directory.to_string_lossy()
        );

        let (exit_code, output) =
            run_one_shot("CMD home-directory smoke test", build_command(resolved));

        assert_eq!(exit_code, 0, "CMD did not exit cleanly: {output}");
        assert!(
            output.to_lowercase().contains(&expected.to_lowercase()),
            "CMD did not start in the user home directory. Expected {expected:?}, output: {output}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn cloned_killer_stops_a_real_powershell_process_without_false_error() {
        let pty_pair = native_pty_system()
            .openpty(sanitize_size(None, None))
            .expect("PTY should be available on Windows");
        let mut child = pty_pair
            .slave
            .spawn_command(powershell_test_command())
            .expect("PowerShell should start inside the PTY");
        let mut killer = child.clone_killer();
        let mut reader = pty_pair
            .master
            .try_clone_reader()
            .expect("PowerShell output should be readable");
        drop(pty_pair.slave);

        let child_exited = Arc::new(AtomicBool::new(false));
        let wait_child_exited = Arc::clone(&child_exited);
        let (cleanup_sender, cleanup_receiver) = mpsc::sync_channel(0);
        let (reader_done_sender, reader_done_receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut buffer = [0_u8; 1024];
            while matches!(reader.read(&mut buffer), Ok(count) if count > 0) {}
            let _ = reader_done_sender.send(());
        });
        let wait_handle = thread::spawn(move || {
            let status = child.wait().expect("PowerShell exit should be observed");
            wait_child_exited.store(true, Ordering::Release);
            cleanup_receiver
                .recv()
                .expect("test should release delayed session cleanup");
            status
        });

        let stop_result = killer.kill();
        let confirmation =
            confirm_windows_stop(stop_result, || Ok(child_exited.load(Ordering::Acquire)));
        drop(pty_pair.master);
        reader_done_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("releasing the PTY master should let output cleanup finish");
        cleanup_sender
            .send(())
            .expect("test should unblock delayed session cleanup");
        confirmation.expect("Stop should succeed before output cleanup removes the session");
        wait_handle
            .join()
            .expect("PowerShell wait thread should finish after cleanup is released");
    }
}
