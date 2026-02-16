# rust48

HP-48 GX calculator emulator written in Rust, compiled to WebAssembly for the browser and packaged as a native Windows desktop app via Tauri.

Source-accurate port of the [x48](https://github.com/gwenhael-le-moine/x48)/droid48 C emulator (~11,300 lines of C) into ~6,100 lines of Rust. Same instruction decoding, memory mapping, BCD arithmetic, timer synchronization, and save file format.

## Architecture

The emulator core is a pure Rust library (`src/`) with no platform dependencies. It compiles to:

- **WebAssembly** via `wasm-pack` + `wasm-bindgen` for the browser
- **Native** via `cargo build` for the Tauri desktop app

The web frontend (`web/`) handles display rendering, SVG button generation, keyboard input, audio, and state persistence via IndexedDB.

```
src/            Rust emulator core (Saturn CPU, MMU, ALU, display, timers, ...)
web/            Web frontend (TypeScript, HTML, CSS, WASM glue)
src-tauri/      Tauri desktop app wrapper
assets/         ROM, RAM, and state files
```

## Building

### Prerequisites

- Rust toolchain (`rustup`)
- `wasm-pack` (`cargo install wasm-pack`)
- Node.js (for esbuild and Tauri CLI)

### Web (WASM)

```sh
# Build Rust to WASM
wasm-pack build --target web --release

# Bundle TypeScript
npx esbuild web/hp48_rust.ts --bundle --format=esm --outfile=web/hp48_rust.js

# Copy WASM to web dir
cp pkg/rust48_bg.wasm web/rust48_bg.wasm

# Serve
cd web && python3 -m http.server 8088
# Open http://localhost:8088/rust.html
```

### Desktop (Tauri)

```sh
# On Windows (or WSL targeting Windows):
cd src-tauri && cargo tauri build
# Output: src-tauri/target/release/rust48.exe
```

### C/Emscripten path (legacy)

The original C emulator can still be built via Emscripten:

```sh
make    # requires emcc
# Open web/index.html
```

### Tests

```sh
cargo test
```

## Two Web Paths

The project contains two independent web frontends:

| | C/Emscripten | Rust/WASM |
|---|---|---|
| Entry | `web/index.html` | `web/rust.html` |
| Bridge | `web/hp48.ts` (IIFE) | `web/hp48_rust.ts` (ESM) |
| Binary | `hp48_emu.js` + `.wasm` + `.data` | `rust48_bg.wasm` |
| Display | 262x142 (2x scale + header) | 131x64 (1x) + 262x12 annunciator |
| Persistence | Emscripten IDBFS | IndexedDB (`hp48_rust` database) |

The Tauri desktop app uses the Rust/WASM path.

## Credits

Based on the x48 emulator by Eddie C. Dost and the droid48 Android port. Saturn CPU architecture by Hewlett-Packard.
