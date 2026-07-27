# fission-web

Browser-native interface for [Fission](https://github.com/fission-systems/Fission) — a Rust-first reverse-engineering and decompilation platform.

Built with [Dioxus](https://dioxuslabs.com/) and compiled to WebAssembly. Railway
serves the static UI and proxies same-origin `/api/*` requests to a private
`fission-backend` service over Railway's internal network.

---

## Table of Contents

- [Architecture](#architecture)
- [Quick Start](#quick-start)
  - [Prerequisites](#prerequisites)
  - [Start the Backend](#start-the-backend)
  - [Open the Web UI](#open-the-web-ui)
- [Development Setup](#development-setup)
  - [Install Toolchain](#install-toolchain)
  - [Local Dev Server](#local-dev-server)
  - [Production Build](#production-build)
- [REST API Reference](#rest-api-reference)
  - [GET /api/status](#get-apistatus)
  - [POST /api/binary](#post-apibinary)
  - [GET /api/functions](#get-apifunctions)
  - [POST /api/decompile/:addr](#post-apidecompileaddr)
  - [GET /api/xrefs/:addr](#get-apixrefsaddr)
- [CI / CD](#ci--cd)
  - [GitHub Actions Workflow](#github-actions-workflow)
  - [Railway Deployment](#railway-deployment)
- [Repository Layout](#repository-layout)
- [Configuration](#configuration)
  - [Dioxus.toml](#dioxustoml)
  - [railway.json](#railwayjson)
  - [Nginx Gateway](#nginx-gateway)
- [Crate Dependencies](#crate-dependencies)
- [WASM Constraints](#wasm-constraints)
- [Troubleshooting](#troubleshooting)
- [Related Projects](#related-projects)
- [License](#license)

---

## Architecture

```
┌──────────────────────────────────────────────────────┐
│  Browser                                             │
│    │ same-origin HTTPS                               │
│    ▼                                                 │
│  Railway: fission-web (Nginx + Dioxus WASM)          │
│    ├── /assets/*  → static WASM/CSS                  │
│    └── /api/*     → private-network reverse proxy    │
└───────────────────────┬──────────────────────────────┘
                        │ railway.internal:7331
┌───────────────────────▼──────────────────────────────┐
│  Railway: fission-backend (Rust / Axum, private)     │
│  ├── fission-loader     — PE/ELF/Mach-O parsing      │
│  ├── fission-decompiler — Rust-Sleigh pipeline       │
│  ├── fission-static     — xrefs, discovery           │
│  └── fission-pcode      — NIR/HIR structuring        │
└──────────────────────────────────────────────────────┘
```

### Design Rationale

Decompilation is inherently CPU-intensive and depends on native Rust libraries (SLEIGH runtime, file I/O, threading). Running the full pipeline inside a browser WASM sandbox is not practical:

- SLEIGH requires runtime file access to `.sla` spec files
- The decompilation pipeline uses `rayon`/`tokio` thread pools
- Binary parsing allocates large memory buffers incompatible with WASM memory limits

The UI remains lightweight: only the Dioxus component tree, state management,
and HTTP client compile to WASM. The heavyweight Rust logic runs in the private
Railway backend. Nginx is the single public gateway, which avoids cross-origin
browser traffic and keeps the backend service off the public internet.

### Local resource model

Yes, Fission resources can run from the user's local machine. In the current
web architecture the browser does not read those paths itself. The local
`fission serve` process resolves SLEIGH artifacts, signatures, FID data, and
type information through Fission's normal resource configuration:

- `--resource-root <path>` when the server is started through a supporting CLI
- `FISSION_RESOURCE_ROOT` for signature/type/FID bundles
- `FISSION_SLEIGH_SPEC_DIR` for the SLEIGH resource tree
- installed, user-data, and Fission workspace resource locations

`GET /api/status` reports the backend resource mode and availability without
exposing absolute host paths. A future in-browser analysis worker must instead
use a bundle explicitly selected by the user or versioned packaged artifacts;
web pages cannot automatically open arbitrary local filesystem paths.

### Railway cloud backend

The canonical backend container and deployment configuration live in the
Fission repository:

- [`Dockerfile`](https://github.com/fission-systems/Fission/blob/main/Dockerfile)
- [`railway.json`](https://github.com/fission-systems/Fission/blob/main/railway.json)
- [Railway deployment guide](https://github.com/fission-systems/Fission/blob/main/docs/RAILWAY.md)

The production frontend is built with an empty `FISSION_WEB_API_URL`, so the
client uses its current origin and Nginx forwards `/api/*` privately. Enter
`FISSION_SERVE_API_TOKEN` through the connection banner at runtime. Never
compile that token into the WASM bundle.

---

## Quick Start

### Prerequisites

| Requirement | Version | Notes |
|---|---|---|
| Rust | stable | Install via [rustup](https://rustup.rs) |
| `wasm32-unknown-unknown` target | — | `rustup target add wasm32-unknown-unknown` |
| Dioxus CLI (`dx`) | 0.7+ | See [Install Toolchain](#install-toolchain) |
| Fission CLI (`fission_cli`) | latest | Built from [fission-systems/Fission](https://github.com/fission-systems/Fission) |

### Start the Backend

Build and run the local analysis server from the Fission repository:

```bash
# Build the CLI (release recommended — significantly faster decompilation)
cd /path/to/Fission
cargo build -p fission-cli --release

# Start the API server on port 7331 (default)
./target/release/fission_cli serve --port 7331
```

Expected output:

```
INFO fission serve  →  http://localhost:7331
INFO Open fission-web in your browser and connect to this server.
```

The server keeps the loaded binary in memory for the duration of the session. Restart to load a different binary or simply upload a new file — the new binary replaces the previous one.

### Open the Web UI

Navigate to the Railway `fission-web` domain or run a local dev build (see
[Local Dev Server](#local-dev-server)).

1. The UI shows a drop zone on first load.
2. Drag-and-drop or click **Choose file** to upload a binary.
3. The file is sent to `fission serve` via `POST /api/binary`.
4. The function list populates in the sidebar.
5. Click any function to decompile it.

> **Local development**: enter `http://localhost:7331` in the backend URL field.
> Production uses the empty same-origin URL and the Railway gateway.

---

## Development Setup

### Install Toolchain

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Dioxus CLI using cargo-binstall (avoids long source compilation)
cargo install cargo-binstall
cargo binstall dioxus-cli --no-confirm

# Verify
dx --version   # should print dioxus-cli 0.7.x
```

### Local Dev Server

```bash
cd /path/to/fission-web

# Hot-reload dev build — starts a local HTTP server at http://localhost:8080
dx serve --platform web
```

The dev server watches source files and recompiles on change. Dioxus 0.7 supports hot-reload for RSX templates without a full rebuild.

To use a backend other than `localhost:7331`, set the server URL in the
connection banner. Leave it empty when testing a same-origin gateway.

### Production Build

```bash
# Output: target/dx/fission-web/release/web/public/
dx build --platform web --release
```

The output directory contains `index.html`, the compiled `.wasm` file, and all
bundled assets. The Railway Docker image copies this directory into Nginx.

To inspect build output size:

```bash
ls -lh target/dx/fission-web/release/web/public/assets/*.wasm
```

The release profile in `Cargo.toml` is configured for minimum WASM size:

```toml
[profile.release]
opt-level = "z"    # optimize for size
lto = true
codegen-units = 1
```

---

## REST API Reference

In production, all endpoints are exposed through the same Railway frontend
origin under `/api`. For local development, `fission serve` defaults to
`http://localhost:7331`.

### GET /api/status

Returns server health and currently loaded binary info.

**Response**

```json
{
  "version": "0.1.0",
  "binary": "example.exe",
  "fn_count": 1247
}
```

| Field | Type | Description |
|---|---|---|
| `version` | `string` | Fission CLI version |
| `binary` | `string \| null` | Name of the currently loaded binary |
| `fn_count` | `number` | Number of discovered functions |

---

### POST /api/binary

Uploads a binary file. Accepts `multipart/form-data` with a single file field.

**Request**

```
Content-Type: multipart/form-data
Body: file=<binary bytes>
```

**Response (200)**

```json
{
  "fn_count": 1247,
  "summary": "PE32+ | 64-bit | 1247 functions | entry 0x1400010a0"
}
```

**Response (422)**

```json
{ "error": "unsupported format" }
```

Supported formats: PE (32/64), ELF (32/64), Mach-O, raw binary. The binary is parsed by `fission-loader` and held in server memory; existing binary is replaced on each upload.

---

### GET /api/functions

Returns the full function list for the currently loaded binary.

**Response (200)**

```json
[
  {
    "addr": 5368717984,
    "name": "main",
    "is_import": false,
    "is_export": true,
    "is_thunk": false,
    "size": 1024
  },
  {
    "addr": 5368715264,
    "name": "printf",
    "is_import": true,
    "is_export": false,
    "is_thunk": true,
    "size": 6
  }
]
```

| Field | Type | Description |
|---|---|---|
| `addr` | `number` | Function start address (decimal) |
| `name` | `string` | Symbol name or auto-generated `sub_<hex>` |
| `is_import` | `bool` | True if the function is an imported symbol |
| `is_export` | `bool` | True if the function is exported |
| `is_thunk` | `bool` | True if the function is a thunk/trampoline |
| `size` | `number` | Function byte size |

**Response (404)** — no binary loaded

```json
{ "error": "no binary loaded" }
```

---

### POST /api/decompile/:addr

Decompiles the function at the given hex address.

**Path parameter**: `addr` — function address as a hex string (without `0x` prefix), e.g. `/api/decompile/1400010a0`.

**Response (200)**

```json
{
  "pseudocode": "int __cdecl main(int argc, char **argv) {\n    ...\n}",
  "nir": "fn main(r14:i32, r15:i64) -> i32 { ... }",
  "fell_back": false,
  "reason": null
}
```

| Field | Type | Description |
|---|---|---|
| `pseudocode` | `string` | C-like decompiled pseudocode |
| `nir` | `string \| null` | Normalized IR (NIR) text, if available |
| `fell_back` | `bool` | True if decompilation fell back to a simpler output |
| `reason` | `string \| null` | Reason for fallback, if applicable |

Decompilation runs on a `tokio::task::spawn_blocking` thread to avoid blocking the async server. Typical latency: 50–500 ms per function depending on binary complexity.

**Response (400)** — invalid address format

```json
{ "error": "invalid address" }
```

**Response (404)** — no binary loaded

```json
{ "error": "no binary loaded" }
```

---

### GET /api/xrefs/:addr

Returns cross-references for the function at the given hex address.

**Path parameter**: `addr` — same format as `/api/decompile/:addr`.

**Response (200)**

```json
{
  "callers": [
    {
      "from_addr": 5368719024,
      "to_addr": 5368717984,
      "kind": "Call",
      "symbol": null,
      "fn_name": "init_app"
    }
  ],
  "callees": [
    {
      "from_addr": 5368717984,
      "to_addr": 5368715264,
      "kind": "Call",
      "symbol": "printf",
      "fn_name": null
    }
  ]
}
```

| Field | Type | Description |
|---|---|---|
| `callers` | `XrefRow[]` | Functions that call the target |
| `callees` | `XrefRow[]` | Functions called by the target |
| `from_addr` | `number` | Source address of the reference |
| `to_addr` | `number \| null` | Target address |
| `kind` | `string` | Reference kind: `Call`, `Jump`, `ConditionalJump`, etc. |
| `symbol` | `string \| null` | Symbol name at the target, if available |
| `fn_name` | `string \| null` | Enclosing function name of the source |

---

## CI / CD

### GitHub Actions Workflow

`.github/workflows/ci.yml` runs on every push to `main` and on pull requests
targeting `main`.

**Steps:**

| Step | Action |
|---|---|
| Checkout | `actions/checkout@v4` |
| Install Rust stable | `dtolnay/rust-toolchain@stable` with `wasm32-unknown-unknown` target |
| Cache Cargo registry | `actions/cache@v4` keyed on `Cargo.lock` hash |
| Install Dioxus CLI | `cargo binstall dioxus-cli --version 0.7.9` |
| `cargo build --target wasm32-unknown-unknown --release` | Runs first to surface readable Rust errors before `dx build` wraps them |
| `dx build --platform web --release` | Produces output in `target/dx/fission-web/release/web/public/` |
| Verify output | Requires the generated `index.html` and WASM asset |

### Railway Deployment

Railway watches `fission-systems/fission-web` and builds `Dockerfile` on each
`main` push. The multi-stage image:

1. installs the pinned Dioxus CLI,
2. compiles the WASM application with a same-origin API base,
3. copies the static output into Nginx, and
4. proxies `/api/*` to `fission-backend.railway.internal:7331`.

No deployment secrets are compiled into the frontend. The bearer token remains
runtime-only browser input.

---

## Repository Layout

```
fission-web/
├── .github/
│   └── workflows/
│       └── ci.yml              # WASM build validation
├── assets/
│   └── style.css               # Global CSS (loaded via Manganis asset! macro)
├── deploy/
│   └── nginx/
│       └── default.conf.template # Static hosting + private API proxy
├── src/
│   ├── main.rs                 # Entry point — console_error_panic_hook + dioxus::launch
│   ├── state.rs                # Re-export of fission_ui::state (AppState, signals)
│   ├── engine.rs               # Re-export stub (engine lives in fission-ui)
│   └── components/
│       ├── mod.rs              # Component module declarations
│       ├── app.rs              # Root App component — title bar, layout, status bar
│       └── dropzone.rs         # File drop/upload zone (web-sys FileReader + spawn_local)
├── Cargo.toml                  # Package manifest and dependencies
├── Cargo.lock                  # Pinned dependency versions
├── Dioxus.toml                 # Dioxus 0.7 project configuration
├── Dockerfile                  # Railway build/runtime image
├── railway.json                # Railway healthcheck and restart policy
├── index.html                  # HTML shell injected by dx build
├── AGENTS.md                   # Agent/contributor guide
└── README.md                   # This file
```

Shared UI components (`Sidebar`, `Editor`, `BottomPanel`, `CommandPalette`, `CFGView`, `XrefsView`) and platform-split engine logic (`run_load`, `run_decompile`, `run_xrefs`) live in the `fission-ui` crate inside the main Fission repository and are referenced here as a git dependency.

---

## Configuration

### Dioxus.toml

```toml
[application]
name = "fission-web"
default_platform = "web"

[web.app]
title = "Fission"

[web.watcher]
reload_html = true

[web.resource]
style = []
script = []
```

Dioxus 0.7 requires this file to be present and uses it for build metadata. The `asset!()` macro in source files (Manganis) resolves paths relative to the crate root — do not use a leading `/` in asset paths.

### railway.json

```json
{
  "$schema": "https://railway.com/railway.schema.json",
  "build": {
    "builder": "DOCKERFILE",
    "dockerfilePath": "Dockerfile"
  },
  "deploy": {
    "healthcheckPath": "/healthz"
  }
}
```

### Nginx Gateway

The runtime container listens on Railway's `PORT`, serves the Dioxus bundle,
and forwards `/api/*` to:

```text
fission-backend.railway.internal:7331
```

`client_max_body_size` is aligned with the backend's 50 MB upload limit.
`FISSION_BACKEND_HOST` and `FISSION_BACKEND_PORT` can override the internal
target without rebuilding the image.

---

## Crate Dependencies

| Crate | Purpose |
|---|---|
| `dioxus` (0.7, `web` feature) | WASM component framework |
| `dioxus-logger` (0.7) | Browser console logging via `tracing` |
| `tracing` | Structured logging |
| `wasm-bindgen` | Rust ↔ JS interop |
| `wasm-bindgen-futures` | `spawn_local` — run async futures from JS event callbacks |
| `web-sys` | FileReader, File, DragEvent, HtmlInputElement, Blob, FormData |
| `js-sys` | Uint8Array, Array, for binary data manipulation |
| `gloo-timers` | WASM-compatible timer utilities |
| `console_error_panic_hook` | Converts Rust panics to readable browser console messages |
| `serde` / `serde_json` | JSON serialization for API responses |
| `fission-ui` (git) | Shared Dioxus components and engine (platform-split) |

`fission-ui` itself depends on `gloo-net` (WASM HTTP client) when compiled for `wasm32` and on `fission-decompiler`, `fission-static`, and `fission-loader` when compiled for native.

---

## WASM Constraints

Running in the browser imposes several constraints that shaped the Option A architecture:

| Constraint | Impact |
|---|---|
| No filesystem access | SLEIGH `.sla` spec files cannot be read at runtime |
| No threads (single-threaded WASM) | `rayon` and `tokio::spawn_blocking` are unavailable |
| Memory limit (~2 GB on most browsers) | Large binaries may exhaust WASM linear memory |
| No native OS calls | `mmap`, `dlopen`, and similar syscalls are unavailable |
| JS async context isolation | Dioxus `spawn()` requires an active Dioxus runtime; callbacks from JS events (FileReader, setTimeout) must use `wasm_bindgen_futures::spawn_local` instead |

The last point is the reason `dropzone.rs` uses `wasm_bindgen_futures::spawn_local` inside the `FileReader.onload` callback rather than Dioxus's own `spawn`. Dioxus `spawn` internally calls `RUNTIMES.with(|r| r.borrow().last().unwrap())` — if no Dioxus render is active on the call stack, this panics with `called Option::unwrap() on a None value`.

---

## Troubleshooting

### "Enter the API token above to begin analysis"

The WASM client has not authenticated with the backend. In production, paste
the Railway bearer token and leave the backend URL empty. For local development,
ensure:

1. `fission_cli serve --port 7331` is running.
2. No firewall rule blocks `localhost:7331`.
3. The browser is not in a restricted environment (e.g. some corporate proxies intercept localhost requests).

### Upload fails silently

Check the browser developer console (F12) for network errors on `POST /api/binary`. Common causes:

- `fission serve` is not running — connection refused.
- Local CORS error — add the development origin to the backend allow-list or
  use the production same-origin Railway gateway.
- Binary format not supported — the server returns `422 Unprocessable Entity`.

### Rust panic in the browser

With `console_error_panic_hook` enabled, Rust panics appear in the browser console as readable backtraces rather than `RuntimeError: unreachable`. Search for lines beginning with `panicked at`.

```
panicked at src/components/app.rs:42:5:
explicit panic
```

If you see `panicked at dioxus-core/src/runtime.rs:223`: a Dioxus hook or `spawn()` was called outside a component render. Use `wasm_bindgen_futures::spawn_local` for JS event callbacks.

### `dx build` fails with "FISSION_SLEIGH_SPEC_DIR not found"

The SLEIGH spec directory must be present before building. In CI this is handled by the `Download fission-utils bundle` step. Locally:

```bash
# Download and extract the utils bundle
curl -fsSL \
  "https://github.com/fission-systems/Fission/releases/download/assets-v1/fission-utils.tar.gz" \
  | tar -xz -C ./

export FISSION_SLEIGH_SPEC_DIR="$PWD/utils/sleigh-specs"
dx build --platform web --release
```

Or point `FISSION_SLEIGH_SPEC_DIR` at your local Fission checkout:

```bash
export FISSION_SLEIGH_SPEC_DIR=/path/to/Fission/utils/sleigh-specs
```

### Railway deployment shows stale content

1. Hard-refresh the browser (`Cmd+Shift+R` / `Ctrl+Shift+R`).
2. Check the Railway `fission-web` deployment status and logs.
3. Confirm `/healthz` and the hashed WASM asset return HTTP 200.

### Large WASM binary size

The WASM bundle is large because it includes all Dioxus component logic, serialization code, and gloo-net. To inspect what contributes most to size:

```bash
cargo install twiggy
twiggy top target/dx/fission-web/release/web/public/assets/*.wasm
```

---

## Related Projects

| Repository | Description |
|---|---|
| [fission-systems/Fission](https://github.com/fission-systems/Fission) | Core decompiler engine (Rust). Contains `fission-cli`, `fission-ui`, `fission-pcode`, `fission-decompiler`, `fission-loader`, `fission-static`. |
| [fission-systems/fission-benchmark](https://github.com/fission-systems/fission-benchmark) | External benchmark runner for quality measurement against real-world binaries. |
| [DioxusLabs/dioxus](https://github.com/DioxusLabs/dioxus) | The Rust UI framework used by fission-web and fission-dioxus. |

---

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE) or the [GNU AGPL v3 full text](https://www.gnu.org/licenses/agpl-3.0.html).
