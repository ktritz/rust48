// WASM interface via wasm-bindgen
// Replaces Emscripten ccall/cwrap with typed wasm-bindgen exports.

use wasm_bindgen::prelude::*;

use crate::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::emulator::Emulator;
use crate::types::Model;

#[wasm_bindgen]
pub struct Hp48 {
    emu: Emulator,
}

#[wasm_bindgen]
impl Hp48 {
    /// Create a new emulator instance.
    /// `rom` — ROM data (nibble or packed byte format).
    /// `ram` — optional RAM data (nibble or packed byte format).
    /// `state` — optional saved state (binary format from save_state).
    #[wasm_bindgen(constructor)]
    pub fn new(rom: &[u8], ram: Option<Vec<u8>>, state: Option<Vec<u8>>) -> Self {
        // Auto-detect model from ROM size
        let model = if rom.len() > ROM_SIZE_SX_PACKED {
            Model::Gx
        } else {
            Model::Sx
        };
        Self {
            emu: Emulator::new(rom, ram.as_deref(), state.as_deref(), model),
        }
    }

    /// Start emulation timers. Call once after construction.
    /// `now_secs` — monotonic time in seconds (e.g. performance.now() / 1000).
    /// `unix_epoch_secs` — wall-clock seconds since Unix epoch, local time
    ///   (e.g. Date.now()/1000 - new Date().getTimezoneOffset()*60).
    pub fn start(&mut self, now_secs: f64, unix_epoch_secs: f64) {
        self.emu.start(now_secs, unix_epoch_secs);
    }

    /// Push a key event into the queue.
    /// Bit 31 clear = press, bit 31 set = release.
    /// Bits [7:4] = row, bits [3:0] = column.
    pub fn push_key_event(&mut self, code: u32) {
        self.emu.keyboard.push_key_event(code);
    }

    /// Get pointer to the RGBA display buffer (for use with WASM memory).
    pub fn display_buffer_ptr(&self) -> *const u8 {
        self.emu.display.rgba.as_ptr()
    }

    pub fn display_width(&self) -> u32 {
        DISPLAY_WIDTH
    }

    pub fn display_height(&self) -> u32 {
        DISPLAY_HEIGHT
    }

    pub fn is_display_dirty(&self) -> bool {
        self.emu.is_display_dirty()
    }

    pub fn clear_display_dirty(&mut self) {
        self.emu.clear_display_dirty();
    }

    /// Get current annunciator state as bitmask.
    pub fn annunciator_state(&self) -> u32 {
        self.emu.annunciator_state()
    }

    /// Get detected speaker frequency in Hz (0 = no tone).
    /// Call every ~20ms from JS.
    pub fn speaker_frequency(&mut self) -> u32 {
        self.emu.speaker_frequency()
    }

    /// Serialize CPU state to binary format (compatible with C version).
    pub fn save_state(&self) -> Vec<u8> {
        self.emu.save_state()
    }

    /// Serialize RAM to packed byte format.
    pub fn save_ram(&self) -> Vec<u8> {
        self.emu.save_ram()
    }

    /// Run one frame of emulation.
    /// `elapsed_ms` — milliseconds since last frame.
    /// `now_secs` — current time in seconds.
    pub fn run_frame(&mut self, elapsed_ms: f64, now_secs: f64) {
        self.emu.run_frame(elapsed_ms, now_secs);
    }
}

/// Packed ROM size threshold for SX model detection
const ROM_SIZE_SX_PACKED: usize = crate::types::ROM_SIZE_SX / 2;
