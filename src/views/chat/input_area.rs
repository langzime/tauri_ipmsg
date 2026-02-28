use dioxus::prelude::*;
use crate::models::OnlineUser;
use crate::tauri::{invoke, invoke_no_args};
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaFile, FaFolder, FaPaperPlane};

const INPUT_AREA_CSS: Asset = asset!("/assets/chat/input_area.css");

#[component]
pub fn InputArea(
    current: Option<OnlineUser>,
) -> Element {
    let mut input = use_signal(|| String::new());

    // Captures for closure
    let current_click_file = current.clone();
    let current_click_file_folder = current.clone();
    let current_click_text = current.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: INPUT_AREA_CSS }
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
                    Icon { width: 20, height: 20, icon: FaFile }
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
                    Icon { width: 20, height: 20, icon: FaFolder }
                }
            }
            textarea {
                value: "{input}",
                oninput: move |ev| input.set(ev.value()),
                onkeydown: move |ev| {
                    if ev.key() == Key::Enter && !ev.modifiers().contains(Modifiers::SHIFT) {
                        ev.prevent_default();
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
            // div { class: "actions",
            //     button {
            //         onclick: move |_| {
            //             let text = input();
            //             if !text.trim().is_empty() {
            //                 if let Some(curr) = current_click_text.as_ref() {
            //                     let addr_str = curr.addr.to_string();
            //                     spawn(async move {
            //                         let _ = invoke::<()>("send_msg", serde_json::json!({ "to": addr_str, "text": text })).await;
            //                     });
            //                     input.set(String::new());
            //                 }
            //             }
            //         },
            //         Icon { width: 16, height: 16, icon: FaPaperPlane }, " 发送"
            //     }
            // }
        }
    }
}
