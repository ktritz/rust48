// Instruction decoder — exact port of step_instruction / decode_group_80 /
// decode_group_1 / decode_8_thru_f from emulate.c
//
// Returns true for illegal instruction (stop), false for normal execution.

use crate::alu::RegId;
use crate::types::*;

const JUMPMASKS: [i32; 8] = [
    -1,          // 0xffffffff
    -16,         // 0xfffffff0
    -256,        // 0xffffff00
    -4096,       // 0xfffff000
    -65536,      // 0xffff0000
    -1048576,    // 0xfff00000
    -16777216,   // 0xff000000
    -268435456,  // 0xf0000000
];

/// Forward-declared Emulator struct lives in emulator.rs.
/// This file adds the instruction-decode methods via `impl Emulator`.
use crate::emulator::Emulator;

impl Emulator {
    // ----------------------------------------------------------------
    // Conditional-jump helper:  used after a test sets saturn.carry.
    //
    //   pc_at_jump = saturn.pc + base_pc_offset   (points at the 2-nibble
    //                                               relative displacement)
    //   If carry is set we take the jump; otherwise we skip 2 nibbles past
    //   pc_at_jump (total = base_pc_offset + 2 nibbles skipped).
    // ----------------------------------------------------------------
    fn cond_jump(&mut self, base_pc_offset: i32) {
        if self.saturn.carry != 0 {
            self.saturn.pc += base_pc_offset;
            let mut op = self.read_nibbles(self.saturn.pc, 2);
            if op != 0 {
                if op & 0x80 != 0 {
                    op |= JUMPMASKS[2];
                }
                self.saturn.pc = (self.saturn.pc + op) & 0xfffff;
            } else {
                self.saturn.pc = self.saturn.pop_return_addr();
            }
        } else {
            self.saturn.pc += base_pc_offset + 2;
        }
    }

    // ================================================================
    //  decode_group_80  —  opcodes 80x
    // ================================================================
    fn decode_group_80(&mut self) -> bool {
        let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
        match op3 {
            0 => {
                // OUT=CS  (copy C[0] to OUT[0])
                self.saturn.pc += 3;
                self.saturn.out[0] = self.saturn.c[0];
                self.check_out_register();
                false
            }
            1 => {
                // OUT=C  (copy C[0..2] to OUT[0..2])
                self.saturn.pc += 3;
                self.saturn.out[0] = self.saturn.c[0];
                self.saturn.out[1] = self.saturn.c[1];
                self.saturn.out[2] = self.saturn.c[2];
                self.check_out_register();
                false
            }
            2 => {
                // A=IN  (copy IN[0..3] to A[0..3])
                self.saturn.pc += 3;
                self.do_in();
                self.saturn.a[0] = self.saturn.in_reg[0];
                self.saturn.a[1] = self.saturn.in_reg[1];
                self.saturn.a[2] = self.saturn.in_reg[2];
                self.saturn.a[3] = self.saturn.in_reg[3];
                false
            }
            3 => {
                // C=IN  (copy IN[0..3] to C[0..3])
                self.saturn.pc += 3;
                self.do_in();
                self.saturn.c[0] = self.saturn.in_reg[0];
                self.saturn.c[1] = self.saturn.in_reg[1];
                self.saturn.c[2] = self.saturn.in_reg[2];
                self.saturn.c[3] = self.saturn.in_reg[3];
                false
            }
            4 => {
                // UNCNFG
                self.saturn.pc += 3;
                self.do_unconfigure();
                false
            }
            5 => {
                // CONFIG
                self.saturn.pc += 3;
                self.saturn.do_configure();
                false
            }
            6 => {
                // C=ID
                self.saturn.pc += 3;
                self.saturn.get_identification();
                false
            }
            7 => {
                // SHUTDN
                self.saturn.pc += 3;
                let now = self.now;
                self.do_shutdown(now);
                false
            }
            8 => {
                let op4 = self.read_nibble(self.saturn.pc + 3) as i32;
                match op4 {
                    0 => {
                        // INTON
                        self.saturn.pc += 4;
                        self.saturn.do_inton();
                        false
                    }
                    1 => {
                        // RSI
                        let _op5 = self.read_nibble(self.saturn.pc + 4);
                        self.saturn.pc += 5;
                        self.do_reset_interrupt_system();
                        false
                    }
                    2 => {
                        // LA(n)
                        let op5 = self.read_nibble(self.saturn.pc + 4) as i32;
                        self.load_constant(RegId::A, (op5 + 1) as usize, self.saturn.pc + 5);
                        self.saturn.pc += 6 + op5;
                        false
                    }
                    3 => {
                        // BUSCB
                        self.saturn.pc += 4;
                        false
                    }
                    4 => {
                        // ABIT=0
                        let op5 = self.read_nibble(self.saturn.pc + 4) as usize;
                        self.saturn.pc += 5;
                        self.saturn.clear_register_bit(RegId::A, op5);
                        false
                    }
                    5 => {
                        // ABIT=1
                        let op5 = self.read_nibble(self.saturn.pc + 4) as usize;
                        self.saturn.pc += 5;
                        self.saturn.set_register_bit(RegId::A, op5);
                        false
                    }
                    8 => {
                        // CBIT=0
                        let op5 = self.read_nibble(self.saturn.pc + 4) as usize;
                        self.saturn.pc += 5;
                        self.saturn.clear_register_bit(RegId::C, op5);
                        false
                    }
                    9 => {
                        // CBIT=1
                        let op5 = self.read_nibble(self.saturn.pc + 4) as usize;
                        self.saturn.pc += 5;
                        self.saturn.set_register_bit(RegId::C, op5);
                        false
                    }
                    6 | 7 | 0xa | 0xb => {
                        // ?ABIT=0, ?ABIT=1, ?CBIT=0, ?CBIT=1
                        let op5 = self.read_nibble(self.saturn.pc + 4) as usize;
                        let reg = if op4 < 8 { RegId::A } else { RegId::C };
                        let t = if op4 == 6 || op4 == 0xa { false } else { true };
                        self.saturn.carry =
                            if self.saturn.get_register_bit(reg, op5) == t { 1 } else { 0 };
                        self.cond_jump(5);
                        false
                    }
                    0xc => {
                        // PC=(A)
                        let addr = Saturn::dat_to_addr(self.saturn.get_reg(RegId::A));
                        self.saturn.pc = self.read_nibbles(addr, 5);
                        false
                    }
                    0xd => {
                        // BUSCD
                        self.saturn.pc += 4;
                        false
                    }
                    0xe => {
                        // PC=(C)
                        let addr = Saturn::dat_to_addr(self.saturn.get_reg(RegId::C));
                        self.saturn.pc = self.read_nibbles(addr, 5);
                        false
                    }
                    0xf => {
                        // INTOFF
                        self.saturn.pc += 4;
                        self.saturn.do_intoff();
                        false
                    }
                    _ => true, // illegal
                }
            }
            9 => {
                // C+P+1
                self.saturn.pc += 3;
                self.saturn.add_p_plus_one(RegId::C);
                false
            }
            0xa => {
                // RESET
                self.saturn.pc += 3;
                self.do_reset();
                false
            }
            0xb => {
                // BUSCC
                self.saturn.pc += 3;
                false
            }
            0xc => {
                // C=P n
                let op4 = self.read_nibble(self.saturn.pc + 3) as usize;
                self.saturn.pc += 4;
                self.saturn.set_register_nibble(RegId::C, op4, self.saturn.p);
                false
            }
            0xd => {
                // P=C n
                let op4 = self.read_nibble(self.saturn.pc + 3) as usize;
                self.saturn.pc += 4;
                self.saturn.p = self.saturn.get_register_nibble(RegId::C, op4);
                false
            }
            0xe => {
                // SREQ?
                self.saturn.pc += 3;
                self.saturn.c[0] = 0;
                self.saturn.sr = 0;
                false
            }
            0xf => {
                // CPEX n
                let op4 = self.read_nibble(self.saturn.pc + 3) as usize;
                self.saturn.pc += 4;
                let t = self.saturn.get_register_nibble(RegId::C, op4);
                self.saturn.set_register_nibble(RegId::C, op4, self.saturn.p);
                self.saturn.p = t;
                false
            }
            _ => true,
        }
    }

    // ================================================================
    //  decode_group_1  —  opcodes 1xx
    // ================================================================
    fn decode_group_1(&mut self) -> bool {
        let op2 = self.read_nibble(self.saturn.pc + 1) as i32;
        match op2 {
            0 => {
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                match op3 {
                    0 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::R0, RegId::A, W_FIELD); false }
                    1 | 5 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::R1, RegId::A, W_FIELD); false }
                    2 | 6 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::R2, RegId::A, W_FIELD); false }
                    3 | 7 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::R3, RegId::A, W_FIELD); false }
                    4 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::R4, RegId::A, W_FIELD); false }
                    8 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::R0, RegId::C, W_FIELD); false }
                    9 | 0xd => { self.saturn.pc += 3; self.saturn.copy_register(RegId::R1, RegId::C, W_FIELD); false }
                    0xa | 0xe => { self.saturn.pc += 3; self.saturn.copy_register(RegId::R2, RegId::C, W_FIELD); false }
                    0xb | 0xf => { self.saturn.pc += 3; self.saturn.copy_register(RegId::R3, RegId::C, W_FIELD); false }
                    0xc => { self.saturn.pc += 3; self.saturn.copy_register(RegId::R4, RegId::C, W_FIELD); false }
                    _ => true,
                }
            }
            1 => {
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                match op3 {
                    0 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::A, RegId::R0, W_FIELD); false }
                    1 | 5 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::A, RegId::R1, W_FIELD); false }
                    2 | 6 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::A, RegId::R2, W_FIELD); false }
                    3 | 7 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::A, RegId::R3, W_FIELD); false }
                    4 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::A, RegId::R4, W_FIELD); false }
                    8 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::C, RegId::R0, W_FIELD); false }
                    9 | 0xd => { self.saturn.pc += 3; self.saturn.copy_register(RegId::C, RegId::R1, W_FIELD); false }
                    0xa | 0xe => { self.saturn.pc += 3; self.saturn.copy_register(RegId::C, RegId::R2, W_FIELD); false }
                    0xb | 0xf => { self.saturn.pc += 3; self.saturn.copy_register(RegId::C, RegId::R3, W_FIELD); false }
                    0xc => { self.saturn.pc += 3; self.saturn.copy_register(RegId::C, RegId::R4, W_FIELD); false }
                    _ => true,
                }
            }
            2 => {
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                match op3 {
                    0 => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::A, RegId::R0, W_FIELD); false }
                    1 | 5 => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::A, RegId::R1, W_FIELD); false }
                    2 | 6 => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::A, RegId::R2, W_FIELD); false }
                    3 | 7 => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::A, RegId::R3, W_FIELD); false }
                    4 => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::A, RegId::R4, W_FIELD); false }
                    8 => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::C, RegId::R0, W_FIELD); false }
                    9 | 0xd => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::C, RegId::R1, W_FIELD); false }
                    0xa | 0xe => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::C, RegId::R2, W_FIELD); false }
                    0xb | 0xf => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::C, RegId::R3, W_FIELD); false }
                    0xc => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::C, RegId::R4, W_FIELD); false }
                    _ => true,
                }
            }
            3 => {
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                match op3 {
                    0 => {
                        // D0=A
                        self.saturn.pc += 3;
                        self.saturn.register_to_address(RegId::A, 0, false);
                        false
                    }
                    1 => {
                        // D1=A
                        self.saturn.pc += 3;
                        self.saturn.register_to_address(RegId::A, 1, false);
                        false
                    }
                    2 => {
                        // AD0EX
                        self.saturn.pc += 3;
                        self.saturn.exchange_reg_dat(RegId::A, 0, A_FIELD);
                        false
                    }
                    3 => {
                        // AD1EX
                        self.saturn.pc += 3;
                        self.saturn.exchange_reg_dat(RegId::A, 1, A_FIELD);
                        false
                    }
                    4 => {
                        // D0=C
                        self.saturn.pc += 3;
                        self.saturn.register_to_address(RegId::C, 0, false);
                        false
                    }
                    5 => {
                        // D1=C
                        self.saturn.pc += 3;
                        self.saturn.register_to_address(RegId::C, 1, false);
                        false
                    }
                    6 => {
                        // CD0EX
                        self.saturn.pc += 3;
                        self.saturn.exchange_reg_dat(RegId::C, 0, A_FIELD);
                        false
                    }
                    7 => {
                        // CD1EX
                        self.saturn.pc += 3;
                        self.saturn.exchange_reg_dat(RegId::C, 1, A_FIELD);
                        false
                    }
                    8 => {
                        // D0=AS
                        self.saturn.pc += 3;
                        self.saturn.register_to_address(RegId::A, 0, true);
                        false
                    }
                    9 => {
                        // D1=AS
                        self.saturn.pc += 3;
                        self.saturn.register_to_address(RegId::A, 1, true);
                        false
                    }
                    0xa => {
                        // AD0XS
                        self.saturn.pc += 3;
                        self.saturn.exchange_reg_dat(RegId::A, 0, IN_FIELD);
                        false
                    }
                    0xb => {
                        // AD1XS
                        self.saturn.pc += 3;
                        self.saturn.exchange_reg_dat(RegId::A, 1, IN_FIELD);
                        false
                    }
                    0xc => {
                        // D0=CS
                        self.saturn.pc += 3;
                        self.saturn.register_to_address(RegId::C, 0, true);
                        false
                    }
                    0xd => {
                        // D1=CS
                        self.saturn.pc += 3;
                        self.saturn.register_to_address(RegId::C, 1, true);
                        false
                    }
                    0xe => {
                        // CD0XS
                        self.saturn.pc += 3;
                        self.saturn.exchange_reg_dat(RegId::C, 0, IN_FIELD);
                        false
                    }
                    0xf => {
                        // CD1XS
                        self.saturn.pc += 3;
                        self.saturn.exchange_reg_dat(RegId::C, 1, IN_FIELD);
                        false
                    }
                    _ => true,
                }
            }
            4 => {
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                // op3 < 8: field = W (0xf), op3 >= 8: field = B (6)
                let code = if op3 < 8 { 0xf_u8 } else { 6_u8 };
                match op3 & 7 {
                    0 => { self.saturn.pc += 3; self.store(RegId::A, code, 0); false }
                    1 => { self.saturn.pc += 3; self.store(RegId::A, code, 1); false }
                    2 => { self.saturn.pc += 3; self.recall(RegId::A, code, 0); false }
                    3 => { self.saturn.pc += 3; self.recall(RegId::A, code, 1); false }
                    4 => { self.saturn.pc += 3; self.store(RegId::C, code, 0); false }
                    5 => { self.saturn.pc += 3; self.store(RegId::C, code, 1); false }
                    6 => { self.saturn.pc += 3; self.recall(RegId::C, code, 0); false }
                    7 => { self.saturn.pc += 3; self.recall(RegId::C, code, 1); false }
                    _ => true,
                }
            }
            5 => {
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                let op4 = self.read_nibble(self.saturn.pc + 3) as i32;
                if op3 >= 8 {
                    // n-nibble DAT operations
                    let n = (op4 + 1) as usize;
                    match op3 & 7 {
                        0 => { self.saturn.pc += 4; self.store_n(RegId::A, n, 0); false }
                        1 => { self.saturn.pc += 4; self.store_n(RegId::A, n, 1); false }
                        2 => { self.saturn.pc += 4; self.recall_n(RegId::A, n, 0); false }
                        3 => { self.saturn.pc += 4; self.recall_n(RegId::A, n, 1); false }
                        4 => { self.saturn.pc += 4; self.store_n(RegId::C, n, 0); false }
                        5 => { self.saturn.pc += 4; self.store_n(RegId::C, n, 1); false }
                        6 => { self.saturn.pc += 4; self.recall_n(RegId::C, n, 0); false }
                        7 => { self.saturn.pc += 4; self.recall_n(RegId::C, n, 1); false }
                        _ => true,
                    }
                } else {
                    // Field-based DAT operations
                    let code = op4 as u8;
                    match op3 {
                        0 => { self.saturn.pc += 4; self.store(RegId::A, code, 0); false }
                        1 => { self.saturn.pc += 4; self.store(RegId::A, code, 1); false }
                        2 => { self.saturn.pc += 4; self.recall(RegId::A, code, 0); false }
                        3 => { self.saturn.pc += 4; self.recall(RegId::A, code, 1); false }
                        4 => { self.saturn.pc += 4; self.store(RegId::C, code, 0); false }
                        5 => { self.saturn.pc += 4; self.store(RegId::C, code, 1); false }
                        6 => { self.saturn.pc += 4; self.recall(RegId::C, code, 0); false }
                        7 => { self.saturn.pc += 4; self.recall(RegId::C, code, 1); false }
                        _ => true,
                    }
                }
            }
            6 => {
                // D0=D0+ (n+1)
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                self.saturn.pc += 3;
                self.saturn.add_address(0, op3 + 1);
                false
            }
            7 => {
                // D1=D1+ (n+1)
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                self.saturn.pc += 3;
                self.saturn.add_address(1, op3 + 1);
                false
            }
            8 => {
                // D0=D0- (n+1)
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                self.saturn.pc += 3;
                self.saturn.add_address(0, -(op3 + 1));
                false
            }
            9 => {
                // D0=(2) addr
                let pc = self.saturn.pc;
                self.load_addr(0, pc + 2, 2);
                self.saturn.pc += 4;
                false
            }
            0xa => {
                // D0=(4) addr
                let pc = self.saturn.pc;
                self.load_addr(0, pc + 2, 4);
                self.saturn.pc += 6;
                false
            }
            0xb => {
                // D0=(5) addr
                let pc = self.saturn.pc;
                self.load_addr(0, pc + 2, 5);
                self.saturn.pc += 7;
                false
            }
            0xc => {
                // D1=D1- (n+1)
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                self.saturn.pc += 3;
                self.saturn.add_address(1, -(op3 + 1));
                false
            }
            0xd => {
                // D1=(2) addr
                let pc = self.saturn.pc;
                self.load_addr(1, pc + 2, 2);
                self.saturn.pc += 4;
                false
            }
            0xe => {
                // D1=(4) addr
                let pc = self.saturn.pc;
                self.load_addr(1, pc + 2, 4);
                self.saturn.pc += 6;
                false
            }
            0xf => {
                // D1=(5) addr
                let pc = self.saturn.pc;
                self.load_addr(1, pc + 2, 5);
                self.saturn.pc += 7;
                false
            }
            _ => true,
        }
    }

    // ================================================================
    //  decode_8_thru_f  —  first nibble 8..F
    // ================================================================
    fn decode_8_thru_f(&mut self, op1: i32) -> bool {
        let op2 = self.read_nibble(self.saturn.pc + 1) as i32;
        match op1 {
            // ----------------------------------------------------------
            // 8xxx — many sub-groups
            // ----------------------------------------------------------
            8 => {
                match op2 {
                    0 => self.decode_group_80(),
                    1 => {
                        let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                        match op3 {
                            0 => { self.saturn.pc += 3; self.saturn.shift_left_circ_register(RegId::A, W_FIELD); false }
                            1 => { self.saturn.pc += 3; self.saturn.shift_left_circ_register(RegId::B, W_FIELD); false }
                            2 => { self.saturn.pc += 3; self.saturn.shift_left_circ_register(RegId::C, W_FIELD); false }
                            3 => { self.saturn.pc += 3; self.saturn.shift_left_circ_register(RegId::D, W_FIELD); false }
                            4 => { self.saturn.pc += 3; self.saturn.shift_right_circ_register(RegId::A, W_FIELD); false }
                            5 => { self.saturn.pc += 3; self.saturn.shift_right_circ_register(RegId::B, W_FIELD); false }
                            6 => { self.saturn.pc += 3; self.saturn.shift_right_circ_register(RegId::C, W_FIELD); false }
                            7 => { self.saturn.pc += 3; self.saturn.shift_right_circ_register(RegId::D, W_FIELD); false }
                            8 => {
                                // R = R +/- CON
                                let op4 = self.read_nibble(self.saturn.pc + 3) as u8;
                                let op5 = self.read_nibble(self.saturn.pc + 4) as i32;
                                let op6 = self.read_nibble(self.saturn.pc + 5) as i32;
                                if op5 < 8 {
                                    // PLUS
                                    match op5 & 3 {
                                        0 => { self.saturn.pc += 6; self.saturn.add_register_constant(RegId::A, op4, op6 + 1); false }
                                        1 => { self.saturn.pc += 6; self.saturn.add_register_constant(RegId::B, op4, op6 + 1); false }
                                        2 => { self.saturn.pc += 6; self.saturn.add_register_constant(RegId::C, op4, op6 + 1); false }
                                        3 => { self.saturn.pc += 6; self.saturn.add_register_constant(RegId::D, op4, op6 + 1); false }
                                        _ => true,
                                    }
                                } else {
                                    // MINUS
                                    match op5 & 3 {
                                        0 => { self.saturn.pc += 6; self.saturn.sub_register_constant(RegId::A, op4, op6 + 1); false }
                                        1 => { self.saturn.pc += 6; self.saturn.sub_register_constant(RegId::B, op4, op6 + 1); false }
                                        2 => { self.saturn.pc += 6; self.saturn.sub_register_constant(RegId::C, op4, op6 + 1); false }
                                        3 => { self.saturn.pc += 6; self.saturn.sub_register_constant(RegId::D, op4, op6 + 1); false }
                                        _ => true,
                                    }
                                }
                            }
                            9 => {
                                // R SRB FIELD
                                let op4 = self.read_nibble(self.saturn.pc + 3) as u8;
                                let op5 = self.read_nibble(self.saturn.pc + 4) as i32;
                                match op5 & 3 {
                                    0 => { self.saturn.pc += 5; self.saturn.shift_right_bit_register(RegId::A, op4); false }
                                    1 => { self.saturn.pc += 5; self.saturn.shift_right_bit_register(RegId::B, op4); false }
                                    2 => { self.saturn.pc += 5; self.saturn.shift_right_bit_register(RegId::C, op4); false }
                                    3 => { self.saturn.pc += 5; self.saturn.shift_right_bit_register(RegId::D, op4); false }
                                    _ => true,
                                }
                            }
                            0xa => {
                                // R = R FIELD, etc.
                                let op4 = self.read_nibble(self.saturn.pc + 3) as u8;
                                let op5 = self.read_nibble(self.saturn.pc + 4) as i32;
                                let op6 = self.read_nibble(self.saturn.pc + 5) as i32;
                                match op5 {
                                    0 => {
                                        // Rn=A/C (field)
                                        match op6 {
                                            0 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::R0, RegId::A, op4); false }
                                            1 | 5 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::R1, RegId::A, op4); false }
                                            2 | 6 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::R2, RegId::A, op4); false }
                                            3 | 7 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::R3, RegId::A, op4); false }
                                            4 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::R4, RegId::A, op4); false }
                                            8 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::R0, RegId::C, op4); false }
                                            9 | 0xd => { self.saturn.pc += 6; self.saturn.copy_register(RegId::R1, RegId::C, op4); false }
                                            0xa | 0xe => { self.saturn.pc += 6; self.saturn.copy_register(RegId::R2, RegId::C, op4); false }
                                            0xb | 0xf => { self.saturn.pc += 6; self.saturn.copy_register(RegId::R3, RegId::C, op4); false }
                                            0xc => { self.saturn.pc += 6; self.saturn.copy_register(RegId::R4, RegId::C, op4); false }
                                            _ => true,
                                        }
                                    }
                                    1 => {
                                        // A/C=Rn (field)
                                        match op6 {
                                            0 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::A, RegId::R0, op4); false }
                                            1 | 5 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::A, RegId::R1, op4); false }
                                            2 | 6 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::A, RegId::R2, op4); false }
                                            3 | 7 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::A, RegId::R3, op4); false }
                                            4 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::A, RegId::R4, op4); false }
                                            8 => { self.saturn.pc += 6; self.saturn.copy_register(RegId::C, RegId::R0, op4); false }
                                            9 | 0xd => { self.saturn.pc += 6; self.saturn.copy_register(RegId::C, RegId::R1, op4); false }
                                            0xa | 0xe => { self.saturn.pc += 6; self.saturn.copy_register(RegId::C, RegId::R2, op4); false }
                                            0xb | 0xf => { self.saturn.pc += 6; self.saturn.copy_register(RegId::C, RegId::R3, op4); false }
                                            0xc => { self.saturn.pc += 6; self.saturn.copy_register(RegId::C, RegId::R4, op4); false }
                                            _ => true,
                                        }
                                    }
                                    2 => {
                                        // AR/CR exchange (field)
                                        match op6 {
                                            0 => { self.saturn.pc += 6; self.saturn.exchange_register(RegId::A, RegId::R0, op4); false }
                                            1 | 5 => { self.saturn.pc += 6; self.saturn.exchange_register(RegId::A, RegId::R1, op4); false }
                                            2 | 6 => { self.saturn.pc += 6; self.saturn.exchange_register(RegId::A, RegId::R2, op4); false }
                                            3 | 7 => { self.saturn.pc += 6; self.saturn.exchange_register(RegId::A, RegId::R3, op4); false }
                                            4 => { self.saturn.pc += 6; self.saturn.exchange_register(RegId::A, RegId::R4, op4); false }
                                            8 => { self.saturn.pc += 6; self.saturn.exchange_register(RegId::C, RegId::R0, op4); false }
                                            9 | 0xd => { self.saturn.pc += 6; self.saturn.exchange_register(RegId::C, RegId::R1, op4); false }
                                            0xa | 0xe => { self.saturn.pc += 6; self.saturn.exchange_register(RegId::C, RegId::R2, op4); false }
                                            0xb | 0xf => { self.saturn.pc += 6; self.saturn.exchange_register(RegId::C, RegId::R3, op4); false }
                                            0xc => { self.saturn.pc += 6; self.saturn.exchange_register(RegId::C, RegId::R4, op4); false }
                                            _ => true,
                                        }
                                    }
                                    _ => true,
                                }
                            }
                            0xb => {
                                let op4 = self.read_nibble(self.saturn.pc + 3) as i32;
                                match op4 {
                                    2 => {
                                        // PC=A
                                        let jumpaddr = Saturn::dat_to_addr(self.saturn.get_reg(RegId::A));
                                        self.saturn.pc = jumpaddr;
                                        false
                                    }
                                    3 => {
                                        // PC=C
                                        let jumpaddr = Saturn::dat_to_addr(self.saturn.get_reg(RegId::C));
                                        self.saturn.pc = jumpaddr;
                                        false
                                    }
                                    4 => {
                                        // A=PC
                                        self.saturn.pc += 4;
                                        let pc = self.saturn.pc;
                                        Saturn::addr_to_dat(pc, &mut self.saturn.a);
                                        false
                                    }
                                    5 => {
                                        // C=PC
                                        self.saturn.pc += 4;
                                        let pc = self.saturn.pc;
                                        Saturn::addr_to_dat(pc, &mut self.saturn.c);
                                        false
                                    }
                                    6 => {
                                        // APCEX
                                        self.saturn.pc += 4;
                                        let jumpaddr = Saturn::dat_to_addr(&self.saturn.a);
                                        let pc = self.saturn.pc;
                                        Saturn::addr_to_dat(pc, &mut self.saturn.a);
                                        self.saturn.pc = jumpaddr;
                                        false
                                    }
                                    7 => {
                                        // CPCEX
                                        self.saturn.pc += 4;
                                        let jumpaddr = Saturn::dat_to_addr(&self.saturn.c);
                                        let pc = self.saturn.pc;
                                        Saturn::addr_to_dat(pc, &mut self.saturn.c);
                                        self.saturn.pc = jumpaddr;
                                        false
                                    }
                                    _ => true,
                                }
                            }
                            0xc => { self.saturn.pc += 3; self.saturn.shift_right_bit_register(RegId::A, W_FIELD); false }
                            0xd => { self.saturn.pc += 3; self.saturn.shift_right_bit_register(RegId::B, W_FIELD); false }
                            0xe => { self.saturn.pc += 3; self.saturn.shift_right_bit_register(RegId::C, W_FIELD); false }
                            0xf => { self.saturn.pc += 3; self.saturn.shift_right_bit_register(RegId::D, W_FIELD); false }
                            _ => true,
                        }
                    }
                    2 => {
                        // CLRHST
                        let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                        self.saturn.pc += 3;
                        self.saturn.clear_hardware_stat(op3);
                        false
                    }
                    3 => {
                        // ?HSTBIT=0
                        let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                        self.saturn.carry = if self.saturn.is_zero_hardware_stat(op3) { 1 } else { 0 };
                        self.cond_jump(3);
                        false
                    }
                    4 => {
                        // CLRST n
                        let op3 = self.read_nibble(self.saturn.pc + 2) as usize;
                        self.saturn.pc += 3;
                        self.saturn.clear_program_stat(op3);
                        false
                    }
                    5 => {
                        // SETST n
                        let op3 = self.read_nibble(self.saturn.pc + 2) as usize;
                        self.saturn.pc += 3;
                        self.saturn.set_program_stat(op3);
                        false
                    }
                    6 => {
                        // ?ST=0 n
                        let op3 = self.read_nibble(self.saturn.pc + 2) as usize;
                        self.saturn.carry = if !self.saturn.get_program_stat(op3) { 1 } else { 0 };
                        self.cond_jump(3);
                        false
                    }
                    7 => {
                        // ?ST=1 n
                        let op3 = self.read_nibble(self.saturn.pc + 2) as usize;
                        self.saturn.carry = if self.saturn.get_program_stat(op3) { 1 } else { 0 };
                        self.cond_jump(3);
                        false
                    }
                    8 => {
                        // ?P#n
                        let op3 = self.read_nibble(self.saturn.pc + 2);
                        self.saturn.carry = if self.saturn.p != op3 { 1 } else { 0 };
                        self.cond_jump(3);
                        false
                    }
                    9 => {
                        // ?P=n
                        let op3 = self.read_nibble(self.saturn.pc + 2);
                        self.saturn.carry = if self.saturn.p == op3 { 1 } else { 0 };
                        self.cond_jump(3);
                        false
                    }
                    0xa => {
                        // Test group A (equality/zero)
                        let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                        match op3 {
                            0 => { self.saturn.carry = if self.saturn.is_equal_register(RegId::A, RegId::B, A_FIELD) { 1 } else { 0 }; }
                            1 => { self.saturn.carry = if self.saturn.is_equal_register(RegId::B, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            2 => { self.saturn.carry = if self.saturn.is_equal_register(RegId::A, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            3 => { self.saturn.carry = if self.saturn.is_equal_register(RegId::C, RegId::D, A_FIELD) { 1 } else { 0 }; }
                            4 => { self.saturn.carry = if self.saturn.is_not_equal_register(RegId::A, RegId::B, A_FIELD) { 1 } else { 0 }; }
                            5 => { self.saturn.carry = if self.saturn.is_not_equal_register(RegId::B, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            6 => { self.saturn.carry = if self.saturn.is_not_equal_register(RegId::A, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            7 => { self.saturn.carry = if self.saturn.is_not_equal_register(RegId::C, RegId::D, A_FIELD) { 1 } else { 0 }; }
                            8 => { self.saturn.carry = if self.saturn.is_zero_register(RegId::A, A_FIELD) { 1 } else { 0 }; }
                            9 => { self.saturn.carry = if self.saturn.is_zero_register(RegId::B, A_FIELD) { 1 } else { 0 }; }
                            0xa => { self.saturn.carry = if self.saturn.is_zero_register(RegId::C, A_FIELD) { 1 } else { 0 }; }
                            0xb => { self.saturn.carry = if self.saturn.is_zero_register(RegId::D, A_FIELD) { 1 } else { 0 }; }
                            0xc => { self.saturn.carry = if self.saturn.is_not_zero_register(RegId::A, A_FIELD) { 1 } else { 0 }; }
                            0xd => { self.saturn.carry = if self.saturn.is_not_zero_register(RegId::B, A_FIELD) { 1 } else { 0 }; }
                            0xe => { self.saturn.carry = if self.saturn.is_not_zero_register(RegId::C, A_FIELD) { 1 } else { 0 }; }
                            0xf => { self.saturn.carry = if self.saturn.is_not_zero_register(RegId::D, A_FIELD) { 1 } else { 0 }; }
                            _ => { return true; }
                        }
                        self.cond_jump(3);
                        false
                    }
                    0xb => {
                        // Test group B (comparison)
                        let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                        match op3 {
                            0 => { self.saturn.carry = if self.saturn.is_greater_register(RegId::A, RegId::B, A_FIELD) { 1 } else { 0 }; }
                            1 => { self.saturn.carry = if self.saturn.is_greater_register(RegId::B, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            2 => { self.saturn.carry = if self.saturn.is_greater_register(RegId::C, RegId::A, A_FIELD) { 1 } else { 0 }; }
                            3 => { self.saturn.carry = if self.saturn.is_greater_register(RegId::D, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            4 => { self.saturn.carry = if self.saturn.is_less_register(RegId::A, RegId::B, A_FIELD) { 1 } else { 0 }; }
                            5 => { self.saturn.carry = if self.saturn.is_less_register(RegId::B, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            6 => { self.saturn.carry = if self.saturn.is_less_register(RegId::C, RegId::A, A_FIELD) { 1 } else { 0 }; }
                            7 => { self.saturn.carry = if self.saturn.is_less_register(RegId::D, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            8 => { self.saturn.carry = if self.saturn.is_greater_or_equal_register(RegId::A, RegId::B, A_FIELD) { 1 } else { 0 }; }
                            9 => { self.saturn.carry = if self.saturn.is_greater_or_equal_register(RegId::B, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            0xa => { self.saturn.carry = if self.saturn.is_greater_or_equal_register(RegId::C, RegId::A, A_FIELD) { 1 } else { 0 }; }
                            0xb => { self.saturn.carry = if self.saturn.is_greater_or_equal_register(RegId::D, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            0xc => { self.saturn.carry = if self.saturn.is_less_or_equal_register(RegId::A, RegId::B, A_FIELD) { 1 } else { 0 }; }
                            0xd => { self.saturn.carry = if self.saturn.is_less_or_equal_register(RegId::B, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            0xe => { self.saturn.carry = if self.saturn.is_less_or_equal_register(RegId::C, RegId::A, A_FIELD) { 1 } else { 0 }; }
                            0xf => { self.saturn.carry = if self.saturn.is_less_or_equal_register(RegId::D, RegId::C, A_FIELD) { 1 } else { 0 }; }
                            _ => { return true; }
                        }
                        self.cond_jump(3);
                        false
                    }
                    0xc => {
                        // GOTO (4-nibble relative)
                        let mut op3 = self.read_nibbles(self.saturn.pc + 2, 4);
                        if op3 & 0x8000 != 0 {
                            op3 |= JUMPMASKS[4];
                        }
                        self.saturn.pc = (self.saturn.pc + op3 + 2) & 0xfffff;
                        false
                    }
                    0xd => {
                        // GOTO (5-nibble absolute)
                        let op3 = self.read_nibbles(self.saturn.pc + 2, 5);
                        self.saturn.pc = op3;
                        false
                    }
                    0xe => {
                        // GOSUB (4-nibble relative)
                        let mut op3 = self.read_nibbles(self.saturn.pc + 2, 4);
                        if op3 & 0x8000 != 0 {
                            op3 |= JUMPMASKS[4];
                        }
                        let jumpaddr = (self.saturn.pc + op3 + 6) & 0xfffff;
                        self.saturn.push_return_addr(self.saturn.pc + 6);
                        self.saturn.pc = jumpaddr;
                        false
                    }
                    0xf => {
                        // GOSUB (5-nibble absolute)
                        let op3 = self.read_nibbles(self.saturn.pc + 2, 5);
                        self.saturn.push_return_addr(self.saturn.pc + 7);
                        self.saturn.pc = op3;
                        false
                    }
                    _ => true,
                }
            }

            // ----------------------------------------------------------
            // 9xxx — register tests with field selector
            // ----------------------------------------------------------
            9 => {
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                if op2 < 8 {
                    let code = op2 as u8;
                    match op3 {
                        0 => { self.saturn.carry = if self.saturn.is_equal_register(RegId::A, RegId::B, code) { 1 } else { 0 }; }
                        1 => { self.saturn.carry = if self.saturn.is_equal_register(RegId::B, RegId::C, code) { 1 } else { 0 }; }
                        2 => { self.saturn.carry = if self.saturn.is_equal_register(RegId::A, RegId::C, code) { 1 } else { 0 }; }
                        3 => { self.saturn.carry = if self.saturn.is_equal_register(RegId::C, RegId::D, code) { 1 } else { 0 }; }
                        4 => { self.saturn.carry = if self.saturn.is_not_equal_register(RegId::A, RegId::B, code) { 1 } else { 0 }; }
                        5 => { self.saturn.carry = if self.saturn.is_not_equal_register(RegId::B, RegId::C, code) { 1 } else { 0 }; }
                        6 => { self.saturn.carry = if self.saturn.is_not_equal_register(RegId::A, RegId::C, code) { 1 } else { 0 }; }
                        7 => { self.saturn.carry = if self.saturn.is_not_equal_register(RegId::C, RegId::D, code) { 1 } else { 0 }; }
                        8 => { self.saturn.carry = if self.saturn.is_zero_register(RegId::A, code) { 1 } else { 0 }; }
                        9 => { self.saturn.carry = if self.saturn.is_zero_register(RegId::B, code) { 1 } else { 0 }; }
                        0xa => { self.saturn.carry = if self.saturn.is_zero_register(RegId::C, code) { 1 } else { 0 }; }
                        0xb => { self.saturn.carry = if self.saturn.is_zero_register(RegId::D, code) { 1 } else { 0 }; }
                        0xc => { self.saturn.carry = if self.saturn.is_not_zero_register(RegId::A, code) { 1 } else { 0 }; }
                        0xd => { self.saturn.carry = if self.saturn.is_not_zero_register(RegId::B, code) { 1 } else { 0 }; }
                        0xe => { self.saturn.carry = if self.saturn.is_not_zero_register(RegId::C, code) { 1 } else { 0 }; }
                        0xf => { self.saturn.carry = if self.saturn.is_not_zero_register(RegId::D, code) { 1 } else { 0 }; }
                        _ => { return true; }
                    }
                } else {
                    let code = (op2 & 7) as u8;
                    match op3 {
                        0 => { self.saturn.carry = if self.saturn.is_greater_register(RegId::A, RegId::B, code) { 1 } else { 0 }; }
                        1 => { self.saturn.carry = if self.saturn.is_greater_register(RegId::B, RegId::C, code) { 1 } else { 0 }; }
                        2 => { self.saturn.carry = if self.saturn.is_greater_register(RegId::C, RegId::A, code) { 1 } else { 0 }; }
                        3 => { self.saturn.carry = if self.saturn.is_greater_register(RegId::D, RegId::C, code) { 1 } else { 0 }; }
                        4 => { self.saturn.carry = if self.saturn.is_less_register(RegId::A, RegId::B, code) { 1 } else { 0 }; }
                        5 => { self.saturn.carry = if self.saturn.is_less_register(RegId::B, RegId::C, code) { 1 } else { 0 }; }
                        6 => { self.saturn.carry = if self.saturn.is_less_register(RegId::C, RegId::A, code) { 1 } else { 0 }; }
                        7 => { self.saturn.carry = if self.saturn.is_less_register(RegId::D, RegId::C, code) { 1 } else { 0 }; }
                        8 => { self.saturn.carry = if self.saturn.is_greater_or_equal_register(RegId::A, RegId::B, code) { 1 } else { 0 }; }
                        9 => { self.saturn.carry = if self.saturn.is_greater_or_equal_register(RegId::B, RegId::C, code) { 1 } else { 0 }; }
                        0xa => { self.saturn.carry = if self.saturn.is_greater_or_equal_register(RegId::C, RegId::A, code) { 1 } else { 0 }; }
                        0xb => { self.saturn.carry = if self.saturn.is_greater_or_equal_register(RegId::D, RegId::C, code) { 1 } else { 0 }; }
                        0xc => { self.saturn.carry = if self.saturn.is_less_or_equal_register(RegId::A, RegId::B, code) { 1 } else { 0 }; }
                        0xd => { self.saturn.carry = if self.saturn.is_less_or_equal_register(RegId::B, RegId::C, code) { 1 } else { 0 }; }
                        0xe => { self.saturn.carry = if self.saturn.is_less_or_equal_register(RegId::C, RegId::A, code) { 1 } else { 0 }; }
                        0xf => { self.saturn.carry = if self.saturn.is_less_or_equal_register(RegId::D, RegId::C, code) { 1 } else { 0 }; }
                        _ => { return true; }
                    }
                }
                self.cond_jump(3);
                false
            }

            // ----------------------------------------------------------
            // Axxx — add / dec with field selector
            // ----------------------------------------------------------
            0xa => {
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                if op2 < 8 {
                    let code = op2 as u8;
                    match op3 {
                        0 => { self.saturn.pc += 3; self.saturn.add_register(RegId::A, RegId::A, RegId::B, code); false }
                        1 => { self.saturn.pc += 3; self.saturn.add_register(RegId::B, RegId::B, RegId::C, code); false }
                        2 => { self.saturn.pc += 3; self.saturn.add_register(RegId::C, RegId::C, RegId::A, code); false }
                        3 => { self.saturn.pc += 3; self.saturn.add_register(RegId::D, RegId::D, RegId::C, code); false }
                        4 => { self.saturn.pc += 3; self.saturn.add_register(RegId::A, RegId::A, RegId::A, code); false }
                        5 => { self.saturn.pc += 3; self.saturn.add_register(RegId::B, RegId::B, RegId::B, code); false }
                        6 => { self.saturn.pc += 3; self.saturn.add_register(RegId::C, RegId::C, RegId::C, code); false }
                        7 => { self.saturn.pc += 3; self.saturn.add_register(RegId::D, RegId::D, RegId::D, code); false }
                        8 => { self.saturn.pc += 3; self.saturn.add_register(RegId::B, RegId::B, RegId::A, code); false }
                        9 => { self.saturn.pc += 3; self.saturn.add_register(RegId::C, RegId::C, RegId::B, code); false }
                        0xa => { self.saturn.pc += 3; self.saturn.add_register(RegId::A, RegId::A, RegId::C, code); false }
                        0xb => { self.saturn.pc += 3; self.saturn.add_register(RegId::C, RegId::C, RegId::D, code); false }
                        0xc => { self.saturn.pc += 3; self.saturn.dec_register(RegId::A, code); false }
                        0xd => { self.saturn.pc += 3; self.saturn.dec_register(RegId::B, code); false }
                        0xe => { self.saturn.pc += 3; self.saturn.dec_register(RegId::C, code); false }
                        0xf => { self.saturn.pc += 3; self.saturn.dec_register(RegId::D, code); false }
                        _ => true,
                    }
                } else {
                    let code = (op2 & 7) as u8;
                    match op3 {
                        0 => { self.saturn.pc += 3; self.saturn.zero_register(RegId::A, code); false }
                        1 => { self.saturn.pc += 3; self.saturn.zero_register(RegId::B, code); false }
                        2 => { self.saturn.pc += 3; self.saturn.zero_register(RegId::C, code); false }
                        3 => { self.saturn.pc += 3; self.saturn.zero_register(RegId::D, code); false }
                        4 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::A, RegId::B, code); false }
                        5 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::B, RegId::C, code); false }
                        6 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::C, RegId::A, code); false }
                        7 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::D, RegId::C, code); false }
                        8 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::B, RegId::A, code); false }
                        9 => { self.saturn.pc += 3; self.saturn.copy_register(RegId::C, RegId::B, code); false }
                        0xa => { self.saturn.pc += 3; self.saturn.copy_register(RegId::A, RegId::C, code); false }
                        0xb => { self.saturn.pc += 3; self.saturn.copy_register(RegId::C, RegId::D, code); false }
                        0xc => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::A, RegId::B, code); false }
                        0xd => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::B, RegId::C, code); false }
                        0xe => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::A, RegId::C, code); false }
                        0xf => { self.saturn.pc += 3; self.saturn.exchange_register(RegId::C, RegId::D, code); false }
                        _ => true,
                    }
                }
            }

            // ----------------------------------------------------------
            // Bxxx — sub / inc / shift with field selector
            // ----------------------------------------------------------
            0xb => {
                let op3 = self.read_nibble(self.saturn.pc + 2) as i32;
                if op2 < 8 {
                    let code = op2 as u8;
                    match op3 {
                        0 => { self.saturn.pc += 3; self.saturn.sub_register(RegId::A, RegId::A, RegId::B, code); false }
                        1 => { self.saturn.pc += 3; self.saturn.sub_register(RegId::B, RegId::B, RegId::C, code); false }
                        2 => { self.saturn.pc += 3; self.saturn.sub_register(RegId::C, RegId::C, RegId::A, code); false }
                        3 => { self.saturn.pc += 3; self.saturn.sub_register(RegId::D, RegId::D, RegId::C, code); false }
                        4 => { self.saturn.pc += 3; self.saturn.inc_register(RegId::A, code); false }
                        5 => { self.saturn.pc += 3; self.saturn.inc_register(RegId::B, code); false }
                        6 => { self.saturn.pc += 3; self.saturn.inc_register(RegId::C, code); false }
                        7 => { self.saturn.pc += 3; self.saturn.inc_register(RegId::D, code); false }
                        8 => { self.saturn.pc += 3; self.saturn.sub_register(RegId::B, RegId::B, RegId::A, code); false }
                        9 => { self.saturn.pc += 3; self.saturn.sub_register(RegId::C, RegId::C, RegId::B, code); false }
                        0xa => { self.saturn.pc += 3; self.saturn.sub_register(RegId::A, RegId::A, RegId::C, code); false }
                        0xb => { self.saturn.pc += 3; self.saturn.sub_register(RegId::C, RegId::C, RegId::D, code); false }
                        0xc => { self.saturn.pc += 3; self.saturn.sub_register(RegId::A, RegId::B, RegId::A, code); false }
                        0xd => { self.saturn.pc += 3; self.saturn.sub_register(RegId::B, RegId::C, RegId::B, code); false }
                        0xe => { self.saturn.pc += 3; self.saturn.sub_register(RegId::C, RegId::A, RegId::C, code); false }
                        0xf => { self.saturn.pc += 3; self.saturn.sub_register(RegId::D, RegId::C, RegId::D, code); false }
                        _ => true,
                    }
                } else {
                    let code = (op2 & 7) as u8;
                    match op3 {
                        0 => { self.saturn.pc += 3; self.saturn.shift_left_register(RegId::A, code); false }
                        1 => { self.saturn.pc += 3; self.saturn.shift_left_register(RegId::B, code); false }
                        2 => { self.saturn.pc += 3; self.saturn.shift_left_register(RegId::C, code); false }
                        3 => { self.saturn.pc += 3; self.saturn.shift_left_register(RegId::D, code); false }
                        4 => { self.saturn.pc += 3; self.saturn.shift_right_register(RegId::A, code); false }
                        5 => { self.saturn.pc += 3; self.saturn.shift_right_register(RegId::B, code); false }
                        6 => { self.saturn.pc += 3; self.saturn.shift_right_register(RegId::C, code); false }
                        7 => { self.saturn.pc += 3; self.saturn.shift_right_register(RegId::D, code); false }
                        8 => { self.saturn.pc += 3; self.saturn.complement_2_register(RegId::A, code); false }
                        9 => { self.saturn.pc += 3; self.saturn.complement_2_register(RegId::B, code); false }
                        0xa => { self.saturn.pc += 3; self.saturn.complement_2_register(RegId::C, code); false }
                        0xb => { self.saturn.pc += 3; self.saturn.complement_2_register(RegId::D, code); false }
                        0xc => { self.saturn.pc += 3; self.saturn.complement_1_register(RegId::A, code); false }
                        0xd => { self.saturn.pc += 3; self.saturn.complement_1_register(RegId::B, code); false }
                        0xe => { self.saturn.pc += 3; self.saturn.complement_1_register(RegId::C, code); false }
                        0xf => { self.saturn.pc += 3; self.saturn.complement_1_register(RegId::D, code); false }
                        _ => true,
                    }
                }
            }

            // ----------------------------------------------------------
            // Cxx — add, A_FIELD shorthand
            // ----------------------------------------------------------
            0xc => {
                match op2 {
                    0 => { self.saturn.pc += 2; self.saturn.add_register(RegId::A, RegId::A, RegId::B, A_FIELD); false }
                    1 => { self.saturn.pc += 2; self.saturn.add_register(RegId::B, RegId::B, RegId::C, A_FIELD); false }
                    2 => { self.saturn.pc += 2; self.saturn.add_register(RegId::C, RegId::C, RegId::A, A_FIELD); false }
                    3 => { self.saturn.pc += 2; self.saturn.add_register(RegId::D, RegId::D, RegId::C, A_FIELD); false }
                    4 => { self.saturn.pc += 2; self.saturn.add_register(RegId::A, RegId::A, RegId::A, A_FIELD); false }
                    5 => { self.saturn.pc += 2; self.saturn.add_register(RegId::B, RegId::B, RegId::B, A_FIELD); false }
                    6 => { self.saturn.pc += 2; self.saturn.add_register(RegId::C, RegId::C, RegId::C, A_FIELD); false }
                    7 => { self.saturn.pc += 2; self.saturn.add_register(RegId::D, RegId::D, RegId::D, A_FIELD); false }
                    8 => { self.saturn.pc += 2; self.saturn.add_register(RegId::B, RegId::B, RegId::A, A_FIELD); false }
                    9 => { self.saturn.pc += 2; self.saturn.add_register(RegId::C, RegId::C, RegId::B, A_FIELD); false }
                    0xa => { self.saturn.pc += 2; self.saturn.add_register(RegId::A, RegId::A, RegId::C, A_FIELD); false }
                    0xb => { self.saturn.pc += 2; self.saturn.add_register(RegId::C, RegId::C, RegId::D, A_FIELD); false }
                    0xc => { self.saturn.pc += 2; self.saturn.dec_register(RegId::A, A_FIELD); false }
                    0xd => { self.saturn.pc += 2; self.saturn.dec_register(RegId::B, A_FIELD); false }
                    0xe => { self.saturn.pc += 2; self.saturn.dec_register(RegId::C, A_FIELD); false }
                    0xf => { self.saturn.pc += 2; self.saturn.dec_register(RegId::D, A_FIELD); false }
                    _ => true,
                }
            }

            // ----------------------------------------------------------
            // Dxx — zero / copy / exchange, A_FIELD shorthand
            // ----------------------------------------------------------
            0xd => {
                match op2 {
                    0 => { self.saturn.pc += 2; self.saturn.zero_register(RegId::A, A_FIELD); false }
                    1 => { self.saturn.pc += 2; self.saturn.zero_register(RegId::B, A_FIELD); false }
                    2 => { self.saturn.pc += 2; self.saturn.zero_register(RegId::C, A_FIELD); false }
                    3 => { self.saturn.pc += 2; self.saturn.zero_register(RegId::D, A_FIELD); false }
                    4 => { self.saturn.pc += 2; self.saturn.copy_register(RegId::A, RegId::B, A_FIELD); false }
                    5 => { self.saturn.pc += 2; self.saturn.copy_register(RegId::B, RegId::C, A_FIELD); false }
                    6 => { self.saturn.pc += 2; self.saturn.copy_register(RegId::C, RegId::A, A_FIELD); false }
                    7 => { self.saturn.pc += 2; self.saturn.copy_register(RegId::D, RegId::C, A_FIELD); false }
                    8 => { self.saturn.pc += 2; self.saturn.copy_register(RegId::B, RegId::A, A_FIELD); false }
                    9 => { self.saturn.pc += 2; self.saturn.copy_register(RegId::C, RegId::B, A_FIELD); false }
                    0xa => { self.saturn.pc += 2; self.saturn.copy_register(RegId::A, RegId::C, A_FIELD); false }
                    0xb => { self.saturn.pc += 2; self.saturn.copy_register(RegId::C, RegId::D, A_FIELD); false }
                    0xc => { self.saturn.pc += 2; self.saturn.exchange_register(RegId::A, RegId::B, A_FIELD); false }
                    0xd => { self.saturn.pc += 2; self.saturn.exchange_register(RegId::B, RegId::C, A_FIELD); false }
                    0xe => { self.saturn.pc += 2; self.saturn.exchange_register(RegId::A, RegId::C, A_FIELD); false }
                    0xf => { self.saturn.pc += 2; self.saturn.exchange_register(RegId::C, RegId::D, A_FIELD); false }
                    _ => true,
                }
            }

            // ----------------------------------------------------------
            // Exx — sub / inc, A_FIELD shorthand
            // ----------------------------------------------------------
            0xe => {
                match op2 {
                    0 => { self.saturn.pc += 2; self.saturn.sub_register(RegId::A, RegId::A, RegId::B, A_FIELD); false }
                    1 => { self.saturn.pc += 2; self.saturn.sub_register(RegId::B, RegId::B, RegId::C, A_FIELD); false }
                    2 => { self.saturn.pc += 2; self.saturn.sub_register(RegId::C, RegId::C, RegId::A, A_FIELD); false }
                    3 => { self.saturn.pc += 2; self.saturn.sub_register(RegId::D, RegId::D, RegId::C, A_FIELD); false }
                    4 => { self.saturn.pc += 2; self.saturn.inc_register(RegId::A, A_FIELD); false }
                    5 => { self.saturn.pc += 2; self.saturn.inc_register(RegId::B, A_FIELD); false }
                    6 => { self.saturn.pc += 2; self.saturn.inc_register(RegId::C, A_FIELD); false }
                    7 => { self.saturn.pc += 2; self.saturn.inc_register(RegId::D, A_FIELD); false }
                    8 => { self.saturn.pc += 2; self.saturn.sub_register(RegId::B, RegId::B, RegId::A, A_FIELD); false }
                    9 => { self.saturn.pc += 2; self.saturn.sub_register(RegId::C, RegId::C, RegId::B, A_FIELD); false }
                    0xa => { self.saturn.pc += 2; self.saturn.sub_register(RegId::A, RegId::A, RegId::C, A_FIELD); false }
                    0xb => { self.saturn.pc += 2; self.saturn.sub_register(RegId::C, RegId::C, RegId::D, A_FIELD); false }
                    0xc => { self.saturn.pc += 2; self.saturn.sub_register(RegId::A, RegId::B, RegId::A, A_FIELD); false }
                    0xd => { self.saturn.pc += 2; self.saturn.sub_register(RegId::B, RegId::C, RegId::B, A_FIELD); false }
                    0xe => { self.saturn.pc += 2; self.saturn.sub_register(RegId::C, RegId::A, RegId::C, A_FIELD); false }
                    0xf => { self.saturn.pc += 2; self.saturn.sub_register(RegId::D, RegId::C, RegId::D, A_FIELD); false }
                    _ => true,
                }
            }

            // ----------------------------------------------------------
            // Fxx — shift / complement, A_FIELD shorthand
            // ----------------------------------------------------------
            0xf => {
                match op2 {
                    0 => { self.saturn.pc += 2; self.saturn.shift_left_register(RegId::A, A_FIELD); false }
                    1 => { self.saturn.pc += 2; self.saturn.shift_left_register(RegId::B, A_FIELD); false }
                    2 => { self.saturn.pc += 2; self.saturn.shift_left_register(RegId::C, A_FIELD); false }
                    3 => { self.saturn.pc += 2; self.saturn.shift_left_register(RegId::D, A_FIELD); false }
                    4 => { self.saturn.pc += 2; self.saturn.shift_right_register(RegId::A, A_FIELD); false }
                    5 => { self.saturn.pc += 2; self.saturn.shift_right_register(RegId::B, A_FIELD); false }
                    6 => { self.saturn.pc += 2; self.saturn.shift_right_register(RegId::C, A_FIELD); false }
                    7 => { self.saturn.pc += 2; self.saturn.shift_right_register(RegId::D, A_FIELD); false }
                    8 => { self.saturn.pc += 2; self.saturn.complement_2_register(RegId::A, A_FIELD); false }
                    9 => { self.saturn.pc += 2; self.saturn.complement_2_register(RegId::B, A_FIELD); false }
                    0xa => { self.saturn.pc += 2; self.saturn.complement_2_register(RegId::C, A_FIELD); false }
                    0xb => { self.saturn.pc += 2; self.saturn.complement_2_register(RegId::D, A_FIELD); false }
                    0xc => { self.saturn.pc += 2; self.saturn.complement_1_register(RegId::A, A_FIELD); false }
                    0xd => { self.saturn.pc += 2; self.saturn.complement_1_register(RegId::B, A_FIELD); false }
                    0xe => { self.saturn.pc += 2; self.saturn.complement_1_register(RegId::C, A_FIELD); false }
                    0xf => { self.saturn.pc += 2; self.saturn.complement_1_register(RegId::D, A_FIELD); false }
                    _ => true,
                }
            }

            _ => true,
        }
    }

    // ================================================================
    //  step_instruction  —  top-level decode, one instruction
    // ================================================================
    pub fn step_instruction(&mut self) -> bool {
        let op0 = self.read_nibble(self.saturn.pc) as i32;

        let stop = match op0 {
            0 => {
                let op1 = self.read_nibble(self.saturn.pc + 1) as i32;
                match op1 {
                    0 => {
                        // RTNSXM
                        self.saturn.xm = 1;
                        self.saturn.pc = self.saturn.pop_return_addr();
                        false
                    }
                    1 => {
                        // RTN
                        self.saturn.pc = self.saturn.pop_return_addr();
                        false
                    }
                    2 => {
                        // RTNSC
                        self.saturn.carry = 1;
                        self.saturn.pc = self.saturn.pop_return_addr();
                        false
                    }
                    3 => {
                        // RTNCC
                        self.saturn.carry = 0;
                        self.saturn.pc = self.saturn.pop_return_addr();
                        false
                    }
                    4 => {
                        // SETHEX
                        self.saturn.pc += 2;
                        self.saturn.hexmode = HEX;
                        false
                    }
                    5 => {
                        // SETDEC
                        self.saturn.pc += 2;
                        self.saturn.hexmode = DEC;
                        false
                    }
                    6 => {
                        // RSTK=C
                        let jumpaddr = Saturn::dat_to_addr(&self.saturn.c);
                        self.saturn.push_return_addr(jumpaddr);
                        self.saturn.pc += 2;
                        false
                    }
                    7 => {
                        // C=RSTK
                        self.saturn.pc += 2;
                        let jumpaddr = self.saturn.pop_return_addr();
                        Saturn::addr_to_dat(jumpaddr, &mut self.saturn.c);
                        false
                    }
                    8 => {
                        // CLRST
                        self.saturn.pc += 2;
                        self.saturn.clear_status();
                        false
                    }
                    9 => {
                        // C=ST
                        self.saturn.pc += 2;
                        self.saturn.status_to_register(RegId::C);
                        false
                    }
                    0xa => {
                        // ST=C
                        self.saturn.pc += 2;
                        self.saturn.register_to_status(RegId::C);
                        false
                    }
                    0xb => {
                        // CSTEX
                        self.saturn.pc += 2;
                        self.saturn.swap_register_status(RegId::C);
                        false
                    }
                    0xc => {
                        // P=P+1
                        self.saturn.pc += 2;
                        if self.saturn.p == 0xf {
                            self.saturn.p = 0;
                            self.saturn.carry = 1;
                        } else {
                            self.saturn.p += 1;
                            self.saturn.carry = 0;
                        }
                        false
                    }
                    0xd => {
                        // P=P-1
                        self.saturn.pc += 2;
                        if self.saturn.p == 0 {
                            self.saturn.p = 0xf;
                            self.saturn.carry = 1;
                        } else {
                            self.saturn.p -= 1;
                            self.saturn.carry = 0;
                        }
                        false
                    }
                    0xe => {
                        // AND/OR register operations
                        let op2 = self.read_nibble(self.saturn.pc + 2) as u8;
                        let op3 = self.read_nibble(self.saturn.pc + 3) as i32;
                        match op3 {
                            0 => { self.saturn.pc += 4; self.saturn.and_register(RegId::A, RegId::A, RegId::B, op2); false }
                            1 => { self.saturn.pc += 4; self.saturn.and_register(RegId::B, RegId::B, RegId::C, op2); false }
                            2 => { self.saturn.pc += 4; self.saturn.and_register(RegId::C, RegId::C, RegId::A, op2); false }
                            3 => { self.saturn.pc += 4; self.saturn.and_register(RegId::D, RegId::D, RegId::C, op2); false }
                            4 => { self.saturn.pc += 4; self.saturn.and_register(RegId::B, RegId::B, RegId::A, op2); false }
                            5 => { self.saturn.pc += 4; self.saturn.and_register(RegId::C, RegId::C, RegId::B, op2); false }
                            6 => { self.saturn.pc += 4; self.saturn.and_register(RegId::A, RegId::A, RegId::C, op2); false }
                            7 => { self.saturn.pc += 4; self.saturn.and_register(RegId::C, RegId::C, RegId::D, op2); false }
                            8 => { self.saturn.pc += 4; self.saturn.or_register(RegId::A, RegId::A, RegId::B, op2); false }
                            9 => { self.saturn.pc += 4; self.saturn.or_register(RegId::B, RegId::B, RegId::C, op2); false }
                            0xa => { self.saturn.pc += 4; self.saturn.or_register(RegId::C, RegId::C, RegId::A, op2); false }
                            0xb => { self.saturn.pc += 4; self.saturn.or_register(RegId::D, RegId::D, RegId::C, op2); false }
                            0xc => { self.saturn.pc += 4; self.saturn.or_register(RegId::B, RegId::B, RegId::A, op2); false }
                            0xd => { self.saturn.pc += 4; self.saturn.or_register(RegId::C, RegId::C, RegId::B, op2); false }
                            0xe => { self.saturn.pc += 4; self.saturn.or_register(RegId::A, RegId::A, RegId::C, op2); false }
                            0xf => { self.saturn.pc += 4; self.saturn.or_register(RegId::C, RegId::C, RegId::D, op2); false }
                            _ => true,
                        }
                    }
                    0xf => {
                        // RTI
                        self.do_return_interrupt();
                        false
                    }
                    _ => true,
                }
            }
            1 => self.decode_group_1(),
            2 => {
                // P = nibble
                let op2 = self.read_nibble(self.saturn.pc + 1);
                self.saturn.pc += 2;
                self.saturn.p = op2;
                false
            }
            3 => {
                // LC(n)  — load constant into C
                let op2 = self.read_nibble(self.saturn.pc + 1) as i32;
                let pc = self.saturn.pc;
                self.load_constant(RegId::C, (op2 + 1) as usize, pc + 2);
                self.saturn.pc += 3 + op2;
                false
            }
            4 => {
                // GOC  — conditional jump if carry set
                let mut op2 = self.read_nibbles(self.saturn.pc + 1, 2);
                if op2 == 0x02 {
                    // NOP3
                    self.saturn.pc += 3;
                } else if self.saturn.carry != 0 {
                    if op2 != 0 {
                        if op2 & 0x80 != 0 {
                            op2 |= JUMPMASKS[2];
                        }
                        self.saturn.pc = (self.saturn.pc + op2 + 1) & 0xfffff;
                    } else {
                        self.saturn.pc = self.saturn.pop_return_addr();
                    }
                } else {
                    self.saturn.pc += 3;
                }
                false
            }
            5 => {
                // GONC  — conditional jump if carry clear
                if self.saturn.carry == 0 {
                    let mut op2 = self.read_nibbles(self.saturn.pc + 1, 2);
                    if op2 != 0 {
                        if op2 & 0x80 != 0 {
                            op2 |= JUMPMASKS[2];
                        }
                        self.saturn.pc = (self.saturn.pc + op2 + 1) & 0xfffff;
                    } else {
                        self.saturn.pc = self.saturn.pop_return_addr();
                    }
                } else {
                    self.saturn.pc += 3;
                }
                false
            }
            6 => {
                // GOTO (3-nibble relative) or NOP / TRAP
                let mut op2 = self.read_nibbles(self.saturn.pc + 1, 3);
                if op2 == 0x003 {
                    // NOP4
                    self.saturn.pc += 4;
                } else if op2 == 0x004 {
                    // TRAP
                    let op3 = self.read_nibbles(self.saturn.pc + 4, 1);
                    self.saturn.pc += 5;
                    if op3 != 0 {
                        return true;
                    }
                } else {
                    if op2 & 0x800 != 0 {
                        op2 |= JUMPMASKS[3];
                    }
                    self.saturn.pc = (op2 + self.saturn.pc + 1) & 0xfffff;
                }
                false
            }
            7 => {
                // GOSUB (3-nibble relative)
                let mut op2 = self.read_nibbles(self.saturn.pc + 1, 3);
                if op2 & 0x800 != 0 {
                    op2 |= JUMPMASKS[3];
                }
                let jumpaddr = (op2 + self.saturn.pc + 4) & 0xfffff;
                self.saturn.push_return_addr(self.saturn.pc + 4);
                self.saturn.pc = jumpaddr;
                false
            }
            _ => {
                // 8..F
                self.decode_8_thru_f(op0)
            }
        };

        stop
    }
}

use crate::cpu::Saturn;
