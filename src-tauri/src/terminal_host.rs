//! Local, authenticated IPC. Only the host owns PTYs and the preferences file.
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{sync_channel, SyncSender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{preferences, private_storage, terminal};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Listener, Manager, State, Window};

const PROTOCOL: u32 = 1;
const MAX_FRAME: u64 = 32 * 1024 * 1024;
const MAX_REQUEST: u64 = 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Serialize, Deserialize)]
struct Endpoint {
    protocol: u32,
    port: u16,
    token: String,
    pid: u32,
}

#[derive(Serialize, Deserialize)]
struct Request {
    protocol: u32,
    token: String,
    command: String,
    args: Value,
}

#[derive(Serialize, Deserialize)]
struct Response {
    value: Option<Value>,
    error: Option<String>,
}

fn read_json<T: serde::de::DeserializeOwned>(
    reader: &mut impl BufRead,
    limit: u64,
) -> Result<T, String> {
    let mut frame = Vec::new();
    reader
        .take(limit + 1)
        .read_until(b'\n', &mut frame)
        .map_err(|e| e.to_string())?;
    if frame.len() as u64 > limit || frame.last() != Some(&b'\n') {
        return Err("Terminal host sent an incomplete or oversized message".into());
    }
    serde_json::from_slice(&frame).map_err(|e| e.to_string())
}

fn write_json(stream: &mut TcpStream, value: &impl Serialize) -> Result<(), String> {
    serde_json::to_writer(&mut *stream, value).map_err(|e| e.to_string())?;
    stream.write_all(b"\n").map_err(|e| e.to_string())
}

fn connect(endpoint: &Endpoint) -> Result<TcpStream, String> {
    if endpoint.protocol != PROTOCOL {
        return Err("Terminal host protocol is incompatible; keep the current interface until its sessions finish".into());
    }
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, endpoint.port));
    let stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
        .map_err(|e| format!("Terminal host unavailable: {e}"))?;
    stream.set_nodelay(true).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;
    Ok(stream)
}

pub struct HostClient {
    endpoint: Endpoint,
    subscribed: Arc<AtomicBool>,
}

impl HostClient {
    fn from_endpoint(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            subscribed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn call(&self, command: &str, args: Value) -> Result<Value, String> {
        let mut stream = connect(&self.endpoint)?;
        write_json(
            &mut stream,
            &Request {
                protocol: PROTOCOL,
                token: self.endpoint.token.clone(),
                command: command.into(),
                args,
            },
        )?;
        let response: Response = read_json(&mut BufReader::new(stream), MAX_FRAME)?;
        match response.error {
            Some(e) => Err(e),
            None => Ok(response.value.unwrap_or(Value::Null)),
        }
    }

    fn subscribe(&self, app: AppHandle) -> Result<(), String> {
        if self.subscribed.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let result = (|| {
            let mut stream = connect(&self.endpoint)?;
            write_json(
                &mut stream,
                &Request {
                    protocol: PROTOCOL,
                    token: self.endpoint.token.clone(),
                    command: "subscribe".into(),
                    args: json!({}),
                },
            )?;
            let mut reader = BufReader::new(stream);
            let ack: Response = read_json(&mut reader, MAX_FRAME)?;
            if let Some(error) = ack.error {
                return Err(error);
            }
            reader
                .get_mut()
                .set_read_timeout(None)
                .map_err(|e| e.to_string())?;
            let subscribed = Arc::clone(&self.subscribed);
            thread::spawn(move || {
                loop {
                    let event: Value = match read_json(&mut reader, MAX_FRAME) {
                        Ok(e) => e,
                        Err(_) => break,
                    };
                    if let Some(name) = event["event"].as_str() {
                        let _ = app.emit(name, &event["payload"]);
                    }
                }
                subscribed.store(false, Ordering::Release);
                let _ = app.emit("terminal-host-disconnected", "The terminal host connection was interrupted. Reload the interface to reconnect; existing tasks may still be running.");
            });
            Ok(())
        })();
        if result.is_err() {
            self.subscribed.store(false, Ordering::Release);
        }
        result
    }

    pub fn ensure(root: &Path) -> Result<Self, String> {
        private_storage::prepare_directory(root)
            .map_err(|e| format!("Unable to protect application data: {e}"))?;
        let endpoint_path = root.join("terminal-host.json");
        for name in [
            "terminal-host.json",
            "terminal-host.json.tmp",
            "preferences.json",
            "preferences.json.tmp",
        ] {
            private_storage::restrict_existing_file(&root.join(name))
                .map_err(|e| format!("Unable to protect {name}: {e}"))?;
        }
        if let Ok(bytes) = fs::read(&endpoint_path) {
            let endpoint: Endpoint = serde_json::from_slice(&bytes).map_err(|_| {
                "Invalid terminal host endpoint; no replacement host was started".to_string()
            })?;
            let client = Self::from_endpoint(endpoint);
            if client.call("ping", json!({})).is_ok() {
                return Ok(client);
            }
            if process_alive(client.endpoint.pid, root) {
                return Err("An existing terminal host is running but cannot be reached or is incompatible. No replacement was started to protect its tasks.".into());
            }
        }
        // A content-addressed private copy stays locked by the host while the installed GUI is replaced.
        let source = std::env::current_exe().map_err(|e| e.to_string())?;
        let bytes = fs::read(&source).map_err(|e| e.to_string())?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        let cache = root.join("terminal-host").join(digest);
        fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
        // NSIS closes processes by the GUI executable name, so the host must use a distinct name.
        let executable = cache.join(if cfg!(windows) {
            "cli-hub-terminal-host.exe"
        } else {
            "cli-hub-terminal-host"
        });
        if !executable.exists() {
            fs::copy(&source, &executable).map_err(|e| e.to_string())?;
        }
        let mut command = Command::new(&executable);
        command
            .arg("--terminal-host")
            .arg(root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // Break away from parent job lifetime, without a console window.
            command.creation_flags(0x0000_0008 | 0x0000_0200 | 0x0100_0000);
        }
        let _child = command
            .spawn()
            .map_err(|e| format!("Unable to detach terminal host: {e}"))?;
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            if let Ok(bytes) = fs::read(&endpoint_path) {
                if let Ok(endpoint) = serde_json::from_slice(&bytes) {
                    let client = Self::from_endpoint(endpoint);
                    if client.call("ping", json!({})).is_ok() {
                        return Ok(client);
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        Err("Terminal host did not become ready. Existing sessions were not stopped.".into())
    }
}

#[cfg(windows)]
fn process_alive(pid: u32, root: &Path) -> bool {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_INVALID_PARAMETER},
        System::Threading::{
            GetExitCodeProcess, OpenProcess, QueryFullProcessImageNameW,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
    };
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return GetLastError() != ERROR_INVALID_PARAMETER;
        }
        let mut code = 0;
        let ok = GetExitCodeProcess(handle, &mut code) != 0;
        // Windows can keep an exited process object alive while another handle is open.
        // Querying its image then fails, so inspect its exit status first.
        if ok && code != 259 {
            CloseHandle(handle);
            return false;
        }
        let mut path = vec![0_u16; 32768];
        let mut length = path.len() as u32;
        let image_ok = QueryFullProcessImageNameW(handle, 0, path.as_mut_ptr(), &mut length) != 0;
        CloseHandle(handle);
        if !ok || !image_ok {
            return true;
        }
        let image = String::from_utf16_lossy(&path[..length as usize]).to_lowercase();
        let prefix = format!("{}\\", root.join("terminal-host").to_string_lossy()).to_lowercase();
        // A reused PID belonging to another application is not our host.
        image.starts_with(&prefix)
    }
}

#[cfg(not(windows))]
fn process_alive(_pid: u32, _root: &Path) -> bool {
    true
}

#[cfg(windows)]
struct HostLock(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl HostLock {
    fn acquire(root: &Path) -> Result<Self, String> {
        use windows_sys::Win32::{
            Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS},
            System::Threading::CreateMutexW,
        };
        let digest = format!("{:x}", Sha256::digest(root.to_string_lossy().as_bytes()));
        let name: Vec<u16> = format!("Local\\CLIHubTerminalHost-{digest}")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        unsafe {
            let handle = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
            if handle.is_null() {
                return Err("Unable to create terminal host singleton lock".into());
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                return Err("Terminal host already running".into());
            }
            Ok(Self(handle))
        }
    }
}
#[cfg(windows)]
impl Drop for HostLock {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

pub fn run(root: PathBuf) {
    #[cfg(windows)]
    let _lock = match HostLock::acquire(&root) {
        Ok(lock) => lock,
        Err(_) => return,
    };
    #[cfg(not(windows))]
    {
        let _ = root;
        return;
    }
    #[cfg(windows)]
    {
        let mut context = tauri::generate_context!();
        context.config_mut().app.windows.clear();
        tauri::Builder::default()
            .manage(Arc::new(terminal::TerminalManager::new()))
            .setup(move |app| {
                app.manage(preferences::PreferencesStore::open(
                    root.join("preferences.json"),
                ));
                // Native hidden window only: no WebView, frontend, tray, updater or single-instance GUI plugin.
                let window = tauri::window::WindowBuilder::new(app, "terminal-host")
                    .visible(false)
                    .build()?;
                start_server(app.handle().clone(), window, root.clone())
                    .map_err(std::io::Error::other)?;
                Ok(())
            })
            .run(context)
            .expect("terminal host failed");
    }
}

type Subscribers = Arc<Mutex<Vec<(usize, SyncSender<Value>)>>>;
static NEXT_SUBSCRIBER: AtomicUsize = AtomicUsize::new(1);
struct Subscription {
    id: usize,
    subscribers: Subscribers,
}
impl Drop for Subscription {
    fn drop(&mut self) {
        if let Ok(mut senders) = self.subscribers.lock() {
            senders.retain(|(id, _)| *id != self.id);
        }
    }
}

fn start_server(app: AppHandle, window: Window, root: PathBuf) -> Result<(), String> {
    private_storage::prepare_directory(&root)
        .map_err(|e| format!("Unable to protect application data: {e}"))?;
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|e| e.to_string())?;
    let mut random = [0_u8; 32];
    getrandom::fill(&mut random).map_err(|e| e.to_string())?;
    let endpoint = Endpoint {
        protocol: PROTOCOL,
        port: listener.local_addr().map_err(|e| e.to_string())?.port(),
        token: random.iter().map(|b| format!("{b:02x}")).collect(),
        pid: std::process::id(),
    };
    let subscribers: Subscribers = Arc::new(Mutex::new(Vec::new()));
    for name in [
        "terminal-output",
        "terminal-exit",
        "session-persistence-error",
    ] {
        let subscribers = Arc::clone(&subscribers);
        app.listen_any(name, move |event| {
            if let Ok(payload) = serde_json::from_str::<Value>(event.payload()) {
                let value = json!({"event": name, "payload": payload});
                if let Ok(mut senders) = subscribers.lock() {
                    // A slow client is disconnected, never blocks PTY output or other clients.
                    senders.retain(|(_, sender)| sender.try_send(value.clone()).is_ok());
                }
            }
        });
    }
    let temporary = root.join("terminal-host.json.tmp");
    let mut file = private_storage::open_for_write(&temporary)
        .map_err(|e| format!("Unable to protect terminal host endpoint: {e}"))?;
    file.write_all(&serde_json::to_vec(&endpoint).map_err(|e| e.to_string())?)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    drop(file);
    if root.join("terminal-host.json").exists() {
        fs::remove_file(root.join("terminal-host.json")).map_err(|e| e.to_string())?;
    }
    fs::rename(temporary, root.join("terminal-host.json")).map_err(|e| e.to_string())?;
    let in_flight = Arc::new(AtomicUsize::new(0));
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            if in_flight.fetch_add(1, Ordering::AcqRel) >= 16 {
                in_flight.fetch_sub(1, Ordering::AcqRel);
                continue;
            }
            let app = app.clone();
            let window = window.clone();
            let endpoint = endpoint.clone();
            let subscribers = Arc::clone(&subscribers);
            let in_flight = Arc::clone(&in_flight);
            thread::spawn(move || {
                let _ = serve_connection(stream, &endpoint, &app, &window, &subscribers);
                in_flight.fetch_sub(1, Ordering::AcqRel);
            });
        }
    });
    Ok(())
}

fn serve_connection(
    mut stream: TcpStream,
    endpoint: &Endpoint,
    app: &AppHandle,
    window: &Window,
    subscribers: &Subscribers,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;
    let request: Request = read_json(
        &mut BufReader::new(stream.try_clone().map_err(|e| e.to_string())?),
        MAX_REQUEST,
    )?;
    if request.token != endpoint.token || request.protocol != PROTOCOL {
        return write_json(
            &mut stream,
            &Response {
                value: None,
                error: Some("Terminal host authentication or protocol rejected".into()),
            },
        );
    }
    if request.command == "subscribe" {
        let (sender, receiver) = sync_channel(64);
        let mut senders = subscribers.lock().map_err(|_| "Event state unavailable")?;
        if senders.len() >= 4 {
            return Err("Too many terminal interface connections".into());
        }
        let id = NEXT_SUBSCRIBER.fetch_add(1, Ordering::Relaxed);
        senders.push((id, sender));
        drop(senders);
        let _subscription = Subscription {
            id,
            subscribers: Arc::clone(subscribers),
        };
        write_json(
            &mut stream,
            &Response {
                value: Some(json!(true)),
                error: None,
            },
        )?;
        // Heartbeats detect closed clients even when their terminals are idle.
        loop {
            let event = match receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(event) => event,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    json!({"event":"terminal-host-heartbeat", "payload":null})
                }
                Err(_) => break,
            };
            write_json(&mut stream, &event)?;
        }
        return Ok(());
    }
    let result = dispatch(app, window, &request.command, request.args);
    let shutdown = request.command == "shutdown_application" && result.is_ok();
    let response = match result {
        Ok(value) => Response {
            value: Some(value),
            error: None,
        },
        Err(error) => Response {
            value: None,
            error: Some(error),
        },
    };
    write_json(&mut stream, &response)?;
    if shutdown {
        app.exit(0);
    }
    Ok(())
}

fn argument<T: serde::de::DeserializeOwned>(args: &Value, name: &str) -> Result<T, String> {
    serde_json::from_value(args.get(name).cloned().unwrap_or(Value::Null))
        .map_err(|e| format!("Invalid {name}: {e}"))
}

fn dispatch(app: &AppHandle, window: &Window, command: &str, args: Value) -> Result<Value, String> {
    let manager = app.state::<Arc<terminal::TerminalManager>>();
    let store = app.state::<preferences::PreferencesStore>();
    match command {
        "ping" => Ok(json!({"protocol": PROTOCOL, "hostVersion": env!("CARGO_PKG_VERSION"), "pid": std::process::id()})),
        "load_app_preferences" => serde_json::to_value(preferences::load_app_preferences(store)?),
        "migrate_legacy_preferences" => serde_json::to_value(preferences::migrate_legacy_preferences(store, argument(&args, "origin")?, argument(&args, "preferences")?)?),
        "save_launch_profiles" => serde_json::to_value(preferences::save_launch_profiles(store, argument(&args, "launchProfiles")?)?),
        "save_recent_directories" => serde_json::to_value(preferences::save_recent_directories(store, argument(&args, "recentDirectories")?)?),
        "save_ui_settings" => serde_json::to_value(preferences::save_ui_settings(store, manager, argument(&args, "uiSettings")?)?),
        "save_workspace_pages" => serde_json::to_value(preferences::save_workspace_pages(store, argument(&args, "workspacePages")?, argument(&args, "activeWorkspaceId")?)?),
        "restore_terminals" => serde_json::to_value(terminal::restore_terminals(window.clone(), manager, store)?),
        "start_terminal" => serde_json::to_value(terminal::start_terminal(window.clone(), manager, argument(&args, "request")?, argument(&args, "cols")?, argument(&args, "rows")?)?),
        "list_active_terminals" => serde_json::to_value(terminal::list_active_terminals(manager)?),
        "write_terminal" => { terminal::write_terminal(manager, argument(&args, "sessionId")?, argument(&args, "data")?)?; Ok(Value::Null) },
        "resize_terminal" => { terminal::resize_terminal(manager, argument(&args, "sessionId")?, argument(&args, "cols")?, argument(&args, "rows")?)?; Ok(Value::Null) },
        "stop_terminal" => { terminal::stop_terminal(manager, argument(&args, "sessionId")?)?; Ok(Value::Null) },
        "stop_all_terminals" => serde_json::to_value(terminal::stop_all_terminals(manager)?),
        "stop_workspace_terminals" => serde_json::to_value(terminal::stop_workspace_terminals(manager, argument(&args, "pageId")?)?),
        "shutdown_application" => { terminal::prepare_shutdown(&manager, &store)?; Ok(Value::Null) },
        _ => return Err("Unknown terminal host command".into()),
    }.map_err(|e| e.to_string())
}

// Keep the WebView invoke contract unchanged; temporary environment values cross IPC only in memory.
macro_rules! proxy {
    ($name:ident $(, $arg:ident : $kind:ty)*) => {
        #[tauri::command]
        pub fn $name(client: State<'_, HostClient>, $($arg: $kind),*) -> Result<Value, String> {
            let args = serde_json::Map::from_iter([
                $((camel_case(stringify!($arg)), serde_json::to_value($arg).map_err(|e| e.to_string())?),)*
            ]);
            client.call(stringify!($name), Value::Object(args))
        }
    };
}

fn camel_case(name: &str) -> String {
    let mut upper = false;
    name.chars()
        .filter_map(|ch| {
            if ch == '_' {
                upper = true;
                None
            } else if upper {
                upper = false;
                Some(ch.to_ascii_uppercase())
            } else {
                Some(ch)
            }
        })
        .collect()
}

proxy!(load_app_preferences);
proxy!(migrate_legacy_preferences, origin: String, preferences: Value);
proxy!(save_launch_profiles, launch_profiles: Value);
proxy!(save_recent_directories, recent_directories: Value);
proxy!(save_ui_settings, ui_settings: Value);
proxy!(save_workspace_pages, workspace_pages: Value, active_workspace_id: String);
proxy!(start_terminal, request: Value, cols: Option<u16>, rows: Option<u16>);
proxy!(list_active_terminals);
proxy!(write_terminal, session_id: u64, data: String);
proxy!(resize_terminal, session_id: u64, cols: u16, rows: u16);
proxy!(stop_terminal, session_id: u64);
proxy!(stop_all_terminals);
proxy!(stop_workspace_terminals, page_id: String);

#[tauri::command]
pub fn restore_terminals(app: AppHandle, client: State<'_, HostClient>) -> Result<Value, String> {
    client.subscribe(app)?;
    client.call("restore_terminals", json!({}))
}

#[tauri::command]
pub fn prepare_interface_update(client: State<'_, HostClient>) -> Result<Value, String> {
    client.call("ping", json!({}))
}

#[tauri::command]
pub fn shutdown_application(app: AppHandle, client: State<'_, HostClient>) -> Result<(), String> {
    client.call("shutdown_application", json!({}))?;
    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn framing_rejects_truncated_and_oversized_messages() {
        assert!(read_json::<Value>(&mut BufReader::new(&b"{}"[..]), 10).is_err());
        assert!(read_json::<Value>(&mut BufReader::new(&b"{}\n"[..]), 2).is_err());
        assert_eq!(
            read_json::<Value>(&mut BufReader::new(&b"{}\n"[..]), 3).unwrap(),
            json!({})
        );
    }
    #[test]
    fn incompatible_endpoint_is_rejected_before_connecting() {
        assert!(connect(&Endpoint {
            protocol: PROTOCOL + 1,
            port: 0,
            token: String::new(),
            pid: 0
        })
        .unwrap_err()
        .contains("incompatible"));
    }
    #[test]
    fn proxy_keeps_frontend_argument_names() {
        assert_eq!(camel_case("active_workspace_id"), "activeWorkspaceId");
        assert_eq!(camel_case("session_id"), "sessionId");
    }

    #[cfg(windows)]
    #[test]
    fn an_unrelated_live_pid_does_not_block_startup_after_pid_reuse() {
        let root = std::env::temp_dir().join("cli-hub-unused-host-root");
        assert!(!process_alive(std::process::id(), &root));
        assert!(!process_alive(u32::MAX, &root));
    }

    #[cfg(windows)]
    #[test]
    fn an_exited_pid_with_a_retained_handle_does_not_block_startup() {
        let root = std::env::temp_dir().join("cli-hub-unused-host-root");
        let mut child = Command::new("cmd.exe")
            .args(["/C", "exit", "0"])
            .spawn()
            .unwrap();
        child.wait().unwrap();
        // Keep Child's process handle open: Windows may still resolve this exited PID.
        assert!(!process_alive(child.id(), &root));
    }

    #[test]
    fn idle_subscriber_disconnect_releases_its_slot() {
        let subscribers: Subscribers = Arc::new(Mutex::new(Vec::new()));
        for id in 0..8 {
            let (sender, _receiver) = sync_channel(64);
            subscribers.lock().unwrap().push((id, sender));
            let subscription = Subscription {
                id,
                subscribers: Arc::clone(&subscribers),
            };
            drop(subscription);
            assert!(subscribers.lock().unwrap().is_empty());
        }
    }
}
