// Speaker frequency detection — exact port of device.c speaker section
// Windowed analysis of OUT register bit 3 toggles.

const HP48_IPS: i64 = 169000;

pub struct Speaker {
    pub last_state: bool,
    pub last_toggle_instr: i64,
    pub win_half_sum: i64,
    pub win_toggle_count: i32,
    pub instr_count: i64,
}

impl Speaker {
    pub fn new() -> Self {
        Self {
            last_state: false,
            last_toggle_instr: 0,
            win_half_sum: 0,
            win_toggle_count: 0,
            instr_count: 0,
        }
    }

    /// Called each instruction to check for speaker toggle.
    /// out_nibble2 is saturn.OUT[2].
    pub fn check_out_register(&mut self, out_nibble2: u8, speaker_counter: &mut i32) {
        let state = (out_nibble2 & 0x8) == 0x8;
        if state != self.last_state {
            let delta = self.instr_count - self.last_toggle_instr;
            self.last_toggle_instr = self.instr_count;

            if self.win_toggle_count > 0 && delta > 0 {
                self.win_half_sum += delta;
            }
            self.win_toggle_count += 1;

            *speaker_counter += 1;
            self.last_state = state;
        }
    }

    /// Called by JS every ~20ms. Returns frequency in Hz, or 0 if no tone.
    pub fn get_frequency(&mut self) -> u32 {
        let count = self.win_toggle_count;
        let sum = self.win_half_sum;

        // Reset window
        self.win_toggle_count = 0;
        self.win_half_sum = 0;

        if count < 4 {
            return 0;
        }

        let intervals = count - 1;
        let avg_half = sum / intervals as i64;
        if avg_half <= 0 {
            return 0;
        }

        let freq = (HP48_IPS / (2 * avg_half)) as i32;

        if freq < 20 || freq > 20000 {
            return 0;
        }

        freq as u32
    }
}
