// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod ipmsg_core;
mod state;
mod config;

use log::{info, error};
use std::net::SocketAddr;
use state::{ChatMessage, OnlineUser, StateCmd, FileInfo};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::Manager;

#[tauri::command]
fn get_unread_counts() -> HashMap<SocketAddr, u32> {
    state::list_unread_counts()
}

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    window.minimize().unwrap();
}

#[tauri::command]
fn maximize_window(window: tauri::Window) {
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().unwrap();
    } else {
        window.maximize().unwrap();
    }
}

#[tauri::command]
async fn close_window(window: tauri::Window) {
    let _ = ipmsg_core::send_exit().await;
    window.close().unwrap();
}

#[tauri::command]
fn list_users() -> Vec<OnlineUser> {
    state::list_online_users()
}

#[tauri::command]
fn list_messages() -> Vec<ChatMessage> {
    state::list_messages()
}

#[tauri::command]
fn get_self() -> Option<OnlineUser> {
    state::get_self_addr_info()
}

#[tauri::command]
async fn send_msg(to: String, text: String) -> Result<(), String> {
    // to is "ip:port"
    if let Ok(addr) = to.parse::<SocketAddr>() {
        ipmsg_core::send_message(addr, text.clone()).await.map_err(|e| e.to_string())?;
        
        // Reconstruct ChatMessage
        let self_info = state::get_self_addr_info();
        if let Some(me) = self_info {
             let msg = ChatMessage {
                 from: me.addr,
                 to: addr,
                 is_me: true,
                 text: text,
                 time: "现在".to_string(),
                 file: None,
             };
             state::dispatch_cmd(StateCmd::PushOutgoing(msg));
        }
        
        Ok(())
    } else {
        Err("Invalid address".to_string())
    }
}

#[tauri::command]
async fn send_files(to: String, paths: Vec<String>) -> Result<(), String> {
    if let Ok(addr) = to.parse::<SocketAddr>() {
        ipmsg_core::send_files(addr, paths.clone()).await.map_err(|e| e.to_string())?;

        // Add to local history
        if let Some(me) = state::get_self_addr_info() {
            for path_str in paths {
                let path = PathBuf::from(&path_str);
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let is_dir = path.is_dir();

                let file_info = FileInfo {
                    packet_no: 0,
                    file_id: 0,
                    name,
                    size,
                    saved: true,
                    received: size,
                    is_dir,
                    local_path: Some(path_str),
                    current_file: None,
                    error: false,
                };
                let msg = ChatMessage {
                    from: me.addr,
                    to: addr,
                    is_me: true,
                    text: "".to_string(),
                    time: "现在".to_string(),
                    file: Some(file_info),
                };
                state::dispatch_cmd(StateCmd::PushOutgoing(msg));
            }
        }
        Ok(())
    } else {
        Err("Invalid address".to_string())
    }
}

#[tauri::command]
#[allow(non_snake_case)]
async fn download_file(
    from: String,
    packetNo: u32,
    fileId: u32,
    savePath: String,
    size: u64,
) -> Result<(), String> {
    let from_addr = from.parse().map_err(|e: std::net::AddrParseError| e.to_string())?;
    tauri::async_runtime::spawn(async move {
        let mut last_update = std::time::Instant::now();
        match ipmsg_core::recv_file(from_addr, packetNo, fileId, size, savePath, |p| {
            if last_update.elapsed() > std::time::Duration::from_millis(100) || p == size {
                state::dispatch_cmd(StateCmd::UpdateProgress {
                    file_id: fileId,
                    packet_no: packetNo,
                    progress: p,
                    file_name: None,
                    saved: None,
                    error: Some(false),
                });
                last_update = std::time::Instant::now();
            }
        })
        .await
        {
            Ok(_) => {
                info!("File downloaded successfully");
                state::dispatch_cmd(StateCmd::UpdateProgress {
                    file_id: fileId,
                    packet_no: packetNo,
                    progress: size,
                    file_name: None,
                    saved: Some(true),
                    error: Some(false),
                });
            },
            Err(e) => {
                error!("File download failed: {}", e);
                state::dispatch_cmd(StateCmd::UpdateProgress {
                    file_id: fileId,
                    packet_no: packetNo,
                    progress: 0,
                    file_name: None,
                    saved: Some(false),
                    error: Some(true),
                });
            },
        }
    });
    Ok(())
}

#[tauri::command]
async fn send_folder(to: String, path: String) -> Result<(), String> {
    if let Ok(addr) = to.parse::<SocketAddr>() {
        ipmsg_core::send_folder(addr, path.clone()).await.map_err(|e| e.to_string())?;

        // Add to local history
        if let Some(me) = state::get_self_addr_info() {
            let path_buf = PathBuf::from(&path);
            let name = path_buf.file_name().unwrap_or_default().to_string_lossy().to_string();
            
            let file_info = FileInfo {
                packet_no: 0,
                file_id: 0,
                name: name.clone(),
                size: 0,
                saved: true,
                received: 0,
                    is_dir: true,
                    local_path: Some(path),
                    current_file: None,
                    error: false,
                };

                let msg = ChatMessage {
                from: me.addr,
                to: addr,
                is_me: true,
                text: format!("[文件夹] {}", name),
                time: "现在".to_string(),
                file: Some(file_info),
            };
            state::dispatch_cmd(StateCmd::PushOutgoing(msg));
        }
        Ok(())
    } else {
        Err("Invalid address".to_string())
    }
}

#[tauri::command]
#[allow(non_snake_case)]
async fn download_folder(
    from: String,
    packetNo: u32,
    fileId: u32,
    savePath: String,
) -> Result<(), String> {
    let from_addr = from.parse().map_err(|e: std::net::AddrParseError| e.to_string())?;
    tauri::async_runtime::spawn(async move {
        let mut last_update = std::time::Instant::now();
        match ipmsg_core::recv_folder(from_addr, packetNo, fileId, savePath, |p, current_file| {
            if last_update.elapsed() > std::time::Duration::from_millis(100) {
                state::dispatch_cmd(StateCmd::UpdateProgress {
                    file_id: fileId,
                    packet_no: packetNo,
                    progress: p,
                    file_name: Some(current_file),
                    saved: None,
                    error: Some(false),
                });
                last_update = std::time::Instant::now();
            }
        })
        .await
        {
            Ok(_) => {
                info!("Folder downloaded successfully");
                state::dispatch_cmd(StateCmd::UpdateProgress {
                    file_id: fileId,
                    packet_no: packetNo,
                    progress: 0,
                    file_name: None,
                    saved: Some(true),
                    error: Some(false),
                });
            },
            Err(e) => {
                error!("Folder download failed: {}", e);
                state::dispatch_cmd(StateCmd::UpdateProgress {
                    file_id: fileId,
                    packet_no: packetNo,
                    progress: 0,
                    file_name: None,
                    saved: Some(false),
                    error: Some(true),
                });
            },
        }
    });
    Ok(())
}

#[tauri::command]
async fn pick_files(app: tauri::AppHandle) -> Vec<String> {
    if let Some(paths) = app.dialog().file().blocking_pick_files() {
        paths.into_iter().map(|p| p.to_string()).collect()
    } else {
        Vec::new()
    }
}

use tauri_plugin_dialog::DialogExt;

#[tauri::command]
#[allow(non_snake_case)]
async fn save_file_dialog(app: tauri::AppHandle, fileName: String) -> Option<String> {
    app.dialog()
        .file()
        .set_file_name(fileName)
        .blocking_save_file()
        .map(|p| p.to_string())
}

#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    app.dialog().file().blocking_pick_folder().map(|p| p.to_string())
}

#[tauri::command]
fn clear_unread(addr: String) {
    if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
        state::dispatch_cmd(StateCmd::ClearUnread { addr: socket_addr });
    }
}

#[tauri::command]
fn get_platform() -> String {
    if cfg!(target_os = "macos") {
        "macos".into()
    } else if cfg!(target_os = "windows") {
        "windows".into()
    } else {
        "linux".into()
    }
}

#[tauri::command]
async fn open_settings_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let window = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("/settings".into()),
        )
        .title("设置")
        .inner_size(400.0, 350.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .center()
        .visible(false)
        .build();

        if let Ok(w) = window {
            let _ = w.set_shadow(true);
        }
    }
}

#[tauri::command]
fn close_sub_window(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
fn show_window(window: tauri::Window) {
    let _ = window.show();
    let _ = window.set_focus();
}

#[tauri::command]
fn get_config(app: tauri::AppHandle) -> config::AppConfig {
    config::load_config(&app)
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, username: String, group: String) -> Result<(), String> {
    let mut config = config::load_config(&app);
    config.user.username = username.clone();
    config.user.group = group.clone();
    
    if let Err(e) = config::save_config(&config, &app) {
        return Err(e.to_string());
    }
    
    // Update core
    ipmsg_core::set_user_info(&username, &group);
    let _ = ipmsg_core::send_broadcast_entry().await;
    
    // Update state
    if let Some(me) = state::get_self_addr_info() {
        state::dispatch_cmd(StateCmd::InitSelf {
            user: username,
            group: group,
            host: me.host,
            addr: me.addr,
        });
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_decorations(false);
                }
            }
            state::init_state(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_users,
            list_messages,
            get_self,
            send_msg,
            send_files,
            send_folder,
            download_file,
            download_folder,
            pick_files,
            pick_folder,
            save_file_dialog,
            clear_unread,
            get_unread_counts,
            minimize_window,
            maximize_window,
            close_window,
            get_platform,
            open_settings_window,
            close_sub_window,
            show_window,
            get_config,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
