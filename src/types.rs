// Types matching the C typedefs from hp48.h

pub type Word1 = u8;
pub type Word4 = u8;
pub type Word8 = u8;
pub type Word12 = u16;
pub type Word16 = u16;
pub type Word20 = i32; // C uses `long` (signed)
pub type Word32 = i32;

pub const NR_RSTK: usize = 8;
pub const NR_PSTAT: usize = 16;
pub const NR_MCTL: usize = 6;

pub const RAM_SIZE_SX: usize = 0x10000;
pub const RAM_SIZE_GX: usize = 0x40000;

pub const ROM_SIZE_SX: usize = 0x080000;
pub const ROM_SIZE_GX: usize = 0x100000;

pub const NIBBLES_PER_ROW: i32 = 0x22;

// Field codes
pub const P_FIELD: u8 = 0;
pub const WP_FIELD: u8 = 1;
pub const XS_FIELD: u8 = 2;
pub const X_FIELD: u8 = 3;
pub const S_FIELD: u8 = 4;
pub const M_FIELD: u8 = 5;
pub const B_FIELD: u8 = 6;
pub const W_FIELD: u8 = 7;
pub const A_FIELD: u8 = 15;
pub const IN_FIELD: u8 = 16;
pub const OUT_FIELD: u8 = 17;
pub const OUTS_FIELD: u8 = 18;

// Hex/Dec modes
pub const DEC: u8 = 10;
pub const HEX: u8 = 16;

// Memory controller indices — SX
pub const MCTL_MMIO_SX: usize = 0;
pub const MCTL_SYSRAM_SX: usize = 1;
pub const MCTL_PORT1_SX: usize = 2;
pub const MCTL_PORT2_SX: usize = 3;
pub const MCTL_EXTRA_SX: usize = 4;
pub const MCTL_SYSROM_SX: usize = 5;

// Memory controller indices — GX
pub const MCTL_MMIO_GX: usize = 0;
pub const MCTL_SYSRAM_GX: usize = 1;
pub const MCTL_BANK_GX: usize = 2;
pub const MCTL_PORT1_GX: usize = 3;
pub const MCTL_PORT2_GX: usize = 4;
pub const MCTL_SYSROM_GX: usize = 5;

// Device flags
pub const DISP_INSTR_OFF: i32 = 0x10;

// Annunciator masks
pub const ANN_LEFT: u8 = 0x81;
pub const ANN_RIGHT: u8 = 0x82;
pub const ANN_ALPHA: u8 = 0x84;
pub const ANN_BATTERY: u8 = 0x88;
pub const ANN_BUSY: u8 = 0x90;
pub const ANN_IO: u8 = 0xa0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Model {
    Sx,
    Gx,
}

#[derive(Clone, Copy, Debug)]
pub struct MemCntl {
    pub unconfigured: i16,
    pub config: [Word20; 2],
}

impl Default for MemCntl {
    fn default() -> Self {
        Self {
            unconfigured: 0,
            config: [0; 2],
        }
    }
}

pub const NIBBLE_MASKS: [i32; 16] = [
    0x0000000f,
    0x000000f0,
    0x00000f00,
    0x0000f000,
    0x000f0000,
    0x00f00000,
    0x0f000000,
    // NOTE: these are i32, so 0xf0000000 would be negative
    // The C code uses `long` which is 32-bit on Emscripten
    -268435456, // 0xf0000000 as i32
    0x0000000f,
    0x000000f0,
    0x00000f00,
    0x0000f000,
    0x000f0000,
    0x00f00000,
    0x0f000000,
    -268435456, // 0xf0000000 as i32
];
