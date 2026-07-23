//! Drop zone — shown when no binary is loaded.
//! Web-specific: uses web-sys FileReader to load binary bytes from the browser.
//! The actual loading logic (load_binary_from_bytes_blocking) lives in fission-ui.

use crate::state::use_app_state;
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use fission_ui::{
    engine::{run_load, LoadResult},
    state::{AppState, LogEntry},
};
use wasm_bindgen::JsCast;
use web_sys::{FileReader, ProgressEvent};

fn read_file_and_load(file: web_sys::File, mut sig: Signal<AppState>) {
    let name  = file.name();
    let reader = FileReader::new().unwrap();
    let reader_clone = reader.clone();
    let name_clone   = name.clone();

    let onload = wasm_bindgen::closure::Closure::once(move |_e: ProgressEvent| {
        let result = reader_clone.result().unwrap();
        let array  = js_sys::Uint8Array::new(&result);
        let bytes  = array.to_vec();

        // Kick off async load via fission-ui engine (wasm32 path: no spawn_blocking)
        spawn(async move {
            {
                let mut s = sig.write();
                s.is_loading_binary = true;
                s.push_log(LogEntry::info(format!("Loading {name_clone}…")));
            }

            match run_load(bytes, name_clone.clone()).await {
                Ok(load) => {
                    let mut s = sig.write();
                    s.binary_name       = Some(load.binary.format.clone());
                    s.binary            = Some(std::sync::Arc::clone(&load.binary));
                    s.functions         = load.functions;
                    s.is_loading_binary = false;
                    s.push_log(LogEntry::info(load.summary));
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
                    "PE, ELF, Mach-O — all analysis runs locally in your browser."
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
                p { class: "dropzone-note", "Your binary never leaves this device." }
            }
        }
    }
}
