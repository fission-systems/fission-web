//! Drop zone — shown when no binary is loaded.
//! Web-specific: uses web-sys FileReader to load binary bytes from the browser.
//! The actual loading logic (load_binary_from_bytes_blocking) lives in fission-ui.

use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use fission_ui::{
    engine::{poll_functions, run_load},
    state::{AppState, LogEntry, use_app_state},
};
use wasm_bindgen::JsCast;
use web_sys::{FileReader, ProgressEvent};

/// Ceiling on how long to keep polling for background discovery results
/// before giving up silently (session TTL default is 30 min server-side;
/// this is just a client-side safety net against polling forever if a
/// session is somehow stuck `analyzing`).
const MAX_POLL_ATTEMPTS: u32 = 200; // ~200 * 1.5s = 5 minutes
const POLL_INTERVAL_MS: u32 = 1500;

pub(crate) fn read_file_and_load(file: web_sys::File, mut sig: Signal<AppState>) {
    let name  = file.name();
    let reader = FileReader::new().unwrap();
    let reader_clone = reader.clone();
    let name_clone   = name.clone();

    let onload = wasm_bindgen::closure::Closure::once(move |_e: ProgressEvent| {
        let result = reader_clone.result().unwrap();
        let array  = js_sys::Uint8Array::new(&result);
        let bytes  = array.to_vec();

        // Use wasm_bindgen_futures::spawn_local instead of Dioxus spawn.
        // The FileReader onload callback fires from a JS event loop context
        // outside any Dioxus render/hook scope, so Dioxus spawn would panic
        // with "called Option::unwrap() on a None value" in the runtime.
        wasm_bindgen_futures::spawn_local(async move {
            {
                let mut s = sig.write();
                s.is_loading_binary = true;
                s.push_log(LogEntry::info(format!("Loading {name_clone}…")));
            }

            match run_load(bytes, name_clone.clone()).await {
                Ok(load) => {
                    let session_id = load.session_id.clone();
                    let analyzing  = load.analyzing;
                    {
                        let mut s = sig.write();
                        s.binary_name        = Some(name_clone);
                        s.binary             = load.binary;
                        s.functions          = load.functions;
                        s.server_session_id  = load.session_id;
                        s.is_loading_binary  = false;
                        s.is_analyzing       = analyzing;
                        s.push_log(LogEntry::info(load.summary));
                    }
                    if analyzing {
                        if let Some(session_id) = session_id {
                            wasm_bindgen_futures::spawn_local(poll_until_analyzed(session_id, sig));
                        }
                    }
                }
                Err(e) => {
                    let mut s = sig.write();
                    s.is_loading_binary = false;
                    s.push_log(LogEntry::error(format!("Load failed: {e}")));
                }
            }
        });
    });

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    reader.read_as_array_buffer(&file).unwrap();
}

/// Re-fetch the function list every `POLL_INTERVAL_MS` until the server
/// reports background CFG discovery is done, then apply the fuller list.
/// The already-loaded loader-only functions stay visible/usable the whole
/// time -- this only ever adds to what's shown, never blocks interaction.
async fn poll_until_analyzed(session_id: String, mut sig: Signal<AppState>) {
    for _ in 0..MAX_POLL_ATTEMPTS {
        gloo_timers::future::TimeoutFuture::new(POLL_INTERVAL_MS).await;

        // Bail if the user switched to a different binary in the meantime.
        if sig.read().server_session_id.as_deref() != Some(session_id.as_str()) {
            return;
        }

        match poll_functions(&session_id).await {
            Ok((functions, still_analyzing)) => {
                let mut s = sig.write();
                s.functions    = functions;
                s.is_analyzing = still_analyzing;
                if !still_analyzing {
                    return;
                }
            }
            Err(_) => return, // session expired or backend unreachable; stop quietly
        }
    }
    sig.write().is_analyzing = false;
}

#[component]
pub fn DropZone() -> Element {
    let state       = use_app_state();
    let is_dragging = use_signal(|| false);
    let mut dragging = is_dragging;

    let drag_cls = if *is_dragging.read() { "dropzone is-dragging" } else { "dropzone" };

    rsx! {
        div {
            class: "{drag_cls}",
            ondragover: move |e| { e.prevent_default(); *dragging.write() = true; },
            ondragleave: move |_| *dragging.write() = false,
            ondrop: move |e| {
                e.prevent_default();
                *dragging.write() = false;
                let native = e.as_web_event();
                if let Some(dt) = native.data_transfer() {
                    if let Some(files) = dt.files() {
                        if let Some(file) = files.get(0) {
                            read_file_and_load(file, state);
                        }
                    }
                }
            },

            div { class: "dropzone-inner",
                div { class: "dropzone-icon",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "48", height: "48",
                        view_box: "0 0 24 24",
                        fill: "none", stroke: "currentColor",
                        stroke_width: "1.2", stroke_linecap: "round",
                        path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                        polyline { points: "17 8 12 3 7 8" }
                        line { x1: "12", y1: "3", x2: "12", y2: "15" }
                    }
                }
                h1 { class: "dropzone-title", "Drop a binary to decompile" }
                p  { class: "dropzone-sub",
                    "PE, ELF, Mach-O — analysis runs on the private Fission backend."
                }
                label { class: "dropzone-btn", r#for: "file-input-dz",
                    "Choose file"
                }
                input {
                    id: "file-input-dz",
                    r#type: "file",
                    style: "display:none",
                    onchange: move |evt| {
                        // web-sys path: grab file from input element
                        use wasm_bindgen::JsCast;
                        if let Some(input) = evt
                            .as_web_event()
                            .target()
                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                        {
                            if let Some(files) = input.files() {
                                if let Some(file) = files.get(0) {
                                    read_file_and_load(file, state);
                                }
                            }
                        }
                    }
                }
                p { class: "dropzone-note", "Analysis runs on the connected Fission backend." }
            }
        }
    }
}
