use dioxus::prelude::*;
use crate::models::{ChatMessage, OnlineUser};
use crate::tauri::{invoke, invoke_no_args};
use std::collections::HashMap;

const CHAT_CSS: Asset = asset!("/assets/chat.css");

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
pub fn Chat() -> Element {
    let list_w = use_signal(|| 200i32);
    let mut dragging = use_signal(|| false);
    let mut start_x = use_signal(|| 0f64);
    let mut start_w = use_signal(|| 260i32);

    let mut active_conv = use_signal(|| 0usize);
    let mut input = use_signal(|| String::new());
    // File download handling would go here
    
    // Polling trigger
    let mut version = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(1000).await;
            version += 1;
        }
    });

    // Resources
    let conversations_resource = use_resource(move || async move {
        let _ = version(); // dependency
        invoke_no_args::<Vec<OnlineUser>>("list_users").await.unwrap_or_default()
    });

    let messages_resource = use_resource(move || async move {
        let _ = version(); // dependency
        invoke_no_args::<Vec<ChatMessage>>("list_messages").await.unwrap_or_default()
    });

    let unread_counts_resource = use_resource(move || async move {
        let _ = version(); // dependency
        invoke_no_args::<HashMap<String, u32>>("get_unread_counts").await.unwrap_or_default()
    });

    let self_resource = use_resource(move || async move {
        invoke_no_args::<Option<OnlineUser>>("get_self").await.unwrap_or_default()
    });

    // Computed data
    let conversations = conversations_resource.cloned().unwrap_or_default();
    let all_msgs = messages_resource.cloned().unwrap_or_default();
    let unread_counts = unread_counts_resource.cloned().unwrap_or_default();
    let self_addr_info = self_resource.cloned().unwrap_or_default();
    let self_addr = self_addr_info.map(|u| u.addr);

    let current = conversations.get(active_conv()).cloned();
    let current_click_text = current.clone();
    let current_click_file_folder = current.clone();
    let current_click_file = current.clone();

    let view_msgs: Vec<ChatMessage> = if let Some(curr) = &current {
        all_msgs
            .into_iter()
            .filter(|m| {
                if let Some(sa) = self_addr {
                    if curr.addr == sa {
                        m.from == sa && m.to == sa
                    } else {
                        (m.from == curr.addr && m.to == sa) || (m.to == curr.addr && m.from == sa)
                    }
                } else {
                    m.from == curr.addr || m.to == curr.addr
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    let active_idx = active_conv();
    let conv_items: Vec<(usize, OnlineUser, u32)> = conversations
        .iter()
        .cloned()
        .enumerate()
        .map(|(idx, conv)| {
            let unread = unread_counts.get(&conv.addr.to_string()).copied().unwrap_or(0); 
            (idx, conv, unread)
        })
        .collect();

    // Auto clear unread would go here

    rsx! {
        document::Link { rel: "stylesheet", href: CHAT_CSS }

        div { id: "chat-app", style: format!("grid-template-columns: {}px {}px 1fr", 64, list_w()),
            div { id: "sidebar",
                // window dragging area
                "data-tauri-drag-region": "true",
                div { class: "avatar", "A" }
            }

            div { id: "list-panel",
                div { class: "splitter",
                    onmousedown: move |ev| {
                        dragging.set(true);
                        start_x.set(ev.client_coordinates().x);
                        start_w.set(list_w());
                    }
                }
                div {
                    class: "list-topbar",
                    "data-tauri-drag-region": "true",
                 }
                div { class: "list-header",
                    input {
                        r#type: "text",
                        placeholder: "搜索",
                    }
                    button { "+" }
                }
                div { class: "conv-list",
                    for (idx, conv, unread) in conv_items.into_iter() {
                        div {
                            class: if active_idx == idx { "conv-item active" } else { "conv-item" },
                            onclick: move |_| {
                                active_conv.set(idx);
                                // clear unread
                                spawn(async move {
                                    let _ = invoke::<()>("clear_unread", conv.addr.to_string()).await;
                                });
                            },
                            div { class: "avatar",
                                {conv.name.chars().next().unwrap_or('C').to_string()}
                                if unread > 0 && active_idx != idx {
                                    div { class: "avatar-badge" }
                                }
                            }
                            div { class: "meta",
                                div { class: "top",
                                    span { class: "name", {conv.name.clone()} }
                                    span { class: "time", "" }
                                }
                                div { class: "bottom",
                                    span { class: "preview", {conv.host.clone()} }
                                }
                            }
                        }
                    }
                }
            }

            div { id: "main-panel",
                div { class: "topbar",
                    "data-tauri-drag-region": "true",
                    div { class: "window-controls",
                        div { class: "win-btn min", onclick: |_| { spawn(async move { let _ = invoke_no_args::<()>("minimize_window").await; }); }, "－" }
                        div { class: "win-btn max", onclick: |_| { spawn(async move { let _ = invoke_no_args::<()>("maximize_window").await; }); }, "□" }
                        div { class: "win-btn close", onclick: |_| { spawn(async move { let _ = invoke_no_args::<()>("close_window").await; }); }, "×" }
                    }
                }
                div { class: "main-header",
                    "data-tauri-drag-region": "true",
                    h2 { {current.as_ref().map(|c| c.host.clone()).unwrap_or_else(|| "会话".into())} }
                    div { class: "tools", span { "🔍" } span { "🔔" } span { "⋮" } }
                }
                div { class: "messages",
                            for (i, msg) in view_msgs.iter().enumerate() {
                                {
                                    let show_date = i == 0 || view_msgs[i - 1].time != msg.time;
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
                                                            div { class: "file-icon", "📄" }
                                                            div { class: "file-info",
                                                                div { class: "file-name", "{file.name}" }
                                                                div { class: "file-size", "{format_size(file.size)}" }
                                                            }
                                                            div { class: "file-status", "已发送" }
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
                                                            let file_icon = if is_dir { "Fn" } else { "📄" };
                                                            rsx! {
                                                                div { class: "file-bubble",
                                                                    div { class: "file-icon", "{file_icon}" }
                                                                    div { class: "file-info",
                                                                        div { class: "file-name", "{file.name}" }
                                                                        div { class: "file-size", "{format_size(file.size)}" }
                                                                    }
                                                                    if file.saved {
                                                                        div { class: "file-status", "已保存" }
                                                                    } else if file.received > 0 && file.received < file.size {
                                                                        {
                                                                            let percent = (file.received as f64 / file.size as f64 * 100.0) as u32;
                                                                            rsx! {
                                                                                div { class: "file-status", "下载中 {percent}%" }
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
                                                                                            if is_dir {
                                                                                                // For folder, pick destination parent directory
                                                                                                if let Ok(Some(parent_path)) = invoke_no_args::<Option<String>>("pick_folder").await {
                                                                                                    let mut path = std::path::PathBuf::from(parent_path);
                                                                                                    path.push(&file_clone.name);
                                                                                                    let save_path = path.to_string_lossy().to_string();
                                                                                                    
                                                                                                    let _ = invoke::<()>("download_folder", serde_json::json!({
                                                                                                        "from": from_addr.to_string(),
                                                                                                        "packet_no": file_clone.packet_no,
                                                                                                        "file_id": file_clone.file_id,
                                                                                                        "save_path": save_path
                                                                                                    })).await;
                                                                                                }
                                                                                            } else {
                                                                                                if let Ok(Some(path)) = invoke::<Option<String>>("save_file_dialog", serde_json::json!({ "file_name": file_clone.name })).await {
                                                                                                    let _ = invoke::<()>("download_file", serde_json::json!({
                                                                                                        "from": from_addr.to_string(),
                                                                                                        "packet_no": file_clone.packet_no,
                                                                                                        "file_id": file_clone.file_id,
                                                                                                        "save_path": path,
                                                                                                        "size": file_clone.size
                                                                                                    })).await;
                                                                                                }
                                                                                            }
                                                                                        });
                                                                                    },
                                                                                    "下载"
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
                div { class: "input-area",
                    div { class: "toolbar",
                        button {
                            title: "发送文件",
                            onclick: move |_| {
                                let current_click = current_click_file.clone();
                                spawn(async move {
                                    if let Some(curr) = current_click {
                                        if let Ok(paths) = invoke_no_args::<Vec<String>>("pick_files").await {
                                            if !paths.is_empty() {
                                                let _ = invoke::<()>("send_files", serde_json::json!({
                                                    "to": curr.addr.to_string(),
                                                    "paths": paths
                                                })).await;
                                            }
                                        }
                                    }
                                });
                            },
                            "📄"
                        }
                        button {
                            title: "发送文件夹",
                            onclick: move |_| {
                                let current_click = current_click_file_folder.clone();
                                spawn(async move {
                                    if let Some(curr) = current_click {
                                        if let Ok(Some(path)) = invoke_no_args::<Option<String>>("pick_folder").await {
                                            let _ = invoke::<()>("send_folder", serde_json::json!({
                                                "to": curr.addr.to_string(),
                                                "path": path
                                            })).await;
                                        }
                                    }
                                });
                            },
                            "📁"
                        }
                    }
                    textarea {
                        value: "{input}",
                        oninput: move |ev| input.set(ev.value()),
                        onkeydown: move |ev| {
                            if ev.key() == Key::Enter && !ev.modifiers().contains(Modifiers::SHIFT) {
                                let text = input();
                                if !text.trim().is_empty() {
                                    if let Some(curr) = current.as_ref() {
                                        let addr_str = curr.addr.to_string();
                                        spawn(async move {
                                            match invoke::<()>("send_msg", serde_json::json!({ "to": addr_str, "text": text })).await {
                                                Ok(_) => {},
                                                Err(e) => println!("Send failed: {}", e),
                                            }
                                        });
                                        input.set(String::new());
                                    }
                                }
                            }
                        }
                    }
                    div { class: "actions",
                        button {
                            onclick: move |_| {
                                let text = input();
                                if !text.trim().is_empty() {
                                    if let Some(curr) = current_click_text.as_ref() {
                                        let addr_str = curr.addr.to_string();
                                        spawn(async move {
                                            let _ = invoke::<()>("send_msg", serde_json::json!({ "to": addr_str, "text": text })).await;
                                        });
                                        input.set(String::new());
                                    }
                                }
                            },
                            "发送"
                        }
                    }
                }
            }
        }
    }
}
