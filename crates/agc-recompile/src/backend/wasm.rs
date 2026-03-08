//! WASM backend — translates TC2 stack bytecode to a WebAssembly module
//! using the yecta [`Reactor`] for control-flow-aware function building.
//!
//! ## Design
//!
//! Each AGC basic block becomes one WASM function.  The TC2 evaluation stack
//! maps directly to the WASM operand stack (all values are `i32`).  The T and
//! OFF registers become WASM locals.  Conditional intra-bytecode jumps
//! (`JUMP_NOT` / `JUMP_IF`) are converted to structured `if/else/end` blocks
//! via recursive descent over the TC2 bytecode.
//!
//! ### WASM module structure
//!
//! ```text
//! imports:
//!   env.mem_read (addr:i32) -> i32        — AGC memory read (host handles register dispatch)
//!   env.mem_write(addr:i32, val:i32)      — AGC memory write
//!   env.chan_read (ch:i32) -> i32         — I/O channel read
//!   env.chan_write(ch:i32, val:i32)       — I/O channel write
//!
//! memory:  1 page (64 KB) — register file at fixed byte offsets:
//!   0: A, 2: L, 4: Q, 6: EB, 8: FB, 10: Z, 12: BB,
//!   14: TMP, 16: EXTEND, 18: INHINT, 20: INSTR_WORD
//!
//! functions: one per basic block (all type: () -> ())
//! exports:   entry-point blocks by name "bb_XXXXX"
//!            linear memory as "memory"
//! ```
//!
//! ### Locals layout per generated function
//!
//! | Index | Name    | Purpose                    |
//! |-------|---------|----------------------------|
//! | 0     | T       | T register (i32)           |
//! | 1     | OFF     | OFF register (i32)         |
//! | 2     | SCR0    | scratch — stores / DUP     |
//! | 3     | SCR1    | scratch — SWAP / wide-ops  |

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use wasm_encoder::{
    BlockType, CodeSection, EntityType, ExportKind, ExportSection, FunctionSection,
    ImportSection, Instruction, MemArg, MemorySection, MemoryType, Module, TypeSection,
    ValType,
};
use yecta::{FuncIdx, Reactor};

use crate::backend::Backend;
use crate::ir::{InstrStream, Terminator};
use agc_lower::bytecode::op;

// ─── WASM memory layout: byte offsets for the register file ───────────────────

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

// ─── Backend struct ───────────────────────────────────────────────────────────

pub struct WasmBackend;

impl Default for WasmBackend {
    fn default() -> Self { WasmBackend }
}

impl Backend for WasmBackend {
    type Output = Vec<u8>;
    type Error = String;

    fn emit(&mut self, stream: &InstrStream) -> Result<Vec<u8>, String> {
        // ── Assign WASM function index to each basic block ───────────────────
        //    Imports occupy 0..NUM_IMPORTS; generated blocks follow.
        let addr_to_fn: BTreeMap<u16, u32> = stream
            .blocks
            .keys()
            .enumerate()
            .map(|(i, &addr)| (addr, NUM_IMPORTS + i as u32))
            .collect();
        let n_blocks = stream.blocks.len() as u32;

        // ── Build functions with the yecta Reactor ───────────────────────────
        let mut reactor: Reactor<()> = Reactor::with_base_func_offset(NUM_IMPORTS);
        let mut ctx = ();

        for (_addr, block) in &stream.blocks {
            // 4 locals: T (i32), OFF (i32), SCR0 (i32), SCR1 (i32)
            reactor
                .next(&mut ctx, core::iter::once((4u32, ValType::I32)), 0)
                .unwrap();

            for rec in &block.instrs {
                let next_pc = ((rec.pc + 1) & 0x7FFF) as i32;
                // Pre-advance Z so Expr::Z reads are correct.
                feed(&mut reactor, &mut ctx, Instruction::I32Const(0))?;
                feed(&mut reactor, &mut ctx, Instruction::I32Const(next_pc))?;
                feed(&mut reactor, &mut ctx, Instruction::I32Store16(mem16(MEM_Z)))?;
                // Expose raw instruction word for LOAD(INSTR_WORD).
                feed(&mut reactor, &mut ctx, Instruction::I32Const(0))?;
                feed(&mut reactor, &mut ctx, Instruction::I32Const(rec.raw_word as i32))?;
                feed(&mut reactor, &mut ctx, Instruction::I32Store16(mem16(MEM_INSTR_WORD)))?;

                emit_bc_segment(&mut reactor, &mut ctx, &rec.bytecode, 0)?;
            }

            emit_terminator(&mut reactor, &mut ctx, &block.terminator, &addr_to_fn)?;

            // Seal (finalize) this function — emits `End` and severs edges.
            reactor
                .seal(&mut ctx, &Instruction::End)
                .unwrap();
        }

        let functions = reactor.into_fns();

        // ── Assemble WASM module ─────────────────────────────────────────────
        let mut module = Module::new();

        // Type section
        //   type 0: () -> ()         (block functions)
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

        // Function section — all blocks use type 0
        let mut funcs = FunctionSection::new();
        for _ in 0..n_blocks {
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

        // Export section — entry points + linear memory
        let mut exports = ExportSection::new();
        exports.export("memory", ExportKind::Memory, 0);
        for &ep in &stream.entry_points {
            if let Some(&fn_idx) = addr_to_fn.get(&ep) {
                let name = format!("bb_{ep:05o}");
                exports.export(&name, ExportKind::Func, fn_idx);
            }
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

// ─── LOAD / STORE with known (compile-time) TC2 address ──────────────────────

/// Emit WASM to push one 16-bit value read from TC2 address `addr`.
fn emit_load_known(
    reactor: &mut Reactor<()>,
    ctx: &mut (),
    addr: u16,
) -> Result<(), String> {
    if let Some(off) = tc2_reg_offset(addr) {
        // Inline register read from WASM linear memory.
        feed(reactor, ctx, Instruction::I32Const(0))?;
        feed(reactor, ctx, Instruction::I32Load16U(mem16(off)))?;
    } else if addr == 0x0007 {
        // ZERO register always reads 0.
        feed(reactor, ctx, Instruction::I32Const(0))?;
    } else if addr >= 0x8000 {
        // Channel read.
        feed(reactor, ctx, Instruction::I32Const((addr - 0x8000) as i32))?;
        feed(reactor, ctx, Instruction::Call(FN_CHAN_READ))?;
        feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
        feed(reactor, ctx, Instruction::I32And)?;
    } else {
        // General AGC memory: call the imported mem_read.
        feed(reactor, ctx, Instruction::I32Const(addr as i32))?;
        feed(reactor, ctx, Instruction::Call(FN_MEM_READ))?;
        feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
        feed(reactor, ctx, Instruction::I32And)?;
    }
    Ok(())
}

/// Emit WASM to pop one value from the operand stack and write it to TC2
/// address `addr`.  If `mask15` is true, the value is first masked to 15 bits.
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
        // Save value to scratch, push base addr 0, reload, store.
        feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::I32Const(0))?;
        feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::I32Store16(mem16(off)))?;
    } else if addr == 0x0007 {
        // ZERO register: writes are ignored.
        feed(reactor, ctx, Instruction::Drop)?;
    } else if addr >= 0x8000 {
        // Channel write: save val, push ch, push val, call.
        feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::I32Const((addr - 0x8000) as i32))?;
        feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::Call(FN_CHAN_WRITE))?;
    } else {
        // General AGC memory: save val, push addr, push val, call.
        feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::I32Const(addr as i32))?;
        feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
        feed(reactor, ctx, Instruction::Call(FN_MEM_WRITE))?;
    }
    Ok(())
}

// ─── TC2 bytecode → WASM (recursive descent) ─────────────────────────────────

/// Translate TC2 bytecode `code[start..]` into WASM instructions.
///
/// Stops at the first `RET` (or end of slice).  Intra-bytecode conditional
/// jumps (`JUMP_NOT` / `JUMP_IF`) are mapped to WASM `if/else/end` blocks
/// via recursive descent — TC2 bytecode jumps are always forward-only, so
/// the resulting blocks are always well-structured.
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
            // ── Control ───────────────────────────────────────────────────
            op::RET => break,

            // ── Stack ─────────────────────────────────────────────────────
            op::DUP => {
                // local.tee SCR0 leaves value on stack, then get SCR0 pushes again.
                feed(reactor, ctx, Instruction::LocalTee(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
            }
            op::SWAP => {
                // (a, b) → (b, a): save b, save a, get b, get a.
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?; // save b
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR1))?; // save a
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?; // push b
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR1))?; // push a
            }
            op::DROP => {
                feed(reactor, ctx, Instruction::Drop)?;
            }

            // ── Arithmetic ────────────────────────────────────────────────
            op::ADD  => { feed(reactor, ctx, Instruction::I32Add)?; }
            op::SUB  => { feed(reactor, ctx, Instruction::I32Sub)?; }
            op::AND  => { feed(reactor, ctx, Instruction::I32And)?; }
            op::OR   => { feed(reactor, ctx, Instruction::I32Or)?; }
            op::XOR  => { feed(reactor, ctx, Instruction::I32Xor)?; }
            op::NOT  => {
                // 16-bit bitwise NOT: XOR with 0xFFFF.
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32Xor)?;
            }
            op::MASK15 => {
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::NEG => {
                // Two's-complement negation: 0 - v.
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Sub)?;
            }
            op::LSHR_STK => {
                // (val, k) → val >> (k & 15).
                feed(reactor, ctx, Instruction::I32Const(15))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32ShrU)?;
            }
            op::LSHL_STK => {
                // (val, k) → (val << (k & 15)) & 0xFFFF.
                feed(reactor, ctx, Instruction::I32Const(15))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Shl)?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }

            // ── Wide-integer ops ──────────────────────────────────────────
            op::IMUL_HI15 => {
                // (a, b) → (sign32(a) * sign32(b)) >> 15.
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Mul)?;
                feed(reactor, ctx, Instruction::I32Const(15))?;
                feed(reactor, ctx, Instruction::I32ShrS)?;
            }
            op::IMUL_LO15 => {
                // (a, b) → (sign32(a) * sign32(b)) & 0x7FFF.
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Extend16S)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Mul)?;
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::IDIV_Q15 => {
                // (hi, lo, d) → ((hi<<15) | (lo & 0x7FFF)) / d, or 0 if d==0.
                // Stack order at entry: hi (bottom), lo, d (top).
                feed(reactor, ctx, Instruction::I32Extend16S)?;    // sign-extend d
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR1))?; // d → SCR1
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;           // lo & 0x7FFF
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?; // lo → SCR0
                feed(reactor, ctx, Instruction::I32Extend16S)?;    // sign-extend hi
                feed(reactor, ctx, Instruction::I32Const(15))?;
                feed(reactor, ctx, Instruction::I32Shl)?;           // hi << 15
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Or)?;            // n = hi<<15 | lo
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR1))?; // d
                feed(reactor, ctx, Instruction::I32DivS)?;          // n / d
            }
            op::IDIV_R15 => {
                // Like IDIV_Q15 but remainder.
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

            // ── OC bit-pattern predicates — all produce 0 or 1 ───────────
            op::IS_POS => {
                // x15 = x & 0x7FFF; result = x15 != 0 && (x15 & 0x4000) == 0.
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalTee(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0))?;
                feed(reactor, ctx, Instruction::I32Ne)?;             // x15 != 0
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0x4000))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Eqz)?;            // (x15 & 0x4000) == 0
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::IS_PLUS_ZERO => {
                // (x & 0x7FFF) == 0.
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Eqz)?;
            }
            op::IS_NEG => {
                // x15 = x & 0x7FFF; result = (x15 & 0x4000) != 0 && x15 != 0x7FFF.
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalTee(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0x4000))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Const(0))?;
                feed(reactor, ctx, Instruction::I32Ne)?;             // bit 14 set
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32Ne)?;             // not minus-zero
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::IS_MINUS_ZERO => {
                // (x & 0x7FFF) == 0x7FFF.
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32Eq)?;
            }
            op::IS_ZERO_OR_NEG => {
                // x15==0 || x15==0x7FFF || (x15 & 0x4000) != 0.
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::LocalTee(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Eqz)?;            // x15 == 0
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, ctx, Instruction::I32Eq)?;             // x15 == 0x7FFF
                feed(reactor, ctx, Instruction::I32Or)?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::I32Const(0x4000))?;
                feed(reactor, ctx, Instruction::I32And)?;
                feed(reactor, ctx, Instruction::I32Const(0))?;
                feed(reactor, ctx, Instruction::I32Ne)?;             // negative
                feed(reactor, ctx, Instruction::I32Or)?;
            }
            op::HAS_OVERFLOW => {
                // ((x >> 14) & 3) == 1 || == 2.
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
            op::BOOL_AND => {
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::BOOL_NOT => {
                feed(reactor, ctx, Instruction::I32Eqz)?;
            }

            // ── T register ────────────────────────────────────────────────
            op::LOAD_T  => { feed(reactor, ctx, Instruction::LocalGet(LOCAL_T))?; }
            op::STORE_T => { feed(reactor, ctx, Instruction::LocalSet(LOCAL_T))?; }

            // ── OFF register ──────────────────────────────────────────────
            op::GET_OFF       => { feed(reactor, ctx, Instruction::LocalGet(LOCAL_OFF))?; }
            op::SET_OFF_STACK => { feed(reactor, ctx, Instruction::LocalSet(LOCAL_OFF))?; }

            // ── OFF-relative memory (dynamic address via import) ───────────
            op::LOAD_OFF => {
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_OFF))?;
                feed(reactor, ctx, Instruction::Call(FN_MEM_READ))?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::STORE_OFF => {
                // Pop val, write to mem[OFF].
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

            // ── Channel (OFF-relative) ────────────────────────────────────
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

            // ── Indirect memory ───────────────────────────────────────────
            op::LOAD_IND => {
                // Pop addr, push mem[addr].
                feed(reactor, ctx, Instruction::Call(FN_MEM_READ))?;
                feed(reactor, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, ctx, Instruction::I32And)?;
            }
            op::STORE_IND => {
                // Stack: (..., val, addr) — addr is top.
                // Emit: save addr to SCR0, save val to SCR1, call mem_write(SCR0, SCR1).
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?; // addr
                feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR1))?; // val
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR1))?;
                feed(reactor, ctx, Instruction::Call(FN_MEM_WRITE))?;
            }

            // ── Two-word instructions ──────────────────────────────────────
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

            // ── Intra-bytecode conditional jumps → WASM if/else/end ───────
            op::JUMP_NOT => {
                // Jump to `target` if TOS == 0 (zero → jump).
                // WASM `if` executes if TOS != 0, so the "not jumped" branch
                // (TOS ≠ 0) goes in the `if` arm, jumped branch in `else`.
                let off = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                feed(reactor, ctx, Instruction::If(BlockType::Empty))?;
                emit_bc_segment(reactor, ctx, code, pc)?;     // not-jumped path
                feed(reactor, ctx, Instruction::Else)?;
                emit_bc_segment(reactor, ctx, code, target)?; // jumped path
                feed(reactor, ctx, Instruction::End)?;
                return Ok(());
            }
            op::JUMP_IF => {
                // Jump to `target` if TOS != 0 (nonzero → jump).
                // WASM `if` executes if TOS != 0 → jumped branch in `if` arm.
                let off = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                feed(reactor, ctx, Instruction::If(BlockType::Empty))?;
                emit_bc_segment(reactor, ctx, code, target)?; // jumped path
                feed(reactor, ctx, Instruction::Else)?;
                emit_bc_segment(reactor, ctx, code, pc)?;     // not-jumped path
                feed(reactor, ctx, Instruction::End)?;
                return Ok(());
            }
            op::JUMP => {
                // Unconditional forward jump: skip to target.
                let off = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                emit_bc_segment(reactor, ctx, code, target)?;
                return Ok(());
            }

            _ => {
                // Unknown opcode — emit unreachable so the validator surfaces it.
                feed(reactor, ctx, Instruction::Unreachable)?;
            }
        }
    }
    Ok(())
}

// ─── Terminator → WASM tail calls ────────────────────────────────────────────

/// Emit the WASM tail-call dispatch for a block terminator.
///
/// The TC2 bytecode has already written the branch target (or `next_pc`) into
/// memory slot `MEM_Z`.  For statically-known targets we emit direct
/// `return_call`; for CCS / indirect we read Z and emit a chain of
/// `if / return_call / end` guards.
fn emit_terminator(
    reactor: &mut Reactor<()>,
    ctx: &mut (),
    term: &Terminator,
    addr_to_fn: &BTreeMap<u16, u32>,
) -> Result<(), String> {
    match term {
        Terminator::FallThrough(addr) | Terminator::Jump(addr) => {
            if let Some(&fn_idx) = addr_to_fn.get(addr) {
                // Use the Reactor's jmp() for direct tail-call edges.
                reactor.jmp(ctx, FuncIdx(fn_idx - NUM_IMPORTS), 0)
                    .unwrap();
            } else {
                feed(reactor, ctx, Instruction::Unreachable)?;
            }
        }

        Terminator::CondBranch { taken, fallthru } => {
            // Read Z from memory; branch to `taken` if Z == taken, else fallthru.
            let taken_fn   = addr_to_fn.get(taken).copied();
            let fallthru_fn = addr_to_fn.get(fallthru).copied();
            feed(reactor, ctx, Instruction::I32Const(0))?;
            feed(reactor, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, ctx, Instruction::I32Const(*taken as i32))?;
            feed(reactor, ctx, Instruction::I32Eq)?;
            feed(reactor, ctx, Instruction::If(BlockType::Empty))?;
            if let Some(fn_idx) = taken_fn {
                feed(reactor, ctx, Instruction::ReturnCall(fn_idx))?;
            } else {
                feed(reactor, ctx, Instruction::Unreachable)?;
            }
            feed(reactor, ctx, Instruction::Else)?;
            if let Some(fn_idx) = fallthru_fn {
                feed(reactor, ctx, Instruction::ReturnCall(fn_idx))?;
            } else {
                feed(reactor, ctx, Instruction::Unreachable)?;
            }
            feed(reactor, ctx, Instruction::End)?;
        }

        Terminator::CcsBranch(targets) => {
            // Read Z; emit a chain of if/return_call guards for each target.
            feed(reactor, ctx, Instruction::I32Const(0))?;
            feed(reactor, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
            for &t in targets.iter() {
                if let Some(&fn_idx) = addr_to_fn.get(&t) {
                    feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                    feed(reactor, ctx, Instruction::I32Const(t as i32))?;
                    feed(reactor, ctx, Instruction::I32Eq)?;
                    feed(reactor, ctx, Instruction::If(BlockType::Empty))?;
                    feed(reactor, ctx, Instruction::ReturnCall(fn_idx))?;
                    feed(reactor, ctx, Instruction::End)?;
                }
            }
            feed(reactor, ctx, Instruction::Unreachable)?;
        }

        Terminator::Indirect { possible_targets } => {
            // Read Z; try each known target.
            feed(reactor, ctx, Instruction::I32Const(0))?;
            feed(reactor, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, ctx, Instruction::LocalSet(LOCAL_SCR0))?;
            for &t in possible_targets.iter() {
                if let Some(&fn_idx) = addr_to_fn.get(&t) {
                    feed(reactor, ctx, Instruction::LocalGet(LOCAL_SCR0))?;
                    feed(reactor, ctx, Instruction::I32Const(t as i32))?;
                    feed(reactor, ctx, Instruction::I32Eq)?;
                    feed(reactor, ctx, Instruction::If(BlockType::Empty))?;
                    feed(reactor, ctx, Instruction::ReturnCall(fn_idx))?;
                    feed(reactor, ctx, Instruction::End)?;
                }
            }
            // No match: unreachable (host should have re-dispatched before calling).
            feed(reactor, ctx, Instruction::Unreachable)?;
        }

        Terminator::Halt => {
            feed(reactor, ctx, Instruction::Unreachable)?;
        }
    }
    Ok(())
}
