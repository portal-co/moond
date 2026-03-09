//! WASM direct backend — translates the AGC instruction stream into a
//! WebAssembly module using the yecta [`Reactor`].
//!
//! ## Design
//!
//! This backend implements [`DirectBackend`], not the block-based [`Backend`]
//! trait.  Callers iterate all 4096 AGC addresses × 2 EXTEND states and feed
//! them in order via [`WasmDirectBackend::feed_instr`]; the backend then
//! returns a valid WASM binary from [`WasmDirectBackend::finish`].
//!
//! ### Reactor function layout
//!
//! Each (address, extend) pair gets one WASM function:
//!
//! ```text
//! reactor_idx(addr, false) = 2 * addr
//! reactor_idx(addr, true)  = 2 * addr + 1
//! WASM function index      = reactor_idx + NUM_IMPORTS
//! ```
//!
//! Calling `reactor.next(ctx, locals, 2)` for every function builds two
//! natural fall-through chains (even = extend=false, odd = extend=true).
//! The extend=false chain is the primary one; the extend=true chain is
//! redirected back to the extend=false chain after each EXTEND=1 instruction
//! via an explicit `jmp()`.
//!
//! ### Function body termination
//!
//! yecta's `add_pred_checked` emits `ReturnCall + ifs×End` for cycle
//! (back-edge) functions but does NOT emit the final function-body `End`.
//! `finish()` appends `End` to every function after `into_fns()`, which is
//! valid in both cases:
//! - Cycle-detected: `[…ReturnCall(f)] + End`   — End in unreachable state ✓
//! - Non-terminated: `[…bytecode…]    + End`   — stack is balanced by TC2 ✓
//!
//! ### WASM module structure
//!
//! ```text
//! imports:
//!   env.mem_read (addr:i32) -> i32
//!   env.mem_write(addr:i32, val:i32)
//!   env.chan_read (ch:i32) -> i32
//!   env.chan_write(ch:i32, val:i32)
//!
//! memory:  1 page (64 KB) — register file at fixed byte offsets:
//!   0: A, 2: L, 4: Q, 6: EB, 8: FB, 10: Z, 12: BB,
//!   14: TMP, 16: EXTEND, 18: INHINT, 20: INSTR_WORD
//!
//! functions: 8192 × type 0  (4096 addrs × 2 extend states)
//! exports:   "memory" + entry-point "bb_OOOOO" names (octal, ext=false)
//! ```

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use wasm_encoder::{
    BlockType, CodeSection, EntityType, ExportKind, ExportSection, FunctionSection,
    ImportSection, Instruction, MemArg, MemorySection, MemoryType, Module, TypeSection,
    ValType,
};
use yecta::{FuncIdx, Reactor};

use agc_isa::InstrType;

use super::{DirectBackend, DirectInstr};
use crate::ir::Terminator;
use agc_lower::bytecode::op;

// ─── WASM memory layout: byte offsets for the AGC register file ───────────────

const MEM_A:          u64 = 0;
const MEM_L:          u64 = 2;
const MEM_Q:          u64 = 4;
const MEM_EB:         u64 = 6;
const MEM_FB:         u64 = 8;
const MEM_Z:          u64 = 10;
const MEM_BB:         u64 = 12;
const MEM_TMP:        u64 = 14;
const MEM_EXTEND:     u64 = 16;
const MEM_INHINT:     u64 = 18;
const MEM_INSTR_WORD: u64 = 20;

// ─── Imported function indices ─────────────────────────────────────────────────

const FN_MEM_READ:  u32 = 0;
const FN_MEM_WRITE: u32 = 1;
const FN_CHAN_READ:  u32 = 2;
const FN_CHAN_WRITE: u32 = 3;
const NUM_IMPORTS:  u32 = 4;

// ─── Local variable indices inside generated functions ────────────────────────

const LOCAL_T:    u32 = 0;
const LOCAL_OFF:  u32 = 1;
const LOCAL_SCR0: u32 = 2;
const LOCAL_SCR1: u32 = 3;

// ─── Helper: reactor index for an (addr, extend) pair ────────────────────────

#[inline]
fn ri(addr: u16, extend: bool) -> u32 {
    2 * addr as u32 + extend as u32
}

// ─── Backend struct ───────────────────────────────────────────────────────────

/// Direct WASM backend powered by the yecta [`Reactor`].
///
/// Feed all 4096 × 2 instructions (in address order, both EXTEND states) with
/// [`feed_instr`], then call [`finish`] to get the assembled WASM module.
///
/// [`feed_instr`]: WasmDirectBackend::feed_instr
/// [`finish`]:     WasmDirectBackend::finish
pub struct WasmDirectBackend {
    reactor: Reactor<()>,
    ctx: (),
    /// Entry-point addresses to export as "bb_OOOOO".
    entry_points: Vec<u16>,
}

impl WasmDirectBackend {
    pub fn new(entry_points: Vec<u16>) -> Self {
        WasmDirectBackend {
            reactor: Reactor::with_base_func_offset(NUM_IMPORTS),
            ctx: (),
            entry_points,
        }
    }
}

impl DirectBackend for WasmDirectBackend {
    type Output = Vec<u8>;
    type Error = String;

    fn feed_instr(&mut self, instr: &DirectInstr) -> Result<(), String> {
        // Create one reactor function per (addr, extend) pair.
        // len=2 builds two parallel fall-through chains:
        //   even chain (extend=false): f0 → f2 → f4 → …
        //   odd  chain (extend=true):  f1 → f3 → f5 → …
        self.reactor
            .next(&mut self.ctx, core::iter::once((4u32, ValType::I32)), 2)
            .unwrap();

        // Pre-advance Z to next_pc so TC2 LOAD(Z) reads are correct.
        let next_pc = instr.addr.wrapping_add(1) & 0x7FFF;
        feed(&mut self.reactor, &mut self.ctx, Instruction::I32Const(0))?;
        feed(&mut self.reactor, &mut self.ctx, Instruction::I32Const(next_pc as i32))?;
        feed(&mut self.reactor, &mut self.ctx, Instruction::I32Store16(mem16(MEM_Z)))?;

        // Expose raw instruction word for LOAD(INSTR_WORD).
        feed(&mut self.reactor, &mut self.ctx, Instruction::I32Const(0))?;
        feed(&mut self.reactor, &mut self.ctx, Instruction::I32Const(instr.raw_word as i32))?;
        feed(&mut self.reactor, &mut self.ctx, Instruction::I32Store16(mem16(MEM_INSTR_WORD)))?;

        // Emit TC2 bytecode (pre-lowered by the caller via decode_direct).
        emit_bc_segment(&mut self.reactor, &mut self.ctx, &instr.bytecode, 0)?;

        // Emit control-flow terminator.
        emit_direct_terminator(&mut self.reactor, &mut self.ctx, instr)?;

        Ok(())
    }

    fn finish(self) -> Result<Vec<u8>, String> {
        let n_funcs = self.reactor.fn_count() as u32;
        let mut functions = self.reactor.into_fns();

        // Every WASM function body must end with End (0x0b) to close the
        // function's implicit block.  Append it unconditionally:
        //   - Cycle-detected functions end with ReturnCall; End in unreachable
        //     state is valid (closes the implicit block).
        //   - All other functions have TC2 bytecode that leaves the stack
        //     empty; a bare End closes the block.
        for f in &mut functions {
            f.instruction(&Instruction::End);
        }

        // ── Assemble WASM module ──────────────────────────────────────────────

        let mut module = Module::new();

        // Type section
        //   type 0: () -> ()         (all generated functions)
        //   type 1: (i32) -> i32     (mem_read, chan_read)
        //   type 2: (i32, i32) -> () (mem_write, chan_write)
        let mut types = TypeSection::new();
        types.ty().function([], []);
        types.ty().function([ValType::I32], [ValType::I32]);
        types.ty().function([ValType::I32, ValType::I32], []);
        module.section(&types);

        // Import section
        let mut imports = ImportSection::new();
        imports.import("env", "mem_read",   EntityType::Function(1));
        imports.import("env", "mem_write",  EntityType::Function(2));
        imports.import("env", "chan_read",  EntityType::Function(1));
        imports.import("env", "chan_write", EntityType::Function(2));
        module.section(&imports);

        // Function section — all functions use type 0
        let mut funcs = FunctionSection::new();
        for _ in 0..n_funcs {
            funcs.function(0);
        }
        module.section(&funcs);

        // Memory section — 1 page (64 KB) for register file
        let mut mems = MemorySection::new();
        mems.memory(MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        module.section(&mems);

        // Export section — entry points (extend=false variant) + linear memory
        let mut exports = ExportSection::new();
        exports.export("memory", ExportKind::Memory, 0);
        // Deduplicate: BTreeSet for ordered, unique entry point export
        let unique_eps: BTreeSet<u16> = self.entry_points.iter().copied().collect();
        for ep in unique_eps {
            let fn_idx = ri(ep, false) + NUM_IMPORTS;
            let name = format!("bb_{ep:05o}");
            exports.export(&name, ExportKind::Func, fn_idx);
        }
        module.section(&exports);

        // Code section
        let mut code = CodeSection::new();
        for f in functions {
            code.function(&f);
        }
        module.section(&code);

        Ok(module.finish())
    }
}

// ─── Terminator emission ──────────────────────────────────────────────────────

/// Emit the WASM control-flow terminator for one AGC instruction.
///
/// - `FallThrough`: handled by the natural `len=2` chain for EXTEND=0
///   non-EXTEND-opcode instructions; explicit `jmp()` otherwise.
/// - `Jump`: unconditional `jmp()` to the target's extend=false function.
/// - `CondBranch`: inline `if / ReturnCall / End` for the taken edge, then
///   `jmp()` for the fall-through edge.
/// - `CcsBranch` / `Indirect`: inline chain of `if / ReturnCall / End` + final
///   `Unreachable`.
/// - `Halt`: `Unreachable`.
fn emit_direct_terminator(
    reactor: &mut Reactor<()>,
    ctx: &mut (),
    instr: &DirectInstr,
) -> Result<(), String> {
    let next_addr = instr.addr.wrapping_add(1) & 0x7FFF;

    match &instr.terminator {
        Terminator::FallThrough(_) => {
            if instr.instr_type == Some(InstrType::Extend) {
                // EXTEND opcode: natural chain goes to (next, ext=false),
                // but the next instruction must be decoded as an extracode.
                reactor
                    .jmp(ctx, FuncIdx(ri(next_addr, true)), 0)
                    .unwrap();
            } else if instr.extend {
                // After an EXTEND=1 instruction, EXTEND resets to false.
                // Natural chain would go to (next, ext=true); redirect.
                reactor
                    .jmp(ctx, FuncIdx(ri(next_addr, false)), 0)
                    .unwrap();
            }
            // else: natural len=2 chain correctly targets (next, ext=false).
        }

        Terminator::Jump(target) => {
            reactor.jmp(ctx, FuncIdx(ri(*target, false)), 0).unwrap();
        }

        Terminator::CondBranch { taken, fallthru } => {
            // Read Z; if Z == taken, tail-call taken; else fall through.
            feed(reactor, ctx, Instruction::I32Const(0))?;
            feed(reactor, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, ctx, Instruction::I32Const(*taken as i32))?;
            feed(reactor, ctx, Instruction::I32Eq)?;
            feed(reactor, ctx, Instruction::If(BlockType::Empty))?;
            feed(reactor, ctx, Instruction::ReturnCall(ri(*taken, false) + NUM_IMPORTS))?;
            feed(reactor, ctx, Instruction::End)?;
            // Fall through to fallthru address.
            reactor.jmp(ctx, FuncIdx(ri(*fallthru, false)), 0).unwrap();
        }

        Terminator::CcsBranch(targets) => {
            // Load Z once; then a chain of if/ReturnCall/End guards.
            feed(reactor, ctx, Instruction::I32Const(0))?;
            feed(reactor, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
            for &t in targets.iter() {
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(t as i32))?;
                feed(reactor, ctx, Instruction::I32Eq)?;
                feed(reactor, ctx, Instruction::If(BlockType::Empty))?;
                feed(reactor, ctx, Instruction::ReturnCall(ri(t, false) + NUM_IMPORTS))?;
                feed(reactor, ctx, Instruction::End)?;
            }
            feed(reactor, ctx, Instruction::Unreachable)?;
        }

        Terminator::Indirect { possible_targets } => {
            // Load Z; chain of if/ReturnCall/End for each known target.
            feed(reactor, ctx, Instruction::I32Const(0))?;
            feed(reactor, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
            for &t in possible_targets.iter() {
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(t as i32))?;
                feed(reactor, ctx, Instruction::I32Eq)?;
                feed(reactor, ctx, Instruction::If(BlockType::Empty))?;
                feed(reactor, ctx, Instruction::ReturnCall(ri(t, false) + NUM_IMPORTS))?;
                feed(reactor, ctx, Instruction::End)?;
            }
            // No match: unreachable (host should dispatch before calling).
            feed(reactor, ctx, Instruction::Unreachable)?;
        }

        Terminator::Halt => {
            feed(reactor, ctx, Instruction::Unreachable)?;
        }
    }
    Ok(())
}

// ─── Instruction helpers ──────────────────────────────────────────────────────

fn mem16(offset: u64) -> MemArg {
    MemArg { offset, align: 1, memory_index: 0 }
}

fn feed(
    reactor: &mut Reactor<()>,
    ctx: &mut (),
    instr: Instruction<'_>,
) -> Result<(), String> {
    reactor.feed(ctx, &instr).map_err(|e| match e {})
}

/// Map a TC2 register address to its WASM linear-memory byte offset.
fn tc2_reg_offset(addr: u16) -> Option<u64> {
    match addr {
        0x0000 => Some(MEM_A),
        0x0001 => Some(MEM_L),
        0x0002 => Some(MEM_Q),
        0x0003 => Some(MEM_EB),
        0x0004 => Some(MEM_FB),
        0x0005 => Some(MEM_Z),
        0x0006 => Some(MEM_BB),
        0xFF00 => Some(MEM_TMP),
        0xFF01 => Some(MEM_EXTEND),
        0xFF02 => Some(MEM_INHINT),
        0xFF03 => Some(MEM_INSTR_WORD),
        _ => None,
    }
}

// ─── LOAD / STORE with a known (compile-time) TC2 address ────────────────────

fn emit_load_known(
    reactor: &mut Reactor<()>,
    ctx: &mut (),
    addr: u16,
) -> Result<(), String> {
    if let Some(off) = tc2_reg_offset(addr) {
        feed(reactor, ctx, Instruction::I32Const(0))?;
        feed(reactor, ctx, Instruction::I32Load16U(mem16(off)))?;
    } else if addr == 0x0007 {
        feed(reactor, ctx, Instruction::I32Const(0))?;
    } else if addr >= 0x8000 {
        feed(reactor, ctx, Instruction::I32Const((addr - 0x8000) as i32))?;
        feed(reactor, ctx, Instruction::Call(FN_CHAN_READ))?;
        feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
        feed(reactor, ctx, Instruction::I32And)?;
    } else {
        feed(reactor, ctx, Instruction::I32Const(addr as i32))?;
        feed(reactor, ctx, Instruction::Call(FN_MEM_READ))?;
        feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
        feed(reactor, ctx, Instruction::I32And)?;
    }
    Ok(())
}

fn emit_store_known(
    reactor: &mut Reactor<()>,
    ctx: &mut (),
    addr: u16,
    mask15: bool,
) -> Result<(), String> {
    if mask15 {
        feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
        feed(reactor, ctx, Instruction::I32And)?;
    }
    if let Some(off) = tc2_reg_offset(addr) {
        feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::I32Const(0))?;
        feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::I32Store16(mem16(off)))?;
    } else if addr == 0x0007 {
        feed(reactor, ctx, Instruction::Drop)?;
    } else if addr >= 0x8000 {
        feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::I32Const((addr - 0x8000) as i32))?;
        feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::Call(FN_CHAN_WRITE))?;
    } else {
        feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::I32Const(addr as i32))?;
        feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::Call(FN_MEM_WRITE))?;
    }
    Ok(())
}

// ─── TC2 bytecode → WASM (recursive descent) ─────────────────────────────────

/// Translate TC2 bytecode `code[start..]` into WASM instructions fed into
/// the reactor.
///
/// Intra-bytecode conditional jumps (`JUMP_NOT` / `JUMP_IF`) are mapped to
/// WASM `if/else/end` blocks via recursive descent — TC2 jumps are always
/// forward-only so the nesting is always well-formed.
fn emit_bc_segment(
    reactor: &mut Reactor<()>,
    ctx: &mut (),
    code: &[u16],
    start: usize,
) -> Result<(), String> {
    let mut pc = start;
    while pc < code.len() {
        let opc = code[pc];
        pc += 1;

        match opc {
            op::RET => break,

            op::DUP => {
                feed(reactor, ctx, Instruction::LocalTee(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
            }
            op::SWAP => {
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR1))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR1))?;
            }
            op::DROP => { feed(reactor, ctx, Instruction::Drop)?; }

            op::ADD  => { feed(reactor, ctx, Instruction::I32Add)?; }
            op::SUB  => { feed(reactor, ctx, Instruction::I32Sub)?; }
            op::AND  => { feed(reactor, ctx, Instruction::I32And)?; }
            op::OR   => { feed(reactor, ctx, Instruction::I32Or)?; }
            op::XOR  => { feed(reactor, ctx, Instruction::I32Xor)?; }
            op::NOT  => {
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32Xor)?;
            }
            op::MASK15 => {
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::NEG => {
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Sub)?;
            }
            op::LSHR_STK => {
                feed(reactor, ctx, Instruction::I32Const(15))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32ShrU)?;
            }
            op::LSHL_STK => {
                feed(reactor, ctx, Instruction::I32Const(15))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Shl)?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }

            op::IMUL_HI15 => {
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Mul)?;
                feed(reactor, ctx, Instruction::I32Const(15))?;
                feed(reactor, ctx, Instruction::I32ShrS)?;
            }
            op::IMUL_LO15 => {
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Mul)?;
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::IDIV_Q15 => {
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR1))?;
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::I32Const(15))?;
                feed(reactor, ctx, Instruction::I32Shl)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Or)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR1))?;
                feed(reactor, ctx, Instruction::I32DivS)?;
            }
            op::IDIV_R15 => {
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR1))?;
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::I32Const(15))?;
                feed(reactor, ctx, Instruction::I32Shl)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Or)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR1))?;
                feed(reactor, ctx, Instruction::I32RemS)?;
            }

            op::IS_POS => {
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalTee(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0))?;
                feed(reactor, ctx, Instruction::I32Ne)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0x4000))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Eqz)?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::IS_PLUS_ZERO => {
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Eqz)?;
            }
            op::IS_NEG => {
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalTee(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0x4000))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Const(0))?;
                feed(reactor, ctx, Instruction::I32Ne)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32Ne)?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::IS_MINUS_ZERO => {
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32Eq)?;
            }
            op::IS_ZERO_OR_NEG => {
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalTee(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Eqz)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32Eq)?;
                feed(reactor, ctx, Instruction::I32Or)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0x4000))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Const(0))?;
                feed(reactor, ctx, Instruction::I32Ne)?;
                feed(reactor, ctx, Instruction::I32Or)?;
            }
            op::HAS_OVERFLOW => {
                feed(reactor, ctx, Instruction::I32Const(14))?;
                feed(reactor, ctx, Instruction::I32ShrU)?;
                feed(reactor, ctx, Instruction::I32Const(3))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalTee(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(1))?;
                feed(reactor, ctx, Instruction::I32Eq)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(2))?;
                feed(reactor, ctx, Instruction::I32Eq)?;
                feed(reactor, ctx, Instruction::I32Or)?;
            }
            op::BOOL_AND => { feed(reactor, ctx, Instruction::I32And)?; }
            op::BOOL_NOT => { feed(reactor, ctx, Instruction::I32Eqz)?; }

            op::LOAD_T  => { feed(reactor, ctx, Instruction::LocalGet(LOCAL_T))?; }
            op::STORE_T => { feed(reactor, ctx, Instruction::LocalSet(LOCAL_T))?; }

            op::GET_OFF       => { feed(reactor, ctx, Instruction::LocalGet(LOCAL_OFF))?; }
            op::SET_OFF_STACK => { feed(reactor, ctx, Instruction::LocalSet(LOCAL_OFF))?; }

            op::LOAD_OFF => {
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_OFF))?;
                feed(reactor, ctx, Instruction::Call(FN_MEM_READ))?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::STORE_OFF => {
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_OFF))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::Call(FN_MEM_WRITE))?;
            }
            op::LOAD_OFF1 => {
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_OFF))?;
                feed(reactor, ctx, Instruction::I32Const(1))?;
                feed(reactor, ctx, Instruction::I32Add)?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::Call(FN_MEM_READ))?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::STORE_OFF1 => {
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_OFF))?;
                feed(reactor, ctx, Instruction::I32Const(1))?;
                feed(reactor, ctx, Instruction::I32Add)?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::Call(FN_MEM_WRITE))?;
            }

            op::LOAD_CHAN_OFF => {
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_OFF))?;
                feed(reactor, ctx, Instruction::I32Const(0x01FF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::Call(FN_CHAN_READ))?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::STORE_CHAN_OFF => {
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_OFF))?;
                feed(reactor, ctx, Instruction::I32Const(0x01FF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::Call(FN_CHAN_WRITE))?;
            }

            op::LOAD_IND => {
                feed(reactor, ctx, Instruction::Call(FN_MEM_READ))?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::STORE_IND => {
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR1))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR1))?;
                feed(reactor, ctx, Instruction::Call(FN_MEM_WRITE))?;
            }

            op::PUSH_IMM => {
                let v = code[pc] as i32; pc += 1;
                feed(reactor, ctx, Instruction::I32Const(v))?;
            }
            op::LOAD => {
                let addr = code[pc] as u16; pc += 1;
                emit_load_known(reactor, ctx, addr)?;
            }
            op::STORE => {
                let addr = code[pc] as u16; pc += 1;
                emit_store_known(reactor, ctx, addr, false)?;
            }
            op::STORE15 => {
                let addr = code[pc] as u16; pc += 1;
                emit_store_known(reactor, ctx, addr, true)?;
            }
            op::SET_OFF => {
                let v = code[pc] as i32; pc += 1;
                feed(reactor, ctx, Instruction::I32Const(v))?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_OFF))?;
            }
            op::LSHR => {
                let k = code[pc] as i32; pc += 1;
                feed(reactor, ctx, Instruction::I32Const(k))?;
                feed(reactor, ctx, Instruction::I32ShrU)?;
            }
            op::LSHL => {
                let k = code[pc] as i32; pc += 1;
                feed(reactor, ctx, Instruction::I32Const(k))?;
                feed(reactor, ctx, Instruction::I32Shl)?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }

            op::JUMP_NOT => {
                let off = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                feed(reactor, ctx, Instruction::If(BlockType::Empty))?;
                emit_bc_segment(reactor, ctx, code, pc)?;
                feed(reactor, ctx, Instruction::Else)?;
                emit_bc_segment(reactor, ctx, code, target)?;
                feed(reactor, ctx, Instruction::End)?;
                return Ok(());
            }
            op::JUMP_IF => {
                let off = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                feed(reactor, ctx, Instruction::If(BlockType::Empty))?;
                emit_bc_segment(reactor, ctx, code, target)?;
                feed(reactor, ctx, Instruction::Else)?;
                emit_bc_segment(reactor, ctx, code, pc)?;
                feed(reactor, ctx, Instruction::End)?;
                return Ok(());
            }
            op::JUMP => {
                let off = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                emit_bc_segment(reactor, ctx, code, target)?;
                return Ok(());
            }

            _ => {
                feed(reactor, ctx, Instruction::Unreachable)?;
            }
        }
    }
    Ok(())
}
