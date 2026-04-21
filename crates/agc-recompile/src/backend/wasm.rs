//! WASM direct backend — translates the AGC instruction stream into a
//! WebAssembly module using the yecta [`Reactor`].
//!
//! ## Generics
//!
//! ```text
//! WasmDirectBackend<'cb, 'ctx, Context = (), Err = String>
//! ```
//!
//! ## Emission-context isolation
//!
//! Every generated WASM function carries **two** TC2 emission contexts:
//!
//! ```text
//! locals_mark (= 0, no params)
//!   outer:  T OFF SCR0 SCR1          ← used for the AGC instruction's own bytecode
//!   nested: T OFF SCR0 SCR1          ← used for injected trap / hook bytecode
//!   virt:   vr[0] … vr[N_VIRT_REGS] ← addresses 0xFF10..0xFF1F (shared by both)
//!   [trap-declared locals follow]
//! ```
//!
//! The scratch locals (`T`, `OFF`, `SCR0`, `SCR1`) of the outer and nested
//! contexts map to **disjoint** WASM locals.  Trap / hook code therefore
//! cannot accidentally clobber the outer instruction's working state.
//!
//! Virtual registers occupy TC2 addresses `0xFF10`–`0xFF1F` (16 cells).
//! These lie outside the 16-bit AGC address space and compile to WASM
//! `local.get / local.set` instead of linear-memory loads/stores.  They
//! are shared between the outer and nested contexts so trap code can
//! communicate with the instruction being trapped.
//!
//! ## Host functions
//!
//! Users may register additional WASM imports with
//! [`WasmDirectBackend::add_host_fn`].  TC2 bytecode can then call them
//! with the [`op::HOST_CALL`] opcode (slot, n\_args, n\_results).
//!
//! Host functions must be registered **before** the first [`feed_instr`] call
//! because their count determines the reactor's `base_func_offset`.
//!
//! ## WASM function index layout
//!
//! ```text
//! 0 .. 3                        standard I/O imports (mem_read, …)
//! 4 .. 4+n_host-1               host function imports (slot 0 … n_host-1)
//! 4+n_host .. 4+n_host+8191     generated functions  (reactor indices 0 … 8191)
//! ```
//!
//! [`feed_instr`]: WasmDirectBackend::feed_instr

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

// ─── Standard imported function indices ──────────────────────────────────────

const FN_MEM_READ:      u32 = 0;
const FN_MEM_WRITE:     u32 = 1;
const FN_CHAN_READ:      u32 = 2;
const FN_CHAN_WRITE:     u32 = 3;
/// Number of standard I/O imports.  Host-function imports are added immediately
/// after these, starting at WASM function index `NUM_IMPORTS`.
pub const NUM_IMPORTS:  u32 = 4;

// ─── Virtual-register address space ──────────────────────────────────────────

/// First TC2 address in the virtual-register range.
///
/// TC2 `LOAD(0xFF10 + i)` / `STORE(0xFF10 + i)` compile to WASM
/// `local.get` / `local.set` on the corresponding virtual-register local.
/// This range is completely outside the 16-bit AGC address space and
/// therefore cannot alias with any AGC memory or register.
pub const VIRT_REG_BASE: u16  = 0xFF10;

/// Number of virtual registers (addresses `0xFF10`..`0xFF1F`).
pub const N_VIRT_REGS: usize = 16;

// ─── Reactor index ────────────────────────────────────────────────────────────

#[inline]
fn ri(addr: u16, extend: bool) -> u32 {
    2 * addr as u32 + extend as u32
}

// ─── WasmSink — generic instruction-emission interface ───────────────────────

/// Minimal WASM instruction emission interface.
///
/// Abstracts over:
/// * [`FeedSink`] — wraps `(&Reactor, tail_idx)`, used for normal AGC bytecode
/// * [`speet_traps::TrapContext`] — used when `emit_bc_segment` is called
///   from inside a trap callback
///
/// All TC2-to-WASM translation helpers (`emit_bc_segment`,
/// `emit_load_known`, `emit_store_known`) are generic over `WasmSink`, so
/// they can be called from both the main translation path and from trap code.
pub(crate) trait WasmSink<Context, Err> {
    fn emit_wasm(&mut self, ctx: &mut Context, instr: &Instruction<'_>) -> Result<(), Err>;
}

/// Binds a [`Reactor`] to a fixed `tail_idx`, implementing [`WasmSink`].
pub(crate) struct FeedSink<'r, Context, Err> {
    pub reactor:  &'r Reactor<Context, Err, Function, LocalPool>,
    pub tail_idx: usize,
}

impl<Context, Err> WasmSink<Context, Err> for FeedSink<'_, Context, Err> {
    #[inline]
    fn emit_wasm(&mut self, ctx: &mut Context, instr: &Instruction<'_>) -> Result<(), Err> {
        self.reactor.feed_to(self.tail_idx, ctx, instr)
    }
}

impl<'a, Context, Err> WasmSink<Context, Err> for speet_traps::TrapContext<'a, Context, Err> {
    #[inline]
    fn emit_wasm(&mut self, ctx: &mut Context, instr: &Instruction<'_>) -> Result<(), Err> {
        speet_traps::TrapContext::emit(self, ctx, instr)
    }
}

// ─── EmitContext — per-function, per-namespace TC2 emission state ─────────────

/// Resolved WASM local indices for the four TC2 scratch variables.
#[derive(Clone, Copy)]
pub(crate) struct LocalIndices {
    pub t:    u32,
    pub off:  u32,
    pub scr0: u32,
    pub scr1: u32,
}

/// Per-function TC2-to-WASM emission context.
///
/// Two contexts are created per function (outer and nested); they share
/// the same `virt_regs` but have independent `li` scratch locals.
/// See the [module-level docs](self) for the layout.
#[derive(Clone)]
pub struct EmitContext {
    /// Scratch WASM locals for T, OFF, SCR0, SCR1 in this namespace.
    pub(crate) li: LocalIndices,
    /// WASM local indices for virtual registers 0xFF10..0xFF1F.
    pub(crate) virt_regs: [u32; N_VIRT_REGS],
    /// WASM function index of the first host function (= NUM_IMPORTS).
    pub(crate) host_fn_base: u32,
}

impl EmitContext {
    /// Return the WASM local index for virtual register at TC2 address `addr`.
    ///
    /// Returns `None` if `addr` is not in the virtual-register range.
    #[inline]
    pub(crate) fn virt_reg(&self, addr: u16) -> Option<u32> {
        let i = addr.wrapping_sub(VIRT_REG_BASE) as usize;
        if i < N_VIRT_REGS { Some(self.virt_regs[i]) } else { None }
    }
}

// ─── HostFnSig ────────────────────────────────────────────────────────────────

/// Signature of a host function callable from TC2 bytecode via
/// [`op::HOST_CALL`].
///
/// Host functions are inserted as WASM imports immediately after the four
/// standard I/O imports.  Slot index `s` maps to WASM function index
/// `NUM_IMPORTS + s`.
#[derive(Clone, Debug)]
pub struct HostFnSig {
    /// WASM module name (typically `"env"`).
    pub module: alloc::string::String,
    /// WASM function name.
    pub name: alloc::string::String,
    /// Number of `i32` parameters consumed from the TC2 stack.
    pub params: u32,
    /// Number of `i32` results pushed onto the TC2 stack (0 or 1).
    pub results: u32,
}

// ─── WasmDirectBackend ────────────────────────────────────────────────────────

/// Direct WASM backend powered by the yecta [`Reactor`].
///
/// Feed all 4096 × 2 instructions with [`feed_instr`], then call
/// [`finish`] to obtain the assembled WASM module.
///
/// [`feed_instr`]: WasmDirectBackend::feed_instr
/// [`finish`]:     WasmDirectBackend::finish
pub struct WasmDirectBackend<'cb, 'ctx, Context = (), Err = String> {
    reactor:      Reactor<Context, Err, Function, LocalPool>,
    entry_points: Vec<u16>,
    tail_idx:     usize,

    layout:       LocalLayout,
    locals_mark:  Mark,

    // ── Outer namespace (AGC instruction's own TC2 bytecode) ──────────────
    slot_t:    LocalSlot,
    slot_off:  LocalSlot,
    slot_scr0: LocalSlot,
    slot_scr1: LocalSlot,

    // ── Nested namespace (injected trap / hook bytecode) ─────────────────
    slot_nested_t:    LocalSlot,
    slot_nested_off:  LocalSlot,
    slot_nested_scr0: LocalSlot,
    slot_nested_scr1: LocalSlot,

    // ── Virtual registers (shared; outside 16-bit AGC address space) ─────
    slot_virt: [LocalSlot; N_VIRT_REGS],

    // ── Host functions and traps ─────────────────────────────────────────
    host_fns: Vec<HostFnSig>,
    traps:    TrapConfig<'cb, 'ctx, Context, Err>,
}

impl<'cb, 'ctx, Context, Err> WasmDirectBackend<'cb, 'ctx, Context, Err> {
    /// Create a new backend with no host functions and no traps.
    pub fn new(entry_points: Vec<u16>) -> Self {
        let mut s = Self {
            reactor:     Reactor::with_base_func_offset(NUM_IMPORTS),
            entry_points,
            tail_idx:    0,
            layout:      LocalLayout::empty(),
            locals_mark: Mark { slot_count: 0, total_locals: 0 },
            slot_t:            LocalSlot::default(),
            slot_off:          LocalSlot::default(),
            slot_scr0:         LocalSlot::default(),
            slot_scr1:         LocalSlot::default(),
            slot_nested_t:    LocalSlot::default(),
            slot_nested_off:  LocalSlot::default(),
            slot_nested_scr0: LocalSlot::default(),
            slot_nested_scr1: LocalSlot::default(),
            slot_virt: [LocalSlot::default(); N_VIRT_REGS],
            host_fns: Vec::new(),
            traps:    TrapConfig::new(),
        };
        s.setup_traps();
        s
    }

    /// Register a host function that TC2 bytecode may call with
    /// [`op::HOST_CALL`].
    ///
    /// Must be called **before** the first [`feed_instr`] call.  Returns
    /// the slot index to embed in the `HOST_CALL` operand.
    ///
    /// [`feed_instr`]: WasmDirectBackend::feed_instr
    pub fn add_host_fn(&mut self, sig: HostFnSig) -> u16 {
        let slot = self.host_fns.len() as u16;
        self.host_fns.push(sig);
        // Generated functions start after all imports.
        self.reactor.set_base_func_offset(NUM_IMPORTS + self.host_fns.len() as u32);
        slot
    }

    /// **Phase 1** — (re-)initialize the layout and let traps declare param slots.
    pub fn setup_traps(&mut self) {
        self.layout = LocalLayout::empty();
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

        // Outer namespace
        self.slot_t    = self.layout.append(1, ValType::I32);
        self.slot_off  = self.layout.append(1, ValType::I32);
        self.slot_scr0 = self.layout.append(1, ValType::I32);
        self.slot_scr1 = self.layout.append(1, ValType::I32);

        // Nested namespace (isolated scratch locals for injected bytecode)
        self.slot_nested_t    = self.layout.append(1, ValType::I32);
        self.slot_nested_off  = self.layout.append(1, ValType::I32);
        self.slot_nested_scr0 = self.layout.append(1, ValType::I32);
        self.slot_nested_scr1 = self.layout.append(1, ValType::I32);

        // Virtual registers (shared between outer and nested)
        for s in &mut self.slot_virt {
            *s = self.layout.append(1, ValType::I32);
        }

        self.traps.declare_locals(CellIdx(0), &mut self.layout);

        // Collect the locals declaration for the new function.
        let fn_locals: alloc::vec::Vec<(u32, ValType)> =
            self.layout.iter_since(&self.locals_mark).collect();
        self.reactor.next_with(ctx, Function::new(fn_locals), 2)?;
        self.tail_idx = self.reactor.fn_count().saturating_sub(1);
        let tail_idx = self.tail_idx;

        // Build the two emission contexts.
        let virt_regs: [u32; N_VIRT_REGS] =
            core::array::from_fn(|i| self.layout.base(self.slot_virt[i]));
        let host_fn_base = NUM_IMPORTS; // host functions start right after standard imports

        let outer_ctx = EmitContext {
            li: LocalIndices {
                t:    self.layout.base(self.slot_t),
                off:  self.layout.base(self.slot_off),
                scr0: self.layout.base(self.slot_scr0),
                scr1: self.layout.base(self.slot_scr1),
            },
            virt_regs,
            host_fn_base,
        };
        let nested_ctx = EmitContext {
            li: LocalIndices {
                t:    self.layout.base(self.slot_nested_t),
                off:  self.layout.base(self.slot_nested_off),
                scr0: self.layout.base(self.slot_nested_scr0),
                scr1: self.layout.base(self.slot_nested_scr1),
            },
            virt_regs,
            host_fn_base,
        };

        // ── Pre-advance Z ─────────────────────────────────────────────────────
        let next_pc = instr.addr.wrapping_add(1) & 0x7FFF;
        let r = &self.reactor;
        feed(r, tail_idx, ctx, Instruction::I32Const(0))?;
        feed(r, tail_idx, ctx, Instruction::I32Const(next_pc as i32))?;
        feed(r, tail_idx, ctx, Instruction::I32Store16(mem16(MEM_Z)))?;

        feed(r, tail_idx, ctx, Instruction::I32Const(0))?;
        feed(r, tail_idx, ctx, Instruction::I32Const(instr.raw_word as i32))?;
        feed(r, tail_idx, ctx, Instruction::I32Store16(mem16(MEM_INSTR_WORD)))?;

        // ── Phase 3a: instruction trap ────────────────────────────────────────
        let speet_info = InstructionInfo {
            pc:   instr.addr as u64,
            len:  1,
            arch: ArchTag::Other,
            class: classify_agc_insn(instr.instr_type),
        };
        let (reactor, traps, layout) = (&mut self.reactor, &mut self.traps, &self.layout);
        let action = traps.on_instruction(&speet_info, ctx, reactor, layout)?;

        if action == TrapAction::Skip {
            emit_direct_terminator(
                &mut self.reactor, self.tail_idx, ctx,
                instr, &mut self.traps, &self.layout, &outer_ctx,
            )?;
            return Ok(());
        }

        // ── Emit TC2 bytecode (outer namespace) ───────────────────────────────
        let mut outer_sink = FeedSink { reactor: &self.reactor, tail_idx };
        emit_bc_segment(&mut outer_sink, ctx, &instr.bytecode, 0, &outer_ctx)?;

        // ── Emit control-flow terminator ──────────────────────────────────────
        emit_direct_terminator(
            &mut self.reactor, self.tail_idx, ctx,
            instr, &mut self.traps, &self.layout, &outer_ctx,
        )?;

        // Expose nested_ctx for downstream trap/hook users.
        let _ = nested_ctx;

        Ok(())
    }

    fn finish(self, ctx: &mut Context) -> Result<Vec<u8>, Err> {
        let n_funcs = self.reactor.fn_count() as u32;
        let base_func_offset = self.reactor.base_func_offset();
        let mut functions = self.reactor.into_fns();

        for f in &mut functions {
            f.instruction(&Instruction::End);
        }

        // ── Assemble WASM module ──────────────────────────────────────────────
        let mut module = Module::new();

        // Type section — standard types + host-function types
        let mut types = TypeSection::new();
        types.ty().function([], []);                                   // type 0: () -> ()
        types.ty().function([ValType::I32], [ValType::I32]);           // type 1: (i32) -> i32
        types.ty().function([ValType::I32, ValType::I32], []);         // type 2: (i32,i32) -> ()

        // Collect unique host-function type signatures (beyond the 3 standard ones).
        let mut host_type_indices: Vec<u32> = Vec::new();
        for hf in &self.host_fns {
            // Build (params: [i32×n], results: [i32×m]) and check if a matching
            // type already exists in the section.
            let params: alloc::vec::Vec<ValType>  = (0..hf.params).map(|_| ValType::I32).collect();
            let results: alloc::vec::Vec<ValType> = (0..hf.results).map(|_| ValType::I32).collect();
            // Linear search among host-function types (typically very few).
            let existing = host_type_indices.iter().enumerate().find(|(i, &ty_idx)| {
                let hf_ref = &self.host_fns[*i];
                hf_ref.params == hf.params && hf_ref.results == hf.results
            });
            if let Some((_, &ty_idx)) = existing {
                host_type_indices.push(ty_idx);
            } else {
                let ty_idx = 3 + host_type_indices.len() as u32; // after standard types
                types.ty().function(params, results);
                host_type_indices.push(ty_idx);
            }
        }
        module.section(&types);

        // Import section — standard + host
        let mut imports = ImportSection::new();
        imports.import("env", "mem_read",   EntityType::Function(1));
        imports.import("env", "mem_write",  EntityType::Function(2));
        imports.import("env", "chan_read",  EntityType::Function(1));
        imports.import("env", "chan_write", EntityType::Function(2));
        for (i, hf) in self.host_fns.iter().enumerate() {
            imports.import(&hf.module, &hf.name, EntityType::Function(host_type_indices[i]));
        }
        module.section(&imports);

        // Function section (all use type 0)
        let mut funcs = FunctionSection::new();
        for _ in 0..n_funcs { funcs.function(0); }
        module.section(&funcs);

        // Memory section
        let mut mems = MemorySection::new();
        mems.memory(MemoryType {
            minimum: 1, maximum: None,
            memory64: false, shared: false, page_size_log2: None,
        });
        module.section(&mems);

        // Export section
        let mut exports = ExportSection::new();
        exports.export("memory", ExportKind::Memory, 0);
        let unique_eps: BTreeSet<u16> = self.entry_points.iter().copied().collect();
        for ep in unique_eps {
            let fn_idx = ri(ep, false) + base_func_offset;
            exports.export(&format!("bb_{ep:05o}"), ExportKind::Func, fn_idx);
        }
        module.section(&exports);

        // Code section
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
            InstrType::Ccs => InsnClass::BRANCH,
            InstrType::Go  => InsnClass::BRANCH | InsnClass::INDIRECT,
            InstrType::Resume | InstrType::Rupt
            | InstrType::Inhint | InstrType::Relint => InsnClass::PRIVILEGED,
            InstrType::Ca  | InstrType::Cs  | InstrType::Ad  | InstrType::Ads
            | InstrType::Su | InstrType::Ts  | InstrType::Xch | InstrType::Lxch
            | InstrType::Qxch | InstrType::Dxch | InstrType::Dca | InstrType::Dcs
            | InstrType::Read | InstrType::Write | InstrType::Rand | InstrType::Wand
            | InstrType::Ror  | InstrType::Wor  | InstrType::Rxor => InsnClass::MEMORY,
            _ => InsnClass::OTHER,
        },
    }
}

// ─── Jump trap helper ─────────────────────────────────────────────────────────

/// Fire the jump trap then, if `Continue`, emit a `reactor.jmp` to
/// `ri(target_addr, target_extend)`.
fn fire_jmp<'cb, 'ctx, Context, Err>(
    reactor:       &mut Reactor<Context, Err, Function, LocalPool>,
    tail_idx:      usize,
    ctx:           &mut Context,
    traps:         &mut TrapConfig<'cb, 'ctx, Context, Err>,
    layout:        &LocalLayout,
    source_pc:     u16,
    kind:          JumpKind,
    target_addr:   u16,
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
    reactor:   &mut Reactor<Context, Err, Function, LocalPool>,
    tail_idx:  usize,
    ctx:       &mut Context,
    instr:     &DirectInstr,
    traps:     &mut TrapConfig<'cb, 'ctx, Context, Err>,
    layout:    &LocalLayout,
    emit_ctx:  &EmitContext,
) -> Result<(), Err> {
    let next_addr  = instr.addr.wrapping_add(1) & 0x7FFF;
    // Generated function base offset — includes both standard and host imports.
    let base       = reactor.base_func_offset();

    match &instr.terminator {
        Terminator::FallThrough(_) => {
            if instr.instr_type == Some(InstrType::Extend) {
                fire_jmp(reactor, tail_idx, ctx, traps, layout,
                    instr.addr, JumpKind::DirectJump, next_addr, true)?;
            } else if instr.extend {
                fire_jmp(reactor, tail_idx, ctx, traps, layout,
                    instr.addr, JumpKind::DirectJump, next_addr, false)?;
            }
        }

        Terminator::Jump(target) => {
            fire_jmp(reactor, tail_idx, ctx, traps, layout,
                instr.addr, JumpKind::DirectJump, *target, false)?;
        }

        Terminator::CondBranch { taken, fallthru } => {
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
                        Instruction::ReturnCall(ri(*taken, false) + base))?;
                }
            }
            feed(reactor, tail_idx, ctx, Instruction::End)?;
            fire_jmp(reactor, tail_idx, ctx, traps, layout,
                instr.addr, JumpKind::ConditionalBranch, *fallthru, false)?;
        }

        Terminator::CcsBranch(targets) => {
            feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
            feed(reactor, tail_idx, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, tail_idx, ctx, Instruction::LocalSet(emit_ctx.li.scr0))?;
            for &t in targets.iter() {
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(emit_ctx.li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(t as i32))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eq)?;
                feed(reactor, tail_idx, ctx, Instruction::If(BlockType::Empty))?;
                {
                    let jinfo = JumpInfo::direct(instr.addr as u64, t as u64,
                        JumpKind::ConditionalBranch);
                    let action = traps.on_jump(&jinfo, ctx, reactor, layout)?;
                    if action == TrapAction::Continue {
                        feed(reactor, tail_idx, ctx,
                            Instruction::ReturnCall(ri(t, false) + base))?;
                    }
                }
                feed(reactor, tail_idx, ctx, Instruction::End)?;
            }
            feed(reactor, tail_idx, ctx, Instruction::Unreachable)?;
        }

        Terminator::Indirect { possible_targets } => {
            feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
            feed(reactor, tail_idx, ctx, Instruction::I32Load16U(mem16(MEM_Z)))?;
            feed(reactor, tail_idx, ctx, Instruction::LocalSet(emit_ctx.li.scr0))?;
            for &t in possible_targets.iter() {
                feed(reactor, tail_idx, ctx, Instruction::LocalGet(emit_ctx.li.scr0))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Const(t as i32))?;
                feed(reactor, tail_idx, ctx, Instruction::I32Eq)?;
                feed(reactor, tail_idx, ctx, Instruction::If(BlockType::Empty))?;
                {
                    let jinfo = JumpInfo::indirect(instr.addr as u64, emit_ctx.li.scr0,
                        JumpKind::IndirectJump);
                    let action = traps.on_jump(&jinfo, ctx, reactor, layout)?;
                    if action == TrapAction::Continue {
                        feed(reactor, tail_idx, ctx,
                            Instruction::ReturnCall(ri(t, false) + base))?;
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

// ─── Low-level helpers (reactor-direct) ──────────────────────────────────────

#[inline]
fn mem16(offset: u64) -> MemArg {
    MemArg { offset, align: 1, memory_index: 0 }
}

/// Thin shim from `&Reactor + tail_idx` to a one-shot instruction emit.
/// Used only in terminator emission which always has a direct reactor borrow.
#[inline]
fn feed<Context, Err>(
    reactor:  &Reactor<Context, Err, Function, LocalPool>,
    tail_idx: usize,
    ctx:      &mut Context,
    instr:    Instruction<'_>,
) -> Result<(), Err> {
    reactor.feed_to(tail_idx, ctx, &instr)
}

/// Map a TC2 address to its WASM linear-memory byte offset (for fixed regs).
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

// ─── WasmSink helpers (generic — used by emit_bc_segment and emit_*_known) ───

#[inline]
fn wfeed<Context, Err, S: WasmSink<Context, Err>>(
    sink: &mut S,
    ctx:  &mut Context,
    instr: Instruction<'_>,
) -> Result<(), Err> {
    sink.emit_wasm(ctx, &instr)
}

fn emit_load_known<Context, Err, S: WasmSink<Context, Err>>(
    sink:     &mut S,
    ctx:      &mut Context,
    addr:     u16,
    emit_ctx: &EmitContext,
) -> Result<(), Err> {
    // Virtual register range — maps to WASM locals (outside AGC memory).
    if let Some(local_idx) = emit_ctx.virt_reg(addr) {
        return wfeed(sink, ctx, Instruction::LocalGet(local_idx));
    }
    if let Some(off) = tc2_reg_offset(addr) {
        wfeed(sink, ctx, Instruction::I32Const(0))?;
        wfeed(sink, ctx, Instruction::I32Load16U(mem16(off)))?;
    } else if addr == 0x0007 {
        wfeed(sink, ctx, Instruction::I32Const(0))?;
    } else if addr >= 0x8000 {
        wfeed(sink, ctx, Instruction::I32Const((addr - 0x8000) as i32))?;
        wfeed(sink, ctx, Instruction::Call(FN_CHAN_READ))?;
        wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
        wfeed(sink, ctx, Instruction::I32And)?;
    } else {
        wfeed(sink, ctx, Instruction::I32Const(addr as i32))?;
        wfeed(sink, ctx, Instruction::Call(FN_MEM_READ))?;
        wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
        wfeed(sink, ctx, Instruction::I32And)?;
    }
    Ok(())
}

fn emit_store_known<Context, Err, S: WasmSink<Context, Err>>(
    sink:     &mut S,
    ctx:      &mut Context,
    addr:     u16,
    mask15:   bool,
    emit_ctx: &EmitContext,
) -> Result<(), Err> {
    if mask15 {
        wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
        wfeed(sink, ctx, Instruction::I32And)?;
    }
    // Virtual register range — maps to WASM locals.
    if let Some(local_idx) = emit_ctx.virt_reg(addr) {
        return wfeed(sink, ctx, Instruction::LocalSet(local_idx));
    }
    if let Some(off) = tc2_reg_offset(addr) {
        wfeed(sink, ctx, Instruction::LocalSet(emit_ctx.li.scr0))?;
        wfeed(sink, ctx, Instruction::I32Const(0))?;
        wfeed(sink, ctx, Instruction::LocalGet(emit_ctx.li.scr0))?;
        wfeed(sink, ctx, Instruction::I32Store16(mem16(off)))?;
    } else if addr == 0x0007 {
        wfeed(sink, ctx, Instruction::Drop)?;
    } else if addr >= 0x8000 {
        wfeed(sink, ctx, Instruction::LocalSet(emit_ctx.li.scr0))?;
        wfeed(sink, ctx, Instruction::I32Const((addr - 0x8000) as i32))?;
        wfeed(sink, ctx, Instruction::LocalGet(emit_ctx.li.scr0))?;
        wfeed(sink, ctx, Instruction::Call(FN_CHAN_WRITE))?;
    } else {
        wfeed(sink, ctx, Instruction::LocalSet(emit_ctx.li.scr0))?;
        wfeed(sink, ctx, Instruction::I32Const(addr as i32))?;
        wfeed(sink, ctx, Instruction::LocalGet(emit_ctx.li.scr0))?;
        wfeed(sink, ctx, Instruction::Call(FN_MEM_WRITE))?;
    }
    Ok(())
}

// ─── TC2 bytecode → WASM (generic over WasmSink) ─────────────────────────────

/// Translate TC2 bytecode `code[start..]` into WASM instructions, emitting
/// through `sink`.
///
/// This function is generic over any [`WasmSink`] so the same translation
/// logic can be driven by both the main reactor path ([`FeedSink`]) and by
/// trap callback paths ([`speet_traps::TrapContext`]).
///
/// The `emit_ctx` determines which WASM locals to use for scratch variables
/// (enabling outer / nested namespace isolation) and provides the host
/// function base index for [`op::HOST_CALL`] dispatch.
pub(crate) fn emit_bc_segment<Context, Err, S: WasmSink<Context, Err>>(
    sink:     &mut S,
    ctx:      &mut Context,
    code:     &[u16],
    start:    usize,
    emit_ctx: &EmitContext,
) -> Result<(), Err> {
    let li = emit_ctx.li;
    let mut pc = start;
    while pc < code.len() {
        let opc = code[pc];
        pc += 1;

        match opc {
            op::RET => break,

            op::DUP => {
                wfeed(sink, ctx, Instruction::LocalTee(li.scr0))?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
            }
            op::SWAP => {
                wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                wfeed(sink, ctx, Instruction::LocalSet(li.scr1))?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr1))?;
            }
            op::DROP => { wfeed(sink, ctx, Instruction::Drop)?; }

            op::ADD  => { wfeed(sink, ctx, Instruction::I32Add)?; }
            op::SUB  => { wfeed(sink, ctx, Instruction::I32Sub)?; }
            op::AND  => { wfeed(sink, ctx, Instruction::I32And)?; }
            op::OR   => { wfeed(sink, ctx, Instruction::I32Or)?; }
            op::XOR  => { wfeed(sink, ctx, Instruction::I32Xor)?; }
            op::NOT  => {
                wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                wfeed(sink, ctx, Instruction::I32Xor)?;
            }
            op::MASK15 => {
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
            }
            op::NEG => {
                wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Const(0))?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Sub)?;
            }
            op::LSHR_STK => {
                wfeed(sink, ctx, Instruction::I32Const(15))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::I32ShrU)?;
            }
            op::LSHL_STK => {
                wfeed(sink, ctx, Instruction::I32Const(15))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::I32Shl)?;
                wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
            }

            op::IMUL_HI15 => {
                wfeed(sink, ctx, Instruction::I32Extend16S)?;
                wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Extend16S)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Mul)?;
                wfeed(sink, ctx, Instruction::I32Const(15))?;
                wfeed(sink, ctx, Instruction::I32ShrS)?;
            }
            op::IMUL_LO15 => {
                wfeed(sink, ctx, Instruction::I32Extend16S)?;
                wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Extend16S)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Mul)?;
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
            }
            op::IDIV_Q15 => {
                wfeed(sink, ctx, Instruction::I32Extend16S)?;
                wfeed(sink, ctx, Instruction::LocalSet(li.scr1))?;
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Extend16S)?;
                wfeed(sink, ctx, Instruction::I32Const(15))?;
                wfeed(sink, ctx, Instruction::I32Shl)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Or)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr1))?;
                wfeed(sink, ctx, Instruction::I32DivS)?;
            }
            op::IDIV_R15 => {
                wfeed(sink, ctx, Instruction::I32Extend16S)?;
                wfeed(sink, ctx, Instruction::LocalSet(li.scr1))?;
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Extend16S)?;
                wfeed(sink, ctx, Instruction::I32Const(15))?;
                wfeed(sink, ctx, Instruction::I32Shl)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Or)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr1))?;
                wfeed(sink, ctx, Instruction::I32RemS)?;
            }

            op::IS_POS => {
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::LocalTee(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Const(0))?;
                wfeed(sink, ctx, Instruction::I32Ne)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Const(0x4000))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::I32Eqz)?;
                wfeed(sink, ctx, Instruction::I32And)?;
            }
            op::IS_PLUS_ZERO => {
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::I32Eqz)?;
            }
            op::IS_NEG => {
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::LocalTee(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Const(0x4000))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::I32Const(0))?;
                wfeed(sink, ctx, Instruction::I32Ne)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32Ne)?;
                wfeed(sink, ctx, Instruction::I32And)?;
            }
            op::IS_MINUS_ZERO => {
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32Eq)?;
            }
            op::IS_ZERO_OR_NEG => {
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::LocalTee(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Eqz)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Const(0x7FFF))?;
                wfeed(sink, ctx, Instruction::I32Eq)?;
                wfeed(sink, ctx, Instruction::I32Or)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Const(0x4000))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::I32Const(0))?;
                wfeed(sink, ctx, Instruction::I32Ne)?;
                wfeed(sink, ctx, Instruction::I32Or)?;
            }
            op::HAS_OVERFLOW => {
                wfeed(sink, ctx, Instruction::I32Const(14))?;
                wfeed(sink, ctx, Instruction::I32ShrU)?;
                wfeed(sink, ctx, Instruction::I32Const(3))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::LocalTee(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Const(1))?;
                wfeed(sink, ctx, Instruction::I32Eq)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::I32Const(2))?;
                wfeed(sink, ctx, Instruction::I32Eq)?;
                wfeed(sink, ctx, Instruction::I32Or)?;
            }
            op::BOOL_AND => { wfeed(sink, ctx, Instruction::I32And)?; }
            op::BOOL_NOT => { wfeed(sink, ctx, Instruction::I32Eqz)?; }

            op::LOAD_T  => { wfeed(sink, ctx, Instruction::LocalGet(li.t))?; }
            op::STORE_T => { wfeed(sink, ctx, Instruction::LocalSet(li.t))?; }

            op::GET_OFF       => { wfeed(sink, ctx, Instruction::LocalGet(li.off))?; }
            op::SET_OFF_STACK => { wfeed(sink, ctx, Instruction::LocalSet(li.off))?; }

            op::LOAD_OFF => {
                wfeed(sink, ctx, Instruction::LocalGet(li.off))?;
                wfeed(sink, ctx, Instruction::Call(FN_MEM_READ))?;
                wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
            }
            op::STORE_OFF => {
                wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                wfeed(sink, ctx, Instruction::LocalGet(li.off))?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::Call(FN_MEM_WRITE))?;
            }
            op::LOAD_OFF1 => {
                wfeed(sink, ctx, Instruction::LocalGet(li.off))?;
                wfeed(sink, ctx, Instruction::I32Const(1))?;
                wfeed(sink, ctx, Instruction::I32Add)?;
                wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::Call(FN_MEM_READ))?;
                wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
            }
            op::STORE_OFF1 => {
                wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                wfeed(sink, ctx, Instruction::LocalGet(li.off))?;
                wfeed(sink, ctx, Instruction::I32Const(1))?;
                wfeed(sink, ctx, Instruction::I32Add)?;
                wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::Call(FN_MEM_WRITE))?;
            }
            op::LOAD_CHAN_OFF => {
                wfeed(sink, ctx, Instruction::LocalGet(li.off))?;
                wfeed(sink, ctx, Instruction::I32Const(0x01FF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::Call(FN_CHAN_READ))?;
                wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
            }
            op::STORE_CHAN_OFF => {
                wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                wfeed(sink, ctx, Instruction::LocalGet(li.off))?;
                wfeed(sink, ctx, Instruction::I32Const(0x01FF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::Call(FN_CHAN_WRITE))?;
            }
            op::LOAD_IND => {
                wfeed(sink, ctx, Instruction::Call(FN_MEM_READ))?;
                wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
            }
            op::STORE_IND => {
                wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                wfeed(sink, ctx, Instruction::LocalSet(li.scr1))?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                wfeed(sink, ctx, Instruction::LocalGet(li.scr1))?;
                wfeed(sink, ctx, Instruction::Call(FN_MEM_WRITE))?;
            }

            op::PUSH_IMM => {
                let v = code[pc] as i32; pc += 1;
                wfeed(sink, ctx, Instruction::I32Const(v))?;
            }
            op::LOAD => {
                let addr = code[pc] as u16; pc += 1;
                emit_load_known(sink, ctx, addr, emit_ctx)?;
            }
            op::STORE => {
                let addr = code[pc] as u16; pc += 1;
                emit_store_known(sink, ctx, addr, false, emit_ctx)?;
            }
            op::STORE15 => {
                let addr = code[pc] as u16; pc += 1;
                emit_store_known(sink, ctx, addr, true, emit_ctx)?;
            }
            op::SET_OFF => {
                let v = code[pc] as i32; pc += 1;
                wfeed(sink, ctx, Instruction::I32Const(v))?;
                wfeed(sink, ctx, Instruction::LocalSet(li.off))?;
            }
            op::LSHR => {
                let k = code[pc] as i32; pc += 1;
                wfeed(sink, ctx, Instruction::I32Const(k))?;
                wfeed(sink, ctx, Instruction::I32ShrU)?;
            }
            op::LSHL => {
                let k = code[pc] as i32; pc += 1;
                wfeed(sink, ctx, Instruction::I32Const(k))?;
                wfeed(sink, ctx, Instruction::I32Shl)?;
                wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                wfeed(sink, ctx, Instruction::I32And)?;
            }

            // ── Host call ─────────────────────────────────────────────────────
            op::HOST_CALL => {
                let slot         = code[pc]; pc += 1;
                let packed       = code[pc]; pc += 1;
                let _n_args      = (packed >> 8) as u32;
                let _n_results   = (packed & 0xFF) as u32;
                // The top _n_args values are already on the WASM stack from
                // prior TC2 pushes; Call consumes them.  _n_results values are
                // left on the stack after the call returns.
                let wasm_fn_idx  = emit_ctx.host_fn_base + slot as u32;
                wfeed(sink, ctx, Instruction::Call(wasm_fn_idx))?;
            }

            // ── Intra-bytecode control flow ───────────────────────────────────
            op::JUMP_NOT => {
                let off    = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                wfeed(sink, ctx, Instruction::If(BlockType::Empty))?;
                emit_bc_segment(sink, ctx, code, pc, emit_ctx)?;
                wfeed(sink, ctx, Instruction::Else)?;
                emit_bc_segment(sink, ctx, code, target, emit_ctx)?;
                wfeed(sink, ctx, Instruction::End)?;
                return Ok(());
            }
            op::JUMP_IF => {
                let off    = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                wfeed(sink, ctx, Instruction::If(BlockType::Empty))?;
                emit_bc_segment(sink, ctx, code, target, emit_ctx)?;
                wfeed(sink, ctx, Instruction::Else)?;
                emit_bc_segment(sink, ctx, code, pc, emit_ctx)?;
                wfeed(sink, ctx, Instruction::End)?;
                return Ok(());
            }
            op::JUMP => {
                let off    = code[pc] as i16; pc += 1;
                let target = (pc as isize + off as isize) as usize;
                emit_bc_segment(sink, ctx, code, target, emit_ctx)?;
                return Ok(());
            }

            _ => {
                wfeed(sink, ctx, Instruction::Unreachable)?;
            }
        }
    }
    Ok(())
}
