// CPU actions — exact port of actions.c
// Interrupts, shutdown, config, register-to-status, return stack, etc.

use crate::alu::RegId;
use crate::cpu::Saturn;
use crate::types::*;

static CONF_TAB_SX: [i16; 6] = [1, 2, 2, 2, 2, 0];
static CONF_TAB_GX: [i16; 6] = [1, 2, 2, 2, 2, 0];
static CHIP_ID: [i32; 12] = [0, 0, 0, 0, 0x05, 0xf6, 0x07, 0xf8, 0x01, 0xf2, 0, 0];

impl Saturn {
    // --- Program status ---

    pub fn clear_program_stat(&mut self, n: usize) {
        self.pstat[n] = 0;
    }

    pub fn set_program_stat(&mut self, n: usize) {
        self.pstat[n] = 1;
    }

    pub fn get_program_stat(&self, n: usize) -> bool {
        self.pstat[n] != 0
    }

    pub fn register_to_status(&mut self, r_idx: RegId) {
        let r = self.get_reg(r_idx).clone();
        for i in 0..12 {
            self.pstat[i] = (r[i / 4] >> (i % 4)) & 1;
        }
    }

    pub fn status_to_register(&mut self, r_idx: RegId) {
        let pstat_copy: [u8; NR_PSTAT] = self.pstat;
        let r = self.get_reg_mut(r_idx);
        for i in 0..12 {
            if pstat_copy[i] != 0 {
                r[i / 4] |= 1 << (i % 4);
            } else {
                r[i / 4] &= !(1 << (i % 4)) & 0xf;
            }
        }
    }

    pub fn swap_register_status(&mut self, r_idx: RegId) {
        let old_pstat: [u8; NR_PSTAT] = self.pstat;
        let r_copy = self.get_reg(r_idx).clone();
        // Compute new pstat from register
        for i in 0..12 {
            self.pstat[i] = (r_copy[i / 4] >> (i % 4)) & 1;
        }
        // Write old pstat into register
        let r = self.get_reg_mut(r_idx);
        for i in 0..12 {
            if old_pstat[i] != 0 {
                r[i / 4] |= 1 << (i % 4);
            } else {
                r[i / 4] &= !(1 << (i % 4)) & 0xf;
            }
        }
    }

    pub fn clear_status(&mut self) {
        for i in 0..12 {
            self.pstat[i] = 0;
        }
    }

    // --- Register bit/nibble access ---

    pub fn set_register_nibble(&mut self, r_idx: RegId, n: usize, val: u8) {
        self.get_reg_mut(r_idx)[n] = val;
    }

    pub fn get_register_nibble(&self, r_idx: RegId, n: usize) -> u8 {
        self.get_reg(r_idx)[n]
    }

    pub fn set_register_bit(&mut self, r_idx: RegId, n: usize) {
        let r = self.get_reg_mut(r_idx);
        r[n / 4] |= 1 << (n % 4);
    }

    pub fn clear_register_bit(&mut self, r_idx: RegId, n: usize) {
        let r = self.get_reg_mut(r_idx);
        r[n / 4] &= !(1 << (n % 4));
    }

    pub fn get_register_bit(&self, r_idx: RegId, n: usize) -> bool {
        let r = self.get_reg(r_idx);
        (r[n / 4] & (1 << (n % 4))) > 0
    }

    // --- Hardware status ---

    pub fn set_hardware_stat(&mut self, op: i32) {
        if op & 1 != 0 { self.xm = 1; }
        if op & 2 != 0 { self.sb = 1; }
        if op & 4 != 0 { self.sr = 1; }
        if op & 8 != 0 { self.mp = 1; }
    }

    pub fn clear_hardware_stat(&mut self, op: i32) {
        if op & 1 != 0 { self.xm = 0; }
        if op & 2 != 0 { self.sb = 0; }
        if op & 4 != 0 { self.sr = 0; }
        if op & 8 != 0 { self.mp = 0; }
    }

    pub fn is_zero_hardware_stat(&self, op: i32) -> bool {
        if op & 1 != 0 && self.xm != 0 { return false; }
        if op & 2 != 0 && self.sb != 0 { return false; }
        if op & 4 != 0 && self.sr != 0 { return false; }
        if op & 8 != 0 && self.mp != 0 { return false; }
        true
    }

    // --- Return stack ---

    pub fn push_return_addr(&mut self, addr: Word20) {
        self.rstkp += 1;
        if self.rstkp >= NR_RSTK as i16 {
            for i in 1..NR_RSTK {
                self.rstk[i - 1] = self.rstk[i];
            }
            self.rstkp -= 1;
        }
        self.rstk[self.rstkp as usize] = addr;
    }

    pub fn pop_return_addr(&mut self) -> Word20 {
        if self.rstkp < 0 {
            return 0;
        }
        let addr = self.rstk[self.rstkp as usize];
        self.rstkp -= 1;
        addr
    }

    // --- Memory config ---

    pub fn do_reset(&mut self, model: Model) {
        for i in 0..6 {
            match model {
                Model::Gx => self.mem_cntl[i].unconfigured = CONF_TAB_GX[i],
                Model::Sx => self.mem_cntl[i].unconfigured = CONF_TAB_SX[i],
            }
            self.mem_cntl[i].config[0] = 0;
            self.mem_cntl[i].config[1] = 0;
        }
    }

    pub fn do_configure(&mut self) {
        let mut conf: i32 = 0;
        for i in (0..=4).rev() {
            conf <<= 4;
            conf |= self.c[i] as i32;
        }

        for i in 0..6 {
            if self.mem_cntl[i].unconfigured != 0 {
                self.mem_cntl[i].unconfigured -= 1;
                self.mem_cntl[i].config[self.mem_cntl[i].unconfigured as usize] = conf;
                break;
            }
        }
    }

    pub fn do_unconfigure(&mut self, model: Model) {
        let mut conf: u32 = 0;
        for i in (0..=4).rev() {
            conf <<= 4;
            conf |= self.c[i] as u32;
        }

        for i in 0..6 {
            if self.mem_cntl[i].config[0] == conf as i32 {
                match model {
                    Model::Gx => self.mem_cntl[i].unconfigured = CONF_TAB_GX[i],
                    Model::Sx => self.mem_cntl[i].unconfigured = CONF_TAB_SX[i],
                }
                self.mem_cntl[i].config[0] = 0;
                self.mem_cntl[i].config[1] = 0;
                break;
            }
        }
    }

    pub fn get_identification(&mut self) -> i32 {
        let mut i = 0;
        while i < 6 {
            if self.mem_cntl[i].unconfigured != 0 {
                break;
            }
            i += 1;
        }

        let id = if i < 6 {
            CHIP_ID[2 * i + (2 - self.mem_cntl[i].unconfigured as usize)]
        } else {
            0
        };

        let mut id_tmp = id;
        for j in 0..3 {
            self.c[j] = (id_tmp & 0x0f) as u8;
            id_tmp >>= 4;
        }
        0
    }

    // --- Address operations ---

    pub fn register_to_address(&mut self, r_idx: RegId, d_sel: u8, s: bool) {
        let n = if s { 4 } else { 5 };
        let r = self.get_reg(r_idx);
        let mut dat = if d_sel == 0 { self.d0 } else { self.d1 };
        for i in 0..n {
            dat &= !NIBBLE_MASKS[i];
            dat |= (r[i] as i32 & 0x0f) << (i as i32 * 4);
        }
        if d_sel == 0 { self.d0 = dat; } else { self.d1 = dat; }
    }

    pub fn address_to_register(&mut self, d_sel: u8, r_idx: RegId, s: bool) {
        let n = if s { 4 } else { 5 };
        let dat = if d_sel == 0 { self.d0 } else { self.d1 };
        let r = self.get_reg_mut(r_idx);
        let mut d = dat;
        for i in 0..n {
            r[i] = (d & 0x0f) as u8;
            d >>= 4;
        }
    }

    pub fn add_address(&mut self, d_sel: u8, add: i32) {
        let dat = if d_sel == 0 { &mut self.d0 } else { &mut self.d1 };
        *dat += add;
        if *dat & 0xfff00000u32 as i32 != 0 {
            self.carry = 1;
        } else {
            self.carry = 0;
        }
        *dat &= 0xfffff;
    }

    pub fn dat_to_addr(dat: &[u8]) -> Word20 {
        let mut addr: Word20 = 0;
        for i in (0..5).rev() {
            addr <<= 4;
            addr |= (dat[i] & 0xf) as i32;
        }
        addr
    }

    pub fn addr_to_dat(addr: Word20, dat: &mut [u8]) {
        let mut a = addr;
        for i in 0..5 {
            dat[i] = (a & 0xf) as u8;
            a >>= 4;
        }
    }

    // --- Interrupt control ---

    pub fn do_inton(&mut self) {
        self.kbd_ien = 1;
    }

    pub fn do_intoff(&mut self) {
        self.kbd_ien = 0;
    }
}
