// ALU operations — exact port of register.c
// All register arithmetic/logic operates on nibble fields.

use crate::cpu::Saturn;
use crate::types::*;

static START_FIELDS: [i32; 19] = [
    -1,  0,  2,  0, 15,  3,  0,  0,
    -1,  0,  2,  0, 15,  3,  0,  0,
     0,  0,  0,
];

static END_FIELDS: [i32; 19] = [
    -1, -1,  2,  2, 15, 14,  1, 15,
    -1, -1,  2,  2, 15, 14,  1,  4,
     3,  2,  0,
];

#[inline]
pub fn get_start(code: u8, p: u8) -> usize {
    let s = START_FIELDS[code as usize];
    if s == -1 { p as usize } else { s as usize }
}

#[inline]
pub fn get_end(code: u8, p: u8) -> usize {
    let e = END_FIELDS[code as usize];
    if e == -1 { p as usize } else { e as usize }
}

impl Saturn {
    pub fn add_register(&mut self, res_idx: RegId, r1_idx: RegId, r2_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let hexmode = self.hexmode as i32;
        let r1 = self.get_reg(r1_idx).clone();
        let r2 = self.get_reg(r2_idx).clone();
        let res = self.get_reg_mut(res_idx);
        let mut c = 0i32;
        for i in s..=e {
            let t = r1[i] as i32 + r2[i] as i32 + c;
            if t < hexmode {
                res[i] = (t & 0xf) as u8;
                c = 0;
            } else {
                res[i] = ((t - hexmode) & 0xf) as u8;
                c = 1;
            }
        }
        self.carry = if c != 0 { 1 } else { 0 };
    }

    pub fn add_p_plus_one(&mut self, r_idx: RegId) {
        let c_init = self.p as i32 + 1;
        let r = self.get_reg_mut(r_idx);
        let mut c = c_init;
        for i in 0..=4 {
            let t = r[i] as i32 + c;
            if t < 16 {
                r[i] = (t & 0xf) as u8;
                c = 0;
            } else {
                r[i] = ((t - 16) & 0xf) as u8;
                c = 1;
            }
        }
        self.carry = if c != 0 { 1 } else { 0 };
    }

    pub fn sub_register(&mut self, res_idx: RegId, r1_idx: RegId, r2_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let hexmode = self.hexmode as i32;
        let r1 = self.get_reg(r1_idx).clone();
        let r2 = self.get_reg(r2_idx).clone();
        let res = self.get_reg_mut(res_idx);
        let mut c = 0i32;
        for i in s..=e {
            let t = r1[i] as i32 - r2[i] as i32 - c;
            if t >= 0 {
                res[i] = (t & 0xf) as u8;
                c = 0;
            } else {
                res[i] = ((t + hexmode) & 0xf) as u8;
                c = 1;
            }
        }
        self.carry = if c != 0 { 1 } else { 0 };
    }

    pub fn complement_2_register(&mut self, r_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let hexmode = self.hexmode as i32;
        let r = self.get_reg_mut(r_idx);
        let mut c = 1i32;
        let mut carry = 0i32;
        for i in s..=e {
            let t = (hexmode - 1) - r[i] as i32 + c;
            if t < hexmode {
                r[i] = (t & 0xf) as u8;
                c = 0;
            } else {
                r[i] = ((t - hexmode) & 0xf) as u8;
                c = 1;
            }
            carry += r[i] as i32;
        }
        self.carry = if carry != 0 { 1 } else { 0 };
    }

    pub fn complement_1_register(&mut self, r_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let hexmode = self.hexmode as i32;
        let r = self.get_reg_mut(r_idx);
        for i in s..=e {
            let t = (hexmode - 1) - r[i] as i32;
            r[i] = (t & 0xf) as u8;
        }
        self.carry = 0;
    }

    pub fn inc_register(&mut self, r_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let hexmode = self.hexmode as i32;
        let r = self.get_reg_mut(r_idx);
        let mut c = 1i32;
        for i in s..=e {
            let t = r[i] as i32 + c;
            if t < hexmode {
                r[i] = (t & 0xf) as u8;
                c = 0;
                break;
            } else {
                r[i] = ((t - hexmode) & 0xf) as u8;
                c = 1;
            }
        }
        self.carry = if c != 0 { 1 } else { 0 };
    }

    pub fn add_register_constant(&mut self, r_idx: RegId, code: u8, val: i32) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r = self.get_reg_mut(r_idx);
        let mut c = val;
        for i in s..=e {
            let t = r[i] as i32 + c;
            if t < 16 {
                r[i] = (t & 0xf) as u8;
                c = 0;
                break;
            } else {
                r[i] = ((t - 16) & 0xf) as u8;
                c = 1;
            }
        }
        self.carry = if c != 0 { 1 } else { 0 };
    }

    pub fn dec_register(&mut self, r_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let hexmode = self.hexmode as i32;
        let r = self.get_reg_mut(r_idx);
        let mut c = 1i32;
        for i in s..=e {
            let t = r[i] as i32 - c;
            if t >= 0 {
                r[i] = (t & 0xf) as u8;
                c = 0;
                break;
            } else {
                r[i] = ((t + hexmode) & 0xf) as u8;
                c = 1;
            }
        }
        self.carry = if c != 0 { 1 } else { 0 };
    }

    pub fn sub_register_constant(&mut self, r_idx: RegId, code: u8, val: i32) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r = self.get_reg_mut(r_idx);
        let mut c = val;
        for i in s..=e {
            let t = r[i] as i32 - c;
            if t >= 0 {
                r[i] = (t & 0xf) as u8;
                c = 0;
                break;
            } else {
                r[i] = ((t + 16) & 0xf) as u8;
                c = 1;
            }
        }
        self.carry = if c != 0 { 1 } else { 0 };
    }

    pub fn zero_register(&mut self, r_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r = self.get_reg_mut(r_idx);
        for i in s..=e {
            r[i] = 0;
        }
    }

    pub fn or_register(&mut self, res_idx: RegId, r1_idx: RegId, r2_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r1 = self.get_reg(r1_idx).clone();
        let r2 = self.get_reg(r2_idx).clone();
        let res = self.get_reg_mut(res_idx);
        for i in s..=e {
            res[i] = (r1[i] | r2[i]) & 0xf;
        }
    }

    pub fn and_register(&mut self, res_idx: RegId, r1_idx: RegId, r2_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r1 = self.get_reg(r1_idx).clone();
        let r2 = self.get_reg(r2_idx).clone();
        let res = self.get_reg_mut(res_idx);
        for i in s..=e {
            res[i] = (r1[i] & r2[i]) & 0xf;
        }
    }

    pub fn copy_register(&mut self, to_idx: RegId, from_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let from = self.get_reg(from_idx).clone();
        let to = self.get_reg_mut(to_idx);
        for i in s..=e {
            to[i] = from[i];
        }
    }

    pub fn exchange_register(&mut self, r1_idx: RegId, r2_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        // Must use temporary to avoid borrow issues
        let mut r1_copy = self.get_reg(r1_idx).clone();
        let r2_copy = self.get_reg(r2_idx).clone();
        for i in s..=e {
            let t = r1_copy[i];
            r1_copy[i] = r2_copy[i];
            // set r2 nibble to old r1 nibble
            self.get_reg_mut(r2_idx)[i] = t;
        }
        let r1 = self.get_reg_mut(r1_idx);
        for i in s..=e {
            r1[i] = r1_copy[i];
        }
    }

    pub fn exchange_reg_dat(&mut self, r_idx: RegId, d_sel: u8, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let mut d = if d_sel == 0 { self.d0 } else { self.d1 };
        let r = self.get_reg_mut(r_idx);
        for i in s..=e {
            let t = r[i];
            r[i] = ((d >> (i as i32 * 4)) & 0x0f) as u8;
            d &= !NIBBLE_MASKS[i];
            d |= (t as i32) << (i as i32 * 4);
        }
        if d_sel == 0 { self.d0 = d; } else { self.d1 = d; }
    }

    pub fn shift_left_register(&mut self, r_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r = self.get_reg_mut(r_idx);
        let mut i = e;
        while i > s {
            r[i] = r[i - 1] & 0x0f;
            i -= 1;
        }
        r[s] = 0;
    }

    pub fn shift_left_circ_register(&mut self, r_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r = self.get_reg_mut(r_idx);
        let t = r[e] & 0x0f;
        let mut i = e;
        while i > s {
            r[i] = r[i - 1] & 0x0f;
            i -= 1;
        }
        r[s] = t;
    }

    pub fn shift_right_register(&mut self, r_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let sb = self.get_reg(r_idx)[s] & 0x0f != 0;
        let r = self.get_reg_mut(r_idx);
        for i in s..e {
            r[i] = r[i + 1] & 0x0f;
        }
        r[e] = 0;
        if sb {
            self.sb = 1;
        }
    }

    pub fn shift_right_circ_register(&mut self, r_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r = self.get_reg_mut(r_idx);
        let t = r[s] & 0x0f;
        for i in s..e {
            r[i] = r[i + 1] & 0x0f;
        }
        r[e] = t;
        if t != 0 {
            self.sb = 1;
        }
    }

    pub fn shift_right_bit_register(&mut self, r_idx: RegId, code: u8) {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r = self.get_reg_mut(r_idx);
        let mut sb: u8 = 0;
        let mut i = e as isize;
        while i >= s as isize {
            let idx = i as usize;
            let t = (((r[idx] >> 1) & 7) | (sb << 3)) & 0x0f;
            sb = r[idx] & 1;
            r[idx] = t;
            i -= 1;
        }
        if sb != 0 {
            self.sb = 1;
        }
    }

    pub fn is_zero_register(&self, r_idx: RegId, code: u8) -> bool {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r = self.get_reg(r_idx);
        for i in s..=e {
            if r[i] & 0xf != 0 {
                return false;
            }
        }
        true
    }

    pub fn is_not_zero_register(&self, r_idx: RegId, code: u8) -> bool {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r = self.get_reg(r_idx);
        for i in s..=e {
            if r[i] & 0xf != 0 {
                return true;
            }
        }
        false
    }

    pub fn is_equal_register(&self, r1_idx: RegId, r2_idx: RegId, code: u8) -> bool {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r1 = self.get_reg(r1_idx);
        let r2 = self.get_reg(r2_idx);
        for i in s..=e {
            if (r1[i] & 0xf) != (r2[i] & 0xf) {
                return false;
            }
        }
        true
    }

    pub fn is_not_equal_register(&self, r1_idx: RegId, r2_idx: RegId, code: u8) -> bool {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r1 = self.get_reg(r1_idx);
        let r2 = self.get_reg(r2_idx);
        for i in s..=e {
            if (r1[i] & 0xf) != (r2[i] & 0xf) {
                return true;
            }
        }
        false
    }

    pub fn is_less_register(&self, r1_idx: RegId, r2_idx: RegId, code: u8) -> bool {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r1 = self.get_reg(r1_idx);
        let r2 = self.get_reg(r2_idx);
        let mut i = e as isize;
        while i >= s as isize {
            let idx = i as usize;
            if (r1[idx] & 0xf) < (r2[idx] & 0xf) {
                return true;
            }
            if (r1[idx] & 0xf) > (r2[idx] & 0xf) {
                return false;
            }
            i -= 1;
        }
        false
    }

    pub fn is_less_or_equal_register(&self, r1_idx: RegId, r2_idx: RegId, code: u8) -> bool {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r1 = self.get_reg(r1_idx);
        let r2 = self.get_reg(r2_idx);
        let mut i = e as isize;
        while i >= s as isize {
            let idx = i as usize;
            if (r1[idx] & 0xf) < (r2[idx] & 0xf) {
                return true;
            }
            if (r1[idx] & 0xf) > (r2[idx] & 0xf) {
                return false;
            }
            i -= 1;
        }
        true
    }

    pub fn is_greater_register(&self, r1_idx: RegId, r2_idx: RegId, code: u8) -> bool {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r1 = self.get_reg(r1_idx);
        let r2 = self.get_reg(r2_idx);
        let mut i = e as isize;
        while i >= s as isize {
            let idx = i as usize;
            if (r1[idx] & 0xf) > (r2[idx] & 0xf) {
                return true;
            }
            if (r1[idx] & 0xf) < (r2[idx] & 0xf) {
                return false;
            }
            i -= 1;
        }
        false
    }

    pub fn is_greater_or_equal_register(&self, r1_idx: RegId, r2_idx: RegId, code: u8) -> bool {
        let s = get_start(code, self.p);
        let e = get_end(code, self.p);
        let r1 = self.get_reg(r1_idx);
        let r2 = self.get_reg(r2_idx);
        let mut i = e as isize;
        while i >= s as isize {
            let idx = i as usize;
            if (r1[idx] & 0xf) < (r2[idx] & 0xf) {
                return false;
            }
            if (r1[idx] & 0xf) > (r2[idx] & 0xf) {
                return true;
            }
            i -= 1;
        }
        true
    }
}

// Register identifiers — used to avoid borrow issues with &mut self
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegId {
    A, B, C, D,
    R0, R1, R2, R3, R4,
}

impl Saturn {
    #[inline]
    pub fn get_reg(&self, id: RegId) -> &[u8; 16] {
        match id {
            RegId::A => &self.a,
            RegId::B => &self.b,
            RegId::C => &self.c,
            RegId::D => &self.d,
            RegId::R0 => &self.r0,
            RegId::R1 => &self.r1,
            RegId::R2 => &self.r2,
            RegId::R3 => &self.r3,
            RegId::R4 => &self.r4,
        }
    }

    #[inline]
    pub fn get_reg_mut(&mut self, id: RegId) -> &mut [u8; 16] {
        match id {
            RegId::A => &mut self.a,
            RegId::B => &mut self.b,
            RegId::C => &mut self.c,
            RegId::D => &mut self.d,
            RegId::R0 => &mut self.r0,
            RegId::R1 => &mut self.r1,
            RegId::R2 => &mut self.r2,
            RegId::R3 => &mut self.r3,
            RegId::R4 => &mut self.r4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_register_hex() {
        let mut sat = Saturn::default();
        sat.hexmode = HEX;
        sat.a = [0x5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.b = [0x3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.add_register(RegId::A, RegId::A, RegId::B, W_FIELD);
        assert_eq!(sat.a[0], 0x8);
        assert_eq!(sat.carry, 0);
    }

    #[test]
    fn test_add_register_hex_carry() {
        let mut sat = Saturn::default();
        sat.hexmode = HEX;
        sat.a = [0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf];
        sat.b = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.add_register(RegId::A, RegId::A, RegId::B, W_FIELD);
        assert_eq!(sat.a, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(sat.carry, 1);
    }

    #[test]
    fn test_add_register_dec() {
        let mut sat = Saturn::default();
        sat.hexmode = DEC;
        sat.a = [7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.b = [5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.add_register(RegId::A, RegId::A, RegId::B, W_FIELD);
        assert_eq!(sat.a[0], 2);
        assert_eq!(sat.a[1], 1);
        assert_eq!(sat.carry, 0);
    }

    #[test]
    fn test_sub_register() {
        let mut sat = Saturn::default();
        sat.hexmode = HEX;
        sat.a = [5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.b = [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.sub_register(RegId::A, RegId::A, RegId::B, W_FIELD);
        assert_eq!(sat.a[0], 2);
        assert_eq!(sat.carry, 0);
    }

    #[test]
    fn test_sub_register_borrow() {
        let mut sat = Saturn::default();
        sat.hexmode = HEX;
        sat.a = [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.b = [5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.sub_register(RegId::A, RegId::A, RegId::B, W_FIELD);
        assert_eq!(sat.a[0], 0xe);
        assert_eq!(sat.a[1], 0xf);
        assert_eq!(sat.carry, 1);
    }

    #[test]
    fn test_shift_right_register() {
        let mut sat = Saturn::default();
        sat.a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0];
        sat.shift_right_register(RegId::A, W_FIELD);
        assert_eq!(sat.a[0], 2);
        assert_eq!(sat.a[14], 0);
        assert_eq!(sat.a[15], 0);
        assert_eq!(sat.sb, 1); // bit 0 of nibble[0] was 1
    }

    #[test]
    fn test_is_less_register() {
        let mut sat = Saturn::default();
        sat.a = [5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.b = [6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(sat.is_less_register(RegId::A, RegId::B, W_FIELD));
        assert!(!sat.is_less_register(RegId::B, RegId::A, W_FIELD));
        assert!(!sat.is_less_register(RegId::A, RegId::A, W_FIELD));
    }

    #[test]
    fn test_p_field() {
        let mut sat = Saturn::default();
        sat.p = 3;
        sat.a = [0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        sat.inc_register(RegId::A, P_FIELD);
        assert_eq!(sat.a[3], 6);
    }
}
