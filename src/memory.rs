// Memory system — exact port of memory.c
// MMU dispatch, MMIO registers, read/write nibble for SX and GX models.

use crate::cpu::{DisplayState, Saturn};
use crate::device::DeviceFlags;
use crate::types::*;

pub struct Memory {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub port1: Vec<u8>,
    pub port2: Vec<u8>,
    pub port1_is_ram: bool,
    pub port1_mask: i32,
    pub port2_is_ram: bool,
    pub port2_mask: i32,
    pub line_counter: i32,
}

impl Memory {
    pub fn new(rom: Vec<u8>, ram: Vec<u8>) -> Self {
        Self {
            rom,
            ram,
            port1: Vec::new(),
            port2: Vec::new(),
            port1_is_ram: false,
            port1_mask: 0,
            port2_is_ram: false,
            port2_mask: 0,
            line_counter: -1,
        }
    }

    #[inline]
    fn calc_crc(saturn: &mut Saturn, nib: u8) -> u8 {
        saturn.crc = ((saturn.crc >> 4) ^ (((saturn.crc ^ nib as u16) & 0xf) * 0x1081)) as u16;
        nib
    }

    // --- MMIO Write ---
    pub fn write_dev_mem(
        &mut self,
        saturn: &mut Saturn,
        display: &mut DisplayState,
        device: &mut DeviceFlags,
        device_check: &mut bool,
        schedule_event: &mut i32,
        addr: i32,
        val: i32,
    ) {
        *device_check = true;
        *schedule_event = 0;

        match addr {
            0x100 => {
                // DISPIO
                if val as u8 != saturn.disp_io {
                    saturn.disp_io = val as u8;
                    display.on = (val & 0x8) != 0;
                    display.offset = val & 0x7;
                    if display.offset > 3 {
                        display.nibs_per_line =
                            (NIBBLES_PER_ROW + saturn.line_offset as i32 + 2) & 0xfff;
                    } else {
                        display.nibs_per_line =
                            (NIBBLES_PER_ROW + saturn.line_offset as i32) & 0xfff;
                    }
                    display.disp_end = display.disp_start
                        + display.nibs_per_line * (display.lines + 1);
                    device.display_touched = DISP_INSTR_OFF;
                }
            }
            0x101 => {
                // CONTRAST CONTROL
                saturn.contrast_ctrl = val as u8;
                display.contrast &= !0x0f;
                display.contrast |= val & 0x0f;
                device.contrast_touched = true;
            }
            0x102 => {
                // DISPLAY TEST (nibble 0) + contrast bit 4
                display.contrast &= !0xf0;
                display.contrast |= (val & 0x1) << 4;
                device.contrast_touched = true;
                saturn.disp_test &= !(NIBBLE_MASKS[0] as u8);
                saturn.disp_test |= (val << 0) as u8;
                device.disp_test_touched = true;
            }
            0x103 => {
                // DISPLAY TEST (nibble 1)
                saturn.disp_test &= !(NIBBLE_MASKS[1] as u8);
                saturn.disp_test |= (val << 4) as u8;
                device.disp_test_touched = true;
            }
            0x104..=0x107 => {
                // CRC
                let off = (addr - 0x104) as usize;
                saturn.crc &= !(NIBBLE_MASKS[off] as u16);
                saturn.crc |= ((val as u16) << (off * 4)) as u16;
            }
            0x108 => {
                saturn.power_status = val as u8;
                device.power_status_touched = true;
            }
            0x109 => {
                saturn.power_ctrl = val as u8;
                device.power_ctrl_touched = true;
            }
            0x10a => {
                saturn.mode = val as u8;
                device.mode_touched = true;
            }
            0x10b | 0x10c => {
                let off = (addr - 0x10b) as usize;
                saturn.annunc &= !(NIBBLE_MASKS[off] as u8);
                saturn.annunc |= ((val as u8) << (off * 4)) as u8;
                display.annunc = saturn.annunc as i32;
                device.ann_touched = true;
            }
            0x10d => {
                saturn.baud = val as u8;
                device.baud_touched = true;
            }
            0x10e => {
                // CARD CONTROL
                saturn.card_ctrl = val as u8;
                if saturn.card_ctrl & 0x02 != 0 {
                    saturn.mp = 1;
                }
                // card_ctrl & 0x01 triggers interrupt — handled by caller
                device.card_ctrl_touched = true;
            }
            0x10f => {
                // CARD STATUS — read only, writes ignored
            }
            0x110 => {
                saturn.io_ctrl = val as u8;
                device.ioc_touched = true;
            }
            0x111 => {
                saturn.rcs = val as u8;
            }
            0x112 => {
                saturn.tcs = val as u8;
            }
            0x113 => {
                // CRER
                saturn.rcs &= 0x0b;
            }
            0x114 | 0x115 => {
                // RBR — read only
            }
            0x116 | 0x117 => {
                // TBR
                let off = (addr - 0x116) as usize;
                saturn.tbr &= !(NIBBLE_MASKS[off] as u8);
                saturn.tbr |= ((val as u8) << (off * 4)) as u8;
                saturn.tcs |= 0x01;
                device.tbr_touched = true;
            }
            0x118 | 0x119 => {
                let off = (addr - 0x118) as usize;
                saturn.sreq &= !(NIBBLE_MASKS[off] as u8);
                saturn.sreq |= ((val as u8) << (off * 4)) as u8;
                device.sreq_touched = true;
            }
            0x11a => {
                saturn.ir_ctrl = val as u8;
                device.ir_ctrl_touched = true;
            }
            0x11b => {
                saturn.base_off = val as u8;
                device.base_off_touched = true;
            }
            0x11c => {
                saturn.lcr = val as u8;
                device.lcr_touched = true;
            }
            0x11d => {
                saturn.lbr = val as u8;
                device.lbr_touched = true;
            }
            0x11e => {
                saturn.scratch = val as u8;
                device.scratch_touched = true;
            }
            0x11f => {
                saturn.base_nibble = val as u8;
                device.base_nibble_touched = true;
            }
            0x120..=0x124 => {
                let off = (addr - 0x120) as usize;
                saturn.disp_addr &= !NIBBLE_MASKS[off];
                saturn.disp_addr |= val << (off as i32 * 4);
                let new_start = saturn.disp_addr & 0xffffe;
                if display.disp_start != new_start {
                    display.disp_start = new_start;
                    display.disp_end = display.disp_start
                        + display.nibs_per_line * (display.lines + 1);
                    device.display_touched = DISP_INSTR_OFF;
                }
            }
            0x125..=0x127 => {
                let off = (addr - 0x125) as usize;
                let old_offset = saturn.line_offset;
                saturn.line_offset &= !(NIBBLE_MASKS[off] as u16);
                saturn.line_offset |= ((val as u16) << (off * 4)) as u16;
                if saturn.line_offset != old_offset {
                    if display.offset > 3 {
                        display.nibs_per_line =
                            (NIBBLES_PER_ROW + saturn.line_offset as i32 + 2) & 0xfff;
                    } else {
                        display.nibs_per_line =
                            (NIBBLES_PER_ROW + saturn.line_offset as i32) & 0xfff;
                    }
                    display.disp_end = display.disp_start
                        + display.nibs_per_line * (display.lines + 1);
                    device.display_touched = DISP_INSTR_OFF;
                }
            }
            0x128 | 0x129 => {
                let off = (addr - 0x128) as usize;
                saturn.line_count &= !(NIBBLE_MASKS[off] as u8);
                saturn.line_count |= ((val as u8) << (off * 4)) as u8;
                self.line_counter = -1;
                let new_lines = saturn.line_count as i32 & 0x3f;
                if display.lines != new_lines {
                    display.lines = if new_lines == 0 { 63 } else { new_lines };
                    display.disp_end = display.disp_start
                        + display.nibs_per_line * (display.lines + 1);
                    device.display_touched = DISP_INSTR_OFF;
                }
            }
            0x12a..=0x12d => {
                let off = (addr - 0x12a) as usize;
                saturn.unknown &= !(NIBBLE_MASKS[off] as u16);
                saturn.unknown |= ((val as u16) << (off * 4)) as u16;
                device.unknown_touched = true;
            }
            0x12e => {
                saturn.t1_ctrl = val as u8;
                device.t1_ctrl_touched = true;
            }
            0x12f => {
                saturn.t2_ctrl = val as u8;
                device.t2_ctrl_touched = true;
            }
            0x130..=0x134 => {
                let off = (addr - 0x130) as usize;
                saturn.menu_addr &= !NIBBLE_MASKS[off];
                saturn.menu_addr |= val << (off as i32 * 4);
                if display.menu_start != saturn.menu_addr {
                    display.menu_start = saturn.menu_addr;
                    display.menu_end = display.menu_start + 0x110;
                    device.display_touched = DISP_INSTR_OFF;
                }
            }
            0x135 | 0x136 => {
                let off = (addr - 0x135) as usize;
                saturn.unknown2 &= !(NIBBLE_MASKS[off] as u8);
                saturn.unknown2 |= ((val as u8) << (off * 4)) as u8;
                device.unknown2_touched = true;
            }
            0x137 => {
                saturn.timer1 = val as i8;
                device.t1_touched = true;
            }
            0x138..=0x13f => {
                let off = (addr - 0x138) as usize;
                saturn.timer2 &= !NIBBLE_MASKS[off];
                saturn.timer2 |= val << (off as i32 * 4);
                device.t2_touched = true;
            }
            _ => {}
        }
    }

    // --- MMIO Read ---
    pub fn read_dev_mem(
        &mut self,
        saturn: &mut Saturn,
        device: &mut DeviceFlags,
        device_check: &mut bool,
        schedule_event: &mut i32,
        addr: i32,
    ) -> u8 {
        match addr {
            0x100 => saturn.disp_io & 0x0f,
            0x101 => saturn.contrast_ctrl & 0x0f,
            0x102 | 0x103 => {
                ((saturn.disp_test >> ((addr - 0x102) * 4)) & 0x0f) as u8
            }
            0x104..=0x107 => {
                ((saturn.crc >> ((addr - 0x104) as u16 * 4)) & 0x0f) as u8
            }
            0x108 => saturn.power_status & 0x0f,
            0x109 => saturn.power_ctrl & 0x0f,
            0x10a => saturn.mode & 0x0f,
            0x10b | 0x10c => {
                ((saturn.annunc >> ((addr - 0x10b) * 4) as u8) & 0x0f) as u8
            }
            0x10d => saturn.baud & 0x0f,
            0x10e => saturn.card_ctrl & 0x0f,
            0x10f => saturn.card_status & 0x0f,
            0x110 => saturn.io_ctrl & 0x0f,
            0x111 => saturn.rcs & 0x0f,
            0x112 => saturn.tcs & 0x0f,
            0x113 => 0x00,
            0x114 | 0x115 => {
                saturn.rcs &= 0x0e;
                device.rbr_touched = true;
                *device_check = true;
                *schedule_event = 0;
                ((saturn.rbr >> ((addr - 0x114) * 4) as u8) & 0x0f) as u8
            }
            0x116 | 0x117 => 0x00,
            0x118 | 0x119 => {
                ((saturn.sreq >> ((addr - 0x118) * 4) as u8) & 0x0f) as u8
            }
            0x11a => saturn.ir_ctrl & 0x0f,
            0x11b => saturn.base_off & 0x0f,
            0x11c => saturn.lcr & 0x0f,
            0x11d => saturn.lbr & 0x0f,
            0x11e => saturn.scratch & 0x0f,
            0x11f => saturn.base_nibble & 0x0f,
            0x120..=0x124 => {
                ((saturn.disp_addr >> ((addr - 0x120) * 4)) & 0x0f) as u8
            }
            0x125..=0x127 => {
                ((saturn.line_offset >> ((addr - 0x125) as u16 * 4)) & 0x0f) as u8
            }
            0x128 | 0x129 => {
                self.line_counter += 1;
                if self.line_counter > 0x3f {
                    self.line_counter = -1;
                }
                let combined = (saturn.line_count as i32 & 0xc0) | (self.line_counter & 0x3f);
                ((combined >> ((addr - 0x128) * 4)) & 0x0f) as u8
            }
            0x12a..=0x12d => {
                ((saturn.unknown >> ((addr - 0x12a) as u16 * 4)) & 0x0f) as u8
            }
            0x12e => saturn.t1_ctrl & 0x0f,
            0x12f => saturn.t2_ctrl & 0x0f,
            0x130..=0x134 => {
                ((saturn.menu_addr >> ((addr - 0x130) * 4)) & 0x0f) as u8
            }
            0x135 | 0x136 => {
                ((saturn.unknown2 >> ((addr - 0x135) * 4) as u8) & 0x0f) as u8
            }
            0x137 => (saturn.timer1 as u8) & 0xf,
            0x138..=0x13f => {
                ((saturn.timer2 >> ((addr - 0x138) * 4)) & 0xf) as u8
            }
            _ => 0x00,
        }
    }

    // --- SX read/write ---

    /// Write a nibble for SX model. Returns true if the write reached a RAM path
    /// where the display nibble check should be performed by the caller.
    pub fn write_nibble_sx(
        &mut self,
        saturn: &mut Saturn,
        display: &mut DisplayState,
        device: &mut DeviceFlags,
        device_check: &mut bool,
        schedule_event: &mut i32,
        addr: i32,
        val: i32,
    ) -> bool {
        let addr = addr & 0xfffff;
        let val = val & 0x0f;
        match (addr >> 16) & 0x0f {
            0 => {
                if addr < 0x140
                    && addr >= 0x100
                    && saturn.mem_cntl[MCTL_MMIO_SX].config[0] == 0x100
                {
                    self.write_dev_mem(
                        saturn, display, device, device_check, schedule_event, addr, val,
                    );
                    return false;
                }
                return false; // write to ROM
            }
            1..=6 => return false, // write to ROM
            7 => {
                if saturn.mem_cntl[MCTL_SYSRAM_SX].config[0] == 0x70000 {
                    if saturn.mem_cntl[MCTL_SYSRAM_SX].config[1] == 0xfc000_i32
                        && addr < 0x74000
                    {
                        self.ram[(addr - 0x70000) as usize] = val as u8;
                    } else if saturn.mem_cntl[MCTL_SYSRAM_SX].config[1] == 0xfe000_i32
                        && addr < 0x72000
                    {
                        self.ram[(addr - 0x70000) as usize] = val as u8;
                    } else if saturn.mem_cntl[MCTL_SYSRAM_SX].config[1] == 0xf0000_i32 {
                        self.ram[(addr - 0x70000) as usize] = val as u8;
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            8..=0xb => {
                if saturn.mem_cntl[MCTL_PORT1_SX].config[0] == 0x80000 {
                    if self.port1_is_ram {
                        self.port1[((addr - 0x80000) & self.port1_mask) as usize] = val as u8;
                    }
                    return false;
                }
                if saturn.mem_cntl[MCTL_PORT2_SX].config[0] == 0x80000 {
                    if self.port2_is_ram {
                        self.port2[((addr - 0x80000) & self.port2_mask) as usize] = val as u8;
                    }
                    return false;
                }
                return false;
            }
            0xc..=0xe => {
                if saturn.mem_cntl[MCTL_PORT1_SX].config[0] == 0xc0000_i32 {
                    if self.port1_is_ram {
                        self.port1[((addr - 0xc0000) & self.port1_mask) as usize] = val as u8;
                    }
                    return false;
                }
                if saturn.mem_cntl[MCTL_PORT2_SX].config[0] == 0xc0000_i32 {
                    if self.port2_is_ram {
                        self.port2[((addr - 0xc0000) & self.port2_mask) as usize] = val as u8;
                    }
                    return false;
                }
                return false;
            }
            0xf => {
                if saturn.mem_cntl[MCTL_SYSRAM_SX].config[0] == 0xf0000_i32 {
                    self.ram[(addr - 0xf0000) as usize] = val as u8;
                } else if saturn.mem_cntl[MCTL_PORT1_SX].config[0] == 0xc0000_i32 {
                    if self.port1_is_ram {
                        self.port1[((addr - 0xc0000) & self.port1_mask) as usize] = val as u8;
                    }
                    return false;
                } else if saturn.mem_cntl[MCTL_PORT2_SX].config[0] == 0xc0000_i32 {
                    if self.port2_is_ram {
                        self.port2[((addr - 0xc0000) & self.port2_mask) as usize] = val as u8;
                    }
                    return false;
                } else {
                    return false;
                }
            }
            _ => return false,
        }
        // RAM write fell through — caller should perform display nibble check
        true
    }

    pub fn read_nibble_sx(&self, saturn: &Saturn, addr: i32) -> u8 {
        let addr = addr & 0xfffff;
        match (addr >> 16) & 0x0f {
            0 => {
                if addr < 0x140 && addr >= 0x100 {
                    if saturn.mem_cntl[MCTL_MMIO_SX].config[0] == 0x100 {
                        return 0; // read_dev_mem handled separately
                    } else {
                        return 0x00;
                    }
                }
                self.rom[addr as usize]
            }
            1..=6 => self.rom[addr as usize],
            7 => {
                if saturn.mem_cntl[MCTL_SYSRAM_SX].config[0] == 0x70000 {
                    if saturn.mem_cntl[MCTL_SYSRAM_SX].config[1] == 0xfc000_i32
                        && addr < 0x74000
                    {
                        return self.ram[(addr - 0x70000) as usize];
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_SX].config[1] == 0xfe000_i32
                        && addr < 0x72000
                    {
                        return self.ram[(addr - 0x70000) as usize];
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_SX].config[1] == 0xf0000_i32 {
                        return self.ram[(addr - 0x70000) as usize];
                    }
                }
                self.rom[addr as usize]
            }
            8..=0xb => {
                if saturn.mem_cntl[MCTL_PORT1_SX].config[0] == 0x80000 {
                    return self.port1[((addr - 0x80000) & self.port1_mask) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT2_SX].config[0] == 0x80000 {
                    return self.port2[((addr - 0x80000) & self.port2_mask) as usize];
                }
                0x00
            }
            0xc..=0xe => {
                if saturn.mem_cntl[MCTL_PORT1_SX].config[0] == 0xc0000_i32 {
                    return self.port1[((addr - 0xc0000) & self.port1_mask) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT2_SX].config[0] == 0xc0000_i32 {
                    return self.port2[((addr - 0xc0000) & self.port2_mask) as usize];
                }
                0x00
            }
            0xf => {
                if saturn.mem_cntl[MCTL_SYSRAM_SX].config[0] == 0xf0000_i32 {
                    return self.ram[(addr - 0xf0000) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT1_SX].config[0] == 0xc0000_i32 {
                    return self.port1[((addr - 0xc0000) & self.port1_mask) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT2_SX].config[0] == 0xc0000_i32 {
                    return self.port2[((addr - 0xc0000) & self.port2_mask) as usize];
                }
                0x00
            }
            _ => 0x00,
        }
    }

    // --- GX read/write ---

    /// Write a nibble for GX model. Returns true if the write reached a RAM path
    /// where the display nibble check should be performed by the caller.
    pub fn write_nibble_gx(
        &mut self,
        saturn: &mut Saturn,
        display: &mut DisplayState,
        device: &mut DeviceFlags,
        device_check: &mut bool,
        schedule_event: &mut i32,
        addr: i32,
        val: i32,
    ) -> bool {
        let addr = addr & 0xfffff;
        let val = val & 0x0f;
        match (addr >> 16) & 0x0f {
            0 => {
                if addr < 0x140
                    && addr >= 0x100
                    && saturn.mem_cntl[MCTL_MMIO_GX].config[0] == 0x100
                {
                    self.write_dev_mem(
                        saturn, display, device, device_check, schedule_event, addr, val,
                    );
                    return false;
                }
                return false;
            }
            1 | 2 | 3 | 5 | 6 => return false,
            4 => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x40000 {
                    self.ram[(addr - 0x40000) as usize] = val as u8;
                } else {
                    return false;
                }
            }
            7 => {
                if addr >= 0x7f000
                    && saturn.mem_cntl[MCTL_BANK_GX].config[0] == 0x7f000
                {
                    return false;
                }
                if addr >= 0x7e000
                    && addr < 0x7f000
                    && saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0x7e000
                {
                    return false;
                }
                if addr >= 0x7e000
                    && addr < 0x7f000
                    && saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0x7e000
                {
                    return false;
                }
                return false;
            }
            8 => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000 {
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfc000_i32
                        && addr < 0x84000
                    {
                        self.ram[(addr - 0x80000) as usize] = val as u8;
                    } else if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfe000_i32
                        && addr < 0x82000
                    {
                        self.ram[(addr - 0x80000) as usize] = val as u8;
                    } else if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xf0000_i32 {
                        self.ram[(addr - 0x80000) as usize] = val as u8;
                    } else if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32 {
                        self.ram[(addr - 0x80000) as usize] = val as u8;
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            9 => {
                if saturn.mem_cntl[MCTL_BANK_GX].config[0] == 0x90000 {
                    if addr < 0x91000 {
                        return false;
                    }
                }
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    self.ram[(addr - 0x80000) as usize] = val as u8;
                } else {
                    return false;
                }
            }
            0xa => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    self.ram[(addr - 0x80000) as usize] = val as u8;
                } else if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xa0000_i32 {
                    if self.port1_is_ram {
                        self.port1[((addr - 0xa0000) & self.port1_mask) as usize] = val as u8;
                    }
                    return false;
                } else {
                    return false;
                }
            }
            0xb => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    self.ram[(addr - 0x80000) as usize] = val as u8;
                } else if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xb0000_i32 {
                    if self.port2_is_ram {
                        let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xb0000);
                        self.port2[(idx & self.port2_mask) as usize] = val as u8;
                    }
                    return false;
                } else {
                    return false;
                }
            }
            0xc => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0xc0000_i32 {
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfc000_i32
                        && addr < 0xc4000
                    {
                        self.ram[(addr - 0xc0000) as usize] = val as u8;
                    } else if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfe000_i32
                        && addr < 0xc2000
                    {
                        self.ram[(addr - 0xc0000) as usize] = val as u8;
                    } else {
                        self.ram[(addr - 0xc0000) as usize] = val as u8;
                    }
                } else if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xc0000_i32 {
                    if self.port1_is_ram {
                        self.port1[((addr - 0xc0000) & self.port1_mask) as usize] = val as u8;
                    }
                    return false;
                } else if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xc0000_i32 {
                    if self.port2_is_ram {
                        let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xc0000);
                        self.port2[(idx & self.port2_mask) as usize] = val as u8;
                    }
                    return false;
                } else {
                    return false;
                }
            }
            0xd..=0xf => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    self.ram[(addr - 0xc0000) as usize] = val as u8;
                } else if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_PORT1_GX].config[1] == 0xc0000_i32
                {
                    if self.port1_is_ram {
                        self.port1[((addr - 0xc0000) & self.port1_mask) as usize] = val as u8;
                    }
                    return false;
                } else if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_PORT2_GX].config[1] == 0xc0000_i32
                {
                    if self.port2_is_ram {
                        let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xc0000);
                        self.port2[(idx & self.port2_mask) as usize] = val as u8;
                    }
                    return false;
                } else {
                    return false;
                }
            }
            _ => return false,
        }
        // RAM write fell through — caller should perform display nibble check
        true
    }

    pub fn read_nibble_gx(&mut self, saturn: &mut Saturn, addr: i32) -> u8 {
        let addr = addr & 0xfffff;
        match (addr >> 16) & 0x0f {
            0 => {
                if addr < 0x140 && addr >= 0x100 {
                    if saturn.mem_cntl[MCTL_MMIO_GX].config[0] == 0x100 {
                        return 0; // read_dev_mem handled by caller
                    } else {
                        return 0x00;
                    }
                }
                self.rom[addr as usize]
            }
            1 | 2 | 3 | 5 | 6 => self.rom[addr as usize],
            4 => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x40000 {
                    return self.ram[(addr - 0x40000) as usize];
                }
                self.rom[addr as usize]
            }
            7 => {
                if addr >= 0x7f000
                    && saturn.mem_cntl[MCTL_BANK_GX].config[0] == 0x7f000
                {
                    if addr == 0x7f000 {
                        saturn.bank_switch = 0;
                    }
                    if addr >= 0x7f040 && addr < 0x7f080 {
                        saturn.bank_switch = ((addr - 0x7f040) / 2) as i16;
                    }
                    return 0x7;
                }
                if addr >= 0x7e000
                    && addr < 0x7f000
                    && saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0x7e000
                {
                    return 0x7;
                }
                if addr >= 0x7e000
                    && addr < 0x7f000
                    && saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0x7e000
                {
                    return 0x7;
                }
                self.rom[addr as usize]
            }
            8 => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000 {
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfc000_i32
                        && addr < 0x84000
                    {
                        return self.ram[(addr - 0x80000) as usize];
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfe000_i32
                        && addr < 0x82000
                    {
                        return self.ram[(addr - 0x80000) as usize];
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xf0000_i32 {
                        return self.ram[(addr - 0x80000) as usize];
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32 {
                        return self.ram[(addr - 0x80000) as usize];
                    }
                }
                self.rom[addr as usize]
            }
            9 => {
                if saturn.mem_cntl[MCTL_MMIO_GX].config[0] == 0x90000 {
                    if addr < 0x91000 {
                        if addr == 0x90000 {
                            saturn.bank_switch = 0;
                        }
                        if addr >= 0x90040 && addr < 0x90080 {
                            saturn.bank_switch = ((addr - 0x90040) / 2) as i16;
                        }
                        return 0x7;
                    }
                }
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return self.ram[(addr - 0x80000) as usize];
                }
                self.rom[addr as usize]
            }
            0xa => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return self.ram[(addr - 0x80000) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xa0000_i32 {
                    return self.port1[((addr - 0xa0000) & self.port1_mask) as usize];
                }
                self.rom[addr as usize]
            }
            0xb => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return self.ram[(addr - 0x80000) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xb0000_i32 {
                    let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xb0000);
                    return self.port2[(idx & self.port2_mask) as usize];
                }
                self.rom[addr as usize]
            }
            0xc => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0xc0000_i32 {
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfc000_i32
                        && addr < 0xc4000
                    {
                        return self.ram[(addr - 0xc0000) as usize];
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfe000_i32
                        && addr < 0xc2000
                    {
                        return self.ram[(addr - 0xc0000) as usize];
                    }
                    return self.ram[(addr - 0xc0000) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xc0000_i32 {
                    return self.port1[((addr - 0xc0000) & self.port1_mask) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xc0000_i32 {
                    let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xc0000);
                    return self.port2[(idx & self.port2_mask) as usize];
                }
                self.rom[addr as usize]
            }
            0xd..=0xf => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return self.ram[(addr - 0xc0000) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_PORT1_GX].config[1] == 0xc0000_i32
                {
                    return self.port1[((addr - 0xc0000) & self.port1_mask) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_PORT2_GX].config[1] == 0xc0000_i32
                {
                    let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xc0000);
                    return self.port2[(idx & self.port2_mask) as usize];
                }
                self.rom[addr as usize]
            }
            _ => 0x00,
        }
    }

    /// Read-only variant of read_nibble_gx for display rendering.
    /// Same address dispatch as read_nibble_gx but takes &self and &Saturn
    /// (no mutation). Bank-switch side effects are skipped since display
    /// addresses never hit bank control registers (0x7f000, 0x90000).
    pub fn read_nibble_gx_display(&self, saturn: &Saturn, addr: i32) -> u8 {
        let addr = addr & 0xfffff;
        match (addr >> 16) & 0x0f {
            0 => {
                if addr < 0x140 && addr >= 0x100 {
                    return 0x00;
                }
                self.rom[addr as usize]
            }
            1 | 2 | 3 | 5 | 6 => self.rom[addr as usize],
            4 => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x40000 {
                    return self.ram[(addr - 0x40000) as usize];
                }
                self.rom[addr as usize]
            }
            7 => {
                if addr >= 0x7f000
                    && saturn.mem_cntl[MCTL_BANK_GX].config[0] == 0x7f000
                {
                    return 0x7;
                }
                if addr >= 0x7e000
                    && addr < 0x7f000
                    && saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0x7e000
                {
                    return 0x7;
                }
                if addr >= 0x7e000
                    && addr < 0x7f000
                    && saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0x7e000
                {
                    return 0x7;
                }
                self.rom[addr as usize]
            }
            8 => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000 {
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfc000_i32
                        && addr < 0x84000
                    {
                        return self.ram[(addr - 0x80000) as usize];
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfe000_i32
                        && addr < 0x82000
                    {
                        return self.ram[(addr - 0x80000) as usize];
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xf0000_i32 {
                        return self.ram[(addr - 0x80000) as usize];
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32 {
                        return self.ram[(addr - 0x80000) as usize];
                    }
                }
                self.rom[addr as usize]
            }
            9 => {
                if saturn.mem_cntl[MCTL_MMIO_GX].config[0] == 0x90000 {
                    if addr < 0x91000 {
                        return 0x7;
                    }
                }
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return self.ram[(addr - 0x80000) as usize];
                }
                self.rom[addr as usize]
            }
            0xa => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return self.ram[(addr - 0x80000) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xa0000_i32 {
                    return self.port1[((addr - 0xa0000) & self.port1_mask) as usize];
                }
                self.rom[addr as usize]
            }
            0xb => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return self.ram[(addr - 0x80000) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xb0000_i32 {
                    let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xb0000);
                    return self.port2[(idx & self.port2_mask) as usize];
                }
                self.rom[addr as usize]
            }
            0xc => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0xc0000_i32 {
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfc000_i32
                        && addr < 0xc4000
                    {
                        return self.ram[(addr - 0xc0000) as usize];
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfe000_i32
                        && addr < 0xc2000
                    {
                        return self.ram[(addr - 0xc0000) as usize];
                    }
                    return self.ram[(addr - 0xc0000) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xc0000_i32 {
                    return self.port1[((addr - 0xc0000) & self.port1_mask) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xc0000_i32 {
                    let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xc0000);
                    return self.port2[(idx & self.port2_mask) as usize];
                }
                self.rom[addr as usize]
            }
            0xd..=0xf => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return self.ram[(addr - 0xc0000) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_PORT1_GX].config[1] == 0xc0000_i32
                {
                    return self.port1[((addr - 0xc0000) & self.port1_mask) as usize];
                }
                if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_PORT2_GX].config[1] == 0xc0000_i32
                {
                    let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xc0000);
                    return self.port2[(idx & self.port2_mask) as usize];
                }
                self.rom[addr as usize]
            }
            _ => 0x00,
        }
    }

    // --- CRC variants (same memory map but wrapped in calc_crc) ---

    pub fn read_nibble_crc_sx(&self, saturn: &mut Saturn, addr: i32) -> u8 {
        let addr = addr & 0xfffff;
        match (addr >> 16) & 0x0f {
            0 => {
                if addr < 0x140 && addr >= 0x100 {
                    if saturn.mem_cntl[MCTL_MMIO_SX].config[0] == 0x100 {
                        return 0; // read_dev_mem handled separately
                    } else {
                        return Self::calc_crc(saturn, 0x00);
                    }
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            1..=6 => Self::calc_crc(saturn, self.rom[addr as usize]),
            7 => {
                if saturn.mem_cntl[MCTL_SYSRAM_SX].config[0] == 0x70000 {
                    if saturn.mem_cntl[MCTL_SYSRAM_SX].config[1] == 0xfc000_i32
                        && addr < 0x74000
                    {
                        return Self::calc_crc(saturn, self.ram[(addr - 0x70000) as usize]);
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_SX].config[1] == 0xfe000_i32
                        && addr < 0x72000
                    {
                        return Self::calc_crc(saturn, self.ram[(addr - 0x70000) as usize]);
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_SX].config[1] == 0xf0000_i32 {
                        return Self::calc_crc(saturn, self.ram[(addr - 0x70000) as usize]);
                    }
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            8..=0xb => {
                if saturn.mem_cntl[MCTL_PORT1_SX].config[0] == 0x80000 {
                    return Self::calc_crc(
                        saturn,
                        self.port1[((addr - 0x80000) & self.port1_mask) as usize],
                    );
                }
                if saturn.mem_cntl[MCTL_PORT2_SX].config[0] == 0x80000 {
                    return Self::calc_crc(
                        saturn,
                        self.port2[((addr - 0x80000) & self.port2_mask) as usize],
                    );
                }
                0x00
            }
            0xc..=0xe => {
                if saturn.mem_cntl[MCTL_PORT1_SX].config[0] == 0xc0000_i32 {
                    return Self::calc_crc(
                        saturn,
                        self.port1[((addr - 0xc0000) & self.port1_mask) as usize],
                    );
                }
                if saturn.mem_cntl[MCTL_PORT2_SX].config[0] == 0xc0000_i32 {
                    return Self::calc_crc(
                        saturn,
                        self.port2[((addr - 0xc0000) & self.port2_mask) as usize],
                    );
                }
                0x00
            }
            0xf => {
                if saturn.mem_cntl[MCTL_SYSRAM_SX].config[0] == 0xf0000_i32 {
                    return Self::calc_crc(saturn, self.ram[(addr - 0xf0000) as usize]);
                }
                if saturn.mem_cntl[MCTL_PORT1_SX].config[0] == 0xc0000_i32 {
                    return Self::calc_crc(
                        saturn,
                        self.port1[((addr - 0xc0000) & self.port1_mask) as usize],
                    );
                }
                if saturn.mem_cntl[MCTL_PORT2_SX].config[0] == 0xc0000_i32 {
                    return Self::calc_crc(
                        saturn,
                        self.port2[((addr - 0xc0000) & self.port2_mask) as usize],
                    );
                }
                0x00
            }
            _ => 0x00,
        }
    }

    pub fn read_nibble_crc_gx(&mut self, saturn: &mut Saturn, addr: i32) -> u8 {
        let addr = addr & 0xfffff;
        match (addr >> 16) & 0x0f {
            0 => {
                if addr < 0x140 && addr >= 0x100 {
                    if saturn.mem_cntl[MCTL_MMIO_GX].config[0] == 0x100 {
                        return 0; // read_dev_mem handled separately
                    } else {
                        return Self::calc_crc(saturn, 0x00);
                    }
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            1 | 2 | 3 | 5 | 6 => Self::calc_crc(saturn, self.rom[addr as usize]),
            4 => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x40000 {
                    return Self::calc_crc(saturn, self.ram[(addr - 0x40000) as usize]);
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            7 => {
                if addr >= 0x7f000
                    && saturn.mem_cntl[MCTL_BANK_GX].config[0] == 0x7f000
                {
                    if addr == 0x7f000 {
                        saturn.bank_switch = 0;
                    }
                    if addr >= 0x7f040 && addr < 0x7f080 {
                        saturn.bank_switch = ((addr - 0x7f040) / 2) as i16;
                    }
                    return 0x7;
                }
                if addr >= 0x7e000
                    && addr < 0x7f000
                    && saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0x7e000
                {
                    return 0x7;
                }
                if addr >= 0x7e000
                    && addr < 0x7f000
                    && saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0x7e000
                {
                    return 0x7;
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            8 => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000 {
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfc000_i32
                        && addr < 0x84000
                    {
                        return Self::calc_crc(saturn, self.ram[(addr - 0x80000) as usize]);
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfe000_i32
                        && addr < 0x82000
                    {
                        return Self::calc_crc(saturn, self.ram[(addr - 0x80000) as usize]);
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xf0000_i32 {
                        return Self::calc_crc(saturn, self.ram[(addr - 0x80000) as usize]);
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32 {
                        return Self::calc_crc(saturn, self.ram[(addr - 0x80000) as usize]);
                    }
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            9 => {
                if saturn.mem_cntl[MCTL_MMIO_GX].config[0] == 0x90000 {
                    if addr < 0x91000 {
                        if addr == 0x90000 {
                            saturn.bank_switch = 0;
                        }
                        if addr >= 0x90040 && addr < 0x90080 {
                            saturn.bank_switch = ((addr - 0x90040) / 2) as i16;
                        }
                        return 0x7;
                    }
                }
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return Self::calc_crc(saturn, self.ram[(addr - 0x80000) as usize]);
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            0xa => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return Self::calc_crc(saturn, self.ram[(addr - 0x80000) as usize]);
                }
                if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xa0000_i32 {
                    return Self::calc_crc(
                        saturn,
                        self.port1[((addr - 0xa0000) & self.port1_mask) as usize],
                    );
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            0xb => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0x80000
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return Self::calc_crc(saturn, self.ram[(addr - 0x80000) as usize]);
                }
                if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xb0000_i32 {
                    let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xb0000);
                    return Self::calc_crc(
                        saturn,
                        self.port2[(idx & self.port2_mask) as usize],
                    );
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            0xc => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0xc0000_i32 {
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfc000_i32
                        && addr < 0xc4000
                    {
                        return Self::calc_crc(saturn, self.ram[(addr - 0xc0000) as usize]);
                    }
                    if saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xfe000_i32
                        && addr < 0xc2000
                    {
                        return Self::calc_crc(saturn, self.ram[(addr - 0xc0000) as usize]);
                    }
                    return Self::calc_crc(saturn, self.ram[(addr - 0xc0000) as usize]);
                }
                if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xc0000_i32 {
                    return Self::calc_crc(
                        saturn,
                        self.port1[((addr - 0xc0000) & self.port1_mask) as usize],
                    );
                }
                if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xc0000_i32 {
                    let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xc0000);
                    return Self::calc_crc(
                        saturn,
                        self.port2[(idx & self.port2_mask) as usize],
                    );
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            0xd..=0xf => {
                if saturn.mem_cntl[MCTL_SYSRAM_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_SYSRAM_GX].config[1] == 0xc0000_i32
                {
                    return Self::calc_crc(saturn, self.ram[(addr - 0xc0000) as usize]);
                }
                if saturn.mem_cntl[MCTL_PORT1_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_PORT1_GX].config[1] == 0xc0000_i32
                {
                    return Self::calc_crc(
                        saturn,
                        self.port1[((addr - 0xc0000) & self.port1_mask) as usize],
                    );
                }
                if saturn.mem_cntl[MCTL_PORT2_GX].config[0] == 0xc0000_i32
                    && saturn.mem_cntl[MCTL_PORT2_GX].config[1] == 0xc0000_i32
                {
                    let idx = ((saturn.bank_switch as i32) << 18) + (addr - 0xc0000);
                    return Self::calc_crc(
                        saturn,
                        self.port2[(idx & self.port2_mask) as usize],
                    );
                }
                Self::calc_crc(saturn, self.rom[addr as usize])
            }
            _ => 0x00,
        }
    }

}
