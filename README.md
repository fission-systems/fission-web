# fission-web

Browser-native decompiler interface for [Fission](https://github.com/fission-systems/Fission).

Built with [Dioxus](https://dioxuslabs.com/) — compiles to WebAssembly and runs entirely in the user's browser.
**No binary data is ever sent to a server.**

## Architecture

```
Vercel (static files only)
  └── fission-web.wasm  ← runs in the user's browser
        ├── fission-loader     (binary parsing)
        ├── fission-decompiler (pseudocode generation)
        └── fission-static     (analysis services)
```

All decompilation computation happens on the **user's CPU** via WebAssembly.

## Development

```bash
# Install Dioxus CLI
cargo install dioxus-cli

# Add WASM target
rustup target add wasm32-unknown-unknown

# Run dev server (hot-reload)
dx serve --platform web

# Production build
dx build --platform web --release
# Output: ./dist/
```

## Deploy

Configured for [Vercel](https://vercel.com) via `vercel.json`.

```bash
vercel deploy --prod
```

## Related

- [fission-systems/Fission](https://github.com/fission-systems/Fission) — core decompiler engine
- [fission-systems/fission-benchmark](https://github.com/fission-systems/fission-benchmark) — quality benchmarks
