//! C source-code backend — translates TC2 stack bytecode to C.
//!
//! ## Design
//!
//! The frontend has already lowered every AGC instruction to a `Vec<u16>`
//! of TC2 stack bytecode via [`agc_lower::lower_sem`].  Branches are encoded
//! in that bytecode as `STORE15(addr::Z) + RET`; all OC arithmetic is
//! expanded to TC2 sequences; NDX operand modification is baked in.
//!
//! The C backend therefore has **no** knowledge of AGC semantics — it only
//! maps TC2 opcodes → C statements, using a local `_stk[]` array for the
//! evaluation stack and a runtime header that provides `agc_vm_read` /
//! `agc_vm_write` / `agc_vm_write15` for memory dispatch.
//!
//! ### Per-instruction C block structure
//!
//! ```c
//! {   /* 0o04000: TCF 0o05000 */
//!     s->z = 0x0801U;                  /* advance PC to next_pc */
//!     s->instr_word = 0x5000U;         /* raw word (for LOAD INSTR_WORD) */
//!     uint16_t _stk[64]; int _sp = 0;
//!     uint16_t _t = 0, _off = 0; (void)_t;
//!     /* bytecode starts here ... */
//!     _stk[_sp++] = 0x1400U;          /* SET_OFF; PUSH_IMM target */
//!     ...
//!     agc_vm_write15(s, 0x0005U, _stk[--_sp]); /* STORE15 addr::Z */
//!     goto _bc_end;                    /* RET */
//!     _bc_end:;
//! }
//! /* terminator: Jump(0o05000) */
//! goto bb_05000;
//! ```
//!
//! ### Memory mapping in the runtime
//!
//! `agc_vm_read(s, addr)` / `agc_vm_write(s, addr, val)` dispatch to:
//! - Addresses 0–7  → CPU register fields (`s->a`, `s->l`, …)
//! - `0xFF00`       → `s->tmp`
//! - `0xFF01`       → `s->extend`
//! - `0xFF02`       → `s->inhint`
//! - `0xFF03`       → `s->instr_word`
//! - `0x8000–81FF`  → `agc_chan_read/write` callbacks
//! - All others     → `s->mem_read/write` callbacks (AGC memory)

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;

use crate::backend::Backend;
use crate::ir::{InstrStream, Terminator};

use agc_lower::bytecode::op;

// ─── Runtime header ───────────────────────────────────────────────────────────

pub static RUNTIME_HEADER: &str = r#"/* agc_runtime.h — TC2 bytecode recompiler runtime
 * All OC arithmetic is compiled into the TC2 bytecode; this header only
 * provides the CPU state struct, memory-dispatch helpers, and TC2 wide-int
 * primitives used by IMUL_HI15 / IMUL_LO15 / IDIV_Q15 / IDIV_R15. */
#ifndef AGC_RUNTIME_H
#define AGC_RUNTIME_H
#include <stdint.h>

/* ── Return codes ────────────────────────────────────────────────────────── */
#define AGC_OK        0
#define AGC_HALT      1   /* no recompiled block at s->z         */
#define AGC_DISPATCH  2   /* indirect branch; re-dispatch on s->z */

/* ── CPU state ───────────────────────────────────────────────────────────── */
typedef struct AgcState {
    uint16_t a;           /* accumulator (16-bit; bit15 = overflow) */
    uint16_t l;           /* L register (15-bit)                    */
    uint16_t q;           /* Q / return-address (15-bit)            */
    uint16_t eb;          /* erasable bank                          */
    uint16_t fb;          /* fixed bank                             */
    uint16_t z;           /* program counter (15-bit)               */
    uint16_t bb;          /* both banks                             */
    uint16_t tmp;         /* non-addressable temporary              */
    uint16_t instr_word;  /* raw instruction word (0xFF03 in VM)    */
    uint8_t  extend;      /* EXTEND flip-flop                       */
    uint8_t  inhint;      /* interrupt-inhibit flag                 */
    uint16_t (*mem_read )(struct AgcState *, uint16_t addr);
    void     (*mem_write)(struct AgcState *, uint16_t addr, uint16_t val);
    uint16_t (*chan_read )(struct AgcState *, uint16_t ch);
    void     (*chan_write)(struct AgcState *, uint16_t ch,  uint16_t val);
} AgcState;

/* ── VM memory dispatch ──────────────────────────────────────────────────── */

static inline uint16_t agc_vm_read(AgcState *s, uint16_t addr) {
    switch (addr) {
        case 0x0000U: return s->a;
        case 0x0001U: return s->l;
        case 0x0002U: return s->q;
        case 0x0003U: return s->eb;
        case 0x0004U: return s->fb;
        case 0x0005U: return s->z;
        case 0x0006U: return s->bb;
        case 0x0007U: return 0U;
        case 0xFF00U: return s->tmp;
        case 0xFF01U: return (uint16_t)s->extend;
        case 0xFF02U: return (uint16_t)s->inhint;
        case 0xFF03U: return s->instr_word;
        default:
            if (addr >= 0x8000U && addr < 0x8200U)
                return s->chan_read(s, (uint16_t)(addr - 0x8000U));
            return s->mem_read(s, addr);
    }
}

static inline void agc_vm_write(AgcState *s, uint16_t addr, uint16_t val) {
    switch (addr) {
        case 0x0000U: s->a  = val;              return;
        case 0x0001U: s->l  = val;              return;
        case 0x0002U: s->q  = val;              return;
        case 0x0003U: s->eb = val;              return;
        case 0x0004U: s->fb = val;              return;
        case 0x0005U: s->z  = val;              return;
        case 0x0006U: s->bb = val;              return;
        case 0x0007U:                           return; /* ZERO — ignore */
        case 0xFF00U: s->tmp        = val;      return;
        case 0xFF01U: s->extend     = (uint8_t)val; return;
        case 0xFF02U: s->inhint     = (uint8_t)val; return;
        case 0xFF03U: s->instr_word = val;      return;
        default:
            if (addr >= 0x8000U && addr < 0x8200U)
                s->chan_write(s, (uint16_t)(addr - 0x8000U), val);
            else
                s->mem_write(s, addr, val);
    }
}

static inline void agc_vm_write15(AgcState *s, uint16_t addr, uint16_t val) {
    agc_vm_write(s, addr, val & 0x7FFFU);
}

/* Channel helpers (used by LOAD_CHAN_OFF / STORE_CHAN_OFF) */
#define agc_chan_read(s,ch)      ((s)->chan_read ((s),(ch)))
#define agc_chan_write(s,ch,v)   ((s)->chan_write((s),(ch),(uint16_t)((v)&0x7FFFU)))

#endif /* AGC_RUNTIME_H */
"#;

// ─── C backend options / struct ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CBackendOptions {
    pub inline_runtime: bool,
    pub function_name: String,
}

impl Default for CBackendOptions {
    fn default() -> Self {
        CBackendOptions {
            inline_runtime: true,
            function_name: "agc_run".into(),
        }
    }
}

pub struct CBackend {
    pub options: CBackendOptions,
}

impl Default for CBackend {
    fn default() -> Self { CBackend { options: CBackendOptions::default() } }
}

impl Backend for CBackend {
    type Output = String;
    type Error = String;

    fn emit(&self, stream: &InstrStream) -> Result<String, String> {
        let mut out = String::new();

        if self.options.inline_runtime {
            out.push_str(RUNTIME_HEADER);
            out.push('\n');
        } else {
            out.push_str("#include \"agc_runtime.h\"\n\n");
        }

        let fn_name = &self.options.function_name;
        out.push_str(&format!("int {fn_name}(AgcState *s) {{\n"));

        // Top-level dispatch switch — re-entry for indirect branches.
        out.push_str("    switch (s->z) {\n");
        for &addr in stream.blocks.keys() {
            out.push_str(&format!("        case 0x{addr:04x}U: goto bb_{addr:05o};\n"));
        }
        out.push_str("        default: return AGC_HALT;\n");
        out.push_str("    }\n\n");

        for (&_addr, block) in &stream.blocks {
            let label = block.label;
            out.push_str(&format!("bb_{label:05o}:  /* 0o{label:07o} */\n"));

            // Emit bytecode translation for each instruction.
            for rec in &block.instrs {
                out.push_str(&format!(
                    "    /* 0o{:07o}: {} 0o{:07o} */\n",
                    rec.pc, rec.mnemonic, rec.operand
                ));
                let next_pc = (rec.pc + 1) & 0x7FFF;
                emit_bytecode_block(&rec.bytecode, next_pc, rec.raw_word, &mut out);
            }

            emit_terminator(&block.terminator, &stream.indirect_targets, &mut out);
            out.push('\n');
        }

        out.push_str("}\n");
        Ok(out)
    }
}

// ─── Bytecode → C translation ─────────────────────────────────────────────────

/// Translate one instruction's TC2 bytecode to a C block.
///
/// Before the bytecode executes, `s->z` is set to `next_pc` so that any
/// `LOAD(addr::Z)` inside the bytecode (e.g. for PcRel computations) sees
/// the already-advanced PC.  If the instruction contains a branch, the
/// bytecode will overwrite `s->z` with the branch target before `RET`.
fn emit_bytecode_block(code: &[u16], next_pc: u16, raw_word: u16, out: &mut String) {
    // First pass: collect all word indices that are jump targets.
    let jump_targets = collect_jump_targets(code);

    out.push_str("    {\n");
    // Pre-advance Z to next_pc so Expr::Z (and PcRel) reads are correct.
    out.push_str(&format!("        s->z = 0x{next_pc:04x}U;\n"));
    // Expose instruction word for Expr::InstrWord.
    out.push_str(&format!("        s->instr_word = 0x{raw_word:04x}U;\n"));
    out.push_str("        uint16_t _stk[64]; int _sp = 0;\n");
    out.push_str("        uint16_t _t = 0, _off = 0; (void)_t;\n");

    let mut pc = 0usize;
    while pc < code.len() {
        // Insert label for jump targets.
        if jump_targets.contains(&pc) {
            out.push_str(&format!("        _j{pc}:;\n"));
        }

        let opc = code[pc];
        pc += 1;

        match opc {
            op::RET => {
                out.push_str("        goto _bc_end;\n");
                // Keep scanning — there may be more code reached via jumps.
            }

            // ── Stack manipulation ─────────────────────────────────────────
            op::DUP  => out.push_str("        _stk[_sp] = _stk[_sp-1]; _sp++;\n"),
            op::SWAP => out.push_str("        { uint16_t _s=_stk[_sp-1]; _stk[_sp-1]=_stk[_sp-2]; _stk[_sp-2]=_s; }\n"),
            op::DROP => out.push_str("        --_sp;\n"),

            // ── TC2 arithmetic ─────────────────────────────────────────────
            op::ADD  => out.push_str("        { uint16_t _b=_stk[--_sp]; _stk[_sp-1]=(uint16_t)(_stk[_sp-1]+_b); }\n"),
            op::SUB  => out.push_str("        { uint16_t _b=_stk[--_sp]; _stk[_sp-1]=(uint16_t)(_stk[_sp-1]-_b); }\n"),
            op::AND  => out.push_str("        { uint16_t _b=_stk[--_sp]; _stk[_sp-1]&=_b; }\n"),
            op::OR   => out.push_str("        { uint16_t _b=_stk[--_sp]; _stk[_sp-1]|=_b; }\n"),
            op::XOR  => out.push_str("        { uint16_t _b=_stk[--_sp]; _stk[_sp-1]^=_b; }\n"),
            op::NOT  => out.push_str("        _stk[_sp-1]=(uint16_t)(~_stk[_sp-1]);\n"),
            op::MASK15 => out.push_str("        _stk[_sp-1]&=0x7FFFU;\n"),
            op::NEG  => out.push_str("        _stk[_sp-1]=(uint16_t)(-(int16_t)_stk[_sp-1]);\n"),

            op::LSHR_STK => out.push_str("        { uint16_t _k=_stk[--_sp]; _stk[_sp-1]=(uint16_t)(_stk[_sp-1]>>(_k&15U)); }\n"),
            op::LSHL_STK => out.push_str("        { uint16_t _k=_stk[--_sp]; _stk[_sp-1]=(uint16_t)((_stk[_sp-1]<<(_k&15U))&0xFFFFU); }\n"),

            // ── TC2 wide-integer ops ───────────────────────────────────────
            op::IMUL_HI15 => out.push_str(
                "        { int32_t _b=(int32_t)(int16_t)_stk[--_sp],_a=(int32_t)(int16_t)_stk[--_sp]; \
                 _stk[_sp++]=(uint16_t)((_a*_b)>>15); }\n"
            ),
            op::IMUL_LO15 => out.push_str(
                "        { int32_t _b=(int32_t)(int16_t)_stk[--_sp],_a=(int32_t)(int16_t)_stk[--_sp]; \
                 _stk[_sp++]=(uint16_t)((_a*_b)&0x7FFF); }\n"
            ),
            op::IDIV_Q15 => out.push_str(
                "        { int64_t _d=(int64_t)(int16_t)_stk[--_sp]; \
                 int64_t _lo=(int64_t)(_stk[--_sp]&0x7FFFU); \
                 int64_t _hi=(int64_t)(int16_t)_stk[--_sp]; \
                 int64_t _n=(_hi<<15)|_lo; \
                 _stk[_sp++]=_d!=0?(uint16_t)(_n/_d):0U; }\n"
            ),
            op::IDIV_R15 => out.push_str(
                "        { int64_t _d=(int64_t)(int16_t)_stk[--_sp]; \
                 int64_t _lo=(int64_t)(_stk[--_sp]&0x7FFFU); \
                 int64_t _hi=(int64_t)(int16_t)_stk[--_sp]; \
                 int64_t _n=(_hi<<15)|_lo; \
                 _stk[_sp++]=_d!=0?(uint16_t)(_n%_d):0U; }\n"
            ),

            // ── OC bit-pattern predicates ──────────────────────────────────
            op::IS_POS => out.push_str(
                "        { uint16_t _x=_stk[--_sp]&0x7FFFU; \
                 _stk[_sp++]=(_x!=0U&&(_x&0x4000U)==0U)?1U:0U; }\n"
            ),
            op::IS_PLUS_ZERO => out.push_str(
                "        { uint16_t _x=_stk[--_sp]&0x7FFFU; _stk[_sp++]=(_x==0U)?1U:0U; }\n"
            ),
            op::IS_NEG => out.push_str(
                "        { uint16_t _x=_stk[--_sp]&0x7FFFU; \
                 _stk[_sp++]=((_x&0x4000U)!=0U&&_x!=0x7FFFU)?1U:0U; }\n"
            ),
            op::IS_MINUS_ZERO => out.push_str(
                "        { uint16_t _x=_stk[--_sp]&0x7FFFU; _stk[_sp++]=(_x==0x7FFFU)?1U:0U; }\n"
            ),
            op::IS_ZERO_OR_NEG => out.push_str(
                "        { uint16_t _x=_stk[--_sp]&0x7FFFU; \
                 _stk[_sp++]=(_x==0U||_x==0x7FFFU||(_x&0x4000U)!=0U)?1U:0U; }\n"
            ),
            op::HAS_OVERFLOW => out.push_str(
                "        { uint16_t _x=_stk[--_sp]; \
                 _stk[_sp++]=(((_x>>14)&3U)==1U||((_x>>14)&3U)==2U)?1U:0U; }\n"
            ),
            op::BOOL_AND => out.push_str("        { uint16_t _b=_stk[--_sp]; _stk[_sp-1]&=_b; }\n"),
            op::BOOL_NOT => out.push_str("        _stk[_sp-1]=(_stk[_sp-1]==0U)?1U:0U;\n"),

            // ── T register ────────────────────────────────────────────────
            op::LOAD_T  => out.push_str("        _stk[_sp++]=_t;\n"),
            op::STORE_T => out.push_str("        _t=_stk[--_sp];\n"),

            // ── OFF register ──────────────────────────────────────────────
            op::GET_OFF       => out.push_str("        _stk[_sp++]=_off;\n"),
            op::SET_OFF_STACK  => out.push_str("        _off=_stk[--_sp];\n"),

            // ── OFF-relative memory ────────────────────────────────────────
            op::LOAD_OFF   => out.push_str("        _stk[_sp++]=agc_vm_read(s,_off);\n"),
            op::STORE_OFF  => out.push_str("        agc_vm_write15(s,_off,_stk[--_sp]);\n"),
            op::LOAD_OFF1  => out.push_str("        _stk[_sp++]=agc_vm_read(s,(uint16_t)(_off+1U));\n"),
            op::STORE_OFF1 => out.push_str("        agc_vm_write15(s,(uint16_t)(_off+1U),_stk[--_sp]);\n"),

            // ── Channel (OFF-relative) ────────────────────────────────────
            op::LOAD_CHAN_OFF  => out.push_str("        _stk[_sp++]=agc_chan_read(s,_off&0x01FFU);\n"),
            op::STORE_CHAN_OFF => out.push_str("        agc_chan_write(s,_off&0x01FFU,_stk[--_sp]);\n"),

            // ── Indirect memory ───────────────────────────────────────────
            op::LOAD_IND  => out.push_str("        { uint16_t _a=_stk[--_sp]; _stk[_sp++]=agc_vm_read(s,_a); }\n"),
            op::STORE_IND => out.push_str("        { uint16_t _a=_stk[--_sp]; agc_vm_write15(s,_a,_stk[--_sp]); }\n"),

            // ── Two-word instructions ──────────────────────────────────────
            op::PUSH_IMM => {
                let v = code[pc]; pc += 1;
                out.push_str(&format!("        _stk[_sp++]=0x{v:04x}U;\n"));
            }
            op::LOAD => {
                let addr = code[pc]; pc += 1;
                out.push_str(&format!("        _stk[_sp++]=agc_vm_read(s,0x{addr:04x}U);\n"));
            }
            op::STORE => {
                let addr = code[pc]; pc += 1;
                out.push_str(&format!("        agc_vm_write(s,0x{addr:04x}U,_stk[--_sp]);\n"));
            }
            op::STORE15 => {
                let addr = code[pc]; pc += 1;
                out.push_str(&format!("        agc_vm_write15(s,0x{addr:04x}U,_stk[--_sp]);\n"));
            }
            op::SET_OFF => {
                let v = code[pc]; pc += 1;
                out.push_str(&format!("        _off=0x{v:04x}U;\n"));
            }
            op::LSHR => {
                let k = code[pc]; pc += 1;
                out.push_str(&format!("        _stk[_sp-1]=(uint16_t)(_stk[_sp-1]>>{k}U);\n"));
            }
            op::LSHL => {
                let k = code[pc]; pc += 1;
                out.push_str(&format!("        _stk[_sp-1]=(uint16_t)((_stk[_sp-1]<<{k}U)&0xFFFFU);\n"));
            }
            op::JUMP => {
                let raw = code[pc] as i16; pc += 1;
                let target = (pc as isize + raw as isize) as usize;
                out.push_str(&format!("        goto _j{target};\n"));
            }
            op::JUMP_IF => {
                let raw = code[pc] as i16; pc += 1;
                let target = (pc as isize + raw as isize) as usize;
                out.push_str(&format!("        if (_stk[--_sp]!=0U) goto _j{target};\n"));
            }
            op::JUMP_NOT => {
                let raw = code[pc] as i16; pc += 1;
                let target = (pc as isize + raw as isize) as usize;
                out.push_str(&format!("        if (_stk[--_sp]==0U) goto _j{target};\n"));
            }

            unknown => {
                out.push_str(&format!("        /* unknown opcode 0x{unknown:04x} */\n"));
            }
        }
    }

    // Emit any label whose target falls on or past the last instruction.
    // (Happens when jump patches target the word just after the last RET.)
    if jump_targets.contains(&pc) {
        out.push_str(&format!("        _j{pc}:;\n"));
    }

    out.push_str("        _bc_end:;\n");
    out.push_str("    }\n");
}

/// Scan the bytecode and return a set of word indices that are jump targets.
fn collect_jump_targets(code: &[u16]) -> BTreeSet<usize> {
    let mut targets = BTreeSet::new();
    let mut pc = 0usize;
    while pc < code.len() {
        let opc = code[pc];
        pc += 1;
        match opc {
            op::JUMP | op::JUMP_IF | op::JUMP_NOT => {
                if pc < code.len() {
                    let off = code[pc] as i16;
                    let target = (pc as isize + 1 + off as isize) as usize;
                    targets.insert(target);
                }
                pc += 1;
            }
            op::PUSH_IMM | op::LOAD | op::STORE | op::STORE15
            | op::SET_OFF | op::LSHR | op::LSHL => { pc += 1; }
            _ => {}
        }
    }
    targets
}

// ─── Terminator dispatch ──────────────────────────────────────────────────────

/// Emit the C control-flow transfer at the end of a basic block.
///
/// By the time this runs, the terminal instruction's bytecode has already
/// executed and `s->z` holds the branch target (or `next_pc` if no branch
/// was taken).  We use direct `goto`s wherever the target is statically known.
fn emit_terminator(
    term: &Terminator,
    indirect_targets: &BTreeSet<u16>,
    out: &mut String,
) {
    match term {
        Terminator::FallThrough(addr) => {
            out.push_str(&format!("    goto bb_{addr:05o};\n"));
        }
        Terminator::Jump(addr) => {
            out.push_str(&format!("    goto bb_{addr:05o};\n"));
        }
        Terminator::CondBranch { taken, fallthru } => {
            // Bytecode set s->z = taken if branch was taken, else s->z = next_pc = fallthru.
            out.push_str(&format!(
                "    if (s->z == 0x{taken:04x}U) goto bb_{taken:05o};\n"
            ));
            out.push_str(&format!("    goto bb_{fallthru:05o};\n"));
        }
        Terminator::CcsBranch(targets) => {
            out.push_str("    switch (s->z) {\n");
            for &t in targets {
                out.push_str(&format!("        case 0x{t:04x}U: goto bb_{t:05o};\n"));
            }
            out.push_str("        default: return AGC_HALT;\n");
            out.push_str("    }\n");
        }
        Terminator::Indirect { possible_targets } => {
            let combined: BTreeSet<u16> = indirect_targets
                .iter()
                .chain(possible_targets.iter())
                .copied()
                .collect();
            out.push_str("    switch (s->z) {\n");
            for &t in &combined {
                out.push_str(&format!("        case 0x{t:04x}U: goto bb_{t:05o};\n"));
            }
            out.push_str("        default: return AGC_DISPATCH;\n");
            out.push_str("    }\n");
        }
        Terminator::Halt => {
            out.push_str("    return AGC_HALT;\n");
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::{BTreeMap, BTreeSet};

    #[test]
    fn runtime_header_has_agc_state_and_vm_read() {
        assert!(RUNTIME_HEADER.contains("AgcState"));
        assert!(RUNTIME_HEADER.contains("agc_vm_read"));
        assert!(RUNTIME_HEADER.contains("agc_vm_write15"));
        // OC arithmetic should NOT be in the runtime (it's in bytecode now).
        assert!(!RUNTIME_HEADER.contains("oc_add"));
        assert!(!RUNTIME_HEADER.contains("oc_neg"));
    }

    #[test]
    fn empty_stream_emits_valid_c() {
        use crate::ir::InstrStream;
        let stream = InstrStream {
            blocks: BTreeMap::new(),
            entry_points: alloc::vec![],
            indirect_targets: BTreeSet::new(),
        };
        let src = CBackend::default().emit(&stream).unwrap();
        assert!(src.contains("int agc_run(AgcState *s)"));
        assert!(src.contains("switch (s->z)"));
        assert!(src.contains("AGC_HALT"));
    }

    #[test]
    fn bytecode_push_imm_load_store_translates() {
        use agc_lower::bytecode::Instr;
        let mut code = alloc::vec![];
        Instr::PushImm(42).encode(&mut code);
        Instr::Store15(5).encode(&mut code);  // STORE15 addr::Z
        Instr::Ret.encode(&mut code);
        let mut out = String::new();
        emit_bytecode_block(&code, 0x0801, 0x1234, &mut out);
        assert!(out.contains("0x002aU"));       // 42 pushed
        assert!(out.contains("agc_vm_write15")); // store
        assert!(out.contains("_bc_end"));
    }

    #[test]
    fn jump_target_label_inserted() {
        use agc_lower::bytecode::Instr;
        // JUMP_NOT at words 0,1.  pc_after_instruction = 2.
        // offset=2 → target = 2+2 = 4 (the first Ret, skipping PushImm at 2,3).
        let mut code = alloc::vec![];
        Instr::JumpNot(2).encode(&mut code);  // words 0,1
        Instr::PushImm(0).encode(&mut code);  // words 2,3 — skipped when cond=0
        Instr::Ret.encode(&mut code);          // word 4: _j4 label expected here
        Instr::Ret.encode(&mut code);          // word 5
        let mut out = String::new();
        emit_bytecode_block(&code, 0x0100, 0x0000, &mut out);
        assert!(out.contains("_j4"), "expected label _j4 at word 4");
    }
}
