// Hardware timers + wall-clock sync — source-accurate port of timer.c
//
// The C code uses two different time encodings:
//   T1_TIMER:   (tv_sec << 9) | (tv_usec / 62500)    — ~512 "ticks" per second
//   RUN/IDLE:   (tv_sec << 13) | ((tv_usec << 7) / 15625)  — ~8192 ticks per second
//
// The T1 encoding is designed so that the low 4 bits count at 16 Hz
// (since 512 mod 16 = 0), allowing `& 0x0f` to extract T1 tick count.
//
// The Rust port uses f64 seconds internally and converts to the C
// encoding at API boundaries (get_timer, get_t1_t2).

pub const T1_TIMER: usize = 0;
pub const RUN_TIMER: usize = 1;
pub const IDLE_TIMER: usize = 2;
pub const NUM_TIMERS: usize = 3;

#[derive(Clone, Copy, Debug, Default)]
pub struct TimerVal {
    pub hi: u32,
    pub lo: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct T1T2Ticks {
    pub t1_ticks: i32,
    pub t2_ticks: i32,
}

#[derive(Clone, Debug)]
pub struct Timer {
    pub start: f64,
    pub stop: f64,
    pub running: bool,
    pub accumulated: f64,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            start: 0.0,
            stop: 0.0,
            running: false,
            accumulated: 0.0,
        }
    }
}

pub struct Timers {
    timers: [Timer; NUM_TIMERS],
    access_time: f64,
}

/// Convert f64 seconds to the C T1 timer encoding: (sec << 9) | (usec / 62500)
/// Returns a 64-bit value where hi = upper bits, lo = lower 32 bits.
pub fn secs_to_t1_encoding(secs: f64) -> u64 {
    if secs <= 0.0 {
        return 0;
    }
    let tv_sec = secs.floor() as u64;
    let tv_usec = ((secs - secs.floor()) * 1_000_000.0) as u64;
    let hi = tv_sec >> 23;
    let lo = ((tv_sec << 9) & 0xffffffff) | (tv_usec / 62500);
    (hi << 32) | lo
}

/// Convert f64 seconds to the C RUN/IDLE timer encoding: (sec << 13) | ((usec << 7) / 15625)
/// Returns a 64-bit value where hi = upper bits, lo = lower 32 bits.
fn secs_to_8192_encoding(secs: f64) -> u64 {
    if secs <= 0.0 {
        return 0;
    }
    let tv_sec = secs.floor() as u64;
    let tv_usec = ((secs - secs.floor()) * 1_000_000.0) as u64;
    let hi = tv_sec >> 19;
    let lo = ((tv_sec << 13) & 0xffffffff) | ((tv_usec << 7) / 15625);
    (hi << 32) | lo
}

impl Timers {
    pub fn new() -> Self {
        Self {
            timers: Default::default(),
            access_time: 0.0,
        }
    }

    pub fn reset_timer(&mut self, n: usize) {
        self.timers[n].running = false;
        self.timers[n].start = 0.0;
        self.timers[n].stop = 0.0;
        self.timers[n].accumulated = 0.0;
    }

    pub fn start_timer(&mut self, n: usize, now: f64) {
        if !self.timers[n].running {
            self.timers[n].running = true;
            self.timers[n].start = now;
        }
    }

    pub fn stop_timer(&mut self, n: usize, now: f64) {
        if self.timers[n].running {
            self.timers[n].running = false;
            self.timers[n].stop = now;
            self.timers[n].accumulated += self.timers[n].stop - self.timers[n].start;
        }
    }

    pub fn restart_timer(&mut self, n: usize, now: f64) {
        self.timers[n].start = now;
        self.timers[n].accumulated = 0.0;
        self.timers[n].running = true;
    }

    /// Get elapsed time on timer n in seconds.
    pub fn get_timer_secs(&self, n: usize, now: f64) -> f64 {
        let mut total = self.timers[n].accumulated;
        if self.timers[n].running {
            total += now - self.timers[n].start;
        }
        total
    }

    /// Get elapsed time as word_64 in 8192 Hz encoding.
    /// Port of get_timer(n) from timer.c.
    /// For T1_TIMER: uses T1 encoding (sec<<9 | usec/62500)
    /// For RUN/IDLE: uses 8192 Hz encoding (sec<<13 | (usec<<7)/15625)
    pub fn get_timer(&self, n: usize, now: f64) -> TimerVal {
        let secs = self.get_timer_secs(n, now);
        let val = if n == T1_TIMER {
            secs_to_t1_encoding(secs)
        } else {
            secs_to_8192_encoding(secs)
        };
        TimerVal {
            hi: (val >> 32) as u32,
            lo: val as u32,
        }
    }

    pub fn set_accesstime(&mut self, now: f64) {
        self.access_time = now;
    }

    pub fn is_running(&self, n: usize) -> bool {
        self.timers[n].running
    }
}
