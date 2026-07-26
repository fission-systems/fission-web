# fission-web Agent Guide

## Overview

fission-web is a Dioxus WASM frontend for the Fission decompiler.
The current production backend is a user-run local `fission serve` process.
The browser uploads the selected binary to that configured backend, which owns
decompilation and resolves SLEIGH/signature resources from the host filesystem.
A future browser-worker backend may use explicitly selected or packaged resource
artifacts; browser code must never claim direct access to arbitrary local paths.

## Structure

```
fission-web/
├── src/
│   ├── main.rs           # dioxus::launch() entry point
│   ├── state.rs          # AppState (WASM-compatible)
│   ├── engine.rs         # Shared analysis backend adapter re-export
│   └── components/
│       ├── mod.rs
│       ├── app.rs        # Root App component
│       ├── sidebar.rs    # Function list
│       ├── editor.rs     # Pseudocode / NIR / Hex tabs
│       └── dropzone.rs   # Drag-and-drop binary loader (browser File API)
├── assets/
│   └── style.css         # Design system
├── index.html            # Dioxus web entry
├── vercel.json           # Vercel static deployment
└── Cargo.toml
```

## Key Differences from fission-dioxus (desktop)

| Concern          | Desktop                   | Web (this crate)               |
|------------------|---------------------------|--------------------------------|
| File loading     | `rfd` native dialog       | browser `<input>` / drag-drop  |
| Background tasks | `tokio::task::spawn_blocking` | `wasm_bindgen_futures::spawn_local` |
| File API         | `std::fs`                 | `web-sys FileReader`           |
| Threading        | OS threads                | async WASM client + local backend |
| Build            | `cargo build`             | `dx build --platform web`      |

## Build / Dev Commands

```bash
# Dev server with hot-reload
dx serve --platform web

# Production build → ./dist/
dx build --platform web --release

# Type check only
cargo check --target wasm32-unknown-unknown
```

## Core Rules

1. Never use `tokio::task::spawn_blocking` — use `wasm_bindgen_futures::spawn_local`.
2. Never use `std::fs` — use `web-sys` File/FileReader API.
3. All Fission core logic must remain in `fission-systems/Fission` crates.
4. This repo contains only the UI adapter layer.
5. Describe the active trust boundary precisely: default analysis sends binary
   bytes to the user-configured `fission serve` backend, normally localhost.
6. Browsers cannot automatically read `FISSION_RESOURCE_ROOT` or arbitrary host
   paths. Local-server resources and explicitly selected browser bundles are
   distinct resource modes.
