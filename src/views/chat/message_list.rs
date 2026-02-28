use dioxus::prelude::*;
use crate::models::{ChatMessage, OnlineUser};
use crate::tauri::{invoke, invoke_no_args};
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaFile, FaFolder, FaDownload, FaRotateRight, FaCheck};

const MESSAGE_LIST_CSS: Asset = asset!("/assets/chat/message_list.css");

fn format_size(size: u64) -> String {
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

#[component]
pub fn MessageList(
    current: Option<OnlineUser>,
    messages: Vec<ChatMessage>,
) -> Element {
    let current_addr = current.as_ref().map(|c| c.addr.to_string()).unwrap_or_default();
    let msg_len = messages.len();
    
    // Track previous state to determine if we should scroll
    // using use_hook to avoid re-renders when updating the tracker
    let scroll_tracker = use_hook(|| std::cell::RefCell::new((0, String::new())));
    
    let should_scroll = {
        let mut tracker = scroll_tracker.borrow_mut();
        if tracker.0 != msg_len || tracker.1 != current_addr {
            tracker.0 = msg_len;
            tracker.1 = current_addr;
            true
        } else {
            false
        }
    };

    if should_scroll {
        spawn(async move {
            // Wait for DOM update
            let mut eval = document::eval(r#"
                requestAnimationFrame(() => {
                    var element = document.getElementById('message-list');
                    if (element) {
                        element.scrollTop = element.scrollHeight;
                    }
                });
            "#);
            let _ = eval.await;
        });
    }

    rsx! {
        document::Link { rel: "stylesheet", href: MESSAGE_LIST_CSS }
        div { class: "messages", id: "message-list",
            for (i, msg) in messages.iter().enumerate() {
                {
                    let show_date = i == 0 || messages[i - 1].time != msg.time;
                    let msg_is_me = msg.is_me;
                    let msg_text = msg.text.clone();
                    let msg_time = msg.time.clone();
                    let msg_file = msg.file.clone();
                    let msg_from = msg.from;
                    let current_name = current.as_ref().map(|c| c.name.clone()).unwrap_or_else(|| "客".into());
                    
                    rsx! {
                        if show_date {
                            div { class: "separator", span { "{msg_time}" } }
                        }
                        if msg_is_me {
                            div { class: "msg msg-right",
                                div { class: "bubble",
                                    if let Some(file) = msg_file {
                                        div { class: "file-bubble",
                                            div { class: "file-icon", Icon { width: 24, height: 24, icon: FaFile } }
                                            div { class: "file-info",
                                                div { class: "file-name", "{file.name}" }
                                                div { class: "file-size", "{format_size(file.size)}" }
                                            }
                                            div { class: "file-status", Icon { width: 12, height: 12, icon: FaCheck }, " 已发送" }
                                        }
                                    } else {
                                        "{msg_text}"
                                    }
                                }
                                div { class: "label", "我" }
                            }
                        } else {
                            div { class: "msg msg-left",
                                div { class: "avatar", {current_name.chars().next().unwrap_or('客').to_string()} }
                                div {
                                    class: "bubble",
                                    if let Some(file) = msg_file {
                                        {
                                            let is_dir = file.is_dir;
                                            rsx! {
                                                div { class: "file-bubble",
                                                    div { class: "file-icon", 
                                                        if is_dir {
                                                            Icon { width: 24, height: 24, icon: FaFolder }
                                                        } else {
                                                            Icon { width: 24, height: 24, icon: FaFile }
                                                        }
                                                    }
                                                    div { class: "file-info",
                                                        div { class: "file-name", "{file.name}" }
                                                        div { class: "file-size", "{format_size(file.size)}" }
                                                    }
                                                    if file.saved {
                                                        div { class: "file-status", Icon { width: 12, height: 12, icon: FaCheck }, " 已保存" }
                                                    } else if file.error {
                                                        div { class: "file-status",
                                                            button {
                                                                onclick: move |_| {
                                                                    // Retry logic (same as download logic)
                                                                    let file_clone = file.clone();
                                                                    let from_addr = msg_from;
                                                                    spawn(async move {
                                                                        if is_dir {
                                                                            if let Ok(Some(parent_path)) = invoke_no_args::<Option<String>>("pick_folder").await {
                                                                                let mut path = std::path::PathBuf::from(parent_path);
                                                                                path.push(&file_clone.name);
                                                                                let save_path = path.to_string_lossy().to_string();
                                                                                
                                                                                let _ = invoke::<()>("download_folder", serde_json::json!({
                                                                                    "from": from_addr.to_string(),
                                                                                    "packetNo": file_clone.packet_no,
                                                                                    "fileId": file_clone.file_id,
                                                                                    "savePath": save_path
                                                                                })).await;
                                                                            }
                                                                        } else {
                                                                            if let Ok(Some(path)) = invoke::<Option<String>>("save_file_dialog", serde_json::json!({ "fileName": file_clone.name })).await {
                                                                                let _ = invoke::<()>("download_file", serde_json::json!({
                                                                                    "from": from_addr.to_string(),
                                                                                    "packetNo": file_clone.packet_no,
                                                                                    "fileId": file_clone.file_id,
                                                                                    "savePath": path,
                                                                                    "size": file_clone.size
                                                                                })).await;
                                                                            }
                                                                        }
                                                                    });
                                                                },
                                                                Icon { width: 12, height: 12, icon: FaRotateRight }, " 重试"
                                                            }
                                                        }
                                                    } else if file.received > 0 && file.received < file.size {
                                                        {
                                                            let percent = (file.received as f64 / file.size as f64 * 100.0) as u32;
                                                            let current_file_name = file.current_file.clone().unwrap_or_else(|| "下载中".to_string());
                                                            rsx! {
                                                                div { class: "file-status", 
                                                                    "{current_file_name} {percent}%" 
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        {
                                                            let file_clone = file.clone();
                                                            let from_addr = msg_from;
                                                            rsx! {
                                                                button {
                                                                    onclick: move |_| {
                                                                        let file_clone = file_clone.clone();
                                                                        let from_addr = from_addr;
                                                                        spawn(async move {
                                                                            web_sys::console::log_1(&"Download button clicked".into());
                                                                            if is_dir {
                                                                                // For folder, pick destination parent directory
                                                                                if let Ok(Some(parent_path)) = invoke_no_args::<Option<String>>("pick_folder").await {
                                                                                    web_sys::console::log_1(&format!("Folder selected: {}", parent_path).into());
                                                                                    let mut path = std::path::PathBuf::from(parent_path);
                                                                                    path.push(&file_clone.name);
                                                                                    let save_path = path.to_string_lossy().to_string();
                                                                                    
                                                                                    match invoke::<()>("download_folder", serde_json::json!({
                                                                                        "from": from_addr.to_string(),
                                                                                        "packetNo": file_clone.packet_no,
                                                                                        "fileId": file_clone.file_id,
                                                                                        "savePath": save_path
                                                                                    })).await {
                                                                                        Ok(_) => web_sys::console::log_1(&"Folder download started".into()),
                                                                                        Err(e) => web_sys::console::log_1(&format!("Folder download failed: {}", e).into()),
                                                                                    }
                                                                                } else {
                                                                                    web_sys::console::log_1(&"Folder selection cancelled".into());
                                                                                }
                                                                            } else {
                                                                                if let Ok(Some(path)) = invoke::<Option<String>>("save_file_dialog", serde_json::json!({ "fileName": file_clone.name })).await {
                                                                                    web_sys::console::log_1(&format!("File save path: {}", path).into());
                                                                                    match invoke::<()>("download_file", serde_json::json!({
                                                                                        "from": from_addr.to_string(),
                                                                                        "packetNo": file_clone.packet_no,
                                                                                        "fileId": file_clone.file_id,
                                                                                        "savePath": path,
                                                                                        "size": file_clone.size
                                                                                    })).await {
                                                                                        Ok(_) => web_sys::console::log_1(&"File download started".into()),
                                                                                        Err(e) => web_sys::console::log_1(&format!("File download failed: {}", e).into()),
                                                                                    }
                                                                                } else {
                                                                                    web_sys::console::log_1(&"File save dialog cancelled".into());
                                                                                }
                                                                            }
                                                                        });
                                                        },
                                                        Icon { width: 12, height: 12, icon: FaDownload }, " 下载"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                            }
                                        }
                                    } else {
                                        "{msg_text}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
