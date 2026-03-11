//! TC2 constant-propagation slicer.
//!
//! Forward symbolic evaluation of TC2 bytecode to determine whether a
//! constant value is stored to [`agc_lower::vm::addr::NEXT_INSTR`] (`0xFF04`).
//!
//! Used by the frontend to constant-fold NDX instructions at decode time.

use agc_lower::{bytecode::op, vm::addr};

/// AGC fixed-memory boundary: addresses ≥ `FIXED_BOUNDARY` are ROM (known at
/// decode time); addresses below are erasable RAM (runtime-dependent).
const FIXED_BOUNDARY: u16 = 0o2000; // 1024 decimal

/// Try to determine the constant value stored to `NEXT_INSTR` (0xFF04) by the
/// TC2 bytecode `code`.
///
/// `next_pc` is the compile-time value of the Z register at the start of the
/// instruction — this is pre-set by the C backend preamble.
///
/// Returns `Some(word)` if the stored value can be determined at compile time,
/// `None` otherwise (erasable operand, indirect through unknown address, etc.).
pub fn slice_next_instr(code: &[u16], memory: &[u16; 4096], next_pc: u16) -> Option<u16> {
    let mut state = SymState::new(next_pc);
    state.run(code, memory)
}

// ─── Symbolic value ───────────────────────────────────────────────────────────

/// A symbolic value: `Some(x)` = compile-time constant, `None` = runtime-only.
type Sym = Option<u16>;

#[inline]
fn sym_mask15(v: Sym) -> Sym { v.map(|x| x & 0x7FFF) }

#[inline]
fn sym_add(a: Sym, b: Sym) -> Sym {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.wrapping_add(b)),
        _ => None,
    }
}

#[inline]
fn sym_and(a: Sym, b: Sym) -> Sym {
    match (a, b) { (Some(a), Some(b)) => Some(a & b), _ => None }
}

#[inline]
fn sym_or(a: Sym, b: Sym) -> Sym {
    match (a, b) { (Some(a), Some(b)) => Some(a | b), _ => None }
}

#[inline]
fn sym_xor(a: Sym, b: Sym) -> Sym {
    match (a, b) { (Some(a), Some(b)) => Some(a ^ b), _ => None }
}

#[inline]
fn sym_not(v: Sym) -> Sym { v.map(|x| !x) }

#[inline]
fn sym_lshr(v: Sym, k: u16) -> Sym { v.map(|x| x >> k) }

#[inline]
fn sym_lshl(v: Sym, k: u16) -> Sym { v.map(|x| (x << k) & 0xFFFF) }

// ─── Symbolic state ───────────────────────────────────────────────────────────

struct SymState {
    stack: Vec<Sym>,
    off:   Sym,           // OFF register
    tmp:   Sym,           // addr::TMP (0xFF00)
    z:     Sym,           // Z register — pre-set to Some(next_pc)
}

impl SymState {
    fn new(next_pc: u16) -> Self {
        SymState {
            stack: Vec::with_capacity(16),
            off:   None,
            tmp:   None,
            z:     Some(next_pc),
        }
    }

    fn push(&mut self, v: Sym) { self.stack.push(v); }

    fn pop(&mut self) -> Sym { self.stack.pop().unwrap_or(None) }

    fn mem_load_known(memory: &[u16; 4096], addr: u16) -> Sym {
        if addr < 4096 && addr >= FIXED_BOUNDARY {
            Some(memory[addr as usize] & 0x7FFF)
        } else {
            None
        }
    }

    /// Run the bytecode symbolically.  Returns the constant stored to
    /// `NEXT_INSTR` if one is found, or `None`.
    fn run(&mut self, code: &[u16], memory: &[u16; 4096]) -> Option<u16> {
        let mut pc = 0usize;

        macro_rules! next_word {
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
                op::RET => break,

                // ── Jumps — on unknown condition, abandon (return None) ────
                op::JUMP => {
                    let off = next_word!() as i16;
                    pc = (pc as isize + off as isize) as usize;
                }
                op::JUMP_IF => {
                    let off = next_word!() as i16;
                    let cond = self.pop();
                    match cond {
                        Some(0) => {} // not taken — continue
                        Some(_) => { pc = (pc as isize + off as isize) as usize; }
                        None    => return None, // unknown branch
                    }
                }
                op::JUMP_NOT => {
                    let off = next_word!() as i16;
                    let cond = self.pop();
                    match cond {
                        Some(0) => { pc = (pc as isize + off as isize) as usize; }
                        Some(_) => {} // not taken — continue
                        None    => return None,
                    }
                }

                // ── Immediates / absolute memory ──────────────────────────
                op::PUSH_IMM => { let v = next_word!(); self.push(Some(v)); }

                op::LOAD => {
                    let a = next_word!();
                    let v = match a {
                        addr::Z         => self.z,
                        addr::TMP       => self.tmp,
                        a if a < 4096   => Self::mem_load_known(memory, a),
                        _               => None,
                    };
                    self.push(v);
                }

                op::STORE => {
                    let a = next_word!();
                    let v = self.pop();
                    if a == addr::TMP { self.tmp = v; }
                    // writes to Z or other regs are side effects we don't fold
                }

                op::STORE15 => {
                    let a = next_word!();
                    let v = self.pop().map(|x| x & 0x7FFF);
                    if a == addr::NEXT_INSTR {
                        return v; // ← this is the result we care about
                    }
                    if a == addr::TMP { self.tmp = v; }
                }

                op::SET_OFF => {
                    let k = next_word!();
                    self.off = Some(k);
                }

                // ── Stack ops ─────────────────────────────────────────────
                op::DUP  => { let v = self.pop(); self.push(v); self.push(v); }
                op::SWAP => { let b = self.pop(); let a = self.pop(); self.push(b); self.push(a); }
                op::DROP => { self.pop(); }

                // ── OFF-relative loads ────────────────────────────────────
                op::LOAD_OFF => {
                    let v = self.off.and_then(|k| Self::mem_load_known(memory, k));
                    self.push(v);
                }
                op::LOAD_OFF1 => {
                    let v = self.off.and_then(|k| {
                        Self::mem_load_known(memory, k.wrapping_add(1))
                    });
                    self.push(v);
                }
                op::GET_OFF => { self.push(self.off); }

                op::STORE_OFF  => { self.pop(); /* write to erasable — not constant */ }
                op::STORE_OFF1 => { self.pop(); }
                op::SET_OFF_STACK => { self.off = self.pop(); }

                // ── Indirect ─────────────────────────────────────────────
                op::LOAD_IND => {
                    let addr_sym = self.pop();
                    let v = addr_sym.and_then(|a| Self::mem_load_known(memory, a));
                    self.push(v);
                }
                op::STORE_IND => {
                    let _addr = self.pop();
                    let _val  = self.pop();
                    // runtime memory write — no constant effect
                }

                // ── Arithmetic (propagate constants) ──────────────────────
                op::ADD  => { let b = self.pop(); let a = self.pop(); self.push(sym_add(a, b)); }
                op::SUB  => { let b = self.pop(); let a = self.pop(); self.push(match (a,b) { (Some(a),Some(b)) => Some(a.wrapping_sub(b)), _ => None }); }
                op::AND  => { let b = self.pop(); let a = self.pop(); self.push(sym_and(a, b)); }
                op::OR   => { let b = self.pop(); let a = self.pop(); self.push(sym_or(a, b)); }
                op::XOR  => { let b = self.pop(); let a = self.pop(); self.push(sym_xor(a, b)); }
                op::NOT  => { let x = self.pop(); self.push(sym_not(x)); }
                op::MASK15 => { let x = self.pop(); self.push(sym_mask15(x)); }
                op::NEG  => { let x = self.pop(); self.push(x.map(|v| v.wrapping_neg())); }

                op::LSHR => { let k = next_word!(); let x = self.pop(); self.push(sym_lshr(x, k)); }
                op::LSHL => { let k = next_word!(); let x = self.pop(); self.push(sym_lshl(x, k)); }

                op::LSHR_STK => { let k = self.pop(); let x = self.pop(); self.push(match (x,k) { (Some(x),Some(k)) => Some(x >> k), _ => None }); }
                op::LSHL_STK => { let k = self.pop(); let x = self.pop(); self.push(match (x,k) { (Some(x),Some(k)) => Some((x << k) & 0xFFFF), _ => None }); }

                // ── OC predicates — push None (condition value is runtime) -
                op::IS_POS | op::IS_PLUS_ZERO | op::IS_NEG | op::IS_MINUS_ZERO
                | op::IS_ZERO_OR_NEG | op::HAS_OVERFLOW => {
                    self.pop();
                    self.push(None); // result depends on runtime OC value
                }
                op::BOOL_AND => { let b = self.pop(); let a = self.pop(); self.push(sym_and(a, b)); }
                op::BOOL_NOT => { let x = self.pop(); self.push(match x { Some(0) => Some(1), Some(_) => Some(0), None => None }); }

                // ── T register ───────────────────────────────────────────
                op::LOAD_T  => { self.push(None); } // T not tracked
                op::STORE_T => { self.pop(); }

                // ── Channels ─────────────────────────────────────────────
                op::LOAD_CHAN_OFF  => { self.push(None); } // I/O is always runtime
                op::STORE_CHAN_OFF => { self.pop(); }

                // ── Wide integer ops — push None (not needed for NDX) ─────
                op::IMUL_HI15 | op::IMUL_LO15 => { self.pop(); self.pop(); self.push(None); }
                op::IDIV_Q15 | op::IDIV_R15   => { self.pop(); self.pop(); self.pop(); self.push(None); }

                // Unknown opcode — bail out.
                _ => return None,
            }
        }

        None // NEXT_INSTR was never set
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use agc_lower::lower::lower_sem;
    use agc_isa::sem_text::parse_sem;

    fn make_memory() -> Box<[u16; 4096]> {
        let mut mem = Box::new([0u16; 4096]);
        // Fill fixed memory (>=0o2000) with known values
        for i in 0o2000..4096usize {
            mem[i] = (i as u16) & 0x7FFF;
        }
        mem
    }

    /// NDX with a fixed-memory operand: the slicer should return a constant.
    #[test]
    fn ndx_fixed_memory_folds_to_constant() {
        let memory = make_memory();
        // NDX semantics: set tmp mem; set next_instr oc_add(deref(Z),tmp)
        let ops = parse_sem("set tmp mem\nset next_instr oc_add(deref(Z),tmp)").unwrap();
        let operand = 0o2001u16; // fixed memory
        let next_pc = 0o2010u16; // some fixed-memory next PC
        let code = lower_sem(operand, &ops);

        let result = slice_next_instr(&code, &memory, next_pc);
        assert!(result.is_some(), "expected constant fold for fixed-memory NDX");

        // Verify: result should equal oc_add(memory[next_pc], memory[operand])
        let mem_k    = memory[operand as usize] as u32 & 0x7FFF;
        let mem_z    = memory[next_pc as usize] as u32 & 0x7FFF;
        let raw      = mem_k + mem_z;
        let carry    = raw >> 15;
        let expected = ((raw + carry) & 0x7FFF) as u16;
        assert_eq!(result.unwrap(), expected, "folded value does not match oc_add");
    }

    /// NDX with an erasable operand: the slicer should return None.
    #[test]
    fn ndx_erasable_memory_returns_none() {
        let memory = make_memory();
        let ops = parse_sem("set tmp mem\nset next_instr oc_add(deref(Z),tmp)").unwrap();
        let operand = 0o0100u16; // erasable memory (< 0o2000)
        let next_pc = 0o2010u16;
        let code = lower_sem(operand, &ops);

        let result = slice_next_instr(&code, &memory, next_pc);
        assert!(result.is_none(), "erasable NDX should not fold");
    }

    /// Default epilogue for a fall-through instruction: returns memory[next_pc].
    #[test]
    fn fallthrough_epilogue_folds_to_next_word() {
        let memory = make_memory();
        // A simple CA-like instruction: set A mem
        let ops = parse_sem("set A mem").unwrap();
        let operand = 0o2005u16;
        let next_pc  = 0o2010u16;
        let code = lower_sem(operand, &ops);

        let result = slice_next_instr(&code, &memory, next_pc);
        // Default epilogue: LOAD Z; LOAD_IND; STORE15 0xFF04 → memory[next_pc]
        assert_eq!(result, Some(memory[next_pc as usize] & 0x7FFF));
    }

    /// Branch instruction (early RET before epilogue): slicer returns None.
    #[test]
    fn branch_instruction_returns_none() {
        let memory = make_memory();
        // TCF-like: branch operand
        let ops = parse_sem("branch operand").unwrap();
        let operand = 0o2020u16;
        let next_pc  = 0o2010u16;
        let code = lower_sem(operand, &ops);

        let result = slice_next_instr(&code, &memory, next_pc);
        // Branch emits early RET — the default epilogue is never reached.
        assert!(result.is_none(), "branch should return None (no NEXT_INSTR store)");
    }
}
