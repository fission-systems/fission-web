//! Fission Web — Dioxus WASM entry point.
//!
//! All decompilation computation runs locally in the user's browser via
//! WebAssembly. No binary data is ever sent to a server.

use dioxus::prelude::*;
use tracing::Level;

mod components;
mod engine;
mod state;

use components::app::App;

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}
