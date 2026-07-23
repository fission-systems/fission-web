//! Application state — mirrors fission-dioxus AppState but adapted for WASM.
//!
//! Key differences from desktop:
//! - Binary data stored as Vec<u8> (loaded via browser File API)
//! - No file paths (browser sandbox)
//! - No tokio::task::spawn_blocking (use wasm_bindgen_futures::spawn_local)

use dioxus::prelude::*;
use std::sync::Arc;

// ── Log entry ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum LogLevel { Info, Warn, Error }

#[derive(Clone, PartialEq)]
pub struct LogEntry {
    pub level:   LogLevel,
    pub message: String,
}

impl LogEntry {
    pub fn info(msg: impl Into<String>)  -> Self { Self { level: LogLevel::Info,  message: msg.into() } }
    pub fn warn(msg: impl Into<String>)  -> Self { Self { level: LogLevel::Warn,  message: msg.into() } }
    pub fn error(msg: impl Into<String>) -> Self { Self { level: LogLevel::Error, message: msg.into() } }
}

// ── Function entry ─────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Default)]
pub struct FunctionEntry {
    pub address:  u64,
    pub name:     String,
    pub is_import: bool,
    pub is_export: bool,
    pub is_thunk:  bool,
}

// ── Editor tab ─────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Default)]
pub enum EditorTab {
    #[default]
    Pseudocode,
    Nir,
    Hex,
}

// ── App state ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    // Binary
    pub binary_name:  Option<String>,
    pub binary_data:  Option<Arc<Vec<u8>>>,
    pub functions:    Vec<FunctionEntry>,

    // Editor
    pub active_tab:          EditorTab,
    pub current_fn_addr:     Option<u64>,
    pub decompiled_code:     Option<String>,
    pub decompiled_nir:      Option<String>,

    // Async guards
    pub is_loading:      bool,
    pub is_decompiling:  bool,

    // Sidebar
    pub sidebar_search:  String,

    // Log
    pub logs: Vec<LogEntry>,

    // Navigation history
    pub nav_history: Vec<u64>,
    pub nav_cursor:  usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            binary_name:     None,
            binary_data:     None,
            functions:       Vec::new(),
            active_tab:      EditorTab::default(),
            current_fn_addr: None,
            decompiled_code: None,
            decompiled_nir:  None,
            is_loading:      false,
            is_decompiling:  false,
            sidebar_search:  String::new(),
            logs:            Vec::new(),
            nav_history:     Vec::new(),
            nav_cursor:      0,
        }
    }
}

impl AppState {
    pub fn push_log(&mut self, entry: LogEntry) {
        self.logs.push(entry);
        if self.logs.len() > 500 {
            self.logs.remove(0);
        }
    }

    pub fn navigate_to(&mut self, addr: u64) {
        if self.nav_history.get(self.nav_cursor) == Some(&addr) { return; }
        if !self.nav_history.is_empty() {
            self.nav_history.truncate(self.nav_cursor + 1);
        }
        self.nav_history.push(addr);
        if self.nav_history.len() > 50 { self.nav_history.remove(0); }
        self.nav_cursor = self.nav_history.len().saturating_sub(1);
    }

    pub fn nav_back(&mut self) -> Option<u64> {
        if self.nav_cursor == 0 { return None; }
        self.nav_cursor -= 1;
        Some(self.nav_history[self.nav_cursor])
    }

    pub fn nav_forward(&mut self) -> Option<u64> {
        if self.nav_cursor + 1 >= self.nav_history.len() { return None; }
        self.nav_cursor += 1;
        Some(self.nav_history[self.nav_cursor])
    }

    pub fn can_nav_back(&self) -> bool {
        self.nav_cursor > 0 && !self.nav_history.is_empty()
    }

    pub fn can_nav_forward(&self) -> bool {
        self.nav_cursor + 1 < self.nav_history.len()
    }
}

// ── Global state hook ──────────────────────────────────────────────────────

pub fn use_app_state() -> Signal<AppState> {
    use_context::<Signal<AppState>>()
}
