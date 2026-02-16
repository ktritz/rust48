# src-tauri/ — Tauri Desktop App

Tauri v2 wrapper that packages the web frontend as a native Windows desktop application.

## How It Works

Tauri embeds the `web/` directory as the frontend and renders it in a WebView2 window. The Rust emulator runs as WASM inside the webview (same as the browser version). No native Rust emulator integration — it's purely a webview wrapper.

## Building

Requires the Rust toolchain, Tauri CLI, and Windows SDK (for WebView2).

```sh
# From project root:
cd src-tauri && cargo tauri build

# Output:
# src-tauri/target/release/rust48.exe
```

## Configuration

- **`tauri.conf.json`** — window size (564x950), entry point (`rust.html`), app identifier
- **`Cargo.toml`** — release profile: LTO, single codegen unit, stripped, abort on panic
- **`capabilities/default.json`** — Tauri permission grants

## WebView2 Notes

The Windows WebView2 runtime has some CSS/canvas differences from desktop browsers:

- `::after` pseudo-elements with `z-index` may not render — use physical divs
- Canvas `width` attribute can override CSS `width: 100%` — set inline styles in JS
- `beforeunload` may not fire reliably — use `visibilitychange` and `pagehide` as fallbacks
