//! Semantics DSL types — fully generic, no special-case variants.
//!
//! All instruction behavior is expressed via:
//! - `Set(Dest, Expr)` — assign expression to destination
//! - `Branch(Expr)` — unconditional branch
//! - `BranchIf(Cond, Expr)` — conditional branch
//! - `SetFlag(Flag)` — set a CPU control flag
//! - `ClearFlag(Flag)` — clear a CPU control flag

use alloc::boxed::Box;
use core::fmt;

// ─── Value destinations ───────────────────────────────────────────────────────

/// Where a value is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dest {
    /// Accumulator A (16-bit; bit 15 = overflow flag).
    A,
    /// L register.
    L,
    /// Q register (return address / remainder).
    Q,
    /// Z register (program counter).
    Z,
    /// Temporary (non-addressable) register.
    Tmp,
    /// Memory cell at the instruction's operand address.
    Mem,
    /// Memory cell at operand address + 1 (double-precision high word).
    MemHi,
    /// I/O channel at the instruction's operand address.
    Channel,
    /// Memory cell at a literal address (e.g. BRUPT/ZRUPT writes).
    MemAt(u16),
    /// Memory cell at the address computed by eval(expr) — for NDX.
    Deref(Box<Expr>),
}

// ─── Value expressions ────────────────────────────────────────────────────────

/// A side-effect-free expression producing a 15-bit (or 16-bit for A) value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    // ── Terminals ─────────────────────────────────────────────────────────────
    /// Constant ones-complement literal.
    Lit(u16),
    /// Register A (full 16-bit including overflow).
    A,
    /// Register L.
    L,
    /// Register Q.
    Q,
    /// Register Z (program counter — value after fetch/PC-advance).
    Z,
    /// Temporary register.
    Tmp,
    /// Memory word at the instruction's operand address.
    Mem,
    /// Memory word at operand address + 1.
    MemHi,
    /// I/O channel word at the instruction's operand address.
    Channel,
    /// The raw operand address (K / E / F / H field from the instruction word).
    Operand,
    /// Z + offset — PC-relative address (post-fetch PC).
    PcRel(i8),
    /// Read from memory at a literal address (e.g. ZRUPT at 0o16).
    MemAt(u16),
    /// Read from memory at address eval(expr) — for NDX.
    Deref(Box<Expr>),
    /// The raw instruction word being executed.
    InstrWord,

    // ── Ones-complement arithmetic ─────────────────────────────────────────
    /// Ones-complement addition (with end-around carry).
    OcAdd(Box<Expr>, Box<Expr>),
    /// Ones-complement subtraction: `a - b`.
    OcSub(Box<Expr>, Box<Expr>),
    /// Ones-complement negation: `~x & 0x7FFF`.
    OcNeg(Box<Expr>),

    // ── Bitwise ───────────────────────────────────────────────────────────────
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),

    // ── Double-precision (30-bit) ─────────────────────────────────────────────
    /// High 15 bits of A × B.
    MulHi(Box<Expr>, Box<Expr>),
    /// Low 15 bits of A × B.
    MulLo(Box<Expr>, Box<Expr>),
    /// Quotient of double-precision (a_hi, a_lo) / divisor.
    DivQ(Box<Expr>, Box<Expr>, Box<Expr>),
    /// Remainder of double-precision (a_hi, a_lo) / divisor.
    DivR(Box<Expr>, Box<Expr>, Box<Expr>),
    /// High word of double-precision addition with carry propagation.
    DpAddHi(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),

    // ── Augment/Diminish/Saturate ─────────────────────────────────────────────
    /// Move value one step away from zero (for AUG).
    Augment(Box<Expr>),
    /// Move value one step toward zero (for DIM).
    Diminish(Box<Expr>),
    /// Clamp to ±max if overflow, else passthrough (for TS).
    Saturate(Box<Expr>),
}

// ─── Conditions ───────────────────────────────────────────────────────────────

/// Boolean predicate on current CPU state for conditional branches.
/// All conditions are expression-based — no hard-coded "A" references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cond {
    /// value > 0 (bit14=0, not zero).
    IsPositive(Box<Expr>),
    /// value == 0x0000 (plus-zero).
    IsPlusZero(Box<Expr>),
    /// bit14=1 AND not minus-zero.
    IsNegative(Box<Expr>),
    /// value == 0x7FFF (minus-zero).
    IsMinusZero(Box<Expr>),
    /// zero or negative (for BZMF).
    IsZeroOrNeg(Box<Expr>),
    /// bits 14 and 15 differ in value (overflow, for TS).
    HasOverflow(Box<Expr>),
    /// Logical AND of two conditions.
    And(Box<Cond>, Box<Cond>),
    /// Logical NOT.
    Not(Box<Cond>),
}

// ─── Control flags ────────────────────────────────────────────────────────────

/// CPU control flip-flops that can be set/cleared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// EXTEND flip-flop — next instruction is extracode.
    Extend,
    /// Interrupt-inhibit flag.
    Inhint,
}

// ─── Semantic operations ──────────────────────────────────────────────────────

/// A single semantic operation that modifies CPU state.
///
/// An instruction's full behavior is a `Vec<SemOp>` executed in order.
/// PC advance (`Z := Z + 1`) is implicit at the *end* unless overridden by
/// a `Branch` or `BranchIf` op (which short-circuits further evaluation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemOp {
    /// Assign `expr` to `dest`.
    Set(Dest, Expr),
    /// Unconditionally set Z to `expr` (suppresses implicit PC advance).
    Branch(Expr),
    /// Conditionally set Z. If `cond` holds, Z := target; else Z stays (already advanced).
    BranchIf(Cond, Expr),
    /// Set a CPU control flag.
    SetFlag(Flag),
    /// Clear a CPU control flag.
    ClearFlag(Flag),
}

// ─── Pretty-printing (text DSL) ───────────────────────────────────────────────

impl fmt::Display for Dest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dest::A           => f.write_str("A"),
            Dest::L           => f.write_str("L"),
            Dest::Q           => f.write_str("Q"),
            Dest::Z           => f.write_str("Z"),
            Dest::Tmp         => f.write_str("tmp"),
            Dest::Mem         => f.write_str("mem"),
            Dest::MemHi       => f.write_str("mem_hi"),
            Dest::Channel     => f.write_str("chan"),
            Dest::MemAt(a)    => write!(f, "mem_at({:#o})", a),
            Dest::Deref(e)    => write!(f, "deref({e})"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Lit(n)              => write!(f, "{:#o}", n),
            Expr::A                   => f.write_str("A"),
            Expr::L                   => f.write_str("L"),
            Expr::Q                   => f.write_str("Q"),
            Expr::Z                   => f.write_str("Z"),
            Expr::Tmp                 => f.write_str("tmp"),
            Expr::Mem                 => f.write_str("mem"),
            Expr::MemHi               => f.write_str("mem_hi"),
            Expr::Channel             => f.write_str("chan"),
            Expr::Operand             => f.write_str("operand"),
            Expr::PcRel(n)            => write!(f, "pc_rel({})", n),
            Expr::MemAt(a)            => write!(f, "mem_at({:#o})", a),
            Expr::Deref(e)            => write!(f, "deref({e})"),
            Expr::InstrWord           => f.write_str("instr_word"),
            Expr::OcAdd(a, b)         => write!(f, "oc_add({a},{b})"),
            Expr::OcSub(a, b)         => write!(f, "oc_sub({a},{b})"),
            Expr::OcNeg(x)            => write!(f, "oc_neg({x})"),
            Expr::And(a, b)           => write!(f, "and({a},{b})"),
            Expr::Or(a, b)            => write!(f, "or({a},{b})"),
            Expr::Xor(a, b)           => write!(f, "xor({a},{b})"),
            Expr::Not(x)              => write!(f, "not({x})"),
            Expr::MulHi(a, b)         => write!(f, "mul_hi({a},{b})"),
            Expr::MulLo(a, b)         => write!(f, "mul_lo({a},{b})"),
            Expr::DivQ(a, b, c)       => write!(f, "div_q({a},{b},{c})"),
            Expr::DivR(a, b, c)       => write!(f, "div_r({a},{b},{c})"),
            Expr::DpAddHi(a, b, c, d) => write!(f, "dp_add_hi({a},{b},{c},{d})"),
            Expr::Augment(x)          => write!(f, "aug({x})"),
            Expr::Diminish(x)         => write!(f, "dim({x})"),
            Expr::Saturate(x)         => write!(f, "sat({x})"),
        }
    }
}

impl fmt::Display for Cond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cond::IsPositive(e)   => write!(f, "is_pos({e})"),
            Cond::IsPlusZero(e)   => write!(f, "is_plus_zero({e})"),
            Cond::IsNegative(e)   => write!(f, "is_neg({e})"),
            Cond::IsMinusZero(e)  => write!(f, "is_minus_zero({e})"),
            Cond::IsZeroOrNeg(e)  => write!(f, "is_zero_or_neg({e})"),
            Cond::HasOverflow(e)  => write!(f, "has_overflow({e})"),
            Cond::And(a, b)       => write!(f, "and({a},{b})"),
            Cond::Not(c)          => write!(f, "not({c})"),
        }
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Flag::Extend => f.write_str("extend"),
            Flag::Inhint => f.write_str("inhint"),
        }
    }
}

impl fmt::Display for SemOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemOp::Set(d, e)       => write!(f, "set {d} {e}"),
            SemOp::Branch(e)       => write!(f, "branch {e}"),
            SemOp::BranchIf(c, e)  => write!(f, "branch_if {c} {e}"),
            SemOp::SetFlag(fl)     => write!(f, "set_flag {fl}"),
            SemOp::ClearFlag(fl)   => write!(f, "clear_flag {fl}"),
        }
    }
}
