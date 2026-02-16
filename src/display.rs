// Display rendering — source-accurate port of lcd.c
// Renders HP-48 display memory to an RGBA buffer for web/native display.
//
// C code renders to 262×142 (2× scale + 14px header).
// Rust renders to 131×64 (1× scale, no header). The rendering
// logic is structurally identical to the C: update_display → draw_row →
// draw_nibble → fill pixel. Colors match the C PIXEL_ON/OFF defines.
//
// Annunciators are NOT rendered in the LCD buffer. The emulator exposes
// annunciator_state() as a bitmask for the frontend to render as UI elements.

use crate::types::NIBBLES_PER_ROW;

pub const DISPLAY_WIDTH: u32 = 131;
pub const DISPLAY_HEIGHT: u32 = 64;
pub const DISP_ROWS: i32 = 64;

// RGBA pixel colors — matching lcd.c defines
const PIXEL_ON_R: u8 = 0x10;
const PIXEL_ON_G: u8 = 0x20;
const PIXEL_ON_B: u8 = 0x10;

const PIXEL_OFF_R: u8 = 0xBC;
const PIXEL_OFF_G: u8 = 0xC4;
const PIXEL_OFF_B: u8 = 0xA5;

pub struct Display {
    pub rgba: Vec<u8>,
    pub dirty: bool,
    pub mapped: bool,
    // Diff buffers — matching C's disp_buf[][] and lcd_buffer[][]
    // Used to avoid redundant RGBA writes. 0xf0 = "invalid" sentinel.
    disp_buf: Vec<Vec<u8>>,
    lcd_buffer: Vec<Vec<u8>>,
    old_offset: i32,
    old_lines: i32,
}

const NIBS_PER_BUFFER_ROW: usize = NIBBLES_PER_ROW as usize + 2;

impl Display {
    pub fn new() -> Self {
        let buf_size = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 4) as usize;
        // Initialize RGBA buffer to LCD background color (matching C init_display)
        let mut rgba = vec![0u8; buf_size];
        for i in 0..(DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize {
            rgba[i * 4] = PIXEL_OFF_R;
            rgba[i * 4 + 1] = PIXEL_OFF_G;
            rgba[i * 4 + 2] = PIXEL_OFF_B;
            rgba[i * 4 + 3] = 0xFF;
        }
        Self {
            rgba,
            dirty: true,
            mapped: true,
            disp_buf: vec![vec![0xf0u8; NIBS_PER_BUFFER_ROW]; DISP_ROWS as usize],
            lcd_buffer: vec![vec![0xf0u8; NIBS_PER_BUFFER_ROW]; DISP_ROWS as usize],
            old_offset: -1,
            old_lines: -1,
        }
    }

    /// Port of fill_display_rgba(x, y, v) from lcd.c.
    /// Writes one nibble (4 pixels wide) at nibble column x, nibble row y.
    /// At 1× scale: each nibble = 4 pixels wide, 1 pixel tall.
    /// (C does 2× scale: 8 pixels wide, 2 pixels tall.)
    fn fill_display_rgba(&mut self, x: i32, y: i32, v: u8) {
        let px = x * 4; // 1× scale (C: x * 8)
        let py = y;     // 1× scale, no header (C: y * 2 + HEADER_HEIGHT)

        if py >= DISPLAY_HEIGHT as i32 {
            return;
        }

        for bit in 0..4i32 {
            let col = px + bit; // 1× scale (C: px + bit * 2)

            if col >= DISPLAY_WIDTH as i32 {
                break;
            }

            let (r, g, b) = if (v >> bit) & 1 != 0 {
                (PIXEL_ON_R, PIXEL_ON_G, PIXEL_ON_B)
            } else {
                (PIXEL_OFF_R, PIXEL_OFF_G, PIXEL_OFF_B)
            };

            // 1× scale: one pixel per HP pixel (C: 2×2 block)
            let offset = (py as u32 * DISPLAY_WIDTH + col as u32) * 4;
            let offset = offset as usize;
            self.rgba[offset] = r;
            self.rgba[offset + 1] = g;
            self.rgba[offset + 2] = b;
            self.rgba[offset + 3] = 0xFF;
        }

        self.dirty = true;
    }

    /// Port of draw_nibble(c, r, val) from lcd.c.
    /// Only redraws if the nibble changed (uses lcd_buffer for diffing).
    fn draw_nibble(&mut self, c: i32, r: i32, val: u8) {
        let val = val & 0x0f;
        if val != self.lcd_buffer[r as usize][c as usize] {
            self.lcd_buffer[r as usize][c as usize] = val;
            self.fill_display_rgba(c, r, val);
        }
    }

    /// Port of draw_row(addr, row) from lcd.c.
    /// Reads nibbles for one display row and updates changed pixels.
    fn draw_row(
        &mut self,
        read_fn: &dyn Fn(i32) -> u8,
        addr: i32,
        row: i32,
        disp_offset: i32,
        disp_lines: i32,
    ) {
        let mut line_length = NIBBLES_PER_ROW;
        if disp_offset > 3 && row <= disp_lines {
            line_length += 2;
        }
        for i in 0..line_length {
            let v = read_fn(addr + i);
            if v != self.disp_buf[row as usize][i as usize] {
                self.disp_buf[row as usize][i as usize] = v;
                self.draw_nibble(i, row, v);
            }
        }
    }

    /// Port of disp_draw_nibble(addr, val) from lcd.c.
    /// Called when a nibble in the main display area is written to RAM.
    pub fn disp_draw_nibble(
        &mut self,
        disp_start: i32,
        nibs_per_line: i32,
        lines: i32,
        addr: i32,
        val: u8,
    ) {
        let offset = addr - disp_start;
        let x = if nibs_per_line != 0 {
            offset % nibs_per_line
        } else {
            offset
        };
        if x < 0 || x > 35 {
            return;
        }
        if nibs_per_line != 0 {
            let y = offset / nibs_per_line;
            if y < 0 || y > 63 {
                return;
            }
            let val = val & 0x0f;
            if val != self.disp_buf[y as usize][x as usize] {
                self.disp_buf[y as usize][x as usize] = val;
                self.draw_nibble(x, y, val);
            }
        } else {
            for y in 0..lines {
                let val = val & 0x0f;
                if val != self.disp_buf[y as usize][x as usize] {
                    self.disp_buf[y as usize][x as usize] = val;
                    self.draw_nibble(x, y, val);
                }
            }
        }
    }

    /// Port of menu_draw_nibble(addr, val) from lcd.c.
    /// Called when a nibble in the menu display area is written to RAM.
    pub fn menu_draw_nibble(
        &mut self,
        menu_start: i32,
        lines: i32,
        addr: i32,
        val: u8,
    ) {
        let offset = addr - menu_start;
        let x = offset % NIBBLES_PER_ROW;
        let y = lines + (offset / NIBBLES_PER_ROW) + 1;
        if y < 0 || y >= DISP_ROWS || x < 0 || x >= NIBBLES_PER_ROW {
            return;
        }
        let val = val & 0x0f;
        if val != self.disp_buf[y as usize][x as usize] {
            self.disp_buf[y as usize][x as usize] = val;
            self.draw_nibble(x, y, val);
        }
    }

    /// Port of update_display() from lcd.c.
    /// This is the main entry point called each display refresh.
    pub fn render(
        &mut self,
        display_on: bool,
        _contrast: i32,
        read_fn: &dyn Fn(i32) -> u8,
        disp_start: i32,
        nibs_per_line: i32,
        lines: i32,
        offset: i32,
        menu_start: i32,
    ) {
        if display_on {
            let mut addr = disp_start;

            // C: if offset changed, invalidate main display area buffers
            if offset != self.old_offset {
                for row in 0..=(lines as usize).min(DISP_ROWS as usize - 1) {
                    for col in 0..NIBS_PER_BUFFER_ROW {
                        self.disp_buf[row][col] = 0xf0;
                        self.lcd_buffer[row][col] = 0xf0;
                    }
                }
                self.old_offset = offset;
            }
            // C: if lines changed, invalidate menu area buffers (rows 56..63)
            if lines != self.old_lines {
                for row in 56..DISP_ROWS as usize {
                    for col in 0..NIBS_PER_BUFFER_ROW {
                        self.disp_buf[row][col] = 0xf0;
                        self.lcd_buffer[row][col] = 0xf0;
                    }
                }
                self.old_lines = lines;
            }

            // Main display area: rows 0 to lines (inclusive)
            let mut i = 0;
            while i <= lines {
                self.draw_row(read_fn, addr, i, offset, lines);
                addr += nibs_per_line;
                i += 1;
            }

            // Menu area: remaining rows up to DISP_ROWS
            if i < DISP_ROWS {
                addr = menu_start;
                while i < DISP_ROWS {
                    self.draw_row(read_fn, addr, i, offset, lines);
                    addr += NIBBLES_PER_ROW;
                    i += 1;
                }
            }
        } else {
            // Display off: clear all nibbles to 0x00 (renders as OFF color)
            // C: memset(disp_buf, 0xf0, ...) then draw_nibble(j, i, 0x00)
            for row in 0..DISP_ROWS as usize {
                for col in 0..NIBS_PER_BUFFER_ROW {
                    self.disp_buf[row][col] = 0xf0;
                }
            }
            for i in 0..DISP_ROWS {
                for j in 0..NIBBLES_PER_ROW {
                    self.draw_nibble(j, i, 0x00);
                }
            }
        }
    }
}
