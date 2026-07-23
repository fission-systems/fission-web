//! WASM-compatible engine — wraps Fission core APIs for browser execution.
//!
//! All heavy computation runs via `wasm_bindgen_futures::spawn_local`
//! (no spawn_blocking in WASM). Single-threaded execution model.

use crate::state::{AppState, FunctionEntry, LogEntry};
use dioxus::prelude::*;

// ── Load binary ────────────────────────────────────────────────────────────
// Called after the user selects a file via the browser File API.
// The raw bytes have already been read by web-sys FileReader.

pub fn load_binary_from_bytes(
    bytes: Vec<u8>,
    name: String,
    mut state: Signal<AppState>,
) {
    use fission_loader::loader::load_binary_from_bytes as fission_load;
    use std::sync::Arc;

    state.write().is_loading = true;
    state.write().push_log(LogEntry::info(format!("Loading {name}…")));

    wasm_bindgen_futures::spawn_local(async move {
        // fission_loader is pure Rust — runs fine in WASM
        match fission_load(&bytes, &name) {
            Ok(loaded) => {
                let fn_count = loaded.functions.len();
                let fns: Vec<FunctionEntry> = loaded.functions.iter().map(|f| FunctionEntry {
                    address:   f.address,
                    name:      f.name.clone(),
                    is_import: f.is_import,
                    is_export: f.is_export,
                    is_thunk:  f.is_thunk_like,
                }).collect();

                let mut s = state.write();
                s.binary_name     = Some(name.clone());
                s.binary_data     = Some(Arc::new(bytes));
                s.functions       = fns;
                s.current_fn_addr = None;
                s.decompiled_code = None;
                s.decompiled_nir  = None;
                s.is_loading      = false;
                s.push_log(LogEntry::info(format!("Loaded — {fn_count} functions")));
            }
            Err(e) => {
                let mut s = state.write();
                s.is_loading = false;
                s.push_log(LogEntry::error(format!("Load failed: {e}")));
            }
        }
    });
}

// ── Decompile ──────────────────────────────────────────────────────────────

pub fn decompile_function(addr: u64, name: String, mut state: Signal<AppState>) {
    use fission_decompiler::{RustSleighDecompileConfig, decompile_with_rust_sleigh_with_facts};
    use fission_static::analysis::decomp::facts::FactStore;
    use std::sync::Arc;

    let binary = state.read().binary_data.clone();
    let Some(binary_bytes) = binary else { return; };

    {
        let mut s = state.write();
        s.is_decompiling  = true;
        s.decompiled_code = None;
        s.decompiled_nir  = None;
        s.push_log(LogEntry::info(format!("Decompiling {name}  @  0x{addr:x}…")));
    }

    wasm_bindgen_futures::spawn_local(async move {
        // All Fission decompiler logic is pure Rust — runs in WASM
        // Note: this blocks the main thread in WASM (no threads available)
        // For large functions a spinner is shown while it runs
        let result = (|| -> Result<(String, Option<String>), String> {
            let loaded = fission_loader::loader::load_binary_from_bytes(&binary_bytes, "binary")
                .map_err(|e| e.to_string())?;
            let binary = Arc::new(loaded);
            let facts = FactStore::build(&binary);
            let config = RustSleighDecompileConfig::default();
            let out = decompile_with_rust_sleigh_with_facts(&binary, addr, &name, &facts, &config)
                .map_err(|e| e.to_string())?;
            Ok((out.pseudocode, out.nir_text))
        })();

        match result {
            Ok((code, nir)) => {
                let bytes = code.len();
                let mut s = state.write();
                s.decompiled_code = Some(code);
                s.decompiled_nir  = nir;
                s.is_decompiling  = false;
                s.push_log(LogEntry::info(format!("Complete  ({bytes} bytes)")));
            }
            Err(e) => {
                let mut s = state.write();
                s.is_decompiling  = false;
                s.push_log(LogEntry::error(format!("Decompile failed: {e}")));
            }
        }
    });
}
