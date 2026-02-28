use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaXmark;
use crate::tauri::invoke_no_args;

const SETTINGS_CSS: Asset = asset!("/assets/chat/settings_modal.css");

#[component]
pub fn Settings() -> Element {
    let platform = use_resource(|| async move {
        invoke_no_args::<String>("get_platform").await.unwrap_or_default()
    });

    let p = platform.value().cloned().unwrap_or_default();
    let is_macos = p == "macos";

    rsx! {
        document::Link { rel: "stylesheet", href: SETTINGS_CSS }
        div { class: "modal-content", style: "width: 100%; height: 100%; border-radius: 10px; overflow: hidden;",
            div { class: if is_macos { "modal-header macos" } else { "modal-header windows" }, "data-tauri-drag-region": "true",
                if is_macos {
                    button { class: "close-btn macos", onclick: move |_| {
                        spawn(async move {
                            let _ = invoke_no_args::<()>("close_sub_window").await;
                        });
                    },
                        Icon { width: 10, height: 10, icon: FaXmark }
                    }
                    h3 { "设置" }
                    div { class: "spacer", style: "width: 12px;" } // Placeholder to balance layout
                } else {
                    h3 { "设置" }
                    button { class: "close-btn windows", onclick: move |_| {
                        spawn(async move {
                            let _ = invoke_no_args::<()>("close_sub_window").await;
                        });
                    },
                        Icon { width: 16, height: 16, icon: FaXmark }
                    }
                }
            }
            div { class: "modal-body",
                div { class: "setting-item",
                    label { "用户名" }
                    input { type: "text", value: "m674729", disabled: true }
                }
                div { class: "setting-item",
                    label { "主机名" }
                    input { type: "text", value: "EE-M674729MAC1.local", disabled: true }
                }
            }
            div { class: "modal-footer",
                button { class: "btn-primary", onclick: move |_| {
                    spawn(async move {
                        let _ = invoke_no_args::<()>("close_sub_window").await;
                    });
                }, "确定" }
            }
        }
    }
}
