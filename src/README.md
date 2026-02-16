# src/ — Rust Emulator Core

Saturn CPU emulator ported from the x48/droid48 C codebase. ~6,100 lines of Rust.

## Module Map

| Module | Lines | C Source | Description |
|--------|-------|----------|-------------|
| `types.rs` | 108 | `hp48.h` | Nibble/word types, ROM/RAM size constants, `Model` enum |
| `cpu.rs` | 253 | `hp48.h` `saturn_t` | CPU registers, PC, flags, return stack |
| `alu.rs` | 602 | `register.c` | Register arithmetic/logic — field-based nibble ops, BCD |
| `decode.rs` | 1346 | `emulate.c` | Instruction decoder — nested match tree for all opcodes |
| `actions.rs` | 274 | `actions.c` | CPU actions: interrupts, shutdown, config, reset |
| `memory.rs` | 1264 | `memory.c` | MMU address mapping, memory-mapped I/O for SX and GX |
| `display.rs` | 264 | `lcd.c` | LCD rendering to RGBA pixel buffer |
| `timer.rs` | 149 | `timer.c` | Hardware timers (T1, T2) and wall-clock sync |
| `keyboard.rs` | 77 | `x48_web.c` | Key matrix and event queue |
| `device.rs` | 35 | `device.c` | Device "touched" flags |
| `speaker.rs` | 70 | `device.c` | Speaker toggle frequency detection |
| `serial.rs` | 18 | `serial.c` | Serial port (stub) |
| `scheduler.rs` | 78 | `emulate.c` | Instruction scheduling and timer checks |
| `persist.rs` | 385 | `init.c` | Binary state serialization (compatible with C save files) |
| `emulator.rs` | 1057 | `main_wasm.c` | Top-level `Emulator` struct composing all modules |
| `platform/wasm.rs` | 100 | — | `wasm-bindgen` exports (`Hp48` struct) |

## Key Design Decisions

- **Central `Emulator` struct** replaces C globals — all cross-module methods are `impl Emulator`
- **`Model` enum** (`Sx`/`Gx`) replaces C function pointers for memory map dispatch
- **Field-based nibble ops** use the same `START_FIELDS`/`END_FIELDS` lookup tables as the C code
- **BCD arithmetic** honors `hexmode` flag (base 10 for decimal, base 16 for hex)
- **Save file format** is byte-compatible with C version (big-endian, same field order)

## C Source

The original C files live in `src/emu/` for reference. They are not part of the Rust build.
