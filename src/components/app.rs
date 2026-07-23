//! Root application component — web platform layout.
//! Platform-specific: title bar (HTML5 file input), dropzone.
//! Shared components (Sidebar, Editor, BottomPanel, etc.) come from fission-ui.

use crate::state::{AppState, LogEntry, init_app_state, use_app_state};
use crate::components::dropzone::DropZone;
use dioxus::prelude::*;
use fission_ui::components::{
    sidebar::Sidebar,
    editor::Editor,
    bottom_panel::BottomPanel,
    command_palette::CommandPalette,
};

const STYLE: Asset = asset!("/assets/style.css");

#[component]
pub fn App() -> Element {
    init_app_state();
    let state = use_app_state();

    let has_binary = state.read().binary.is_some();

    let (indicator_cls, status_text) = {
        let s = state.read();
        if s.is_loading_binary   { ("status-indicator busy",     "Loading")     }
        else if s.is_decompiling { ("status-indicator busy",     "Decompiling") }
        else if s.binary.is_some() { ("status-indicator ready", "Ready")        }
        else                     { ("status-indicator inactive", "Idle")         }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.googleapis.com"
        }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600&family=JetBrains+Mono:wght@400;500&display=swap"
        }

        div {
            class: "app-container",
            // ── Title bar ─────────────────────────────────────────────────
            div { class: "title-bar",
                div { class: "title-logo",
                    div { class: "logo-mark",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "22", height: "22",
                            view_box: "0 0 24 24",
                            fill: "none", stroke: "currentColor",
                            stroke_width: "1.8", stroke_linecap: "round",
                            path { d: "M13 2L3 14h9l-1 8 10-12h-9l1-8z" }
                        }
                    }
                    span { class: "logo-wordmark", "Fission" }
                    span { class: "logo-badge", "WEB" }
                }
                div { class: "title-center",
                    if let Some(name) = state.read().binary_name.clone() {
                        span { class: "binary-name", "{name}" }
                    }
                }
                div { class: "title-right",
                    label { class: "open-btn", r#for: "file-input-web",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "14", height: "14",
                            view_box: "0 0 24 24",
                            fill: "none", stroke: "currentColor",
                            stroke_width: "2", stroke_linecap: "round",
                            path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" }
                        }
                        "Open Binary"
                    }
                    input {
                        id: "file-input-web",
                        r#type: "file",
                        style: "display:none",
                        onchange: move |_| {}
                    }
                }
            }

            // ── Main workspace ─────────────────────────────────────────────
            div { class: "main-workspace",
                if has_binary {
                    div { class: "sidebar-wrapper",
                        Sidebar {}
                    }
                    div { class: "editor-area",
                        div { class: "editor-region",
                            Editor {}
                        }
                        div { style: "height: 180px; min-height: 180px; overflow: hidden; display: flex; flex-direction: column;",
                            BottomPanel {}
                        }
                    }
                } else {
                    DropZone {}
                }
            }

            // ── Status bar ─────────────────────────────────────────────────
            div { class: "status-bar",
                div { class: "status-segment",
                    div { class: "{indicator_cls}" }
                    span { "{status_text}" }
                }
                if state.read().binary.is_some() {
                    div { class: "status-segment",
                        "{state.read().functions.len()} functions"
                    }
                }
                div { class: "status-segment status-right",
                    span { class: "status-hint",
                        "All computation runs locally in your browser"
                    }
                }
            }

            // Command palette (shared from fission-ui)
            CommandPalette {}
        }
    }
}
