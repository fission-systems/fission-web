//! Pseudocode / NIR / Hex editor panel.

use crate::state::{EditorTab, use_app_state};
use dioxus::prelude::*;

#[component]
pub fn Editor() -> Element {
    let mut state = use_app_state();
    let tab = state.read().active_tab.clone();
    let is_decompiling = state.read().is_decompiling;
    let fn_name = state.read().current_fn_addr.and_then(|addr| {
        state.read().functions.iter()
            .find(|f| f.address == addr)
            .map(|f| if f.name.is_empty() { format!("sub_{:x}", addr) } else { f.name.clone() })
    });

    let tab_cls = |t: &EditorTab| if *t == tab { "tab is-active" } else { "tab" };

    rsx! {
        div { class: "editor-container",
            // Tab bar
            div { class: "editor-tabs",
                div { class: "breadcrumb",
                    span { class: "breadcrumb-sep", "fission" }
                    if let Some(name) = &fn_name {
                        span { class: "breadcrumb-sep", "/" }
                        span { class: "breadcrumb-fn", "{name}" }
                    }
                }
                div { class: "tab-group",
                    div {
                        class: tab_cls(&EditorTab::Pseudocode),
                        onclick: move |_| state.write().active_tab = EditorTab::Pseudocode,
                        "Pseudocode"
                    }
                    div {
                        class: tab_cls(&EditorTab::Nir),
                        onclick: move |_| state.write().active_tab = EditorTab::Nir,
                        "NIR"
                    }
                    div {
                        class: tab_cls(&EditorTab::Hex),
                        onclick: move |_| state.write().active_tab = EditorTab::Hex,
                        "Hex"
                    }
                }
            }

            // Body
            div { class: "editor-body",
                if is_decompiling {
                    div { class: "editor-decompiling",
                        div { class: "spinner spinner-lg" }
                        span { "Decompiling… (running in your browser)" }
                    }
                } else {
                    match tab {
                        EditorTab::Pseudocode => {
                            let code = state.read().decompiled_code.clone();
                            if let Some(code) = code {
                                rsx! { pre { class: "code-view", "{code}" } }
                            } else {
                                rsx! {
                                    div { class: "editor-placeholder",
                                        "Select a function to decompile."
                                    }
                                }
                            }
                        }
                        EditorTab::Nir => {
                            let nir = state.read().decompiled_nir.clone();
                            if let Some(nir) = nir {
                                rsx! { pre { class: "code-view nir-view", "{nir}" } }
                            } else {
                                rsx! {
                                    div { class: "editor-placeholder",
                                        "NIR not available."
                                    }
                                }
                            }
                        }
                        EditorTab::Hex => {
                            let data = state.read().binary_data.clone();
                            if let Some(data) = data {
                                let hex = hex_dump(&data, 4096);
                                rsx! { pre { class: "code-view hex-view", "{hex}" } }
                            } else {
                                rsx! { div { class: "editor-placeholder", "No binary loaded." } }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn hex_dump(data: &[u8], limit: usize) -> String {
    let data = &data[..data.len().min(limit)];
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let offset = i * 16;
        let hex: String = chunk.iter()
            .enumerate()
            .map(|(j, b)| if j == 8 { format!(" {:02x}", b) } else { format!("{:02x} ", b) })
            .collect();
        let ascii: String = chunk.iter()
            .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
            .collect();
        out.push_str(&format!("{offset:08x}  {hex:<49}  {ascii}\n"));
    }
    if data.len() == limit {
        out.push_str("\n[truncated]\n");
    }
    out
}
