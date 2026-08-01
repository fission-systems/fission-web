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
    engine::{
        fetch_server_status, get_server_url, set_server_api_token, set_server_url,
    },
    protocol::{AnalysisBackendKind, ResourceAccessMode},
};
use crate::components::dropzone::{DropZone, read_file_and_load};
use dioxus::web::WebEventExt;
use wasm_bindgen::JsCast;

const STYLE: Asset = asset!("assets/style.css");

/// Ping /api/status and update server_connected in AppState.
async fn check_server(mut state: Signal<fission_ui::state::AppState>) {
    let status = fetch_server_status().await.ok();
    let mut app = state.write();
    app.server_connected = status.is_some();
    app.backend_status = status;
}

#[component]
pub fn App() -> Element {
    init_app_state();
    let mut state = use_app_state();
    let mut backend_url = use_signal(get_server_url);
    let mut api_token = use_signal(String::new);

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

    let has_binary = state.read().binary_name.is_some();
    let server_connected = state.read().server_connected;
    let resource_status = state.read().backend_status.clone();
    let backend_name = resource_status
        .as_ref()
        .map(|status| match status.capabilities.backend {
            AnalysisBackendKind::NativeProcess => "native",
            AnalysisBackendKind::LocalHttp => "local",
            AnalysisBackendKind::CloudHttp => "cloud",
            AnalysisBackendKind::BrowserWorker => "browser worker",
        })
        .unwrap_or("backend");
    let resource_text = resource_status.as_ref().map(|status| {
        let location = match status.capabilities.resource_access {
            ResourceAccessMode::HostFilesystem => "host filesystem",
            ResourceAccessMode::BrowserSelectedBundle => "selected browser bundle",
            ResourceAccessMode::PackagedArtifacts => "packaged artifacts",
        };
        let sleigh = if status.resources.sleigh_artifacts {
            "SLEIGH ready"
        } else {
            "SLEIGH missing"
        };
        let signatures = if status.resources.signatures {
            "signatures ready"
        } else {
            "signatures optional"
        };
        format!("{location} · {sleigh} · {signatures}")
    });
    let backend_hint =
        resource_text.unwrap_or_else(|| "resource status unavailable".to_string());

    let (indicator_cls, status_text) = {
        let s = state.read();
        if s.is_loading_binary     { ("status-indicator busy",     "Loading")     }
        else if s.is_decompiling   { ("status-indicator busy",     "Decompiling") }
        else if s.has_binary_loaded() { ("status-indicator ready",   "Ready")        }
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
                        onchange: move |evt| {
                            if let Some(input) = evt
                                .as_web_event()
                                .target()
                                .and_then(|target| {
                                    target.dyn_into::<web_sys::HtmlInputElement>().ok()
                                })
                            {
                                if let Some(files) = input.files() {
                                    if let Some(file) = files.get(0) {
                                        read_file_and_load(file, state);
                                    }
                                }
                            }
                        }
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
                        "Can't reach the Fission backend. If it's running elsewhere, or your \
                         deployment requires a token, set them below."
                    }
                    form {
                        class: "backend-connect",
                        // A `type=password` input outside a `<form>` trips
                        // browsers' password-manager heuristics (logged as a
                        // console warning, and some autofill/save-password
                        // prompts just don't engage) -- wrapping it here and
                        // driving the connect action off `onsubmit` instead
                        // of the button's own `onclick` means Enter in
                        // either field submits too, which is also just the
                        // behavior a text field next to a button implies.
                        onsubmit: move |e| {
                            e.prevent_default();
                            let url = backend_url.read().trim().to_string();
                            let token = api_token.read().clone();
                            set_server_url(url.clone());
                            set_server_api_token(token);
                            state.write().server_url = url;
                            wasm_bindgen_futures::spawn_local(async move {
                                check_server(state).await;
                            });
                        },
                        input {
                            class: "backend-input backend-url-input",
                            r#type: "url",
                            value: "{backend_url}",
                            placeholder: "Same origin, or http://localhost:7331 for development",
                            aria_label: "Fission backend URL",
                            oninput: move |event| {
                                *backend_url.write() = event.value();
                            }
                        }
                        input {
                            class: "backend-input backend-token-input",
                            r#type: "password",
                            value: "{api_token}",
                            placeholder: "API token (only if your deployment requires one)",
                            autocomplete: "off",
                            aria_label: "Fission backend API token",
                            oninput: move |event| {
                                *api_token.write() = event.value();
                            }
                        }
                        button {
                            class: "backend-connect-btn",
                            r#type: "submit",
                            "Connect"
                        }
                    }
                    a {
                        class: "server-banner-link",
                        href: "https://github.com/fission-systems/Fission/blob/main/docs/RAILWAY.md",
                        target: "_blank",
                        "Deployment guide"
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
                if state.read().has_binary_loaded() {
                    div { class: "status-segment",
                        "{state.read().functions.len()} functions"
                    }
                }
                div { class: "status-segment status-right",
                    if server_connected {
                        div { class: "status-indicator ready" }
                        span {
                            class: "status-hint",
                            title: "Resource mode and availability reported by the connected Fission backend.",
                            "fission {backend_name} · {backend_hint}"
                        }
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
