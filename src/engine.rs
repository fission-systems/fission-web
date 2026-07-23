// engine.rs — re-export shared engine from fission-ui
// Web-platform async wrappers (run_load, run_decompile, run_xrefs) are
// already gated with #[cfg(target_arch = "wasm32")] inside fission-ui.
pub use fission_ui::engine::*;
