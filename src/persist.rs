// State serialization — binary compatible with init.c save format
// Reads/writes the same format so existing /persist/hp48 files Just Work.

use crate::cpu::Saturn;
use crate::types::*;

const X48_MAGIC: u32 = 0x48503438;
const VERSION_MAJOR: u8 = 0x04;
const VERSION_MINOR: u8 = 0x04;
const PATCHLEVEL: u8 = 0x00;
const COMPILE_VERSION: u8 = 0x00;

// --- Read helpers (big-endian, matching C read_8/read_16/read_32) ---

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_8(&mut self) -> Option<u8> {
        if self.pos >= self.data.len() { return None; }
        let v = self.data[self.pos];
        self.pos += 1;
        Some(v)
    }

    fn read_char(&mut self) -> Option<u8> {
        self.read_8()
    }

    fn read_16(&mut self) -> Option<u16> {
        if self.pos + 2 > self.data.len() { return None; }
        let v = (self.data[self.pos] as u16) << 8 | self.data[self.pos + 1] as u16;
        self.pos += 2;
        Some(v)
    }

    fn read_32(&mut self) -> Option<u32> {
        if self.pos + 4 > self.data.len() { return None; }
        let v = (self.data[self.pos] as u32) << 24
            | (self.data[self.pos + 1] as u32) << 16
            | (self.data[self.pos + 2] as u32) << 8
            | self.data[self.pos + 3] as u32;
        self.pos += 4;
        Some(v)
    }
}

// --- Write helpers ---

struct Writer {
    data: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self { data: Vec::with_capacity(512) }
    }

    fn write_8(&mut self, val: u8) {
        self.data.push(val);
    }

    fn write_char(&mut self, val: u8) {
        self.data.push(val);
    }

    fn write_16(&mut self, val: u16) {
        self.data.push((val >> 8) as u8);
        self.data.push(val as u8);
    }

    fn write_32(&mut self, val: u32) {
        self.data.push((val >> 24) as u8);
        self.data.push((val >> 16) as u8);
        self.data.push((val >> 8) as u8);
        self.data.push(val as u8);
    }
}

/// Initialize Saturn CPU to default state (port of init_saturn)
pub fn init_saturn(saturn: &mut Saturn, model: Model) {
    *saturn = Saturn::default();
    saturn.pc = 0x00000;
    saturn.magic = X48_MAGIC;
    saturn.t1_tick = 8192;
    saturn.t2_tick = 16;
    saturn.i_per_s = 0;
    saturn.version = [VERSION_MAJOR, VERSION_MINOR, PATCHLEVEL, COMPILE_VERSION];
    saturn.hexmode = HEX;
    saturn.rstkp = -1;
    saturn.intenable = 1;
    saturn.int_pending = 0;
    saturn.kbd_ien = 1;
    saturn.timer1 = 0;
    saturn.timer2 = 0x2000;
    saturn.bank_switch = 0;
    for i in 0..NR_MCTL {
        if i == 0 {
            saturn.mem_cntl[i].unconfigured = 1;
        } else if i == 5 {
            saturn.mem_cntl[i].unconfigured = 0;
        } else {
            saturn.mem_cntl[i].unconfigured = 2;
        }
        saturn.mem_cntl[i].config[0] = 0;
        saturn.mem_cntl[i].config[1] = 0;
    }
    let _ = model; // mem_cntl init is same for both models at reset
}

/// Apply saturn_config_init settings (port of saturn_config_init)
pub fn saturn_config_init(saturn: &mut Saturn) {
    saturn.version = [VERSION_MAJOR, VERSION_MINOR, PATCHLEVEL, COMPILE_VERSION];
    saturn.rcs = 0;
    saturn.tcs = 0;
    saturn.lbr = 0;
}

/// Read v0.4.0 state from byte buffer into Saturn struct.
/// Returns true on success.
pub fn read_state(data: &[u8], saturn: &mut Saturn) -> bool {
    let mut r = Reader::new(data);

    // Magic
    let magic = match r.read_32() { Some(v) => v, None => return false };
    if magic != X48_MAGIC {
        return false;
    }
    saturn.magic = magic;

    // Version
    for i in 0..4 {
        saturn.version[i] = match r.read_char() { Some(v) => v, None => return false };
    }

    // Check version — we only support 0.4.0+
    let v = ((saturn.version[0] as u32) << 24)
        | ((saturn.version[1] as u32) << 16)
        | ((saturn.version[2] as u32) << 8)
        | saturn.version[3] as u32;
    if v < 0x00040000 {
        return false; // Old format not supported in Rust port
    }

    // Read v0.4.0 format fields (exact order from read_version_0_4_0_file)
    // Use a macro to avoid repeating the match pattern for each field
    macro_rules! r8 { ($r:expr) => { match $r.read_8() { Some(v) => v, None => return false } } }
    macro_rules! r16 { ($r:expr) => { match $r.read_16() { Some(v) => v, None => return false } } }
    macro_rules! r32 { ($r:expr) => { match $r.read_32() { Some(v) => v, None => return false } } }

    for i in 0..16 { saturn.a[i] = r8!(r); }
    for i in 0..16 { saturn.b[i] = r8!(r); }
    for i in 0..16 { saturn.c[i] = r8!(r); }
    for i in 0..16 { saturn.d[i] = r8!(r); }
    saturn.d0 = r32!(r) as i32;
    saturn.d1 = r32!(r) as i32;
    saturn.p = r8!(r);
    saturn.pc = r32!(r) as i32;
    for i in 0..16 { saturn.r0[i] = r8!(r); }
    for i in 0..16 { saturn.r1[i] = r8!(r); }
    for i in 0..16 { saturn.r2[i] = r8!(r); }
    for i in 0..16 { saturn.r3[i] = r8!(r); }
    for i in 0..16 { saturn.r4[i] = r8!(r); }
    for i in 0..4 { saturn.in_reg[i] = r8!(r); }
    for i in 0..3 { saturn.out[i] = r8!(r); }
    saturn.carry = r8!(r);
    for i in 0..NR_PSTAT { saturn.pstat[i] = r8!(r); }
    saturn.xm = r8!(r);
    saturn.sb = r8!(r);
    saturn.sr = r8!(r);
    saturn.mp = r8!(r);
    saturn.hexmode = r8!(r);
    for i in 0..NR_RSTK { saturn.rstk[i] = r32!(r) as i32; }
    saturn.rstkp = r16!(r) as i16;
    for i in 0..9 { saturn.keybuf.rows[i] = r16!(r) as i16; }
    saturn.intenable = r8!(r);
    saturn.int_pending = r8!(r);
    saturn.kbd_ien = r8!(r);
    saturn.disp_io = r8!(r);
    saturn.contrast_ctrl = r8!(r);
    saturn.disp_test = r8!(r);
    saturn.crc = r16!(r);
    saturn.power_status = r8!(r);
    saturn.power_ctrl = r8!(r);
    saturn.mode = r8!(r);
    saturn.annunc = r8!(r);
    saturn.baud = r8!(r);
    saturn.card_ctrl = r8!(r);
    saturn.card_status = r8!(r);
    saturn.io_ctrl = r8!(r);
    saturn.rcs = r8!(r);
    saturn.tcs = r8!(r);
    saturn.rbr = r8!(r);
    saturn.tbr = r8!(r);
    saturn.sreq = r8!(r);
    saturn.ir_ctrl = r8!(r);
    saturn.base_off = r8!(r);
    saturn.lcr = r8!(r);
    saturn.lbr = r8!(r);
    saturn.scratch = r8!(r);
    saturn.base_nibble = r8!(r);
    saturn.disp_addr = r32!(r) as i32;
    saturn.line_offset = r16!(r);
    saturn.line_count = r8!(r);
    saturn.unknown = r16!(r);
    saturn.t1_ctrl = r8!(r);
    saturn.t2_ctrl = r8!(r);
    saturn.menu_addr = r32!(r) as i32;
    saturn.unknown2 = r8!(r);
    saturn.timer1 = r8!(r) as i8;
    saturn.timer2 = r32!(r) as i32;
    saturn.t1_instr = r32!(r) as i32;
    saturn.t2_instr = r32!(r) as i32;
    saturn.t1_tick = r16!(r) as i16;
    saturn.t2_tick = r16!(r) as i16;
    saturn.i_per_s = r32!(r) as i32;
    saturn.bank_switch = r16!(r) as i16;
    for i in 0..NR_MCTL {
        saturn.mem_cntl[i].unconfigured = r16!(r) as i16;
        saturn.mem_cntl[i].config[0] = r32!(r) as i32;
        saturn.mem_cntl[i].config[1] = r32!(r) as i32;
    }

    true
}

/// Write Saturn state to byte buffer in v0.4.0 format.
pub fn write_state(saturn: &Saturn) -> Vec<u8> {
    let mut w = Writer::new();

    // Magic + version
    w.write_32(saturn.magic);
    for i in 0..4 { w.write_char(saturn.version[i]); }

    // Registers A-D
    for i in 0..16 { w.write_8(saturn.a[i]); }
    for i in 0..16 { w.write_8(saturn.b[i]); }
    for i in 0..16 { w.write_8(saturn.c[i]); }
    for i in 0..16 { w.write_8(saturn.d[i]); }

    // D0, D1
    w.write_32(saturn.d0 as u32);
    w.write_32(saturn.d1 as u32);

    // P, PC
    w.write_8(saturn.p);
    w.write_32(saturn.pc as u32);

    // R0-R4
    for i in 0..16 { w.write_8(saturn.r0[i]); }
    for i in 0..16 { w.write_8(saturn.r1[i]); }
    for i in 0..16 { w.write_8(saturn.r2[i]); }
    for i in 0..16 { w.write_8(saturn.r3[i]); }
    for i in 0..16 { w.write_8(saturn.r4[i]); }

    // IN, OUT
    for i in 0..4 { w.write_8(saturn.in_reg[i]); }
    for i in 0..3 { w.write_8(saturn.out[i]); }

    // CARRY, PSTAT
    w.write_8(saturn.carry);
    for i in 0..NR_PSTAT { w.write_8(saturn.pstat[i]); }

    // Hardware status
    w.write_8(saturn.xm);
    w.write_8(saturn.sb);
    w.write_8(saturn.sr);
    w.write_8(saturn.mp);
    w.write_8(saturn.hexmode);

    // Return stack
    for i in 0..NR_RSTK { w.write_32(saturn.rstk[i] as u32); }
    w.write_16(saturn.rstkp as u16);

    // Key buffer
    for i in 0..9 { w.write_16(saturn.keybuf.rows[i] as u16); }

    // Interrupt state
    w.write_8(saturn.intenable);
    w.write_8(saturn.int_pending);
    w.write_8(saturn.kbd_ien);

    // Display/IO
    w.write_8(saturn.disp_io);
    w.write_8(saturn.contrast_ctrl);
    w.write_8(saturn.disp_test);
    w.write_16(saturn.crc);
    w.write_8(saturn.power_status);
    w.write_8(saturn.power_ctrl);
    w.write_8(saturn.mode);
    w.write_8(saturn.annunc);
    w.write_8(saturn.baud);
    w.write_8(saturn.card_ctrl);
    w.write_8(saturn.card_status);
    w.write_8(saturn.io_ctrl);
    w.write_8(saturn.rcs);
    w.write_8(saturn.tcs);
    w.write_8(saturn.rbr);
    w.write_8(saturn.tbr);
    w.write_8(saturn.sreq);
    w.write_8(saturn.ir_ctrl);
    w.write_8(saturn.base_off);
    w.write_8(saturn.lcr);
    w.write_8(saturn.lbr);
    w.write_8(saturn.scratch);
    w.write_8(saturn.base_nibble);
    w.write_32(saturn.disp_addr as u32);
    w.write_16(saturn.line_offset);
    w.write_8(saturn.line_count);
    w.write_16(saturn.unknown);
    w.write_8(saturn.t1_ctrl);
    w.write_8(saturn.t2_ctrl);
    w.write_32(saturn.menu_addr as u32);
    w.write_8(saturn.unknown2);
    w.write_char(saturn.timer1 as u8);
    w.write_32(saturn.timer2 as u32);
    w.write_32(saturn.t1_instr as u32);
    w.write_32(saturn.t2_instr as u32);
    w.write_16(saturn.t1_tick as u16);
    w.write_16(saturn.t2_tick as u16);
    w.write_32(saturn.i_per_s as u32);
    w.write_16(saturn.bank_switch as u16);
    for i in 0..NR_MCTL {
        w.write_16(saturn.mem_cntl[i].unconfigured as u16);
        w.write_32(saturn.mem_cntl[i].config[0] as u32);
        w.write_32(saturn.mem_cntl[i].config[1] as u32);
    }

    w.data
}

/// Load ROM from byte array. Handles both nibble format (1 nibble per byte)
/// and packed format (2 nibbles per byte, low nibble first).
pub fn load_rom(data: &[u8], expected_size: usize) -> Vec<u8> {
    if data.len() == expected_size {
        // Already in nibble format
        data.to_vec()
    } else if data.len() == expected_size / 2 {
        // Packed byte format — expand to nibbles
        let mut nibbles = Vec::with_capacity(expected_size);
        for &byte in data {
            nibbles.push(byte & 0x0f);
            nibbles.push((byte >> 4) & 0x0f);
        }
        nibbles
    } else {
        // Try to use whatever we have
        let mut nibbles = Vec::with_capacity(expected_size);
        if data.len() < expected_size && data.len() > expected_size / 2 {
            // Assume nibble format, pad with zeros
            nibbles.extend_from_slice(data);
            nibbles.resize(expected_size, 0);
        } else {
            // Assume packed, expand
            for &byte in data {
                nibbles.push(byte & 0x0f);
                nibbles.push((byte >> 4) & 0x0f);
            }
            nibbles.resize(expected_size, 0);
        }
        nibbles
    }
}

/// Load RAM from byte array. Same format handling as ROM.
pub fn load_ram(data: &[u8], expected_size: usize) -> Vec<u8> {
    load_rom(data, expected_size)
}

/// Pack nibble array to byte array (2 nibbles per byte, low nibble first).
pub fn pack_nibbles(nibbles: &[u8]) -> Vec<u8> {
    let mut packed = Vec::with_capacity(nibbles.len() / 2);
    for chunk in nibbles.chunks(2) {
        let lo = chunk[0] & 0x0f;
        let hi = if chunk.len() > 1 { chunk[1] & 0x0f } else { 0 };
        packed.push(lo | (hi << 4));
    }
    packed
}
