//! Root application component.

use crate::state::{AppState, use_app_state};
use crate::components::{sidebar::Sidebar, editor::Editor, dropzone::DropZone};
use dioxus::prelude::*;

const STYLE: Asset = asset!("/assets/style.css");

#[component]
pub fn App() -> Element {
    use_context_provider(|| Signal::new(AppState::default()));
    let state = use_app_state();

    let has_binary = state.read().binary_name.is_some();

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

        div { class: "app",
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
                    // File input trigger
                    label { class: "open-btn", r#for: "file-input",
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
                        id: "file-input",
                        r#type: "file",
                        style: "display:none",
                        onchange: move |evt| {
                            // Handled by DropZone engine
                            let _ = evt;
                        }
                    }
                }
            }

            // ── Main layout ───────────────────────────────────────────────
            div { class: "main-layout",
                if has_binary {
                    Sidebar {}
                    Editor {}
                } else {
                    DropZone {}
                }
            }

            // ── Status bar ───────────────────────────────────────────────
            div { class: "status-bar",
                if state.read().is_loading {
                    div { class: "status-segment",
                        div { class: "spinner-sm" }
                        "Loading…"
                    }
                } else if state.read().is_decompiling {
                    div { class: "status-segment",
                        div { class: "spinner-sm" }
                        "Decompiling…"
                    }
                } else if let Some(name) = state.read().binary_name.clone() {
                    div { class: "status-segment",
                        span { class: "status-dot status-ok" }
                        "{name}"
                    }
                    div { class: "status-segment",
                        "{state.read().functions.len()} functions"
                    }
                }
                div { class: "status-right",
                    span { class: "status-hint",
                        "All computation runs locally in your browser"
                    }
                }
            }
        }
    }
}
