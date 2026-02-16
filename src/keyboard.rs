// Keyboard handling — key matrix + event queue
// Exact port of x48_web.c key_event / GetEvent / push_key_event

use crate::cpu::Saturn;

pub struct Keyboard {
    pub event_queue: Vec<u32>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            event_queue: Vec::new(),
        }
    }

    /// Push a key event from JS/WASM.
    /// Bit 31 clear = press, bit 31 set = release.
    /// Low bits = Saturn keycode (0x8000 for ON, else row<<4|col).
    pub fn push_key_event(&mut self, code: u32) {
        self.event_queue.push(code);
    }

    /// Process queued key events into the Saturn key matrix.
    /// Returns true if any event was processed AND a keyboard interrupt
    /// should be triggered (i.e., a new key was pressed).
    pub fn process_events(&mut self, saturn: &mut Saturn) -> bool {
        if self.event_queue.is_empty() {
            return false;
        }
        let mut need_kbd_int = false;

        // Process all queued events (drain in FIFO order)
        let events: Vec<u32> = self.event_queue.drain(..).collect();
        for code in events {
            let press = (code & 0x80000000) == 0;
            let keycode = (code & 0x7fffffff) as i32;

            if press {
                if keycode == 0x8000 {
                    // ON key press: set bit 15 on all rows, always interrupt
                    for i in 0..9 {
                        saturn.keybuf.rows[i] |= 0x8000u16 as i16;
                    }
                    need_kbd_int = true;
                } else {
                    let r = (keycode >> 4) as usize;
                    let c = 1i16 << (keycode & 0xf);
                    if r < 9 {
                        if (saturn.keybuf.rows[r] & c) == 0 {
                            // Key was not already pressed — trigger interrupt
                            if saturn.kbd_ien != 0 {
                                need_kbd_int = true;
                            }
                            saturn.keybuf.rows[r] |= c;
                        }
                    }
                }
            } else {
                // Release
                if keycode == 0x8000 {
                    // ON key release: clear all keybuf (matches C memset)
                    for i in 0..9 {
                        saturn.keybuf.rows[i] = 0;
                    }
                } else {
                    let r = (keycode >> 4) as usize;
                    let c = 1i16 << (keycode & 0xf);
                    if r < 9 {
                        saturn.keybuf.rows[r] &= !c;
                    }
                }
            }
        }
        need_kbd_int
    }
}
