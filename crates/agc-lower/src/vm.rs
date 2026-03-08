//! TC2 16-bit stack VM for executing lowered AGC bytecode.
//!
//! The VM is purely TC2 — no OC-specific arithmetic opcodes. All OC semantics
//! are lowered to sequences of the TC2 instructions found in this VM by the
//! [`crate::lower`] module.
//!
//! ## Memory layout
//!
//! | VM address range   | Contents                                    |
//! |--------------------|---------------------------------------------|
//! | `0x0000`–`0x0FFF` | AGC memory space (12-bit, identity-mapped)  |
//! | `0x8000`–`0x81FF` | I/O channels (`CHAN_BASE + channel_number`) |
//! | `0xFF00`           | AGC non-addressable `tmp` register          |
//! | `0xFF01`           | EXTEND flag (0 or 1)                        |
//! | `0xFF02`           | INHINT flag (0 or 1)                        |
//! | `0xFF03`           | Raw instruction word being executed         |
//!
//! AGC registers are at their natural 12-bit addresses (A=0, L=1, Q=2, Z=5 …).

use agc_interp::cpu as oc;
use crate::bytecode::op;

// ─── Address constants ────────────────────────────────────────────────────────

/// VM memory addresses for AGC architectural registers and internal state.
pub mod addr {
    pub const A:           u16 = 0x0000;
    pub const L:           u16 = 0x0001;
    pub const Q:           u16 = 0x0002;
    pub const EB:          u16 = 0x0003;
    pub const FB:          u16 = 0x0004;
    pub const Z:           u16 = 0x0005;
    pub const BB:          u16 = 0x0006;
    pub const ZERO:        u16 = 0x0007;
    pub const TMP:         u16 = 0xFF00;
    pub const EXTEND:      u16 = 0xFF01;
    pub const INHINT:      u16 = 0xFF02;
    pub const INSTR_WORD:  u16 = 0xFF03;
    /// I/O channel N maps to `CHAN_BASE + N` (N: 0–511).
    pub const CHAN_BASE:   u16 = 0x8000;
}

// ─── VM error ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    DivideByZero,
    UnknownOpcode(u16),
    StackUnderflow,
}

impl core::fmt::Display for VmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VmError::DivideByZero       => f.write_str("division by zero"),
            VmError::UnknownOpcode(opc) => write!(f, "unknown opcode 0x{opc:04X}"),
            VmError::StackUnderflow     => f.write_str("stack underflow"),
        }
    }
}

// ─── VM ───────────────────────────────────────────────────────────────────────

/// TC2 16-bit stack VM.
pub struct BytecodeVm {
    /// Flat 65 536-word memory (16-bit words — "16-bit bytes" for future tagging).
    pub mem: Box<[u16; 65536]>,
    /// Main temporary register `T`.
    pub t: u16,
    /// Operand/offset register `OFF` (loaded with instruction operand before each exec).
    pub off: u16,
    stack: Vec<u16>,
}

impl BytecodeVm {
    pub fn new() -> Self {
        BytecodeVm {
            mem:   Box::new([0u16; 65536]),
            t:     0,
            off:   0,
            stack: Vec::with_capacity(64),
        }
    }

    // ── Memory ────────────────────────────────────────────────────────────

    #[inline] pub fn mem_load(&self, a: u16)     -> u16 { self.mem[a as usize] }
    #[inline] pub fn mem_store(&mut self, a: u16, v: u16)  { self.mem[a as usize] = v; }
    #[inline] pub fn mem_store15(&mut self, a: u16, v: u16) { self.mem[a as usize] = v & 0x7FFF; }

    #[inline]
    pub fn chan_load(&self, ch: u16) -> u16 {
        self.mem[(addr::CHAN_BASE.wrapping_add(ch & 0x01FF)) as usize]
    }
    #[inline]
    pub fn chan_store(&mut self, ch: u16, v: u16) {
        self.mem[(addr::CHAN_BASE.wrapping_add(ch & 0x01FF)) as usize] = v & 0x7FFF;
    }

    // ── Stack ─────────────────────────────────────────────────────────────

    #[inline] fn push(&mut self, v: u16) { self.stack.push(v); }
    #[inline] fn pop(&mut self) -> Result<u16, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    // ── Execution ─────────────────────────────────────────────────────────

    /// Execute bytecode from word offset 0 until `RET` or end of slice.
    pub fn exec(&mut self, code: &[u16]) -> Result<(), VmError> {
        let mut pc: usize = 0;

        // Read next word and advance pc.
        macro_rules! next {
            () => {{
                let v = code.get(pc).copied().unwrap_or(0);
                pc += 1;
                v
            }};
        }

        loop {
            let Some(&opc) = code.get(pc) else { break };
            pc += 1;

            match opc {
                // ── Control ───────────────────────────────────────────────
                op::RET => break,

                op::JUMP => {
                    let off = next!() as i16;
                    pc = (pc as isize + off as isize) as usize;
                }
                op::JUMP_IF => {
                    let off = next!() as i16;
                    if self.pop()? != 0 {
                        pc = (pc as isize + off as isize) as usize;
                    }
                }
                op::JUMP_NOT => {
                    let off = next!() as i16;
                    if self.pop()? == 0 {
                        pc = (pc as isize + off as isize) as usize;
                    }
                }

                // ── Immediate / absolute memory ───────────────────────────
                op::PUSH_IMM => { let v = next!(); self.push(v); }
                op::LOAD  => { let a = next!(); let v = self.mem_load(a);   self.push(v); }
                op::STORE => { let a = next!(); let v = self.pop()?;         self.mem_store(a, v); }
                op::STORE15 => { let a = next!(); let v = self.pop()?;       self.mem_store15(a, v); }
                op::SET_OFF => { self.off = next!(); }

                // ── Shifts (immediate) ─────────────────────────────────────
                op::LSHR => { let k = next!(); let x = self.pop()?; self.push(x >> k); }
                op::LSHL => { let k = next!(); let x = self.pop()?; self.push((x << k) & 0xFFFF); }
                op::LSHR_STK => {
                    let k = self.pop()?;
                    let x = self.pop()?;
                    self.push(x >> k);
                }
                op::LSHL_STK => {
                    let k = self.pop()?;
                    let x = self.pop()?;
                    self.push((x << k) & 0xFFFF);
                }

                // ── OFF-relative memory ────────────────────────────────────
                op::LOAD_OFF    => { let v = self.mem_load(self.off);               self.push(v); }
                op::STORE_OFF   => { let v = self.pop()?; let a = self.off;         self.mem_store15(a, v); }
                op::LOAD_OFF1   => { let v = self.mem_load(self.off.wrapping_add(1)); self.push(v); }
                op::STORE_OFF1  => { let v = self.pop()?; let a = self.off.wrapping_add(1); self.mem_store15(a, v); }
                op::GET_OFF     => { let o = self.off; self.push(o); }
                op::SET_OFF_STACK => { self.off = self.pop()?; }

                // ── Channel / indirect ─────────────────────────────────────
                op::LOAD_CHAN_OFF  => { let v = self.chan_load(self.off);   self.push(v); }
                op::STORE_CHAN_OFF => { let v = self.pop()?; let c = self.off; self.chan_store(c, v); }
                op::LOAD_IND      => { let a = self.pop()?; let v = self.mem_load(a); self.push(v); }
                op::STORE_IND     => {
                    // Stack: [..., val, addr]  (addr on top)
                    let a = self.pop()?;
                    let v = self.pop()?;
                    self.mem_store15(a, v);
                }

                // ── T register ────────────────────────────────────────────
                op::LOAD_T  => { let v = self.t; self.push(v); }
                op::STORE_T => { self.t = self.pop()?; }

                // ── Stack manipulation ─────────────────────────────────────
                op::DUP  => { let v = self.pop()?; self.push(v); self.push(v); }
                op::SWAP => { let b = self.pop()?; let a = self.pop()?; self.push(b); self.push(a); }
                op::DROP => { self.pop()?; }

                // ── TC2 arithmetic ─────────────────────────────────────────
                op::ADD  => { let b = self.pop()?; let a = self.pop()?; self.push(a.wrapping_add(b)); }
                op::SUB  => { let b = self.pop()?; let a = self.pop()?; self.push(a.wrapping_sub(b)); }
                op::AND  => { let b = self.pop()?; let a = self.pop()?; self.push(a & b); }
                op::OR   => { let b = self.pop()?; let a = self.pop()?; self.push(a | b); }
                op::XOR  => { let b = self.pop()?; let a = self.pop()?; self.push(a ^ b); }
                op::NOT  => { let x = self.pop()?; self.push(!x); }
                op::MASK15 => { let x = self.pop()?; self.push(x & 0x7FFF); }
                op::NEG  => { let x = self.pop()?; self.push(x.wrapping_neg()); }

                // ── TC2 wide integer ops (not OC-specific) ────────────────

                // Signed 16×16 → upper 15 bits of 30-bit product (= product >> 15)
                op::IMUL_HI15 => {
                    let b = self.pop()? as i16 as i32;
                    let a = self.pop()? as i16 as i32;
                    let p = a * b;
                    self.push((p >> 15) as u16);
                }
                // Lower 15 bits of 30-bit signed product (= product & 0x7FFF)
                op::IMUL_LO15 => {
                    let b = self.pop()? as i16 as i32;
                    let a = self.pop()? as i16 as i32;
                    let p = a * b;
                    self.push((p & 0x7FFF) as u16);
                }
                // Signed 15-bit divide quotient.
                // Stack: [..., a_hi (i16), a_lo (15-bit unsigned mag), divisor (i16)]
                op::IDIV_Q15 => {
                    let div = self.pop()? as i16 as i64;
                    if div == 0 { return Err(VmError::DivideByZero); }
                    let a_lo = (self.pop()? & 0x7FFF) as i64;
                    let a_hi = self.pop()? as i16 as i64;
                    let dividend = (a_hi << 15) | a_lo;
                    self.push((dividend / div) as u16);
                }
                op::IDIV_R15 => {
                    let div = self.pop()? as i16 as i64;
                    if div == 0 { return Err(VmError::DivideByZero); }
                    let a_lo = (self.pop()? & 0x7FFF) as i64;
                    let a_hi = self.pop()? as i16 as i64;
                    let dividend = (a_hi << 15) | a_lo;
                    self.push((dividend % div) as u16);
                }

                // ── OC bit-pattern predicates ──────────────────────────────
                // These check bit patterns only — the same values could be
                // interpreted as TC2; the OC meaning is context given by the lowering.

                op::IS_POS => {
                    let x = self.pop()? & 0x7FFF;
                    self.push(if oc::is_positive(x) { 1 } else { 0 });
                }
                op::IS_PLUS_ZERO => {
                    let x = self.pop()? & 0x7FFF;
                    self.push(if oc::is_plus_zero(x) { 1 } else { 0 });
                }
                op::IS_NEG => {
                    let x = self.pop()? & 0x7FFF;
                    self.push(if oc::is_negative(x) { 1 } else { 0 });
                }
                op::IS_MINUS_ZERO => {
                    let x = self.pop()? & 0x7FFF;
                    self.push(if oc::is_minus_zero(x) { 1 } else { 0 });
                }
                op::IS_ZERO_OR_NEG => {
                    let x = self.pop()? & 0x7FFF;
                    let r = oc::is_plus_zero(x) || oc::is_minus_zero(x) || oc::is_negative(x);
                    self.push(if r { 1 } else { 0 });
                }
                op::HAS_OVERFLOW => {
                    let x = self.pop()?;
                    self.push(if oc::has_overflow(x) { 1 } else { 0 });
                }
                op::BOOL_AND => {
                    let b = self.pop()?; let a = self.pop()?;
                    self.push(a & b);
                }
                op::BOOL_NOT => {
                    let x = self.pop()?;
                    self.push(if x == 0 { 1 } else { 0 });
                }

                unknown => return Err(VmError::UnknownOpcode(unknown)),
            }
        }

        Ok(())
    }
}

impl Default for BytecodeVm {
    fn default() -> Self { BytecodeVm::new() }
}
