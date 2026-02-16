// Saturn CPU state — exact port of saturn_t from hp48.h

use crate::types::*;

#[derive(Clone, Debug)]
pub struct KeyState {
    pub rows: [i16; 9],
}

impl Default for KeyState {
    fn default() -> Self {
        Self { rows: [0; 9] }
    }
}

#[derive(Clone, Debug)]
pub struct DisplayState {
    pub on: bool,
    pub disp_start: i32,
    pub disp_end: i32,
    pub offset: i32,
    pub lines: i32,
    pub nibs_per_line: i32,
    pub contrast: i32,
    pub menu_start: i32,
    pub menu_end: i32,
    pub annunc: i32,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            on: false,
            disp_start: 0,
            disp_end: 0,
            offset: 0,
            lines: 0,
            nibs_per_line: 0,
            contrast: 0,
            menu_start: 0,
            menu_end: 0,
            annunc: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Saturn {
    pub magic: u32,
    pub version: [u8; 4],

    // Working registers A-D (16 nibbles each)
    pub a: [u8; 16],
    pub b: [u8; 16],
    pub c: [u8; 16],
    pub d: [u8; 16],

    // Data pointers D0, D1
    pub d0: Word20,
    pub d1: Word20,

    // Pointer register P
    pub p: Word4,

    // Program counter
    pub pc: Word20,

    // Scratch registers R0-R4
    pub r0: [u8; 16],
    pub r1: [u8; 16],
    pub r2: [u8; 16],
    pub r3: [u8; 16],
    pub r4: [u8; 16],

    // IN/OUT registers
    pub in_reg: [u8; 4],
    pub out: [u8; 3],

    // Carry flag
    pub carry: Word1,

    // Program status (16 flags)
    pub pstat: [u8; NR_PSTAT],

    // Hardware status bits
    pub xm: u8,
    pub sb: u8,
    pub sr: u8,
    pub mp: u8,

    // Hex/Dec mode (10 or 16)
    pub hexmode: Word4,

    // Return stack
    pub rstk: [Word20; NR_RSTK],
    pub rstkp: i16,

    // Keyboard buffer
    pub keybuf: KeyState,

    // Interrupt state
    pub intenable: u8,
    pub int_pending: u8,
    pub kbd_ien: u8,

    // Display I/O register
    pub disp_io: Word4,

    // Contrast control
    pub contrast_ctrl: Word4,
    pub disp_test: Word8,

    // CRC register
    pub crc: Word16,

    // Power
    pub power_status: Word4,
    pub power_ctrl: Word4,

    // Mode
    pub mode: Word4,

    // Annunciators
    pub annunc: Word8,

    // Serial
    pub baud: Word4,
    pub card_ctrl: Word4,
    pub card_status: Word4,
    pub io_ctrl: Word4,
    pub rcs: Word4,
    pub tcs: Word4,
    pub rbr: Word8,
    pub tbr: Word8,
    pub sreq: Word8,
    pub ir_ctrl: Word4,
    pub base_off: Word4,
    pub lcr: Word4,
    pub lbr: Word4,
    pub scratch: Word4,
    pub base_nibble: Word4,

    // Display address registers
    pub disp_addr: Word20,
    pub line_offset: Word12,
    pub line_count: Word8,

    // Unknown device registers
    pub unknown: Word16,

    // Timer control
    pub t1_ctrl: Word4,
    pub t2_ctrl: Word4,

    // Menu address
    pub menu_addr: Word20,

    pub unknown2: Word8,

    // Timers (timer1 is signed in C: `char`)
    pub timer1: i8,
    pub timer2: Word32,

    // Timer scheduling
    pub t1_instr: i32,
    pub t2_instr: i32,
    pub t1_tick: i16,
    pub t2_tick: i16,
    pub i_per_s: i32,

    // Bank switching (GX)
    pub bank_switch: i16,

    // Memory controllers
    pub mem_cntl: [MemCntl; NR_MCTL],
}

impl Default for Saturn {
    fn default() -> Self {
        Self {
            magic: 0,
            version: [0; 4],
            a: [0; 16],
            b: [0; 16],
            c: [0; 16],
            d: [0; 16],
            d0: 0,
            d1: 0,
            p: 0,
            pc: 0,
            r0: [0; 16],
            r1: [0; 16],
            r2: [0; 16],
            r3: [0; 16],
            r4: [0; 16],
            in_reg: [0; 4],
            out: [0; 3],
            carry: 0,
            pstat: [0; NR_PSTAT],
            xm: 0,
            sb: 0,
            sr: 0,
            mp: 0,
            hexmode: HEX,
            rstk: [0; NR_RSTK],
            rstkp: -1,
            keybuf: KeyState::default(),
            intenable: 0,
            int_pending: 0,
            kbd_ien: 0,
            disp_io: 0,
            contrast_ctrl: 0,
            disp_test: 0,
            crc: 0,
            power_status: 0,
            power_ctrl: 0,
            mode: 0,
            annunc: 0,
            baud: 0,
            card_ctrl: 0,
            card_status: 0,
            io_ctrl: 0,
            rcs: 0,
            tcs: 0,
            rbr: 0,
            tbr: 0,
            sreq: 0,
            ir_ctrl: 0,
            base_off: 0,
            lcr: 0,
            lbr: 0,
            scratch: 0,
            base_nibble: 0,
            disp_addr: 0,
            line_offset: 0,
            line_count: 0,
            unknown: 0,
            t1_ctrl: 0,
            t2_ctrl: 0,
            menu_addr: 0,
            unknown2: 0,
            timer1: 0,
            timer2: 0,
            t1_instr: 0,
            t2_instr: 0,
            t1_tick: 8,
            t2_tick: 16,
            i_per_s: 0,
            bank_switch: 0,
            mem_cntl: [MemCntl::default(); NR_MCTL],
        }
    }
}
