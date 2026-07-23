//! Root application component — web platform layout.
//! Platform-specific: title bar (HTML5 file input), dropzone.
//! Shared components (Sidebar, Editor, BottomPanel, etc.) come from fission-ui.

use dioxus::prelude::*;
use fission_ui::{
    state::{init_app_state, use_app_state},
    components::{
        sidebar::Sidebar,
        editor::Editor,
        bottom_panel::BottomPanel,
        command_palette::CommandPalette,
    },
    engine::get_server_url,
};
use crate::components::dropzone::DropZone;

const STYLE: Asset = asset!("assets/style.css");

/// Ping /api/status and update server_connected in AppState.
async fn check_server(mut state: Signal<fission_ui::state::AppState>) {
    let url = get_server_url();
    let ok = gloo_net::http::Request::get(&format!("{url}/api/status"))
        .send()
        .await
        .map(|r| r.ok())
        .unwrap_or(false);
    state.write().server_connected = ok;
}

#[component]
pub fn App() -> Element {
    init_app_state();
    let state = use_app_state();

    // ── Server connectivity check (poll every 5 s) ────────────────────────────
    use_effect(move || {
        // Initial check immediately
        let s = state;
        wasm_bindgen_futures::spawn_local(async move {
            check_server(s).await;
        });
        // Subsequent checks every 5 seconds
        let interval = gloo_timers::callback::Interval::new(5_000, move || {
            let s = state;
            wasm_bindgen_futures::spawn_local(async move {
                check_server(s).await;
            });
        });
        // Keep the interval alive for the component lifetime
        interval.forget();
    });

    let has_binary = state.read().binary.is_some();
    let server_connected = state.read().server_connected;

    let (indicator_cls, status_text) = {
        let s = state.read();
        if s.is_loading_binary     { ("status-indicator busy",     "Loading")     }
        else if s.is_decompiling   { ("status-indicator busy",     "Decompiling") }
        else if s.binary.is_some() { ("status-indicator ready",   "Ready")        }
        else                       { ("status-indicator inactive", "Idle")         }
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

            // ── Server disconnected banner ──────────────────────────────────
            if !server_connected {
                div { class: "server-banner",
                    div { class: "server-banner-icon",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "16", height: "16",
                            view_box: "0 0 24 24",
                            fill: "none", stroke: "currentColor",
                            stroke_width: "2", stroke_linecap: "round",
                            circle { cx: "12", cy: "12", r: "10" }
                            line { x1: "12", y1: "8", x2: "12", y2: "12" }
                            line { x1: "12", y1: "16", x2: "12.01", y2: "16" }
                        }
                    }
                    span { class: "server-banner-text",
                        "Backend not reachable — start "
                        code { "fission_cli serve --port 7331" }
                        " on your local machine to enable analysis."
                    }
                    a {
                        class: "server-banner-link",
                        href: "https://github.com/fission-systems/Fission#readme",
                        target: "_blank",
                        "Setup guide"
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
                    if server_connected {
                        div { class: "status-indicator ready" }
                        span { class: "status-hint", "fission serve connected" }
                    } else {
                        div { class: "status-indicator busy" }
                        span { class: "status-hint status-hint-warn", "fission serve not running" }
                    }
                }
            }

            // Command palette (shared from fission-ui)
            CommandPalette {}
        }
    }
}
