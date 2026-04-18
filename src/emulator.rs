// Top-level Emulator struct — composes all modules.
// Port of main_wasm.c frame_callback + emulate.c schedule() + device.c check_devices()

use crate::alu::{get_end, get_start, RegId};
use crate::cpu::{DisplayState, Saturn};
use crate::device::DeviceFlags;
use crate::display::Display;
use crate::keyboard::Keyboard;
use crate::memory::Memory;
use crate::persist;
use crate::scheduler::*;
use crate::serial::Serial;
use crate::speaker::Speaker;
use crate::timer::*;
use crate::types::*;

const TARGET_IPS: f64 = 5_000_000.0; // ~27x real Saturn speed for snappy UI
const TARGET_IPS_BEEP: f64 = 184000.0; // Original speed during speaker activity
const MAX_INSTRUCTIONS_PER_FRAME: i32 = 100_000;

pub struct Emulator {
    pub saturn: Saturn,
    pub mem: Memory,
    pub display_state: DisplayState,
    pub display: Display,
    pub device: DeviceFlags,
    pub keyboard: Keyboard,
    pub speaker: Speaker,
    pub serial: Serial,
    pub sched: Scheduler,
    pub timers: Timers,
    pub model: Model,

    // Runtime flags
    pub got_alarm: bool,
    pub interrupt_called: bool,
    pub is_shutdown: bool,
    pub first_press: bool,
    pub now: f64, // current time in seconds, updated each frame

    // Speaker throttle: snapshot of speaker_counter from previous frame
    last_speaker_counter: i32,

    // Time offset: maps monotonic now (performance.now/1000) to local epoch seconds
    epoch_offset: f64,
    // HP-48 absolute time state (port of C globals time_offset, set_0_time)
    time_offset: u64, // unix_0_time + set_0_time (HP-48 epoch + user adjustment)
    set_0_time: u64,  // user time adjustment (normally 0, modified by drift correction)
}

impl Emulator {
    pub fn new(
        rom_data: &[u8],
        ram_data: Option<&[u8]>,
        state_data: Option<&[u8]>,
        model: Model,
    ) -> Self {
        let rom_size = match model {
            Model::Sx => ROM_SIZE_SX,
            Model::Gx => ROM_SIZE_GX,
        };
        let ram_size = match model {
            Model::Sx => RAM_SIZE_SX,
            Model::Gx => RAM_SIZE_GX,
        };

        let rom = persist::load_rom(rom_data, rom_size);
        let ram = match ram_data {
            Some(data) => persist::load_ram(data, ram_size),
            None => vec![0u8; ram_size],
        };

        let mut saturn = Saturn::default();
        let mut has_state = false;

        if let Some(state) = state_data {
            if persist::read_state(state, &mut saturn) {
                has_state = true;
            }
        }

        if !has_state {
            persist::init_saturn(&mut saturn, model);
        }

        persist::saturn_config_init(&mut saturn);
        // Match C code: clear card_status after loading state (init.c:831)
        saturn.card_status = 0;

        // Initialize display state from saturn registers
        let mut display_state = DisplayState::default();
        display_state.on = (saturn.disp_io & 0x8) != 0;
        display_state.offset = (saturn.disp_io & 0x7) as i32;
        display_state.contrast = saturn.contrast_ctrl as i32;
        display_state.annunc = saturn.annunc as i32;
        display_state.disp_start = saturn.disp_addr & 0xffffe;
        display_state.menu_start = saturn.menu_addr;
        display_state.menu_end = display_state.menu_start + 0x110;
        display_state.lines = if (saturn.line_count & 0x3f) == 0 {
            63
        } else {
            (saturn.line_count & 0x3f) as i32
        };
        if display_state.offset > 3 {
            display_state.nibs_per_line =
                (NIBBLES_PER_ROW + saturn.line_offset as i32 + 2) & 0xfff;
        } else {
            display_state.nibs_per_line =
                (NIBBLES_PER_ROW + saturn.line_offset as i32) & 0xfff;
        }
        display_state.disp_end =
            display_state.disp_start + display_state.nibs_per_line * (display_state.lines + 1);

        let mut sched = Scheduler::new();
        sched.init(saturn.t1_tick, saturn.t2_tick, saturn.timer1);

        let device = DeviceFlags {
            display_touched: 1,
            contrast_touched: true,
            baud_touched: true,
            ann_touched: true,
            ..DeviceFlags::default()
        };

        Self {
            saturn,
            mem: Memory::new(rom, ram),
            display_state,
            display: Display::new(),
            device,
            keyboard: Keyboard::new(),
            speaker: Speaker::new(),
            serial: Serial::new(),
            sched,
            timers: Timers::new(),
            model,
            got_alarm: false,
            interrupt_called: false,
            is_shutdown: false,
            first_press: true,
            last_speaker_counter: 0,
            now: 0.0,
            epoch_offset: 0.0,
            time_offset: 0,
            set_0_time: 0,
        }
    }

    /// Start the emulation timers (call once after construction).
    /// `now` — performance.now()/1000 for timer tracking.
    /// `unix_epoch_secs` — local epoch seconds for HP-48 absolute clock.
    pub fn start(&mut self, now: f64, unix_epoch_secs: f64) {
        // Store mapping from monotonic time to local epoch time
        self.epoch_offset = unix_epoch_secs - now;

        // HP-48 epoch offset: ticks for THU 01.01.1970 00:00:00
        const UNIX_0_TIME: u64 = (0x0001cf2e_u64 << 32) | 0x8f800000;
        self.set_0_time = 0;
        self.time_offset = UNIX_0_TIME; // unix_0_time + set_0_time (set_0_time starts at 0)

        // Match C web version (persist_ready): only start RUN_TIMER.
        // T1_TIMER starts lazily via check_devices when firmware touches T1 register.
        self.timers.set_accesstime(now);
        // Do NOT call set_accesstime_ram() here. The HP-48 firmware maintains a system
        // integrity checksum over system RAM that covers ACCESSTIME. Overwriting it after
        // the firmware has stored its checksum triggers a warm-start recovery on wake,
        // which clears the stack. The timer offsets (time_offset, epoch_offset) are enough
        // for get_t1_t2() to compute correct T2 values from the existing RAM ACCESSTIME.
        self.timers.start_timer(RUN_TIMER, now);
    }

    // -----------------------------------------------------------------------
    // Memory access
    // -----------------------------------------------------------------------

    #[inline]
    pub fn read_nibble(&mut self, addr: i32) -> u8 {
        let a = addr & 0xfffff;
        // Check for MMIO range
        if a >= 0x100 && a < 0x140 {
            let mmio_idx = match self.model {
                Model::Sx => MCTL_MMIO_SX,
                Model::Gx => MCTL_MMIO_GX,
            };
            if self.saturn.mem_cntl[mmio_idx].config[0] == 0x100 {
                return self.mem.read_dev_mem(
                    &mut self.saturn,
                    &mut self.device,
                    &mut self.sched.device_check,
                    &mut self.sched.schedule_event,
                    a,
                );
            }
        }
        match self.model {
            Model::Sx => self.mem.read_nibble_sx(&self.saturn, addr),
            Model::Gx => self.mem.read_nibble_gx(&mut self.saturn, addr),
        }
    }

    /// Read n nibbles from addr, assembling into an i32 (low nibble first).
    #[inline]
    pub fn read_nibbles(&mut self, addr: i32, n: i32) -> i32 {
        let mut val: i32 = 0;
        for i in (0..n).rev() {
            val <<= 4;
            val |= (self.read_nibble(addr + i) & 0xf) as i32;
        }
        val
    }

    /// Check speaker toggle after OUT register write (wraps speaker.check_out_register)
    pub fn check_out_register(&mut self) {
        self.speaker
            .check_out_register(self.saturn.out[2], &mut self.device.speaker_counter);
    }

    #[inline]
    pub fn write_nibble(&mut self, addr: i32, val: i32) {
        let needs_display_check = match self.model {
            Model::Sx => self.mem.write_nibble_sx(
                &mut self.saturn,
                &mut self.display_state,
                &mut self.device,
                &mut self.sched.device_check,
                &mut self.sched.schedule_event,
                addr,
                val,
            ),
            Model::Gx => self.mem.write_nibble_gx(
                &mut self.saturn,
                &mut self.display_state,
                &mut self.device,
                &mut self.sched.device_check,
                &mut self.sched.schedule_event,
                addr,
                val,
            ),
        };
        // Display nibble update — port of the display check at the end of
        // write_nibble_sx/gx in memory.c (calls disp_draw_nibble/menu_draw_nibble)
        if needs_display_check {
            let addr = addr & 0xfffff;
            if self.device.display_touched != 0 {
                return;
            }
            let ds = &self.display_state;
            if addr >= ds.disp_start && addr < ds.disp_end {
                self.display.disp_draw_nibble(
                    ds.disp_start,
                    ds.nibs_per_line,
                    ds.lines,
                    addr,
                    val as u8,
                );
            }
            if self.display_state.lines == 63 {
                return;
            }
            let ds = &self.display_state;
            if addr >= ds.menu_start && addr < ds.menu_end {
                self.display.menu_draw_nibble(
                    ds.menu_start,
                    ds.lines,
                    addr,
                    val as u8,
                );
            }
        }
    }

    #[inline]
    pub fn read_nibble_crc(&mut self, addr: i32) -> u8 {
        let a = addr & 0xfffff;
        // Check for MMIO range — read value and apply CRC
        if a >= 0x100 && a < 0x140 {
            let mmio_idx = match self.model {
                Model::Sx => MCTL_MMIO_SX,
                Model::Gx => MCTL_MMIO_GX,
            };
            if self.saturn.mem_cntl[mmio_idx].config[0] == 0x100 {
                let val = self.mem.read_dev_mem(
                    &mut self.saturn,
                    &mut self.device,
                    &mut self.sched.device_check,
                    &mut self.sched.schedule_event,
                    a,
                );
                // Apply CRC (same formula as Memory::calc_crc)
                self.saturn.crc = ((self.saturn.crc >> 4)
                    ^ (((self.saturn.crc ^ val as u16) & 0xf) * 0x1081))
                    as u16;
                return val;
            }
        }
        match self.model {
            Model::Sx => self.mem.read_nibble_crc_sx(&mut self.saturn, addr),
            Model::Gx => self.mem.read_nibble_crc_gx(&mut self.saturn, addr),
        }
    }

    // -----------------------------------------------------------------------
    // Store / Recall (port of actions.c store/recall/store_n/recall_n)
    // -----------------------------------------------------------------------

    /// Store register field to memory at D0 or D1
    pub fn store(&mut self, r_idx: RegId, code: u8, d_sel: u8) {
        let s = get_start(code, self.saturn.p);
        let e = get_end(code, self.saturn.p);
        let r = self.saturn.get_reg(r_idx).clone();
        let mut dat = if d_sel == 0 {
            self.saturn.d0
        } else {
            self.saturn.d1
        };
        for i in s..=e {
            self.write_nibble(dat, r[i] as i32);
            dat += 1;
        }
    }

    /// Store n nibbles from register[0..n] to memory at D0 or D1
    pub fn store_n(&mut self, r_idx: RegId, n: usize, d_sel: u8) {
        let r = self.saturn.get_reg(r_idx).clone();
        let mut dat = if d_sel == 0 {
            self.saturn.d0
        } else {
            self.saturn.d1
        };
        for i in 0..n {
            self.write_nibble(dat, r[i] as i32);
            dat += 1;
        }
    }

    /// Read from memory at D0 or D1 into register field (with CRC)
    pub fn recall(&mut self, r_idx: RegId, code: u8, d_sel: u8) {
        let s = get_start(code, self.saturn.p);
        let e = get_end(code, self.saturn.p);
        let mut dat = if d_sel == 0 {
            self.saturn.d0
        } else {
            self.saturn.d1
        };
        let mut vals = [0u8; 16];
        for i in s..=e {
            vals[i] = self.read_nibble_crc(dat);
            dat += 1;
        }
        let r = self.saturn.get_reg_mut(r_idx);
        for i in s..=e {
            r[i] = vals[i];
        }
    }

    /// Read n nibbles from memory into register[0..n] (with CRC)
    pub fn recall_n(&mut self, r_idx: RegId, n: usize, d_sel: u8) {
        let mut dat = if d_sel == 0 {
            self.saturn.d0
        } else {
            self.saturn.d1
        };
        let mut vals = [0u8; 16];
        for i in 0..n {
            vals[i] = self.read_nibble_crc(dat);
            dat += 1;
        }
        let r = self.saturn.get_reg_mut(r_idx);
        for i in 0..n {
            r[i] = vals[i];
        }
    }

    /// Load n nibbles from addr into register starting at P (port of load_constant)
    pub fn load_constant(&mut self, r_idx: RegId, n: usize, addr: i32) {
        let mut p = self.saturn.p as usize;
        let mut vals = [0u8; 16];
        for i in 0..n {
            vals[i] = self.read_nibble(addr + i as i32);
        }
        let r = self.saturn.get_reg_mut(r_idx);
        for i in 0..n {
            r[p] = vals[i];
            p = (p + 1) & 0xf;
        }
    }

    /// Load n nibbles from addr into D0 or D1 (port of load_addr)
    pub fn load_addr(&mut self, d_sel: u8, addr: i32, n: usize) {
        let mut dat = if d_sel == 0 {
            self.saturn.d0
        } else {
            self.saturn.d1
        };
        for i in 0..n {
            dat &= !NIBBLE_MASKS[i];
            dat |= (self.read_nibble(addr + i as i32) as i32) << (i as i32 * 4);
        }
        if d_sel == 0 {
            self.saturn.d0 = dat;
        } else {
            self.saturn.d1 = dat;
        }
    }

    /// Load n nibbles from addr into register[0..n] (port of load_address)
    pub fn load_address(&mut self, r_idx: RegId, addr: i32, n: usize) {
        let mut vals = [0u8; 16];
        for i in 0..n {
            vals[i] = self.read_nibble(addr + i as i32);
        }
        let r = self.saturn.get_reg_mut(r_idx);
        for i in 0..n {
            r[i] = vals[i];
        }
    }

    // -----------------------------------------------------------------------
    // Wrappers for Saturn methods needing model or now
    // -----------------------------------------------------------------------

    pub fn do_unconfigure(&mut self) {
        self.saturn.do_unconfigure(self.model);
    }

    pub fn do_reset(&mut self) {
        self.saturn.do_reset(self.model);
    }

    // -----------------------------------------------------------------------
    // get_t1_t2 — full port of timer.c get_t1_t2()
    // -----------------------------------------------------------------------

    /// Compute T1 and T2 tick values. Full port of get_t1_t2() from timer.c.
    /// T1: from the T1 timer in 512 Hz encoding.
    /// T2: computed from ACCESSTIME in RAM vs current wall-clock time.
    pub fn get_t1_t2(&mut self) -> T1T2Ticks {
        // RAM offsets for ACCESSTIME
        const ACCESSTIME_SX: usize = 0x52;
        const ACCESSTIME_GX: usize = 0x58;

        let now = self.now;

        // --- T1: get T1 timer value in T1 encoding ---
        let t1_secs = self.timers.get_timer_secs(T1_TIMER, now);
        let t1_val = secs_to_t1_encoding(t1_secs);
        let t1_ticks = t1_val as u32 as i32;

        // --- T2: compute from ACCESSTIME in RAM ---
        // Current local epoch time in 8192 Hz ticks
        let epoch_now = now + self.epoch_offset;
        let tv_sec = epoch_now.floor() as u64;
        let tv_usec = ((epoch_now - epoch_now.floor()) * 1_000_000.0) as u64;
        let stop_hi = tv_sec >> 19;
        let stop_lo = ((tv_sec << 13) & 0xffffffff) | ((tv_usec << 7) / 15625);
        let mut stop: u64 = (stop_hi << 32) | stop_lo;

        // Add time_offset (unix_0_time + set_0_time)
        stop = stop.wrapping_add(self.time_offset);

        // Read 13-nibble ACCESSTIME from RAM
        let accesstime_loc = match self.model {
            Model::Gx => ACCESSTIME_GX,
            Model::Sx => ACCESSTIME_SX,
        };
        let mut access_time: u64 = 0;
        for i in (0..13).rev() {
            access_time <<= 4;
            if accesstime_loc + i < self.mem.ram.len() {
                access_time |= (self.mem.ram[accesstime_loc + i] & 0xf) as u64;
            }
        }

        // access_time = access_time - stop (ACCESSTIME - current_ticks)
        access_time = access_time.wrapping_sub(stop);

        // Extract lo/hi for 64-bit comparison
        let at_lo = access_time as u32;
        let at_hi = (access_time >> 32) as u32;

        if self.sched.adj_time_pending {
            // Inside interrupt or debugger — don't adjust time
            let timer2 = self.saturn.timer2;
            if (timer2 >= 0 && (at_lo & 0x80000000) != 0)
                || ((timer2 as u32) > at_lo)
            {
                return T1T2Ticks { t1_ticks, t2_ticks: at_lo as i32 };
            } else {
                self.saturn.t2_tick = self.saturn.t2_tick.wrapping_add(1);
                return T1T2Ticks { t1_ticks, t2_ticks: timer2 };
            }
        }

        // Compute drift: adj_time = access_time - saturn.timer2 (sign-extended)
        let timer2_64: u64 = if self.saturn.timer2 < 0 {
            self.saturn.timer2 as i64 as u64
        } else {
            self.saturn.timer2 as u64
        };
        let adj_time = access_time.wrapping_sub(timer2_64);
        let adj_hi = (adj_time >> 32) as u32;

        // delta = abs(adj_time)
        let delta = if adj_hi & 0x8000000 != 0 {
            0u64.wrapping_sub(adj_time)
        } else {
            adj_time
        };
        let delta_hi = (delta >> 32) as u32;
        let delta_lo = delta as u32;

        // If drift > 0x3C000 ticks (~half a minute), adjust time_offset
        if delta_hi != 0 || delta_lo > 0x3c000 {
            self.set_0_time = self.set_0_time.wrapping_add(adj_time);
            self.time_offset = self.time_offset.wrapping_add(adj_time);
            // Recompute access_time with adjusted offset
            access_time = access_time.wrapping_sub(adj_time);
        }

        // Sanity check
        let at_lo = access_time as u32;
        let timer2 = self.saturn.timer2;
        if (timer2 >= 0 && (at_lo & 0x80000000) != 0)
            || ((timer2 as u32) > at_lo)
        {
            T1T2Ticks { t1_ticks, t2_ticks: at_lo as i32 }
        } else {
            self.saturn.t2_tick = self.saturn.t2_tick.wrapping_add(1);
            T1T2Ticks { t1_ticks, t2_ticks: timer2 }
        }
    }

    // -----------------------------------------------------------------------
    // Keyboard — do_in (port of actions.c)
    // -----------------------------------------------------------------------

    pub fn do_in(&mut self) {
        let mut out: i32 = 0;
        for i in (0..=2).rev() {
            out <<= 4;
            out |= self.saturn.out[i] as i32;
        }
        let mut in_val: i32 = 0;
        for i in 0..9 {
            if out & (1 << i) != 0 {
                in_val |= self.saturn.keybuf.rows[i] as i32;
            }
        }

        // Keyboard debounce patch (from x48 SVN)
        if self.saturn.pc == 0x00E31
            && !self.first_press
            && ((out & 0x10 != 0 && in_val & 0x1 != 0)
                || (out & 0x40 != 0 && in_val & 0x7 != 0)
                || (out & 0x80 != 0 && in_val & 0x2 != 0))
        {
            for i in 0..9 {
                if out & (1 << i) != 0 {
                    self.saturn.keybuf.rows[i] = 0;
                }
            }
            self.first_press = true;
        } else {
            self.first_press = false;
        }

        for i in 0..4 {
            self.saturn.in_reg[i] = (in_val & 0xf) as u8;
            in_val >>= 4;
        }
    }

    // -----------------------------------------------------------------------
    // Shutdown (port of actions.c)
    // -----------------------------------------------------------------------

    pub fn do_shutdown(&mut self, now: f64) {
        if self.device.display_touched != 0 {
            self.device.display_touched = 0;
            self.update_display();
        }

        self.timers.stop_timer(RUN_TIMER, now);
        self.timers.start_timer(IDLE_TIMER, now);

        // Check if OUT register is all zeros
        if self.saturn.out[0] == 0 && self.saturn.out[1] == 0 && self.saturn.out[2] == 0 {
            self.saturn.intenable = 1;
            self.saturn.int_pending = 0;
        }

        self.is_shutdown = true;
    }

    pub fn do_shutdown_check(&mut self, now: f64) {
        let mut wake = false;

        if self.got_alarm {
            self.got_alarm = false;

            let ticks = self.get_t1_t2();
            if self.saturn.t2_ctrl & 0x01 != 0 {
                self.saturn.timer2 = ticks.t2_ticks;
            }
            self.saturn.timer1 = (self.sched.set_t1 - ticks.t1_ticks) as i8;
            self.sched.set_t1 = ticks.t1_ticks;

            self.interrupt_called = false;

            if self.saturn.timer2 <= 0 {
                if self.saturn.t2_ctrl & 0x04 != 0 {
                    wake = true;
                }
                if self.saturn.t2_ctrl & 0x02 != 0 {
                    wake = true;
                    self.saturn.t2_ctrl |= 0x08;
                    self.do_interrupt();
                }
            }

            if self.saturn.timer1 <= 0 {
                self.saturn.timer1 &= 0x0f;
                if self.saturn.t1_ctrl & 0x04 != 0 {
                    wake = true;
                }
                if self.saturn.t1_ctrl & 0x03 != 0 {
                    wake = true;
                    self.saturn.t1_ctrl |= 0x08;
                    self.do_interrupt();
                }
            }

            if !wake {
                self.interrupt_called = false;
                self.serial.receive_char();
                if self.interrupt_called {
                    wake = true;
                }
            }
        }

        // Process key events every cycle for responsiveness
        self.interrupt_called = false;
        if self.keyboard.process_events(&mut self.saturn) {
            self.do_kbd_int();
            if self.interrupt_called {
                wake = true;
            }
        }

        if wake {
            self.is_shutdown = false;
            self.timers.stop_timer(IDLE_TIMER, now);
            self.timers.start_timer(RUN_TIMER, now);
        }
    }

    // -----------------------------------------------------------------------
    // Interrupts (port of actions.c)
    // -----------------------------------------------------------------------

    pub fn do_interrupt(&mut self) {
        self.interrupt_called = true;
        if self.saturn.intenable != 0 {
            self.saturn.push_return_addr(self.saturn.pc);
            self.saturn.pc = 0xf;
            self.saturn.intenable = 0;
        }
    }

    pub fn do_kbd_int(&mut self) {
        self.interrupt_called = true;
        if self.saturn.intenable != 0 {
            self.saturn.push_return_addr(self.saturn.pc);
            self.saturn.pc = 0xf;
            self.saturn.intenable = 0;
        } else {
            self.saturn.int_pending = 1;
        }
    }

    pub fn do_return_interrupt(&mut self) {
        if self.saturn.int_pending != 0 {
            self.saturn.int_pending = 0;
            self.saturn.intenable = 0;
            self.saturn.pc = 0xf;
        } else {
            self.saturn.pc = self.saturn.pop_return_addr();
            self.saturn.intenable = 1;

            if self.sched.adj_time_pending {
                self.sched.schedule_event = 0;
                self.sched.sched_adjtime = 0;
            }
        }
    }

    pub fn do_reset_interrupt_system(&mut self) {
        self.saturn.kbd_ien = 1;
        let mut gen_intr = false;
        for i in 0..9 {
            if self.saturn.keybuf.rows[i] != 0 {
                gen_intr = true;
                break;
            }
        }
        if gen_intr {
            self.do_kbd_int();
        }
    }

    // -----------------------------------------------------------------------
    // Display
    // -----------------------------------------------------------------------

    pub fn update_display(&mut self) {
        let ds = &self.display_state;
        let saturn = &self.saturn;
        let mem = &self.mem;
        let model = self.model;

        self.display.render(
            ds.on,
            ds.contrast,
            &|addr| match model {
                Model::Sx => mem.read_nibble_sx(saturn, addr),
                Model::Gx => mem.read_nibble_gx_display(saturn, addr),
            },
            ds.disp_start,
            ds.nibs_per_line,
            ds.lines,
            ds.offset,
            ds.menu_start,
        );
    }

    // -----------------------------------------------------------------------
    // Check devices (port of device.c check_devices)
    // -----------------------------------------------------------------------

    pub fn check_devices(&mut self, now: f64) {
        if self.device.display_touched > 0 {
            self.device.display_touched -= 1;
            if self.device.display_touched == 0 {
                self.update_display();
            }
        }
        if self.device.display_touched > 0 {
            self.sched.device_check = true;
        }
        if self.device.contrast_touched {
            self.device.contrast_touched = false;
        }
        if self.device.ann_touched {
            self.device.ann_touched = false;
        }
        if self.device.baud_touched {
            self.device.baud_touched = false;
            self.serial.set_baud(self.saturn.baud);
        }
        if self.device.ioc_touched {
            self.device.ioc_touched = false;
            if (self.saturn.io_ctrl & 0x02 != 0) && (self.saturn.rcs & 0x01 != 0) {
                self.do_interrupt();
            }
        }
        if self.device.rbr_touched {
            self.device.rbr_touched = false;
            self.serial.receive_char();
        }
        if self.device.tbr_touched {
            self.device.tbr_touched = false;
            self.serial.transmit_char();
        }
        if self.device.t1_touched {
            self.saturn.t1_instr = 0;
            self.sched.sched_timer1 = self.saturn.t1_tick as i32;
            self.timers.restart_timer(T1_TIMER, now);
            self.sched.set_t1 = self.saturn.timer1 as i32;
            self.device.t1_touched = false;
        }
        if self.device.t2_touched {
            self.saturn.t2_instr = 0;
            self.sched.sched_timer2 = self.saturn.t2_tick as i32;
            self.device.t2_touched = false;
        }

        // Speaker toggle detection
        self.speaker
            .check_out_register(self.saturn.out[2], &mut self.device.speaker_counter);
    }

    // -----------------------------------------------------------------------
    // Schedule (port of emulate.c schedule())
    // -----------------------------------------------------------------------

    pub fn schedule(&mut self, now: f64) {
        let steps = self.sched.instructions.wrapping_sub(self.sched.old_sched_instr) as i32;
        self.sched.old_sched_instr = self.sched.instructions;

        // Timer 2
        self.sched.sched_timer2 -= steps;
        if self.sched.sched_timer2 <= 0 {
            if self.saturn.intenable == 0 {
                self.sched.sched_timer2 = SCHED_TIMER2;
            } else {
                self.sched.sched_timer2 = self.saturn.t2_tick as i32;
            }
            self.saturn.t2_instr += steps;
            if self.saturn.t2_ctrl & 0x01 != 0 {
                self.saturn.timer2 -= 1;
            }
            if self.saturn.timer2 == 0 && (self.saturn.t2_ctrl & 0x02 != 0) {
                self.saturn.t2_ctrl |= 0x08;
                self.do_interrupt();
            }
        }
        self.sched.schedule_event = self.sched.sched_timer2;

        // Device check
        if self.sched.device_check {
            self.sched.device_check = false;
            self.sched.sched_display -= steps;
            if self.sched.sched_display <= 0 {
                if self.device.display_touched != 0 {
                    self.device.display_touched -= steps;
                }
                if self.device.display_touched < 0 {
                    self.device.display_touched = 1;
                }
            }
            self.check_devices(now);
            self.sched.sched_display = SCHED_NEVER;
            if self.device.display_touched != 0 {
                if self.device.display_touched < self.sched.sched_display {
                    self.sched.sched_display = self.device.display_touched - 1;
                }
                if self.sched.sched_display < self.sched.schedule_event {
                    self.sched.schedule_event = self.sched.sched_display;
                }
            }
        }

        // Receive
        self.sched.sched_receive -= steps;
        if self.sched.sched_receive <= 0 {
            self.sched.sched_receive = SCHED_RECEIVE;
            if (self.saturn.rcs & 0x01) == 0 {
                self.serial.receive_char();
            }
        }
        if self.sched.sched_receive < self.sched.schedule_event {
            self.sched.schedule_event = self.sched.sched_receive;
        }

        // Adjust time
        self.sched.sched_adjtime -= steps;
        if self.sched.sched_adjtime <= 0 {
            self.sched.sched_adjtime = SCHED_ADJTIME;

            if self.saturn.pc < SRVC_IO_START || self.saturn.pc > SRVC_IO_END {
                let ticks = self.get_t1_t2();
                if self.saturn.t2_ctrl & 0x01 != 0 {
                    self.saturn.timer2 = ticks.t2_ticks;
                }

                if (self.saturn.t2_ctrl & 0x08) == 0 && self.saturn.timer2 <= 0 {
                    if self.saturn.t2_ctrl & 0x02 != 0 {
                        self.saturn.t2_ctrl |= 0x08;
                        self.do_interrupt();
                    }
                }

                self.sched.adj_time_pending = false;

                self.saturn.timer1 = (self.sched.set_t1 - ticks.t1_ticks) as i8;
                if (self.saturn.t1_ctrl & 0x08) == 0 && self.saturn.timer1 <= 0 {
                    if self.saturn.t1_ctrl & 0x02 != 0 {
                        self.saturn.t1_ctrl |= 0x08;
                        self.do_interrupt();
                    }
                }
                self.saturn.timer1 &= 0x0f;
            } else {
                self.sched.adj_time_pending = true;
            }
        }
        if self.sched.sched_adjtime < self.sched.schedule_event {
            self.sched.schedule_event = self.sched.sched_adjtime;
        }

        // Timer 1
        self.sched.sched_timer1 -= steps;
        if self.sched.sched_timer1 <= 0 {
            if self.saturn.intenable == 0 {
                self.sched.sched_timer1 = SCHED_TIMER1;
            } else {
                self.sched.sched_timer1 = self.saturn.t1_tick as i32;
            }
            self.saturn.t1_instr += steps;
            self.saturn.timer1 = (self.saturn.timer1 - 1) & 0xf;
            if self.saturn.timer1 == 0 && (self.saturn.t1_ctrl & 0x02 != 0) {
                self.saturn.t1_ctrl |= 0x08;
                self.do_interrupt();
            }
        }
        if self.sched.sched_timer1 < self.sched.schedule_event {
            self.sched.schedule_event = self.sched.sched_timer1;
        }

        // Statistics
        self.sched.sched_statistics -= steps;
        if self.sched.sched_statistics <= 0 {
            self.sched.sched_statistics = SCHED_STATISTICS;
            let run = self.timers.get_timer(RUN_TIMER, now);
            self.sched.s_1 = (run.hi << 19) | (run.lo >> 13);
            self.sched.s_16 = (run.hi << 23) | (run.lo >> 9);
            let delta_t_1 = self.sched.s_1.wrapping_sub(self.sched.old_s_1);
            let delta_t_16 = self.sched.s_16.wrapping_sub(self.sched.old_s_16);
            self.sched.old_s_1 = self.sched.s_1;
            self.sched.old_s_16 = self.sched.s_16;
            let delta_i = self.sched.instructions.wrapping_sub(self.sched.old_stat_instr);
            self.sched.old_stat_instr = self.sched.instructions;
            if delta_t_1 > 0 {
                self.sched.t1_i_per_tick = ((NR_SAMPLES - 1) * self.sched.t1_i_per_tick
                    + (delta_i as i32 / delta_t_16 as i32))
                    / NR_SAMPLES;
                self.sched.t2_i_per_tick = self.sched.t1_i_per_tick / 512;
                self.saturn.i_per_s = ((NR_SAMPLES - 1) * self.saturn.i_per_s
                    + (delta_i as i32 / delta_t_1 as i32))
                    / NR_SAMPLES;
            } else {
                self.sched.t1_i_per_tick = 8192;
                self.sched.t2_i_per_tick = 16;
            }
            self.saturn.t1_tick = self.sched.t1_i_per_tick as i16;
            self.saturn.t2_tick = self.sched.t2_i_per_tick as i16;
        }
        if self.sched.sched_statistics < self.sched.schedule_event {
            self.sched.schedule_event = self.sched.sched_statistics;
        }

        // Instruction rollover
        self.sched.sched_instr_rollover -= steps;
        if self.sched.sched_instr_rollover <= 0 {
            self.sched.sched_instr_rollover = SCHED_INSTR_ROLLOVER;
            self.sched.instructions = 1;
            self.sched.old_sched_instr = 1;
            self.timers.reset_timer(RUN_TIMER);
            self.timers.reset_timer(IDLE_TIMER);
            self.timers.start_timer(RUN_TIMER, now);
        }
        if self.sched.sched_instr_rollover < self.sched.schedule_event {
            self.sched.schedule_event = self.sched.sched_instr_rollover;
        }

        self.sched.schedule_event -= 1;

        // Process keyboard events
        if self.keyboard.process_events(&mut self.saturn) {
            self.do_kbd_int();
        }

        if self.got_alarm {
            self.got_alarm = false;
        }
    }

    // -----------------------------------------------------------------------
    // Frame callback (port of main_wasm.c frame_callback)
    // -----------------------------------------------------------------------

    pub fn run_frame(&mut self, elapsed_ms: f64, now: f64) {
        self.now = now;

        // Cap elapsed time to avoid huge bursts after tab switch
        let elapsed = if elapsed_ms > 100.0 {
            100.0
        } else {
            elapsed_ms
        };

        // Throttle to original speed during beeps so firmware delay loops
        // produce correct wall-clock duration.
        // Detect active beep: speaker_counter is still incrementing since last frame.
        let sc = self.device.speaker_counter;
        let beeping = sc > self.last_speaker_counter;
        self.last_speaker_counter = sc;
        let ips = if beeping { TARGET_IPS_BEEP } else { TARGET_IPS };
        let mut target = (ips * elapsed / 1000.0) as i32;
        if target > MAX_INSTRUCTIONS_PER_FRAME {
            target = MAX_INSTRUCTIONS_PER_FRAME;
        }
        if target < 1 {
            target = 1;
        }

        self.got_alarm = true;

        if self.is_shutdown {
            self.do_shutdown_check(now);
            return;
        }

        for _ in 0..target {
            self.speaker.instr_count += 1;
            self.sched.instructions += 1;
            self.step_instruction();

            if self.sched.schedule_event <= 0 {
                self.schedule(now);
            } else {
                self.sched.schedule_event -= 1;
            }

            if self.is_shutdown {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Save state
    // -----------------------------------------------------------------------

    pub fn save_state(&self) -> Vec<u8> {
        persist::write_state(&self.saturn)
    }

    pub fn save_ram(&self) -> Vec<u8> {
        persist::pack_nibbles(&self.mem.ram)
    }

    // -----------------------------------------------------------------------
    // Display access (for WASM interface)
    // -----------------------------------------------------------------------

    pub fn display_buffer(&self) -> &[u8] {
        &self.display.rgba
    }

    pub fn is_display_dirty(&self) -> bool {
        self.display.dirty
    }

    pub fn clear_display_dirty(&mut self) {
        self.display.dirty = false;
    }

    pub fn annunciator_state(&self) -> u32 {
        self.saturn.annunc as u32
    }

    pub fn speaker_frequency(&mut self) -> u32 {
        self.speaker.get_frequency()
    }
}
