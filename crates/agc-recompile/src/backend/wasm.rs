//! WASM direct backend — translates the AGC instruction stream into a
//! WebAssembly module using the yecta [`Reactor`].
//!
//! ## Generics
//!
//! ```text
//! WasmDirectBackend<'cb, 'ctx, Context = (), Err = String>
//! ```
//!
//! * `'cb / 'ctx` — lifetimes of installed [`TrapConfig`] callbacks.
//! * `Context`   — user context threaded through every emission call.
//! * `Err`       — error type returned by [`DirectBackend::feed_instr`] /
//!                 [`DirectBackend::finish`] and by all trap callbacks.
//!
//! The underlying [`Reactor`] is always `Reactor<Context, Err, Function, LocalPool>`;
//! `F` and `P` are not exposed as type parameters on the backend itself.
//!
//! ## Local-variable layout
//!
//! AGC functions are `() → ()` — all persistent state lives in linear memory.
//! The four scratch locals are per-function non-parameter locals, declared via
//! [`LocalLayout`] on every `feed_instr` call:
//!
//! ```text
//! locals_mark.total_locals == 0   (no params; traps may add param slots here)
//! local[locals_mark+0]  T     i32  TC2 T register
//! local[locals_mark+1]  OFF   i32  TC2 OFF address register
//! local[locals_mark+2]  SCR0  i32  scratch 0
//! local[locals_mark+3]  SCR1  i32  scratch 1
//! …                     trap-declared locals follow
//! ```
//!
//! ## Reactor function layout
//!
//! ```text
//! reactor_idx(addr, false) = 2 * addr
//! reactor_idx(addr, true)  = 2 * addr + 1
//! WASM function index      = reactor_idx + NUM_IMPORTS
//! ```
//!
//! ## WASM module structure
//!
//! ```text
//! imports:
//!   env.mem_read (addr:i32) -> i32
//!   env.mem_write(addr:i32, val:i32)
//!   env.chan_read (ch:i32) -> i32
//!   env.chan_write(ch:i32, val:i32)
//!
//! memory:  1 page (64 KB) — register file at fixed byte offsets:
//!   0:A  2:L  4:Q  6:EB  8:FB  10:Z  12:BB  14:TMP  16:EXTEND  18:INHINT  20:INSTR_WORD
//!
//! functions: 8192 (4096 addrs × 2 extend states)
//! exports:   "memory" + entry-point "bb_OOOOO" names
//! ```

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use wasm_encoder::{
    BlockType, CodeSection, EntityType, ExportKind, ExportSection, Function, FunctionSection,
    ImportSection, Instruction, MemArg, MemorySection, MemoryType, Module, TypeSection, ValType,
};

use yecta::{FuncIdx, LocalLayout, LocalPool, LocalPoolBackend, LocalSlot, Mark, Reactor};
use yecta::layout::CellIdx;

use speet_traps::{
    ArchTag, InsnClass, InstructionInfo, JumpInfo, JumpKind, TrapAction, TrapConfig,
};

use agc_isa::InstrType;

use super::{DirectBackend, DirectInstr};
use crate::ir::Terminator;
use agc_lower::bytecode::op;

// ─── WASM memory layout ───────────────────────────────────────────────────────

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

// ─── Imported function indices ────────────────────────────────────────────────

const FN_MEM_READ:  u32 = 0;
const FN_MEM_WRITE: u32 = 1;
const FN_CHAN_READ:  u32 = 2;
const FN_CHAN_WRITE: u32 = 3;
const NUM_IMPORTS:  u32 = 4;

// ─── Reactor index ────────────────────────────────────────────────────────────

#[inline]
fn ri(addr: u16, extend: bool) -> u32 {
    2 * addr as u32 + extend as u32
}

// ─── Resolved local indices ───────────────────────────────────────────────────

/// Resolved WASM local indices for the four AGC scratch variables.
///
/// Computed once per function from the [`LocalLayout`] after `declare_locals`
/// returns; passed into every emission helper so they do not need to access
/// the layout directly.
#[derive(Clone, Copy)]
struct LocalIndices {
    t:    u32,
    off:  u32,
    scr0: u32,
    scr1: u32,
}

// ─── Backend struct ───────────────────────────────────────────────────────────

/// Direct WASM backend powered by the yecta [`Reactor`].
///
/// # Type parameters
///
/// * `'cb`  — lifetime of installed trap callback borrows.
/// * `'ctx` — lifetime of data captured by trap callbacks.
/// * `Context` — user context threaded through every emission call (default `()`).
/// * `Err`     — error type (default `String`).
///
/// ## Typical construction
///
/// ```ignore
/// // No traps — unit context, string errors:
/// let mut backend = WasmDirectBackend::new(entry_points);
/// backend.feed_instr(&mut (), &instr)?;
/// let wasm = backend.finish(&mut ())?;
///
/// // With a jump trap:
/// let mut my_trap = MyJumpTrap::new();
/// let mut backend = WasmDirectBackend::new(entry_points);
/// backend.set_jump_trap(&mut my_trap);
/// ```
pub struct WasmDirectBackend<
    'cb,
    'ctx,
    Context = (),
    Err     = String,
> {
    reactor:      Reactor<Context, Err, Function, LocalPool>,
    entry_points: Vec<u16>,
    tail_idx:     usize,

    /// Unified local layout.  The mark is placed after any trap-declared
    /// parameter locals (none by default, since AGC fns have no parameters).
    layout:       LocalLayout,
    locals_mark:  Mark,

    /// Slot handles for the four scratch locals (valid after `setup_traps`).
    slot_t:    LocalSlot,
    slot_off:  LocalSlot,
    slot_scr0: LocalSlot,
    slot_scr1: LocalSlot,

    /// Pluggable instruction-level and jump-level trap hooks.
    traps: TrapConfig<'cb, 'ctx, Context, Err>,
}

impl<'cb, 'ctx, Context, Err> WasmDirectBackend<'cb, 'ctx, Context, Err> {
    /// Create a new backend with no traps installed.
    ///
    /// [`setup_traps`] is called automatically; call it again after installing
    /// traps via [`set_instruction_trap`] / [`set_jump_trap`].
    ///
    /// [`setup_traps`]:          WasmDirectBackend::setup_traps
    /// [`set_instruction_trap`]: WasmDirectBackend::set_instruction_trap
    /// [`set_jump_trap`]:        WasmDirectBackend::set_jump_trap
    pub fn new(entry_points: Vec<u16>) -> Self {
        let mut s = Self {
            reactor:     Reactor::with_base_func_offset(NUM_IMPORTS),
            entry_points,
            tail_idx:    0,
            layout:      LocalLayout::empty(),
            locals_mark: Mark { slot_count: 0, total_locals: 0 },
            slot_t:    LocalSlot::default(),
            slot_off:  LocalSlot::default(),
            slot_scr0: LocalSlot::default(),
            slot_scr1: LocalSlot::default(),
            traps: TrapConfig::new(),
        };
        s.setup_traps();
        s
    }

    /// **Phase 1** — (re-)initialize the layout and let traps declare parameter slots.
    ///
    /// AGC functions carry no wasm parameters (state lives in linear memory),
    /// so the params mark is always at `total_locals = 0` unless a trap adds
    /// parameter slots.  Must be called again after installing or removing a
    /// trap so that the layout reflects the current trap set.
    pub fn setup_traps(&mut self) {
        self.layout = LocalLayout::empty();
        // AGC functions are () → (); no arch parameter slots.
        self.traps.declare_params(CellIdx(0), &mut self.layout);
        self.locals_mark = self.layout.mark();
    }

    /// Install an instruction trap and re-run `setup_traps`.
    pub fn set_instruction_trap(
        &mut self,
        trap: &'cb mut (dyn speet_traps::InstructionTrap<Context, Err> + 'ctx),
    ) {
        self.traps.set_instruction_trap(trap);
        self.setup_traps();
    }

    /// Install a jump trap and re-run `setup_traps`.
    pub fn set_jump_trap(
        &mut self,
        trap: &'cb mut (dyn speet_traps::JumpTrap<Context, Err> + 'ctx),
    ) {
        self.traps.set_jump_trap(trap);
        self.setup_traps();
    }
}

impl<'cb, 'ctx, Context, Err> DirectBackend<Context>
    for WasmDirectBackend<'cb, 'ctx, Context, Err>
where
    Err: core::fmt::Display,
{
    type Output = Vec<u8>;
    type Error  = Err;

    fn feed_instr(&mut self, ctx: &mut Context, instr: &DirectInstr) -> Result<(), Err> {
        // ── Phase 2: per-function local setup ────────────────────────────────
        self.layout.rewind(&self.locals_mark);
        self.slot_t    = self.layout.append(1, ValType::I32);
        self.slot_off  = self.layout.append(1, ValType::I32);
        self.slot_scr0 = self.layout.append(1, ValType::I32);
        self.slot_scr1 = self.layout.append(1, ValType::I32);
        self.traps.declare_locals(CellIdx(0), &mut self.layout);

        // Open the yecta function.  len=2 builds two parallel fall-through
        // chains (even = extend=false, odd = extend=true).
        let fn_locals: alloc::vec::Vec<(u32, ValType)> =
            self.layout.iter_since(&self.locals_mark).collect();
        self.reactor.next_with(ctx, Function::new(fn_locals), 2)?;
        self.tail_idx = self.reactor.fn_count().saturating_sub(1);

        let tail_idx = self.tail_idx;

        // Resolve local indices once for this function.
        let li = LocalIndices {
            t:    self.layout.base(self.slot_t),
            off:  self.layout.base(self.slot_off),
            scr0: self.layout.base(self.slot_scr0),
            scr1: self.layout.base(self.slot_scr1),
        };

        // ── Pre-advance Z ─────────────────────────────────────────────────────
        let next_pc = instr.addr.wrapping_add(1) & 0x7FFF;
        feed(&self.reactor, tail_idx, ctx, Instruction::I32Const(0))?;
        feed(&self.reactor, tail_idx, ctx, Instruction::I32Const(next_pc as i32))?;
        feed(&self.reactor, tail_idx, ctx, Instruction::I32Store16(mem16(MEM_Z)))?;

        // ── Expose raw instruction word ───────────────────────────────────────
        feed(&self.reactor, tail_idx, ctx, Instruction::I32Const(0))?;
        feed(&self.reactor, tail_idx, ctx, Instruction::I32Const(instr.raw_word as i32))?;
        feed(&self.reactor, tail_idx, ctx, Instruction::I32Store16(mem16(MEM_INSTR_WORD)))?;

        // ── Phase 3a: instruction trap ────────────────────────────────────────
        let info = InstructionInfo {
            pc:   instr.addr as u64,
            len:  1,
            arch: ArchTag::Other,
            class: classify_agc_insn(instr.instr_type),
        };
        // Split-borrow: reactor (mutable for EmitSink coercion), traps, layout
        // are three distinct fields — Rust split-borrows permit this.
        let (reactor, traps, layout) = (&mut self.reactor, &mut self.traps, &self.layout);
        let action = traps.on_instruction(&info, ctx, reactor, layout)?;

        if action == TrapAction::Skip {
            // Trap already emitted the skip snippet.  Emit a well-formed
            // terminator (fallthrough or jmp) and return.
            emit_direct_terminator(
                &mut self.reactor, self.tail_idx, ctx,
                instr, &mut self.traps, &self.layout, li,
            )?;
            return Ok(());
        }

        // ── Emit TC2 bytecode ─────────────────────────────────────────────────
        emit_bc_segment(&self.reactor, tail_idx, ctx, &instr.bytecode, 0, li)?;

        // ── Emit control-flow terminator ──────────────────────────────────────
        emit_direct_terminator(
            &mut self.reactor, self.tail_idx, ctx,
            instr, &mut self.traps, &self.layout, li,
        )?;

        Ok(())
    }

    fn finish(self, ctx: &mut Context) -> Result<Vec<u8>, Err> {
        let n_funcs = self.reactor.fn_count() as u32;
        let mut functions = self.reactor.into_fns();

        // Close every function body with an End opcode.  F = Function is
        // fixed for this backend; call wasm_encoder::Function::instruction
        // directly (it is infallible and does not need a Context).
        for f in &mut functions {
            f.instruction(&Instruction::End);
        }

        // ── Assemble WASM module ──────────────────────────────────────────────
        let mut module = Module::new();

        // Types
        let mut types = TypeSection::new();
        types.ty().function([], []);              // type 0: () -> ()
        types.ty().function([ValType::I32], [ValType::I32]); // type 1: read
        types.ty().function([ValType::I32, ValType::I32], []); // type 2: write
        module.section(&types);

        // Imports
        let mut imports = ImportSection::new();
        imports.import("env", "mem_read",   EntityType::Function(1));
        imports.import("env", "mem_write",  EntityType::Function(2));
        imports.import("env", "chan_read",  EntityType::Function(1));
        imports.import("env", "chan_write", EntityType::Function(2));
        module.section(&imports);

        // Functions (all type 0)
        let mut funcs = FunctionSection::new();
        for _ in 0..n_funcs { funcs.function(0); }
        module.section(&funcs);

        // Memory
        let mut mems = MemorySection::new();
        mems.memory(MemoryType {
            minimum: 1, maximum: None,
            memory64: false, shared: false, page_size_log2: None,
        });
        module.section(&mems);

        // Exports
        let mut exports = ExportSection::new();
        exports.export("memory", ExportKind::Memory, 0);
        let unique_eps: BTreeSet<u16> = self.entry_points.iter().copied().collect();
        for ep in unique_eps {
            let fn_idx = ri(ep, false) + NUM_IMPORTS;
            let name = format!("bb_{ep:05o}");
            exports.export(&name, ExportKind::Func, fn_idx);
        }
        module.section(&exports);

        // Code
        let mut code = CodeSection::new();
        for f in functions { code.function(&f); }
        module.section(&code);

        Ok(module.finish())
    }
}

// ─── AGC instruction classification ──────────────────────────────────────────

fn classify_agc_insn(instr_type: Option<InstrType>) -> InsnClass {
    match instr_type {
        None => InsnClass::OTHER,
        Some(t) => match t {
            InstrType::Tc | InstrType::Tcf | InstrType::Bzf | InstrType::Bzmf
                => InsnClass::BRANCH,
            InstrType::Ccs
                => InsnClass::BRANCH,
            InstrType::Go
                => InsnClass::BRANCH | InsnClass::INDIRECT,
            InstrType::Resume | InstrType::Rupt | InstrType::Inhint | InstrType::Relint
                => InsnClass::PRIVILEGED,
            InstrType::Ca  | InstrType::Cs  | InstrType::Ad  | InstrType::Ads
            | InstrType::Su | InstrType::Ts  | InstrType::Xch | InstrType::Lxch
            | InstrType::Qxch | InstrType::Dxch | InstrType::Dca | InstrType::Dcs
            | InstrType::Read | InstrType::Write | InstrType::Rand | InstrType::Wand
            | InstrType::Ror  | InstrType::Wor  | InstrType::Rxor
                => InsnClass::MEMORY,
            _ => InsnClass::OTHER,
        },
    }
}

// ─── Jump trap helper ─────────────────────────────────────────────────────────

/// Fire the jump trap for `(source, target)` then, if `Continue`, call
/// `reactor.jmp` to `ri(target_addr, target_extend)`.
fn fire_jmp<'cb, 'ctx, Context, Err>(
    reactor: &mut Reactor<Context, Err, Function, LocalPool>,
    tail_idx: usize,
    ctx: &mut Context,
    traps: &mut TrapConfig<'cb, 'ctx, Context, Err>,
    layout: &LocalLayout,
    source_pc: u16,
    kind: JumpKind,
    target_addr: u16,
    target_extend: bool,
) -> Result<(), Err> {
    let jinfo = JumpInfo::direct(source_pc as u64, target_addr as u64, kind);
    let action = traps.on_jump(&jinfo, ctx, reactor, layout)?;
    if action == TrapAction::Continue {
        reactor.jmp(tail_idx, ctx, FuncIdx(ri(target_addr, target_extend)), 0)?;
    }
    Ok(())
}

// ─── Terminator emission ──────────────────────────────────────────────────────

fn emit_direct_terminator<'cb, 'ctx, Context, Err>(
    reactor: &mut Reactor<Context, Err, Function, LocalPool>,
    tail_idx: usize,
    ctx: &mut Context,
    instr: &DirectInstr,
    traps: &mut TrapConfig<'cb, 'ctx, Context, Err>,
    layout: &LocalLayout,
    li: LocalIndices,
) -> Result<(), Err> {
    let next_addr = instr.addr.wrapping_add(1) & 0x7FFF;

    match &instr.terminator {
        Terminator::FallThrough(_) => {
            if instr.instr_type == Some(InstrType::Extend) {
                // EXTEND opcode: next instruction must decode as extracode.
                fire_jmp(reactor, tail_idx, ctx, traps, layout,
                    instr.addr, JumpKind::DirectJump, next_addr, true)?;
            } else if instr.extend {
                // After an extended instruction EXTEND resets; redirect chain.
                fire_jmp(reactor, tail_idx, ctx, traps, layout,
                    instr.addr, JumpKind::DirectJump, next_addr, false)?;
            }
            // else: natural len=2 chain targets (next, ext=false) — no jmp needed.
        }

        Terminator::Jump(target) => {
            fire_jmp(reactor, tail_idx, ctx, traps, layout,
                instr.addr, JumpKind::DirectJump, *target, false)?;
        }

        Terminator::CondBranch { taken, fallthru } => {
            // Inline if-guard for the taken edge.
            feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
            feed(reactor, tail_idx, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, tail_idx, ctx, Instruction::I32Const(*taken as i32))?;
            feed(reactor, tail_idx, ctx, Instruction::I32Eq)?;
            feed(reactor, tail_idx, ctx, Instruction::If(BlockType::Empty))?;
            {
                let jinfo = JumpInfo::direct(instr.addr as u64, *taken as u64,
                    JumpKind::ConditionalBranch);
                let action = traps.on_jump(&jinfo, ctx, reactor, layout)?;
                if action == TrapAction::Continue {
                    feed(reactor, tail_idx, ctx,
                        Instruction::ReturnCall(ri(*taken, false) + NUM_IMPORTS))?;
                }
            }
            feed(reactor, tail_idx, ctx, Instruction::End)?;
            // Fall-through edge.
            fire_jmp(reactor, tail_idx, ctx, traps, layout,
                instr.addr, JumpKind::ConditionalBranch, *fallthru, false)?;
        }

        Terminator::CcsBranch(targets) => {
            feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
            feed(reactor, tail_idx, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
            for &t in targets.iter() {
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(t as i32))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eq)?;
                feed(reactor, tail_idx, ctx, Instruction::If(BlockType::Empty))?;
                {
                    let jinfo = JumpInfo::direct(instr.addr as u64, t as u64,
                        JumpKind::ConditionalBranch);
                    let action = traps.on_jump(&jinfo, ctx, reactor, layout)?;
                    if action == TrapAction::Continue {
                        feed(reactor, tail_idx, ctx,
                            Instruction::ReturnCall(ri(t, false) + NUM_IMPORTS))?;
                    }
                }
                feed(reactor, tail_idx, ctx, Instruction::End)?;
            }
            feed(reactor, tail_idx, ctx, Instruction::Unreachable)?;
        }

        Terminator::Indirect { possible_targets } => {
            feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
            feed(reactor, tail_idx, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
            for &t in possible_targets.iter() {
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(t as i32))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eq)?;
                feed(reactor, tail_idx, ctx, Instruction::If(BlockType::Empty))?;
                {
                    // For indirect jumps, report the SCR0 local as the runtime target.
                    let jinfo = JumpInfo::indirect(instr.addr as u64, li.scr0,
                        JumpKind::IndirectJump);
                    let action = traps.on_jump(&jinfo, ctx, reactor, layout)?;
                    if action == TrapAction::Continue {
                        feed(reactor, tail_idx, ctx,
                            Instruction::ReturnCall(ri(t, false) + NUM_IMPORTS))?;
                    }
                }
                feed(reactor, tail_idx, ctx, Instruction::End)?;
            }
            feed(reactor, tail_idx, ctx, Instruction::Unreachable)?;
        }

        Terminator::Halt => {
            feed(reactor, tail_idx, ctx, Instruction::Unreachable)?;
        }
    }
    Ok(())
}

// ─── Low-level emission helpers ───────────────────────────────────────────────

#[inline]
fn mem16(offset: u64) -> MemArg {
    MemArg { offset, align: 1, memory_index: 0 }
}

/// Thin wrapper around `Reactor::feed_to` that drops the `&` → `&` noise.
#[inline]
fn feed<Context, Err>(
    reactor: &Reactor<Context, Err, Function, LocalPool>,
    tail_idx: usize,
    ctx: &mut Context,
    instr: Instruction<'_>,
) -> Result<(), Err> {
    reactor.feed_to(tail_idx, ctx, &instr)
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

// ─── LOAD / STORE with compile-time TC2 address ──────────────────────────────

fn emit_load_known<Context, Err>(
    reactor: &Reactor<Context, Err, Function, LocalPool>,
    tail_idx: usize,
    ctx: &mut Context,
    addr: u16,
) -> Result<(), Err> {
    if let Some(off) = tc2_reg_offset(addr) {
        feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
        feed(reactor, tail_idx, ctx, Instruction::I32Load16U(mem16(off)))?;
    } else if addr == 0x0007 {
        feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
    } else if addr >= 0x8000 {
        feed(reactor, tail_idx, ctx, Instruction::I32Const((addr - 0x8000) as i32))?;
        feed(reactor, tail_idx, ctx, Instruction::Call(FN_CHAN_READ))?;
        feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
        feed(reactor, tail_idx, ctx, Instruction::I32And)?;
    } else {
        feed(reactor, tail_idx, ctx, Instruction::I32Const(addr as i32))?;
        feed(reactor, tail_idx, ctx, Instruction::Call(FN_MEM_READ))?;
        feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
        feed(reactor, tail_idx, ctx, Instruction::I32And)?;
    }
    Ok(())
}

fn emit_store_known<Context, Err>(
    reactor: &Reactor<Context, Err, Function, LocalPool>,
    tail_idx: usize,
    ctx: &mut Context,
    addr: u16,
    mask15: bool,
    li: LocalIndices,
) -> Result<(), Err> {
    if mask15 {
        feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
        feed(reactor, tail_idx, ctx, Instruction::I32And)?;
    }
    if let Some(off) = tc2_reg_offset(addr) {
        feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
        feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
        feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
        feed(reactor, tail_idx, ctx, Instruction::I32Store16(mem16(off)))?;
    } else if addr == 0x0007 {
        feed(reactor, tail_idx, ctx, Instruction::Drop)?;
    } else if addr >= 0x8000 {
        feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
        feed(reactor, tail_idx, ctx, Instruction::I32Const((addr - 0x8000) as i32))?;
        feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
        feed(reactor, tail_idx, ctx, Instruction::Call(FN_CHAN_WRITE))?;
    } else {
        feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
        feed(reactor, tail_idx, ctx, Instruction::I32Const(addr as i32))?;
        feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
        feed(reactor, tail_idx, ctx, Instruction::Call(FN_MEM_WRITE))?;
    }
    Ok(())
}

// ─── TC2 bytecode → WASM ─────────────────────────────────────────────────────

fn emit_bc_segment<Context, Err>(
    reactor: &Reactor<Context, Err, Function, LocalPool>,
    tail_idx: usize,
    ctx: &mut Context,
    code: &[u16],
    start: usize,
    li: LocalIndices,
) -> Result<(), Err> {
    let mut pc = start;
    while pc < code.len() {
        let opc = code[pc];
        pc += 1;

        match opc {
            op::RET => break,

            op::DUP => {
                feed(reactor, tail_idx, ctx, Instruction::LocalTee(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
            }
            op::SWAP => {
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr1))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr1))?;
            }
            op::DROP => { feed(reactor, tail_idx, ctx, Instruction::Drop)?; }

            op::ADD  => { feed(reactor, tail_idx, ctx, Instruction::I32Add)?; }
            op::SUB  => { feed(reactor, tail_idx, ctx, Instruction::I32Sub)?; }
            op::AND  => { feed(reactor, tail_idx, ctx, Instruction::I32And)?; }
            op::OR   => { feed(reactor, tail_idx, ctx, Instruction::I32Or)?; }
            op::XOR  => { feed(reactor, tail_idx, ctx, Instruction::I32Xor)?; }
            op::NOT  => {
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Xor)?;
            }
            op::MASK15 => {
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
            }
            op::NEG => {
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Sub)?;
            }
            op::LSHR_STK => {
                feed(reactor, tail_idx, ctx, Instruction::I32Const(15))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::I32ShrU)?;
            }
            op::LSHL_STK => {
                feed(reactor, tail_idx, ctx, Instruction::I32Const(15))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Shl)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
            }

            op::IMUL_HI15 => {
                feed(reactor, tail_idx, ctx, Instruction::I32Extend16S)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Extend16S)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Mul)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(15))?;
                feed(reactor, tail_idx, ctx, Instruction::I32ShrS)?;
            }
            op::IMUL_LO15 => {
                feed(reactor, tail_idx, ctx, Instruction::I32Extend16S)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Extend16S)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Mul)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
            }
            op::IDIV_Q15 => {
                feed(reactor, tail_idx, ctx, Instruction::I32Extend16S)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr1))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Extend16S)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(15))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Shl)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Or)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr1))?;
                feed(reactor, tail_idx, ctx, Instruction::I32DivS)?;
            }
            op::IDIV_R15 => {
                feed(reactor, tail_idx, ctx, Instruction::I32Extend16S)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr1))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Extend16S)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(15))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Shl)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Or)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr1))?;
                feed(reactor, tail_idx, ctx, Instruction::I32RemS)?;
            }

            op::IS_POS => {
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalTee(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Ne)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x4000))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eqz)?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
            }
            op::IS_PLUS_ZERO => {
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eqz)?;
            }
            op::IS_NEG => {
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalTee(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x4000))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Ne)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Ne)?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
            }
            op::IS_MINUS_ZERO => {
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eq)?;
            }
            op::IS_ZERO_OR_NEG => {
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalTee(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eqz)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x7FFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eq)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Or)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x4000))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Ne)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Or)?;
            }
            op::HAS_OVERFLOW => {
                feed(reactor, tail_idx, ctx, Instruction::I32Const(14))?;
                feed(reactor, tail_idx, ctx, Instruction::I32ShrU)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(3))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalTee(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(1))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eq)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(2))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eq)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Or)?;
            }
            op::BOOL_AND => { feed(reactor, tail_idx, ctx, Instruction::I32And)?; }
            op::BOOL_NOT => { feed(reactor, tail_idx, ctx, Instruction::I32Eqz)?; }

            op::LOAD_T  => { feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.t))?; }
            op::STORE_T => { feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.t))?; }

            op::GET_OFF       => { feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.off))?; }
            op::SET_OFF_STACK => { feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.off))?; }

            op::LOAD_OFF => {
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.off))?;
                feed(reactor, tail_idx, ctx, Instruction::Call(FN_MEM_READ))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
            }
            op::STORE_OFF => {
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.off))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::Call(FN_MEM_WRITE))?;
            }
            op::LOAD_OFF1 => {
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.off))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(1))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Add)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::Call(FN_MEM_READ))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
            }
            op::STORE_OFF1 => {
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.off))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(1))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Add)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::Call(FN_MEM_WRITE))?;
            }

            op::LOAD_CHAN_OFF => {
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.off))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x01FF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::Call(FN_CHAN_READ))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
            }
            op::STORE_CHAN_OFF => {
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.off))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0x01FF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::Call(FN_CHAN_WRITE))?;
            }

            op::LOAD_IND => {
                feed(reactor, tail_idx, ctx, Instruction::Call(FN_MEM_READ))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
            }
            op::STORE_IND => {
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.scr1))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(li.scr1))?;
                feed(reactor, tail_idx, ctx, Instruction::Call(FN_MEM_WRITE))?;
            }

            op::PUSH_IMM => {
                let v = code[pc] as i32; pc += 1;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(v))?;
            }
            op::LOAD => {
                let addr = code[pc] as u16; pc += 1;
                emit_load_known(reactor, tail_idx, ctx, addr)?;
            }
            op::STORE => {
                let addr = code[pc] as u16; pc += 1;
                emit_store_known(reactor, tail_idx, ctx, addr, false, li)?;
            }
            op::STORE15 => {
                let addr = code[pc] as u16; pc += 1;
                emit_store_known(reactor, tail_idx, ctx, addr, true, li)?;
            }
            op::SET_OFF => {
                let v = code[pc] as i32; pc += 1;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(v))?;
                feed(reactor, tail_idx, ctx, Instruction::LocalSet(li.off))?;
            }
            op::LSHR => {
                let k = code[pc] as i32; pc += 1;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(k))?;
                feed(reactor, tail_idx, ctx, Instruction::I32ShrU)?;
            }
            op::LSHL => {
                let k = code[pc] as i32; pc += 1;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(k))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Shl)?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(0xFFFF))?;
                feed(reactor, tail_idx, ctx, Instruction::I32And)?;
            }

            op::JUMP_NOT => {
                let off = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                feed(reactor, tail_idx, ctx, Instruction::If(BlockType::Empty))?;
                emit_bc_segment(reactor, tail_idx, ctx, code, pc, li)?;
                feed(reactor, tail_idx, ctx, Instruction::Else)?;
                emit_bc_segment(reactor, tail_idx, ctx, code, target, li)?;
                feed(reactor, tail_idx, ctx, Instruction::End)?;
                return Ok(());
            }
            op::JUMP_IF => {
                let off = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                feed(reactor, tail_idx, ctx, Instruction::If(BlockType::Empty))?;
                emit_bc_segment(reactor, tail_idx, ctx, code, target, li)?;
                feed(reactor, tail_idx, ctx, Instruction::Else)?;
                emit_bc_segment(reactor, tail_idx, ctx, code, pc, li)?;
                feed(reactor, tail_idx, ctx, Instruction::End)?;
                return Ok(());
            }
            op::JUMP => {
                let off = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                emit_bc_segment(reactor, tail_idx, ctx, code, target, li)?;
                return Ok(());
            }

            _ => {
                feed(reactor, tail_idx, ctx, Instruction::Unreachable)?;
            }
        }
    }
    Ok(())
}
