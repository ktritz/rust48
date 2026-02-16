// Instruction scheduling — exact port of schedule() from emulate.c

pub const SRVC_IO_START: i32 = 0x3c0;
pub const SRVC_IO_END: i32 = 0x5ec;

pub const SCHED_INSTR_ROLLOVER: i32 = 0x3fffffff;
pub const SCHED_RECEIVE: i32 = 0x7ff;
pub const SCHED_ADJTIME: i32 = 0x1ffe;
pub const SCHED_TIMER1: i32 = 0x1e00;
pub const SCHED_TIMER2: i32 = 0xf;
pub const SCHED_STATISTICS: i32 = 0x7ffff;
pub const SCHED_NEVER: i32 = 0x7fffffff;
pub const NR_SAMPLES: i32 = 10;

pub struct Scheduler {
    pub instructions: u32,
    pub old_instr: u32,
    pub schedule_event: i32,
    pub device_check: bool,
    pub adj_time_pending: bool,
    pub set_t1: i32,

    pub sched_instr_rollover: i32,
    pub sched_receive: i32,
    pub sched_adjtime: i32,
    pub sched_timer1: i32,
    pub sched_timer2: i32,
    pub sched_statistics: i32,
    pub sched_display: i32,

    pub t1_i_per_tick: i32,
    pub t2_i_per_tick: i32,

    // Statistics sampling
    pub s_1: u32,
    pub s_16: u32,
    pub old_s_1: u32,
    pub old_s_16: u32,

    pub old_sched_instr: u32,
    pub old_stat_instr: u32,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            instructions: 0,
            old_instr: 0,
            schedule_event: 0,
            device_check: false,
            adj_time_pending: false,
            set_t1: 0,
            sched_instr_rollover: SCHED_INSTR_ROLLOVER,
            sched_receive: SCHED_RECEIVE,
            sched_adjtime: SCHED_ADJTIME,
            sched_timer1: SCHED_TIMER1,
            sched_timer2: SCHED_TIMER2,
            sched_statistics: SCHED_STATISTICS,
            sched_display: SCHED_NEVER,
            t1_i_per_tick: 8192,
            t2_i_per_tick: 16,
            s_1: 0,
            s_16: 0,
            old_s_1: 0,
            old_s_16: 0,
            old_sched_instr: 0,
            old_stat_instr: 0,
        }
    }

    pub fn init(&mut self, t1_tick: i16, t2_tick: i16, timer1: i8) {
        self.sched_timer1 = t1_tick as i32;
        self.t1_i_per_tick = t1_tick as i32;
        self.sched_timer2 = t2_tick as i32;
        self.t2_i_per_tick = t2_tick as i32;
        self.set_t1 = timer1 as i32;
    }
}
