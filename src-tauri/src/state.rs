use crate::ipmsg_core::{detect_self_addr, start_ipmsg, Event, protocol::PORT};
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

#[derive(Clone, Serialize, Deserialize)]
pub struct OnlineUser {
    pub name: String,
    pub group: String,
    pub host: String,
    pub addr: SocketAddr,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub packet_no: u32,
    pub file_id: u32,
    pub name: String,
    pub size: u64,
    pub saved: bool,
    pub received: u64,
    pub is_dir: bool,
    pub local_path: Option<String>,
    pub current_file: Option<String>,
    #[serde(default)]
    pub error: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub from: SocketAddr,
    pub to: SocketAddr,
    pub is_me: bool,
    pub text: String,
    pub time: String,
    pub file: Option<FileInfo>,
}

#[derive(Clone)]
pub struct CoreState {
    pub online_users: HashMap<SocketAddr, OnlineUser>,
    pub messages: Vec<ChatMessage>,
    pub unread_counts: HashMap<SocketAddr, u32>,
    pub self_addr: Option<SocketAddr>,
}

impl CoreState {
    fn new() -> Self {
        Self {
            online_users: HashMap::new(),
            messages: Vec::new(),
            unread_counts: HashMap::new(),
            self_addr: None,
        }
    }
}

// Global state managed by Tauri, but we use Lazy static for compatibility with existing logic structure
// In a pure Tauri app, we should put this in AppHandle::manage
pub static STATE_SNAPSHOT: Lazy<Mutex<CoreState>> = Lazy::new(|| Mutex::new(CoreState::new()));
static STATE_CMD_TX: OnceCell<mpsc::Sender<StateCmd>> = OnceCell::new();

fn state_cmd_tx() -> &'static mpsc::Sender<StateCmd> {
    STATE_CMD_TX.get().expect("STATE_CMD_TX not initialized")
}

fn normalize_addr(addr: SocketAddr) -> SocketAddr {
    addr
}

fn default_self_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), PORT)
}

pub enum StateCmd {
    InitSelf {
        user: String,
        group: String,
        host: String,
        addr: SocketAddr,
    },
    ApplyEvent(Event),
    PushOutgoing(ChatMessage),
    UpdateProgress {
        file_id: u32,
        packet_no: u32,
        progress: u64,
        file_name: Option<String>,
        saved: Option<bool>,
        error: Option<bool>,
    },
    ClearUnread {
        addr: SocketAddr,
    },
}

pub async fn run_state_manager(mut rx: mpsc::Receiver<StateCmd>, app_handle: tauri::AppHandle) {
    let mut state = CoreState::new();
    
    // Load history
    let history_path = app_handle.path().app_data_dir().unwrap_or_else(|_| PathBuf::from(".")).join("history.json");
    if let Ok(content) = fs::read_to_string(&history_path) {
        if let Ok(msgs) = serde_json::from_str::<Vec<ChatMessage>>(&content) {
            state.messages = msgs;
        }
    }

    while let Some(cmd) = rx.recv().await {
        let mut changed = false;
        match cmd {
            StateCmd::InitSelf { user, group, host, addr } => {
                let addr_norm = normalize_addr(addr);
                state.self_addr = Some(addr_norm);
                state.online_users.insert(
                    addr_norm,
                    OnlineUser {
                        name: user,
                        group,
                        host,
                        addr: addr_norm,
                    },
                );
                changed = true;
            }
            StateCmd::ApplyEvent(ev) => match ev {
                Event::Online { user, group, host, addr } => {
                    let addr_norm = normalize_addr(addr);
                    if Some(addr_norm) == state.self_addr {
                        // ignore self
                    } else {
                        state.online_users.insert(
                            addr_norm,
                            OnlineUser {
                                name: user,
                                group,
                                host,
                                addr: addr_norm,
                            },
                        );
                        changed = true;
                    }
                }
                Event::Offline { addr, .. } => {
                    let addr_norm = normalize_addr(addr);
                    if state.online_users.remove(&addr_norm).is_some() {
                        changed = true;
                    }
                }
                Event::Message {
                    from,
                    user,
                    host,
                    text,
                } => {
                    let from_norm = normalize_addr(from);
                    let to = state.self_addr.unwrap_or_else(default_self_addr);
                    state.messages.push(ChatMessage {
                        from: from_norm,
                        to,
                        is_me: false,
                        text,
                        time: "现在".into(),
                        file: None,
                    });
                    state.online_users.entry(from_norm).or_insert(OnlineUser {
                        name: user,
                        group: String::new(),
                        host,
                        addr: from_norm,
                    });
                    *state.unread_counts.entry(from_norm).or_insert(0) += 1;
                    changed = true;
                }
                Event::FileOffer {
                    from,
                    user,
                    host,
                    packet_no,
                    file_id,
                    name,
                    size,
                    is_dir,
                } => {
                    let from_norm = normalize_addr(from);
                    let to = state.self_addr.unwrap_or_else(default_self_addr);
                    let text = if is_dir {
                        format!("[文件夹] {}", name)
                    } else {
                        format!("[文件] {} ({})", name, format_size(size))
                    };
                    state.messages.push(ChatMessage {
                        from: from_norm,
                        to,
                        is_me: false,
                        text,
                        time: "现在".into(),
                        file: Some(FileInfo {
                            packet_no,
                            file_id,
                            name,
                            size,
                            saved: false,
                            received: 0,
                            is_dir,
                            local_path: None,
                            current_file: None,
                            error: false,
                        }),
                    });
                    state.online_users.entry(from_norm).or_insert(OnlineUser {
                        name: user,
                        group: String::new(),
                        host,
                        addr: from_norm,
                    });
                    *state.unread_counts.entry(from_norm).or_insert(0) += 1;
                    changed = true;
                }
                _ => {}
            },
            StateCmd::PushOutgoing(msg) => {
                state.messages.push(msg);
                changed = true;
            }
            StateCmd::UpdateProgress {
                file_id,
                packet_no,
                progress,
                file_name,
                saved,
                error,
            } => {
                for m in state.messages.iter_mut().rev() {
                    if let Some(f) = &mut m.file {
                        if f.packet_no == packet_no && f.file_id == file_id {
                            f.received = progress;
                            if let Some(name) = file_name {
                                f.current_file = Some(name);
                            }
                            if let Some(s) = saved {
                                f.saved = s;
                            }
                            if let Some(e) = error {
                                f.error = e;
                            }
                            break;
                        }
                    }
                }
                changed = true;
            }
            StateCmd::ClearUnread { addr } => {
                let addr_norm = normalize_addr(addr);
                if state.unread_counts.remove(&addr_norm).is_some() {
                    changed = true;
                }
            }
        }

        if changed {
            {
                let mut snap = STATE_SNAPSHOT.lock().unwrap();
                *snap = state.clone();
            }
            // Save history
            if let Ok(json) = serde_json::to_string(&state.messages) {
                if let Some(parent) = history_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&history_path, json);
            }

            // Emit event to frontend
            let _ = app_handle.emit("state-changed", ());
        }
    }
}

pub fn format_size(size: u64) -> String {
    let s = size as f64;
    if s < 1024.0 {
        format!("{:.0} B", s)
    } else if s < 1024.0 * 1024.0 {
        format!("{:.1} KB", s / 1024.0)
    } else if s < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB", s / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", s / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn init_state(app_handle: tauri::AppHandle) {
    let (tx, rx) = mpsc::channel(1024);
    let _ = STATE_CMD_TX.set(tx);

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let handle_for_manager = handle.clone();
        tauri::async_runtime::spawn(run_state_manager(rx, handle_for_manager));

        match start_ipmsg().await {
            Ok((rx, port)) => {
                let mut ipmsg_rx: tokio::sync::broadcast::Receiver<Event> = rx;
                
                // Init self with actual port
                let self_name = whoami::username();
                let self_host = hostname::get()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| "host".into());
                let self_addr = detect_self_addr(port)
                    .unwrap_or_else(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port));
                    
                let _ = state_cmd_tx().try_send(StateCmd::InitSelf {
                    user: self_name,
                    group: "Main".to_string(),
                    host: self_host,
                    addr: self_addr,
                });
                
                loop {
                    match ipmsg_rx.recv().await {
                        Ok(ev) => {
                            let _ = state_cmd_tx().try_send(StateCmd::ApplyEvent(ev));
                        },
                        Err(_) => break,
                    }
                }
            }
            Err(e) => {
                println!("UDP 服务启动失败: {}", e);
                let _ = handle.dialog()
                    .message(format!("UDP 服务启动失败: {}\n端口 2425 可能被占用。\n请关闭其他占用该端口的程序后重试。", e))
                    .kind(MessageDialogKind::Error)
                    .title("启动错误")
                    .blocking_show();
                std::process::exit(1);
            }
        }
    });
}

pub fn dispatch_cmd(cmd: StateCmd) {
    if let Err(e) = state_cmd_tx().try_send(cmd) {
        log::warn!("Failed to dispatch command: {}", e);
    }
}

pub fn list_online_users() -> Vec<OnlineUser> {
    let state = STATE_SNAPSHOT.lock().unwrap();
    let mut users: Vec<OnlineUser> = state.online_users.values().cloned().collect();
    users.sort_by(|a, b| a.addr.cmp(&b.addr));
    users
}

pub fn list_messages() -> Vec<ChatMessage> {
    let state = STATE_SNAPSHOT.lock().unwrap();
    state.messages.clone()
}

#[allow(dead_code)]
pub fn get_unread_count(addr: SocketAddr) -> u32 {
    let state = STATE_SNAPSHOT.lock().unwrap();
    state.unread_counts.get(&addr).copied().unwrap_or(0)
}

pub fn get_self_addr_info() -> Option<OnlineUser> {
    let state = STATE_SNAPSHOT.lock().unwrap();
    if let Some(addr) = state.self_addr {
        state.online_users.get(&addr).cloned()
    } else {
        None
    }
}

pub fn list_unread_counts() -> HashMap<SocketAddr, u32> {
    let state = STATE_SNAPSHOT.lock().unwrap();
    state.unread_counts.clone()
}
