//! Structured, machine-parseable instruction semantics representation.
//!
//! Each [`InstrSpec`] carries an optional [`SemExpr`] sequence that completely
//! describes the instruction's effect on CPU state.  This representation is:
//!
//! * **Compilable** — a typed Rust ADT; the interpreter in [`crate::exec`] evaluates it.
//! * **Machine-readable** — fully structured; no freeform strings in the execution path.
//! * **Parseable** — a compact text DSL can be parsed back into `Vec<SemOp>` via
//!   [`parse_sem`].
//!
//! # Text DSL
//!
//! ```text
//! set A mem            -- A := memory[operand_address]
//! set A oc_add(A,mem)  -- A := OC_ADD(A, memory[operand_address])
//! set Q Z              -- Q := Z
//! set Z operand        -- Z := operand address literal
//! set mem A            -- memory[operand_address] := A
//! branch_eq K          -- if A == 0: Z := operand, else Z := Z+1
//! if A_pos; set Z Z+K  -- conditional operation
//! set_flag extend      -- set EXTEND flip-flop
//! clear_flag extend    -- clear EXTEND flip-flop
//! ```
//!
//! Any line beginning with `--` or `#` is a comment.

use std::fmt;

// ─── Value destinations ───────────────────────────────────────────────────────

/// Where a value is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dest {
    /// Accumulator A (16-bit; bit 15 = overflow flag).
    A,
    /// L register.
    L,
    /// Q register (return address / remainder).
    Q,
    /// Z register (program counter).
    Z,
    /// Memory cell at the instruction's operand address.
    Mem,
    /// Memory cell at operand address + 1 (double-precision high word).
    MemHi,
    /// I/O channel at the instruction's operand address.
    Channel,
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
    /// Register Z (program counter — value *before* this instruction).
    Z,
    /// Memory word at the instruction's operand address.
    Mem,
    /// Memory word at operand address + 1.
    MemHi,
    /// I/O channel word at the instruction's operand address.
    Channel,
    /// The raw operand address (K / E / F / H field from the instruction word).
    Operand,

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
    /// Quotient of double-precision (A<<15|L) ÷ divisor.
    DivQ(Box<Expr>, Box<Expr>),
    /// Remainder of double-precision (A<<15|L) ÷ divisor.
    DivR(Box<Expr>, Box<Expr>),

    // ── Augment/Diminish ─────────────────────────────────────────────────────
    /// Move value one step away from zero (for AUG).
    Augment(Box<Expr>),
    /// Move value one step toward zero (for DIM).
    Diminish(Box<Expr>),
}

// ─── Conditions ───────────────────────────────────────────────────────────────

/// Boolean predicate on current CPU state for conditional branches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cond {
    /// A is strictly positive (> 0, not plus-zero, not negative).
    APos,
    /// A == +0 (all zeros in 15-bit field).
    APlusZero,
    /// A is strictly negative (bit 14 set, not minus-zero).
    ANeg,
    /// A == -0 (all ones in 15-bit field).
    AMinusZero,
    /// A == 0 or A < 0 (A ≤ 0, zero or negative; for BZMF).
    AZeroOrNeg,
    /// Accumulator overflow flag set.
    AOverflow,
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
/// a `Branch` or `BranchCond` op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemOp {
    /// Assign `expr` to `dest`.
    Set(Dest, Expr),

    /// Unconditionally set Z to `expr` (branch — suppresses implicit PC advance).
    Branch(Expr),

    /// Conditionally set Z.  If `cond` holds, Z := taken; otherwise Z := Z + 1.
    BranchCond {
        cond: Cond,
        taken: Expr,
    },

    /// The four-way CCS branch: test value at `operand`, store in A,
    /// then skip 0/1/2/3 instructions depending on pos/+0/neg/-0.
    CcsBranch,

    /// Set a CPU control flag.
    SetFlag(Flag),

    /// Clear a CPU control flag.
    ClearFlag(Flag),

    /// Save interrupt state (BRUPT ← current instruction, ZRUPT ← Z)
    /// and vector to the interrupt handler address.
    Interrupt,

    /// Resume from interrupt: Z ← ZRUPT, next instruction ← BRUPT.
    Resume,
}

// ─── Pretty-printing (text DSL) ───────────────────────────────────────────────

impl fmt::Display for Dest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dest::A       => f.write_str("A"),
            Dest::L       => f.write_str("L"),
            Dest::Q       => f.write_str("Q"),
            Dest::Z       => f.write_str("Z"),
            Dest::Mem     => f.write_str("mem"),
            Dest::MemHi   => f.write_str("mem_hi"),
            Dest::Channel => f.write_str("chan"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Lit(n)          => write!(f, "{:#o}", n),
            Expr::A               => f.write_str("A"),
            Expr::L               => f.write_str("L"),
            Expr::Q               => f.write_str("Q"),
            Expr::Z               => f.write_str("Z"),
            Expr::Mem             => f.write_str("mem"),
            Expr::MemHi           => f.write_str("mem_hi"),
            Expr::Channel         => f.write_str("chan"),
            Expr::Operand         => f.write_str("operand"),
            Expr::OcAdd(a, b)     => write!(f, "oc_add({a},{b})"),
            Expr::OcSub(a, b)     => write!(f, "oc_sub({a},{b})"),
            Expr::OcNeg(x)        => write!(f, "oc_neg({x})"),
            Expr::And(a, b)       => write!(f, "and({a},{b})"),
            Expr::Or(a, b)        => write!(f, "or({a},{b})"),
            Expr::Xor(a, b)       => write!(f, "xor({a},{b})"),
            Expr::Not(x)          => write!(f, "not({x})"),
            Expr::MulHi(a, b)     => write!(f, "mul_hi({a},{b})"),
            Expr::MulLo(a, b)     => write!(f, "mul_lo({a},{b})"),
            Expr::DivQ(a, b)      => write!(f, "div_q({a},{b})"),
            Expr::DivR(a, b)      => write!(f, "div_r({a},{b})"),
            Expr::Augment(x)      => write!(f, "aug({x})"),
            Expr::Diminish(x)     => write!(f, "dim({x})"),
        }
    }
}

impl fmt::Display for Cond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cond::APos        => f.write_str("A_pos"),
            Cond::APlusZero   => f.write_str("A_plus_zero"),
            Cond::ANeg        => f.write_str("A_neg"),
            Cond::AMinusZero  => f.write_str("A_minus_zero"),
            Cond::AZeroOrNeg  => f.write_str("A_zero_or_neg"),
            Cond::AOverflow   => f.write_str("A_overflow"),
            Cond::And(a, b)   => write!(f, "and({a},{b})"),
            Cond::Not(c)      => write!(f, "not({c})"),
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
            SemOp::Set(d, e)               => write!(f, "set {d} {e}"),
            SemOp::Branch(e)               => write!(f, "branch {e}"),
            SemOp::BranchCond { cond, taken } => write!(f, "branch_if {cond} {taken}"),
            SemOp::CcsBranch               => f.write_str("ccs_branch"),
            SemOp::SetFlag(fl)             => write!(f, "set_flag {fl}"),
            SemOp::ClearFlag(fl)           => write!(f, "clear_flag {fl}"),
            SemOp::Interrupt               => f.write_str("interrupt"),
            SemOp::Resume                  => f.write_str("resume"),
        }
    }
}

/// Serialize a semantics sequence to the text DSL.
pub fn sem_to_text(ops: &[SemOp]) -> String {
    ops.iter().map(|op| op.to_string()).collect::<Vec<_>>().join("\n")
}

// ─── Text DSL parser ──────────────────────────────────────────────────────────

/// Parse a newline-delimited semantics DSL string into a `Vec<SemOp>`.
///
/// Returns `Err(message)` on the first unrecognised line.
pub fn parse_sem(text: &str) -> Result<Vec<SemOp>, String> {
    let mut ops = Vec::new();
    for (lineno, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        // Skip comments and blank lines
        if line.is_empty() || line.starts_with("--") || line.starts_with('#') {
            continue;
        }
        let op = parse_op(line)
            .map_err(|e| format!("line {}: {}: {:?}", lineno + 1, e, line))?;
        ops.push(op);
    }
    Ok(ops)
}

fn parse_op(s: &str) -> Result<SemOp, &'static str> {
    // Tokenise by the first whitespace to get the verb
    let (verb, rest) = split_first_word(s);
    match verb {
        "set" => {
            let (dest_s, expr_s) = split_first_word(rest);
            let dest = parse_dest(dest_s)?;
            let expr = parse_expr(expr_s.trim())?;
            Ok(SemOp::Set(dest, expr))
        }
        "branch" => {
            let expr = parse_expr(rest.trim())?;
            Ok(SemOp::Branch(expr))
        }
        "branch_if" => {
            let (cond_s, target_s) = split_first_word(rest);
            let cond  = parse_cond(cond_s)?;
            let taken = parse_expr(target_s.trim())?;
            Ok(SemOp::BranchCond { cond, taken })
        }
        "ccs_branch"  => Ok(SemOp::CcsBranch),
        "set_flag"    => Ok(SemOp::SetFlag(parse_flag(rest.trim())?)),
        "clear_flag"  => Ok(SemOp::ClearFlag(parse_flag(rest.trim())?)),
        "interrupt"   => Ok(SemOp::Interrupt),
        "resume"      => Ok(SemOp::Resume),
        _             => Err("unknown semantic verb"),
    }
}

fn parse_dest(s: &str) -> Result<Dest, &'static str> {
    match s {
        "A"       => Ok(Dest::A),
        "L"       => Ok(Dest::L),
        "Q"       => Ok(Dest::Q),
        "Z"       => Ok(Dest::Z),
        "mem"     => Ok(Dest::Mem),
        "mem_hi"  => Ok(Dest::MemHi),
        "chan"     => Ok(Dest::Channel),
        _         => Err("unknown destination"),
    }
}

fn parse_flag(s: &str) -> Result<Flag, &'static str> {
    match s {
        "extend" => Ok(Flag::Extend),
        "inhint" => Ok(Flag::Inhint),
        _        => Err("unknown flag"),
    }
}

fn parse_cond(s: &str) -> Result<Cond, &'static str> {
    match s {
        "A_pos"         => Ok(Cond::APos),
        "A_plus_zero"   => Ok(Cond::APlusZero),
        "A_neg"         => Ok(Cond::ANeg),
        "A_minus_zero"  => Ok(Cond::AMinusZero),
        "A_zero_or_neg" => Ok(Cond::AZeroOrNeg),
        "A_overflow"    => Ok(Cond::AOverflow),
        _               => Err("unknown condition"),
    }
}

/// Recursive expression parser (handles nested parenthesised calls).
pub fn parse_expr(s: &str) -> Result<Expr, &'static str> {
    let s = s.trim();

    // ── Atoms ──────────────────────────────────────────────────────────────
    match s {
        "A"       => return Ok(Expr::A),
        "L"       => return Ok(Expr::L),
        "Q"       => return Ok(Expr::Q),
        "Z"       => return Ok(Expr::Z),
        "mem"     => return Ok(Expr::Mem),
        "mem_hi"  => return Ok(Expr::MemHi),
        "chan"     => return Ok(Expr::Channel),
        "operand" => return Ok(Expr::Operand),
        _         => {}
    }

    // ── Octal / decimal literal ────────────────────────────────────────────
    if s.starts_with("0o") || s.starts_with("0O") {
        let n = u16::from_str_radix(&s[2..], 8).map_err(|_| "bad octal literal")?;
        return Ok(Expr::Lit(n));
    }
    if let Ok(n) = s.parse::<u16>() {
        return Ok(Expr::Lit(n));
    }

    // ── Function call: verb(args) ──────────────────────────────────────────
    if let Some(paren) = s.find('(') {
        if s.ends_with(')') {
            let verb = &s[..paren];
            let inner = &s[paren + 1..s.len() - 1];

            // Unary
            match verb {
                "oc_neg"  => return Ok(Expr::OcNeg(Box::new(parse_expr(inner)?))),
                "not"     => return Ok(Expr::Not(Box::new(parse_expr(inner)?))),
                "aug"     => return Ok(Expr::Augment(Box::new(parse_expr(inner)?))),
                "dim"     => return Ok(Expr::Diminish(Box::new(parse_expr(inner)?))),
                _         => {}
            }

            // Binary — split inner at the top-level comma
            let mid = top_level_comma(inner).ok_or("expected two arguments")?;
            let lhs = parse_expr(&inner[..mid])?;
            let rhs = parse_expr(&inner[mid + 1..])?;
            return match verb {
                "oc_add"  => Ok(Expr::OcAdd(Box::new(lhs), Box::new(rhs))),
                "oc_sub"  => Ok(Expr::OcSub(Box::new(lhs), Box::new(rhs))),
                "and"     => Ok(Expr::And(Box::new(lhs), Box::new(rhs))),
                "or"      => Ok(Expr::Or(Box::new(lhs), Box::new(rhs))),
                "xor"     => Ok(Expr::Xor(Box::new(lhs), Box::new(rhs))),
                "mul_hi"  => Ok(Expr::MulHi(Box::new(lhs), Box::new(rhs))),
                "mul_lo"  => Ok(Expr::MulLo(Box::new(lhs), Box::new(rhs))),
                "div_q"   => Ok(Expr::DivQ(Box::new(lhs), Box::new(rhs))),
                "div_r"   => Ok(Expr::DivR(Box::new(lhs), Box::new(rhs))),
                _         => Err("unknown expression function"),
            };
        }
    }

    Err("unrecognised expression")
}

/// Find the index of the top-level comma in a string (not inside parens).
fn top_level_comma(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => { if depth > 0 { depth -= 1; } }
            ',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();
    if let Some(pos) = s.find(|c: char| c.is_whitespace()) {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s, "")
    }
}

// ─── Built-in semantics table ─────────────────────────────────────────────────

/// Return the canonical semantics text (DSL) for a given [`crate::spec::InstrType`].
///
/// Used by [`crate::spec::InstrSpec`] builder helpers and by tests.
pub fn builtin_sem_text(mnemonic: &str) -> Option<&'static str> {
    match mnemonic {
        "TC" => Some(
            "set Q Z\n\
             branch operand"
        ),
        "TCF" => Some(
            "branch operand"
        ),
        "CCS" => Some(
            "ccs_branch"
        ),
        "BZF" => Some(
            "branch_if A_plus_zero operand"
        ),
        "BZMF" => Some(
            "branch_if A_zero_or_neg operand"
        ),
        "CA" => Some(
            "set A mem"
        ),
        "CS" => Some(
            "set A oc_neg(mem)"
        ),
        "DCA" => Some(
            "set L mem\n\
             set A mem_hi"
        ),
        "DCS" => Some(
            "set L oc_neg(mem)\n\
             set A oc_neg(mem_hi)"
        ),
        "TS" => Some(
            "set mem A\n\
             branch_if A_overflow operand"
        ),
        "XCH" => Some(
            "set mem A\n\
             set A mem"  // Note: executor handles atomicity
        ),
        "LXCH" => Some(
            "set mem L\n\
             set L mem"
        ),
        "QXCH" => Some(
            "set mem Q\n\
             set Q mem"
        ),
        "DXCH" => Some(
            "set mem A\n\
             set mem_hi L\n\
             set A mem\n\
             set L mem_hi"
        ),
        "NDX" => Some(
            "set A mem"  // Executor applies index to next instruction word
        ),
        "AD" => Some(
            "set A oc_add(A,mem)"
        ),
        "SU" => Some(
            "set A oc_sub(A,mem)"
        ),
        "MP" => Some(
            "set A mul_hi(A,mem)\n\
             set L mul_lo(A,mem)"
        ),
        "DV" => Some(
            "set A div_q(A,mem)\n\
             set L div_r(A,mem)"
        ),
        "ADS" => Some(
            "set mem oc_add(A,mem)\n\
             set A mem"
        ),
        "DAS" => Some(
            "set mem oc_add(L,mem)\n\
             set mem_hi oc_add(A,mem_hi)"
        ),
        "INCR" => Some(
            "set mem oc_add(mem,0o1)"
        ),
        "AUG" => Some(
            "set mem aug(mem)"
        ),
        "DIM" => Some(
            "set mem dim(mem)"
        ),
        "MSU" => Some(
            "set A oc_sub(A,mem)"
        ),
        "MSK" => Some(
            "set A and(A,mem)"
        ),
        "READ" => Some(
            "set A chan"
        ),
        "WRITE" => Some(
            "set chan A"
        ),
        "RAND" => Some(
            "set A and(A,chan)"
        ),
        "WAND" => Some(
            "set chan and(A,chan)"
        ),
        "ROR" => Some(
            "set A or(A,chan)"
        ),
        "WOR" => Some(
            "set chan or(A,chan)"
        ),
        "RXOR" => Some(
            "set A xor(A,chan)"
        ),
        "EXTEND" => Some(
            "set_flag extend"
        ),
        "INHINT" => Some(
            "set_flag inhint"
        ),
        "RELINT" => Some(
            "clear_flag inhint"
        ),
        "RESUME" => Some(
            "resume"
        ),
        "RUPT" => Some(
            "interrupt"
        ),
        "GO" => Some(
            "branch 0o4000"
        ),
        // Counter instructions — each increments/decrements the counter cell
        "PINC"  => Some("set mem oc_add(mem,0o1)"),
        "MINC"  => Some("set mem oc_sub(mem,0o1)"),
        "DINC"  => Some("set mem dim(mem)"),
        "PCDU"  => Some("set mem oc_add(mem,0o1)"),
        "MCDU"  => Some("set mem oc_sub(mem,0o1)"),
        "SHINC" => Some("set mem oc_add(mem,0o1)"),
        "SHANC" => Some("set mem oc_add(mem,0o1)"),
        _ => None,
    }
}

/// Parse the canonical semantics for a mnemonic into a `Vec<SemOp>`.
pub fn builtin_sem(mnemonic: &str) -> Option<Vec<SemOp>> {
    builtin_sem_text(mnemonic).and_then(|t| parse_sem(t).ok())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_simple() {
        let text = "set A mem\nset Z operand";
        let ops  = parse_sem(text).unwrap();
        assert_eq!(ops.len(), 2);
        let back = sem_to_text(&ops);
        assert_eq!(back, text);
    }

    #[test]
    fn round_trip_nested() {
        let text = "set A oc_add(A,mem)";
        let ops  = parse_sem(text).unwrap();
        let back = sem_to_text(&ops);
        assert_eq!(back, text);
    }

    #[test]
    fn parse_branch_if() {
        let text = "branch_if A_plus_zero operand";
        let ops  = parse_sem(text).unwrap();
        assert_eq!(ops.len(), 1);
        assert!(matches!(&ops[0], SemOp::BranchCond { cond: Cond::APlusZero, .. }));
    }

    #[test]
    fn parse_flags() {
        let ops = parse_sem("set_flag extend\nclear_flag inhint").unwrap();
        assert_eq!(ops[0], SemOp::SetFlag(Flag::Extend));
        assert_eq!(ops[1], SemOp::ClearFlag(Flag::Inhint));
    }

    #[test]
    fn builtin_tc_semantics() {
        let ops = builtin_sem("TC").unwrap();
        assert_eq!(ops.len(), 2);
        assert!(matches!(&ops[0], SemOp::Set(Dest::Q, Expr::Z)));
        assert!(matches!(&ops[1], SemOp::Branch(Expr::Operand)));
    }

    #[test]
    fn comments_ignored() {
        let text = "-- this is a comment\n# also ignored\nset A mem";
        let ops  = parse_sem(text).unwrap();
        assert_eq!(ops.len(), 1);
    }
}
