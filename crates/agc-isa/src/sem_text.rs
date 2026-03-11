//! Text DSL parser and serializer for the AGC semantics DSL.
//!
//! # Text DSL format
//!
//! - `set A mem` — Set dest to expr
//! - `branch operand` — Unconditional branch
//! - `branch_if is_pos(A) pc_rel(0)` — Conditional branch
//! - `set_flag extend` — Set flag
//! - `clear_flag inhint` — Clear flag
//! - Comments: lines starting with `--` or `#`
//!
//! ## Expr terminals
//! `A`, `L`, `Q`, `Z`, `tmp`, `mem`, `mem_hi`, `chan`, `operand`, `instr_word`,
//! `pc_rel(n)`, `mem_at(0o17)`, `deref(Z)`, `0o12345` (octal literals), decimal literals
//!
//! ## Expr functions
//! `oc_add(a,b)`, `oc_sub(a,b)`, `oc_neg(x)`, `and(a,b)`, `or(a,b)`, `xor(a,b)`,
//! `not(x)`, `mul_hi(a,b)`, `mul_lo(a,b)`, `div_q(a,b,c)`, `div_r(a,b,c)`,
//! `dp_add_hi(a,b,c,d)`, `aug(x)`, `dim(x)`, `sat(x)`
//!
//! ## Cond functions
//! `is_pos(e)`, `is_plus_zero(e)`, `is_neg(e)`, `is_minus_zero(e)`,
//! `is_zero_or_neg(e)`, `has_overflow(e)`, `and(c1,c2)`, `not(c)`

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use crate::semantics::{Cond, Dest, Expr, Flag, SemOp};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Parse a newline-delimited semantics DSL string into a `Vec<SemOp>`.
///
/// Returns `Err(message)` on the first unrecognised line.
pub fn parse_sem(text: &str) -> Result<Vec<SemOp>, &'static str> {
    let mut ops = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        // Skip comments and blank lines
        if line.is_empty() || line.starts_with("--") || line.starts_with('#') {
            continue;
        }
        let op = parse_op(line)?;
        ops.push(op);
    }
    Ok(ops)
}

/// Serialize a semantics sequence to the text DSL.
pub fn sem_to_text(ops: &[SemOp]) -> String {
    use core::fmt::Write;
    let mut out = String::new();
    for (i, op) in ops.iter().enumerate() {
        if i > 0 { out.push('\n'); }
        let _ = write!(out, "{}", op);
    }
    out
}

// ─── Operation parser ─────────────────────────────────────────────────────────

fn parse_op(s: &str) -> Result<SemOp, &'static str> {
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
            Ok(SemOp::BranchIf(cond, taken))
        }
        "set_flag"   => Ok(SemOp::SetFlag(parse_flag(rest.trim())?)),
        "clear_flag" => Ok(SemOp::ClearFlag(parse_flag(rest.trim())?)),
        _            => Err("unknown semantic verb"),
    }
}

// ─── Destination parser ───────────────────────────────────────────────────────

fn parse_dest(s: &str) -> Result<Dest, &'static str> {
    match s {
        "A"       => return Ok(Dest::A),
        "L"       => return Ok(Dest::L),
        "Q"       => return Ok(Dest::Q),
        "Z"       => return Ok(Dest::Z),
        "tmp"     => return Ok(Dest::Tmp),
        "mem"     => return Ok(Dest::Mem),
        "mem_hi"  => return Ok(Dest::MemHi),
        "chan"       => return Ok(Dest::Channel),
        "next_instr" => return Ok(Dest::NextInstr),
        _            => {}
    }
    // Function-form destinations: mem_at(N) or deref(expr)
    if let Some(paren) = s.find('(') {
        if s.ends_with(')') {
            let verb = &s[..paren];
            let inner = &s[paren + 1..s.len() - 1];
            match verb {
                "mem_at" => {
                    let n = parse_u16_literal(inner)?;
                    return Ok(Dest::MemAt(n));
                }
                "deref" => {
                    let e = parse_expr(inner)?;
                    return Ok(Dest::Deref(Box::new(e)));
                }
                _ => {}
            }
        }
    }
    Err("unknown destination")
}

// ─── Flag parser ──────────────────────────────────────────────────────────────

fn parse_flag(s: &str) -> Result<Flag, &'static str> {
    match s {
        "extend" => Ok(Flag::Extend),
        "inhint" => Ok(Flag::Inhint),
        _        => Err("unknown flag"),
    }
}

// ─── Condition parser ─────────────────────────────────────────────────────────

pub fn parse_cond(s: &str) -> Result<Cond, &'static str> {
    let s = s.trim();
    // All conditions are function-form: verb(args)
    let paren = s.find('(').ok_or("expected condition function")?;
    if !s.ends_with(')') { return Err("unclosed paren in condition"); }
    let verb = &s[..paren];
    let inner = &s[paren + 1..s.len() - 1];

    match verb {
        "is_pos"         => Ok(Cond::IsPositive(Box::new(parse_expr(inner)?))),
        "is_plus_zero"   => Ok(Cond::IsPlusZero(Box::new(parse_expr(inner)?))),
        "is_neg"         => Ok(Cond::IsNegative(Box::new(parse_expr(inner)?))),
        "is_minus_zero"  => Ok(Cond::IsMinusZero(Box::new(parse_expr(inner)?))),
        "is_zero_or_neg" => Ok(Cond::IsZeroOrNeg(Box::new(parse_expr(inner)?))),
        "has_overflow"   => Ok(Cond::HasOverflow(Box::new(parse_expr(inner)?))),
        "and" => {
            let mid = top_level_comma(inner).ok_or("expected two args to and()")?;
            let lhs = parse_cond(&inner[..mid])?;
            let rhs = parse_cond(&inner[mid + 1..])?;
            Ok(Cond::And(Box::new(lhs), Box::new(rhs)))
        }
        "not" => Ok(Cond::Not(Box::new(parse_cond(inner)?))),
        _ => Err("unknown condition function"),
    }
}

// ─── Expression parser ────────────────────────────────────────────────────────

/// Recursive expression parser (handles nested parenthesised calls).
pub fn parse_expr(s: &str) -> Result<Expr, &'static str> {
    let s = s.trim();

    // ── Atoms ──────────────────────────────────────────────────────────────
    match s {
        "A"          => return Ok(Expr::A),
        "L"          => return Ok(Expr::L),
        "Q"          => return Ok(Expr::Q),
        "Z"          => return Ok(Expr::Z),
        "tmp"        => return Ok(Expr::Tmp),
        "mem"        => return Ok(Expr::Mem),
        "mem_hi"     => return Ok(Expr::MemHi),
        "chan"        => return Ok(Expr::Channel),
        "operand"    => return Ok(Expr::Operand),
        "instr_word" => return Ok(Expr::InstrWord),
        _            => {}
    }

    // ── Numeric literal ────────────────────────────────────────────────────
    if s.starts_with("0o") || s.starts_with("0O") {
        let n = parse_u16_octal(&s[2..])?;
        return Ok(Expr::Lit(n));
    }
    if let Ok(n) = parse_u16_decimal(s) {
        return Ok(Expr::Lit(n));
    }

    // ── Function call: verb(args) ──────────────────────────────────────────
    if let Some(paren) = s.find('(') {
        if s.ends_with(')') {
            let verb = &s[..paren];
            let inner = &s[paren + 1..s.len() - 1];

            // Unary functions
            match verb {
                "oc_neg" => return Ok(Expr::OcNeg(Box::new(parse_expr(inner)?))),
                "not"    => return Ok(Expr::Not(Box::new(parse_expr(inner)?))),
                "aug"    => return Ok(Expr::Augment(Box::new(parse_expr(inner)?))),
                "dim"    => return Ok(Expr::Diminish(Box::new(parse_expr(inner)?))),
                "sat"    => return Ok(Expr::Saturate(Box::new(parse_expr(inner)?))),
                "deref"  => return Ok(Expr::Deref(Box::new(parse_expr(inner)?))),
                "pc_rel" => {
                    let n = parse_i8(inner)?;
                    return Ok(Expr::PcRel(n));
                }
                "mem_at" => {
                    let n = parse_u16_literal(inner)?;
                    return Ok(Expr::MemAt(n));
                }
                _ => {}
            }

            // Binary functions
            let mid = top_level_comma(inner).ok_or("expected two arguments")?;
            let lhs_s = &inner[..mid];
            let rhs_s = &inner[mid + 1..];

            match verb {
                "oc_add"  => {
                    let lhs = parse_expr(lhs_s)?;
                    let rhs = parse_expr(rhs_s)?;
                    return Ok(Expr::OcAdd(Box::new(lhs), Box::new(rhs)));
                }
                "oc_sub"  => {
                    let lhs = parse_expr(lhs_s)?;
                    let rhs = parse_expr(rhs_s)?;
                    return Ok(Expr::OcSub(Box::new(lhs), Box::new(rhs)));
                }
                "and"     => {
                    let lhs = parse_expr(lhs_s)?;
                    let rhs = parse_expr(rhs_s)?;
                    return Ok(Expr::And(Box::new(lhs), Box::new(rhs)));
                }
                "or"      => {
                    let lhs = parse_expr(lhs_s)?;
                    let rhs = parse_expr(rhs_s)?;
                    return Ok(Expr::Or(Box::new(lhs), Box::new(rhs)));
                }
                "xor"     => {
                    let lhs = parse_expr(lhs_s)?;
                    let rhs = parse_expr(rhs_s)?;
                    return Ok(Expr::Xor(Box::new(lhs), Box::new(rhs)));
                }
                "mul_hi"  => {
                    let lhs = parse_expr(lhs_s)?;
                    let rhs = parse_expr(rhs_s)?;
                    return Ok(Expr::MulHi(Box::new(lhs), Box::new(rhs)));
                }
                "mul_lo"  => {
                    let lhs = parse_expr(lhs_s)?;
                    let rhs = parse_expr(rhs_s)?;
                    return Ok(Expr::MulLo(Box::new(lhs), Box::new(rhs)));
                }
                // 3-argument functions: div_q(a,b,c), div_r(a,b,c)
                "div_q" | "div_r" => {
                    // rhs_s contains "b,c"
                    let mid2 = top_level_comma(rhs_s).ok_or("div_q/div_r needs 3 args")?;
                    let a = parse_expr(lhs_s)?;
                    let b = parse_expr(&rhs_s[..mid2])?;
                    let c = parse_expr(&rhs_s[mid2 + 1..])?;
                    if verb == "div_q" {
                        return Ok(Expr::DivQ(Box::new(a), Box::new(b), Box::new(c)));
                    } else {
                        return Ok(Expr::DivR(Box::new(a), Box::new(b), Box::new(c)));
                    }
                }
                // 4-argument function: dp_add_hi(a,b,c,d)
                "dp_add_hi" => {
                    let mid2 = top_level_comma(rhs_s).ok_or("dp_add_hi needs 4 args")?;
                    let rest2 = &rhs_s[mid2 + 1..];
                    let mid3 = top_level_comma(rest2).ok_or("dp_add_hi needs 4 args")?;
                    let a = parse_expr(lhs_s)?;
                    let b = parse_expr(&rhs_s[..mid2])?;
                    let c = parse_expr(&rest2[..mid3])?;
                    let d = parse_expr(&rest2[mid3 + 1..])?;
                    return Ok(Expr::DpAddHi(Box::new(a), Box::new(b), Box::new(c), Box::new(d)));
                }
                _ => return Err("unknown expression function"),
            }
        }
    }

    Err("unrecognised expression")
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Find the index of the top-level (not nested) comma.
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
        (&s[..pos], s[pos + 1..].trim_start())
    } else {
        (s, "")
    }
}

fn parse_u16_literal(s: &str) -> Result<u16, &'static str> {
    let s = s.trim();
    if s.starts_with("0o") || s.starts_with("0O") {
        parse_u16_octal(&s[2..])
    } else {
        parse_u16_decimal(s).map_err(|_| "bad integer literal")
    }
}

fn parse_u16_octal(s: &str) -> Result<u16, &'static str> {
    let mut result: u32 = 0;
    for c in s.chars() {
        if !c.is_ascii_digit() || c > '7' { return Err("bad octal digit"); }
        result = result * 8 + (c as u32 - '0' as u32);
        if result > 0xFFFF { return Err("octal literal out of range"); }
    }
    Ok(result as u16)
}

fn parse_u16_decimal(s: &str) -> Result<u16, ()> {
    let mut result: u32 = 0;
    if s.is_empty() { return Err(()); }
    for c in s.chars() {
        if !c.is_ascii_digit() { return Err(()); }
        result = result * 10 + (c as u32 - '0' as u32);
        if result > 0xFFFF { return Err(()); }
    }
    Ok(result as u16)
}

fn parse_i8(s: &str) -> Result<i8, &'static str> {
    let s = s.trim();
    let (neg, digits) = if s.starts_with('-') { (true, &s[1..]) } else { (false, s) };
    let mut result: i32 = 0;
    for c in digits.chars() {
        if !c.is_ascii_digit() { return Err("bad i8 literal"); }
        result = result * 10 + (c as i32 - '0' as i32);
    }
    if neg { result = -result; }
    if result < -128 || result > 127 { return Err("i8 out of range"); }
    Ok(result as i8)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantics::{Dest, Expr, SemOp, Cond, Flag};

    #[test]
    fn round_trip_simple() {
        let text = "set A mem\nbranch operand";
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
    fn parse_branch_if_is_plus_zero() {
        let text = "branch_if is_plus_zero(A) operand";
        let ops  = parse_sem(text).unwrap();
        assert_eq!(ops.len(), 1);
        assert!(matches!(&ops[0], SemOp::BranchIf(Cond::IsPlusZero(_), _)));
    }

    #[test]
    fn parse_pc_rel() {
        let text = "branch_if is_pos(A) pc_rel(0)";
        let ops  = parse_sem(text).unwrap();
        assert!(matches!(&ops[0], SemOp::BranchIf(Cond::IsPositive(_), Expr::PcRel(0))));
    }

    #[test]
    fn parse_flags() {
        let ops = parse_sem("set_flag extend\nclear_flag inhint").unwrap();
        assert_eq!(ops[0], SemOp::SetFlag(Flag::Extend));
        assert_eq!(ops[1], SemOp::ClearFlag(Flag::Inhint));
    }

    #[test]
    fn comments_ignored() {
        let text = "-- this is a comment\n# also ignored\nset A mem";
        let ops  = parse_sem(text).unwrap();
        assert_eq!(ops.len(), 1);
    }

    #[test]
    fn parse_mem_at() {
        let text = "set mem_at(0o16) Z";
        let ops  = parse_sem(text).unwrap();
        assert_eq!(ops.len(), 1);
        assert!(matches!(&ops[0], SemOp::Set(Dest::MemAt(0o16), Expr::Z)));
    }

    #[test]
    fn parse_div_q_three_args() {
        let text = "set A div_q(tmp,L,mem)";
        let ops  = parse_sem(text).unwrap();
        let back = sem_to_text(&ops);
        assert_eq!(back, text);
    }

    #[test]
    fn parse_dp_add_hi_four_args() {
        let text = "set mem_hi dp_add_hi(A,L,mem_hi,tmp)";
        let ops  = parse_sem(text).unwrap();
        let back = sem_to_text(&ops);
        assert_eq!(back, text);
    }

    #[test]
    fn parse_ccs_generic() {
        let text = "set A mem\nbranch_if is_pos(A) pc_rel(0)\nbranch_if is_plus_zero(A) pc_rel(1)\nbranch_if is_neg(A) pc_rel(2)\nbranch pc_rel(3)";
        let ops  = parse_sem(text).unwrap();
        assert_eq!(ops.len(), 5);
    }

    #[test]
    fn parse_sat() {
        let text = "set A sat(tmp)";
        let ops = parse_sem(text).unwrap();
        assert!(matches!(&ops[0], SemOp::Set(Dest::A, Expr::Saturate(_))));
    }

    #[test]
    fn parse_deref() {
        let text = "set deref(Z) oc_add(deref(Z),tmp)";
        let ops = parse_sem(text).unwrap();
        let back = sem_to_text(&ops);
        assert_eq!(back, text);
    }

    #[test]
    fn parse_has_overflow() {
        let text = "branch_if has_overflow(tmp) pc_rel(1)";
        let ops = parse_sem(text).unwrap();
        assert!(matches!(&ops[0], SemOp::BranchIf(Cond::HasOverflow(_), Expr::PcRel(1))));
    }

    #[test]
    fn tc_semantics_roundtrip() {
        let text = "set Q Z\nbranch operand";
        let ops = parse_sem(text).unwrap();
        let back = sem_to_text(&ops);
        assert_eq!(back, text);
    }
}
