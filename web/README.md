# web/ — Web Frontend

Browser UI for the HP-48 GX emulator. Handles display rendering, SVG button generation, keyboard/touch input, audio output, and state persistence.

## Two Independent Paths

### Rust/WASM path (active)

- **`rust.html`** — entry point
- **`hp48_rust.ts`** — TypeScript bridge (ESM module)
- **`hp48_rust.js`** — esbuild bundle of the above
- **`rust48_bg.wasm`** — Rust emulator compiled to WASM via `wasm-pack`

Build:
```sh
wasm-pack build --target web --release
npx esbuild web/hp48_rust.ts --bundle --format=esm --outfile=web/hp48_rust.js
cp pkg/rust48_bg.wasm web/rust48_bg.wasm
```

### C/Emscripten path (legacy)

- **`index.html`** — entry point
- **`hp48.ts`** — TypeScript bridge (IIFE)
- **`hp48.js`** — esbuild bundle of the above
- **`hp48_emu.js`** / **`hp48_emu.wasm`** / **`hp48_emu.data`** — Emscripten output

Build: `make` (requires `emcc`)

## Shared Code

Both paths share:

- **`style.css`** — calculator skin styling (beveled display, button grid layout)
- **`favicon.png`** — browser tab icon
- **`assets/`** — ROM, RAM, and state files loaded at startup

## SVG Button System

49 calculator buttons are generated at runtime from a `BUTTONS` data array. Features:

- Per-button SVG with gradient body, shadow, highlight, and bevel effects
- Shift labels (left-shift purple, right-shift teal) with math text rendering
- Alpha labels (A-Z) at bottom-right corners
- Superscript/italic handling for math symbols
- Shift arrow keys with gradient-matched stroke
- Light gray background on number pad keys (1-9) and right-shift

## Display

- **LCD canvas**: 131x64 pixels (1x scale), CSS-scaled to fill container
- **Annunciator canvas**: 262x12 pixels (2x DPI for crisp icons), XBM bitmaps for 6 indicators
- `image-rendering: pixelated` for sharp pixel scaling

## Persistence

State is saved to IndexedDB (`hp48_rust` database) via auto-save every 30 seconds, plus on page close/hide. On startup, IndexedDB is checked first, falling back to bundled `assets/` files.

## Dev Server

```sh
cd web && python3 -m http.server 8088
# Open http://localhost:8088/rust.html
```
