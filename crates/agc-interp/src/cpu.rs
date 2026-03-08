//! AGC Block-2 CPU state and ones-complement arithmetic helpers.
//!
//! # Memory map (12-bit address space, 0o0000 – 0o7777)
//!
//! | Range         | Description                                 |
//! |---------------|---------------------------------------------|
//! | 0o0000–0o0007 | Central registers (A, L, Q, EB, FB, Z, BB, ZERO) |
//! | 0o0010–0o1777 | Erasable memory (1016 words, bank-switched) |
//! | 0o2000–0o7777 | Fixed memory (6144 words, bank-switched)    |

pub const ERASABLE_SIZE: usize = 1024;  // 0o0000–0o1777 (includes CP registers 0–7)
pub const FIXED_SIZE:    usize = 6144;  // 0o2000–0o7777 (6 × 1024 words)
pub const CHANNEL_COUNT: usize = 512;   // 9-bit channel address

/// Index of the BRUPT register in erasable memory (interrupt saved-instruction).
pub const BRUPT_ADDR: u16 = 0o17;
/// Index of the ZRUPT register in erasable memory (interrupt saved-PC).
pub const ZRUPT_ADDR: u16 = 0o16;
/// Restart address used by GO.
pub const GO_ADDR:    u16 = 0o4000;

// ─── CPU registers ────────────────────────────────────────────────────────────

/// Central-register indices (double as low erasable addresses).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Reg {
    A    = 0o00,  // Accumulator (16-bit: bit15=overflow, bits14:0=data)
    L    = 0o01,  // Lower accumulator / multiplier-quotient
    Q    = 0o02,  // Return address / remainder
    Eb   = 0o03,  // Erasable bank register
    Fb   = 0o04,  // Fixed bank register
    Z    = 0o05,  // Program counter
    Bb   = 0o06,  // Both banks (derived: FB||EB)
    Zero = 0o07,  // Hardwired zero
}

// ─── CPU state ────────────────────────────────────────────────────────────────

/// Full AGC Block-2 CPU state.
pub struct Cpu {
    // ── Central registers (also addressable via erasable 0o00–0o07) ──────────
    /// Accumulator: 16-bit (bit 15 = overflow flag, bits 14:0 = 15-bit OC data).
    pub a: u16,
    /// L register (15-bit).
    pub l: u16,
    /// Q register / return address (15-bit).
    pub q: u16,
    /// Erasable bank (3 bits stored in low bits).
    pub eb: u16,
    /// Fixed bank (5 bits stored in low bits).
    pub fb: u16,
    /// Program counter (12-bit address, incremented after fetch).
    pub z: u16,
    /// Both banks (read-only derived; write splits to EB and FB).
    pub bb: u16,

    // ── Internal temporary registers (not addressable by programs) ────────────
    /// Non-addressable temporary register (used by XCH, DXCH, etc.).
    pub tmp: u16,
    /// Raw instruction word currently being executed.
    pub instr_word: u16,

    // ── Control flip-flops ────────────────────────────────────────────────────
    /// EXTEND flip-flop: set by the EXTEND instruction, cleared after next instr.
    pub extend: bool,
    /// Interrupt inhibit flag (set by INHINT, cleared by RELINT).
    pub inhint: bool,
    /// Pending interrupt vector (0 = none).
    pub pending_interrupt: Option<u16>,

    // ── Memory ────────────────────────────────────────────────────────────────
    /// Erasable memory including CP registers at 0o0000–0o0007.
    pub erasable: [u16; ERASABLE_SIZE],
    /// Fixed memory: flat array covering 0o2000–0o7777.
    pub fixed: [u16; FIXED_SIZE],

    // ── I/O channels ─────────────────────────────────────────────────────────
    pub channels: [u16; CHANNEL_COUNT],
}

impl Cpu {
    /// Create a zeroed CPU (all memory and registers = plus-zero).
    pub fn new() -> Self {
        Cpu {
            a: 0, l: 0, q: 0,
            eb: 0, fb: 0, z: 0o4000, bb: 0,
            tmp: 0,
            instr_word: 0,
            extend: false,
            inhint: false,
            pending_interrupt: None,
            erasable: [0; ERASABLE_SIZE],
            fixed: [0; FIXED_SIZE],
            channels: [0; CHANNEL_COUNT],
        }
    }

    // ── Register read/write ───────────────────────────────────────────────────

    /// Read a central register by address (0o00–0o07).
    pub fn read_reg(&self, addr: u16) -> u16 {
        match addr & 0o7 {
            0o0 => self.a & 0x7FFF,  // Strip overflow when reading as data
            0o1 => self.l & 0x7FFF,
            0o2 => self.q & 0x7FFF,
            0o3 => self.eb & 0x7FFF,
            0o4 => self.fb & 0x7FFF,
            0o5 => self.z & 0x7FFF,
            0o6 => self.bb & 0x7FFF,
            _   => 0, // ZERO — hardwired
        }
    }

    /// Write a central register by address (0o00–0o07).
    pub fn write_reg(&mut self, addr: u16, value: u16) {
        match addr & 0o7 {
            0o0 => { self.a = value; }  // Keep overflow bit
            0o1 => { self.l = value & 0x7FFF; }
            0o2 => { self.q = value & 0x7FFF; }
            0o3 => { self.eb = value & 0x7; self.update_bb(); }
            0o4 => { self.fb = value & 0x1F; self.update_bb(); }
            0o5 => { self.z = value & 0x7FFF; }
            0o6 => { self.split_bb(value); }
            _   => {}  // ZERO — ignore writes
        }
    }

    fn update_bb(&mut self) {
        self.bb = ((self.fb & 0x1F) << 3) | (self.eb & 0x7);
    }

    fn split_bb(&mut self, bb: u16) {
        self.eb = bb & 0x7;
        self.fb = (bb >> 3) & 0x1F;
        self.bb = bb & 0x7FFF;
    }

    // ── Memory read/write ─────────────────────────────────────────────────────

    /// Read a 15-bit word from the given 12-bit address.
    pub fn mem_read(&self, addr: u16) -> u16 {
        let addr = addr & 0x0FFF; // 12-bit address space
        if addr <= 0o7 {
            self.read_reg(addr)
        } else if addr <= 0o1777 {
            self.erasable[addr as usize] & 0x7FFF
        } else {
            let offset = (addr - 0o2000) as usize;
            if offset < FIXED_SIZE {
                self.fixed[offset] & 0x7FFF
            } else {
                0
            }
        }
    }

    /// Write a 15-bit word to the given 12-bit address (bit 15 stripped for memory).
    pub fn mem_write(&mut self, addr: u16, value: u16) {
        let addr = addr & 0x0FFF;
        let value = value & 0x7FFF; // Memory stores 15 bits only
        if addr <= 0o7 {
            self.write_reg(addr, value);
        } else if addr <= 0o1777 {
            self.erasable[addr as usize] = value;
        } else {
            let offset = (addr - 0o2000) as usize;
            if offset < FIXED_SIZE {
                self.fixed[offset] = value;
            }
        }
    }

    // ── I/O channel access ────────────────────────────────────────────────────

    pub fn channel_read(&self, ch: u16) -> u16 {
        let ch = (ch as usize) % CHANNEL_COUNT;
        self.channels[ch] & 0x7FFF
    }

    pub fn channel_write(&mut self, ch: u16, value: u16) {
        let ch = (ch as usize) % CHANNEL_COUNT;
        self.channels[ch] = value & 0x7FFF;
    }

    // ── Fetch-decode cycle ────────────────────────────────────────────────────

    /// Fetch the instruction word at the current PC (Z) without advancing Z.
    pub fn fetch(&self) -> u16 {
        self.mem_read(self.z)
    }

    /// Advance Z to the next instruction address.
    pub fn advance_pc(&mut self) {
        self.z = (self.z + 1) & 0x7FFF;
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu::new()
    }
}

// ─── Ones-complement arithmetic ───────────────────────────────────────────────

/// Convert a 15-bit OC word to a sign-extended i32.
pub fn oc15_to_i32(w: u16) -> i32 {
    let w = w & 0x7FFF;
    if w & 0x4000 == 0 {
        w as i32
    } else if w == 0x7FFF {
        // Minus-zero → treat as 0
        0
    } else {
        -(((!w) & 0x7FFF) as i32)
    }
}

/// Convert an i32 to a 15-bit OC word.
pub fn i32_to_oc15(n: i32) -> u16 {
    if n >= 0 {
        (n.min(0x3FFF) as u16) & 0x7FFF
    } else {
        let magnitude = (-n).min(0x3FFF) as u16;
        (!magnitude) & 0x7FFF
    }
}

/// Ones-complement 15-bit addition with end-around carry.
pub fn oc_add(a: u16, b: u16) -> (u16, bool) {
    let a = (a & 0x7FFF) as u32;
    let b = (b & 0x7FFF) as u32;
    let raw = a + b;
    let result = if raw >= 0x8000 {
        raw - 0x7FFF
    } else {
        raw
    };
    let result = (result & 0x7FFF) as u16;
    let a_neg = (a & 0x4000) != 0;
    let b_neg = (b & 0x4000) != 0;
    let r_neg = (result & 0x4000) != 0;
    let overflowed = (a_neg == b_neg) && (a_neg != r_neg);
    (result, overflowed)
}

/// Ones-complement 15-bit subtraction: `a - b`.
pub fn oc_sub(a: u16, b: u16) -> (u16, bool) {
    let neg_b = (!b) & 0x7FFF;
    oc_add(a, neg_b)
}

/// True if the 15-bit OC word represents plus-zero.
pub fn is_plus_zero(w: u16) -> bool { (w & 0x7FFF) == 0x0000 }

/// True if the 15-bit OC word represents minus-zero.
pub fn is_minus_zero(w: u16) -> bool { (w & 0x7FFF) == 0x7FFF }

/// True if the 15-bit OC word is negative (sign bit set) and not minus-zero.
pub fn is_negative(w: u16) -> bool { (w & 0x4000) != 0 && !is_minus_zero(w) }

/// True if the 15-bit OC word is strictly positive.
pub fn is_positive(w: u16) -> bool { !is_plus_zero(w) && (w & 0x4000) == 0 }

/// Compute overflow state of a 16-bit accumulator value.
pub fn accumulator_overflow(a: u16) -> (bool, bool) {
    let b14 = (a & 0x4000) != 0;
    let b15 = (a & 0x8000) != 0;
    match (b15, b14) {
        (false, true)  => (true, false),  // positive overflow
        (true,  false) => (false, true),  // negative overflow
        _              => (false, false), // normal
    }
}

/// True if a 16-bit value has overflow (bits 14 and 15 differ).
pub fn has_overflow(a: u16) -> bool {
    let b14 = (a & 0x4000) != 0;
    let b15 = (a & 0x8000) != 0;
    b14 != b15
}

/// Diminish a 15-bit OC value toward zero by 1.
pub fn oc_diminish(w: u16) -> u16 {
    let w = w & 0x7FFF;
    if is_plus_zero(w) || is_minus_zero(w) { return w; }
    if is_positive(w) {
        if w == 0x0001 { 0x0000 } else { w - 1 }
    } else {
        let neg_w = (!w) & 0x7FFF;
        if neg_w == 0x0001 { 0x7FFF }
        else { (!((neg_w - 1) & 0x7FFF)) & 0x7FFF }
    }
}

/// Augment a 15-bit OC value: move away from zero by 1.
pub fn oc_augment(w: u16) -> u16 {
    let w = w & 0x7FFF;
    if is_plus_zero(w) || is_minus_zero(w) { return w; }
    if is_positive(w) {
        let (r, _) = oc_add(w, 0x0001);
        r
    } else {
        let (r, _) = oc_add(w, 0x7FFE); // add -1 in OC
        r
    }
}

/// Saturate: clamp to ±max if overflow, else passthrough (for TS).
pub fn oc_saturate(a: u16) -> u16 {
    let (pos_ov, neg_ov) = accumulator_overflow(a);
    if pos_ov { 0x3FFF }
    else if neg_ov { 0x4000 }
    else { a & 0x7FFF }
}

/// 30-bit ones-complement multiply. Returns (high_15bit, low_15bit).
pub fn oc_mul(a: u16, b: u16) -> (u16, u16) {
    let ai = oc15_to_i32(a);
    let bi = oc15_to_i32(b);
    let product: i64 = (ai as i64) * (bi as i64);
    let hi = (product >> 15) as i32;
    let lo = (product & 0x7FFF) as i32;
    (i32_to_oc15(hi), i32_to_oc15(lo))
}

/// 15-bit ones-complement divide. Returns `None` on divide-by-zero.
pub fn oc_div(a_hi: u16, a_lo: u16, divisor: u16) -> Option<(u16, u16)> {
    let dividend: i64 = ((oc15_to_i32(a_hi) as i64) << 15) | (oc15_to_i32(a_lo) as i64).abs();
    let div = oc15_to_i32(divisor) as i64;
    if div == 0 { return None; }
    let q = (dividend / div) as i32;
    let r = (dividend % div) as i32;
    Some((i32_to_oc15(q), i32_to_oc15(r)))
}

/// High word of double-precision addition with carry propagation.
/// dp_add_hi(a_hi, a_lo, b_hi, b_lo): hi word of (a_hi,a_lo) + (b_hi,b_lo)
pub fn dp_add_hi(a_hi: u16, a_lo: u16, b_hi: u16, b_lo: u16) -> u16 {
    let (_, carry) = oc_add(a_lo, b_lo);
    let carry_word: u16 = if carry { 1 } else { 0 };
    let (hi_sum, _) = oc_add(oc_add(a_hi, b_hi).0, carry_word);
    hi_sum
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plus_zero_minus_zero() {
        assert!(is_plus_zero(0x0000));
        assert!(is_minus_zero(0x7FFF));
        assert!(!is_negative(0x7FFF));
        assert!(!is_positive(0x0000));
    }

    #[test]
    fn oc_add_simple() {
        let (r, ov) = oc_add(1, 1);
        assert_eq!(r, 2);
        assert!(!ov);
    }

    #[test]
    fn oc_add_end_around_carry() {
        let (r, _ov) = oc_add(0x7FFE, 0x0002);
        assert_eq!(r, 0x0001);
    }

    #[test]
    fn oc_sub_simple() {
        let (r, _) = oc_sub(5, 3);
        assert_eq!(r, 2);
    }

    #[test]
    fn round_trip_i32() {
        for n in [-16383i32, -1, 0, 1, 16383] {
            let w = i32_to_oc15(n);
            let back = oc15_to_i32(w);
            assert_eq!(back, n, "round-trip failed for {}", n);
        }
    }

    #[test]
    fn mem_read_write() {
        let mut cpu = Cpu::new();
        cpu.mem_write(0o100, 42);
        assert_eq!(cpu.mem_read(0o100), 42);
    }

    #[test]
    fn register_zero_writes_ignored() {
        let mut cpu = Cpu::new();
        cpu.write_reg(Reg::Zero as u16, 0xDEAD);
        assert_eq!(cpu.read_reg(Reg::Zero as u16), 0);
    }

    #[test]
    fn saturate_overflow() {
        // Positive overflow: bits 14=1, 15=0
        assert_eq!(oc_saturate(0b0100_0000_0000_0000u16), 0x3FFF);
        // Negative overflow: bits 14=0, 15=1
        assert_eq!(oc_saturate(0b1000_0000_0000_0000u16), 0x4000);
        // Normal value
        assert_eq!(oc_saturate(5u16), 5);
    }

    #[test]
    fn tmp_field_exists() {
        let mut cpu = Cpu::new();
        cpu.tmp = 42;
        assert_eq!(cpu.tmp, 42);
    }
}
