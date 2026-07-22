//! WASM direct backend — translates the AGC instruction stream into a
//! WebAssembly module using the yecta [`Reactor`].
//!
//! ## Register-local optimisation
//!
//! AGC registers `0x0000`–`0x0006` (A, L, Q, EB, FB, Z, BB), special
//! registers `0xFF00`–`0xFF03` (TMP, EXTEND, INHINT, INSTR\_WORD) and virtual
//! registers `0xFF10`–`0xFF1F` are all promoted from linear-memory loads/stores
//! to dedicated WASM `local.get`/`local.set` instructions.  This eliminates
//! two load/store instructions per register access in the common case.
//!
//! ## Static address folding via `ConstPeek`
//!
//! `emit_bc_segment` is generic over [`WasmSink`], which extends
//! [`wax_core::build::ConstPeek`].  When emitting `LOAD_OFF`, `STORE_OFF`,
//! `LOAD_IND`, or `STORE_IND`, the backend calls `sink.peek_local_i32` /
//! `sink.peek_stack_i32` (provided for free by [`FeedSink`] via the reactor's
//! shadow constant-stack) to check whether the effective address is a
//! compile-time constant.  If it is, the instruction is compiled directly to
//! a register-local or known-address access.
//!
//! ## Dynamic address fallback
//!
//! When the address cannot be resolved statically, the backend flushes all
//! register-locals to their linear-memory backing locations, calls the host
//! `mem_read` / `mem_write` import (which observes linear memory), then
//! restores the register-locals from linear memory.  This keeps the register-
//! local optimisation correct even when AGC code performs self-modifying
//! accesses via `LOAD_IND` / `STORE_IND`.
//!
//! ## Emission-context isolation (outer / nested)
//!
//! Each WASM function carries two sets of scratch locals (T, OFF, SCR0, SCR1)
//! to prevent trap/hook code from clobbering the outer instruction's working
//! registers.  The register-file locals (A … INSTR\_WORD) and virtual-register
//! locals are **shared** between both contexts.

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use wasm_encoder::{
    BlockType, CodeSection, EntityType, ExportKind, ExportSection, Function, FunctionSection,
    ImportSection, Instruction, MemArg, MemorySection, MemoryType, Module, TypeSection, ValType,
};

use yecta::{FuncIdx, LocalLayout, LocalPool, LocalPoolBackend, LocalSlot, Mark, Reactor};
use yecta::layout::CellIdx;

use wax_core::build::ConstPeek;

use speet_traps::{
    ArchTag, InsnClass, InstructionInfo, JumpInfo, JumpKind, TrapAction, TrapConfig,
};

use agc_isa::InstrType;

use super::{DirectBackend, DirectFunctionKey, DirectInstr};
use crate::ir::Terminator;
use agc_lower::bytecode::op;

// ─── WASM linear-memory layout (register backing store) ──────────────────────
// Used only for flush/restore around dynamic-address accesses.

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

const FN_MEM_READ:     u32 = 0;
const FN_MEM_WRITE:    u32 = 1;
const FN_CHAN_READ:    u32 = 2;
const FN_CHAN_WRITE:   u32 = 3;
/// Number of standard I/O imports (before any host functions).
pub const NUM_IMPORTS: u32 = 4;

// ─── Virtual-register address space ──────────────────────────────────────────

/// First TC2 address in the virtual-register range (`0xFF10`).
pub const VIRT_REG_BASE: u16  = 0xFF10;
/// Number of virtual registers (addresses `0xFF10`..`0xFF1F`).
pub const N_VIRT_REGS: usize = 16;

// ─── Helper ───────────────────────────────────────────────────────────────────

#[inline]
fn ri(addr: u16, extend: bool) -> u32 {
    2 * addr as u32 + extend as u32
}

#[inline]
fn planned_function_index(
    functions: &BTreeMap<DirectFunctionKey, u32>,
    addr: u16,
    extend: bool,
) -> u32 {
    functions
        .get(&DirectFunctionKey::new(addr & 0x0FFF, extend))
        .copied()
        .unwrap_or_else(|| ri(addr, extend))
}

// ─── WasmSink — generic emission + constant-peek interface ───────────────────

/// Instruction emission interface for TC2-to-WASM helpers, extending
/// [`ConstPeek`] so helpers can query the constant-folding shadow stack.
///
/// Two implementations are provided:
/// * [`FeedSink`]  — wraps `(&Reactor, tail_idx)`; fully implements `ConstPeek`.
/// * [`speet_traps::TrapContext`]  — uses the `ConstPeek` defaults (all `None`),
///   triggering the flush/restore dynamic fallback for every address access.
pub(crate) trait WasmSink<Context, Err>: ConstPeek {
    fn emit_wasm(&mut self, ctx: &mut Context, instr: &Instruction<'_>) -> Result<(), Err>;
}

/// Binds a [`Reactor`] to a fixed `tail_idx`, implementing [`WasmSink`] with
/// full [`ConstPeek`] support.
pub(crate) struct FeedSink<'r, Context, Err> {
    pub reactor:  &'r Reactor<Context, Err, Function, LocalPool>,
    pub tail_idx: usize,
}

impl<Context, Err> ConstPeek for FeedSink<'_, Context, Err> {
    /// Delegates to the reactor's public `ConstPeek` implementation.
    /// Since `FeedSink` always holds `tail_idx == reactor.fn_count() - 1`
    /// during a single `feed_instr` call, the reactor's tail-entry peek
    /// is exactly the right view.
    fn peek_stack_i32(&self, depth: usize) -> Option<i32> {
        self.reactor.peek_stack_i32(depth)
    }
    fn peek_local_i32(&self, local_idx: u32) -> Option<i32> {
        self.reactor.peek_local_i32(local_idx)
    }
}

impl<Context, Err> WasmSink<Context, Err> for FeedSink<'_, Context, Err> {
    #[inline]
    fn emit_wasm(&mut self, ctx: &mut Context, instr: &Instruction<'_>) -> Result<(), Err> {
        self.reactor.feed_to(self.tail_idx, ctx, instr)
    }
}

// TrapContext's ConstPeek is implemented in speet-traps (ConstPeek always returns None).
impl<'a, Context, Err> WasmSink<Context, Err> for speet_traps::TrapContext<'a, Context, Err> {
    #[inline]
    fn emit_wasm(&mut self, ctx: &mut Context, instr: &Instruction<'_>) -> Result<(), Err> {
        speet_traps::TrapContext::emit(self, ctx, instr)
    }
}

// ─── RegisterLocals — WASM local indices for the AGC register file ────────────

/// WASM local indices for the eleven fixed AGC/TC2 registers promoted to locals.
///
/// These are **shared** between the outer and nested emission contexts:
/// both contexts represent the same AGC register state.
#[derive(Clone, Copy)]
pub(crate) struct RegisterLocals {
    pub a:          u32,  // 0x0000  A
    pub l:          u32,  // 0x0001  L
    pub q:          u32,  // 0x0002  Q
    pub eb:         u32,  // 0x0003  EB
    pub fb:         u32,  // 0x0004  FB
    pub z:          u32,  // 0x0005  Z  (pre-advanced to next_pc at function entry)
    pub bb:         u32,  // 0x0006  BB
    // 0x0007 = ZERO — constant 0, no local needed
    pub tmp:        u32,  // 0xFF00  TMP
    pub extend:     u32,  // 0xFF01  EXTEND
    pub inhint:     u32,  // 0xFF02  INHINT
    pub instr_word: u32,  // 0xFF03  INSTR_WORD
}

/// Number of register-file locals per function (11: A,L,Q,EB,FB,Z,BB,TMP,EXTEND,INHINT,INSTR_WORD).
pub const N_REG_LOCALS: usize = 11;

/// `(local_idx, linear_mem_offset)` pairs for flush/restore operations.
/// Only the registers that have a linear-memory backing location.
fn reg_mem_pairs(rl: &RegisterLocals) -> [(u32, u64); N_REG_LOCALS] {
    [
        (rl.a,          MEM_A),
        (rl.l,          MEM_L),
        (rl.q,          MEM_Q),
        (rl.eb,         MEM_EB),
        (rl.fb,         MEM_FB),
        (rl.z,          MEM_Z),
        (rl.bb,         MEM_BB),
        (rl.tmp,        MEM_TMP),
        (rl.extend,     MEM_EXTEND),
        (rl.inhint,     MEM_INHINT),
        (rl.instr_word, MEM_INSTR_WORD),
    ]
}

// ─── EmitContext ──────────────────────────────────────────────────────────────

/// Per-namespace TC2-to-WASM emission context.
#[derive(Clone, Copy)]
pub(crate) struct LocalIndices {
    pub t:    u32,
    pub off:  u32,
    pub scr0: u32,
    pub scr1: u32,
}

/// Per-function TC2-to-WASM emission context.
///
/// Two contexts are created per function (outer / nested).  They share
/// `regs` and `virt_regs` but have independent `li` scratch locals.
#[derive(Clone)]
pub struct EmitContext {
    pub(crate) li:           LocalIndices,
    pub(crate) regs:         RegisterLocals,   // shared between outer and nested
    pub(crate) virt_regs:    [u32; N_VIRT_REGS],
    pub(crate) host_fn_base: u32,
}

impl EmitContext {
    #[inline]
    pub(crate) fn virt_reg(&self, addr: u16) -> Option<u32> {
        let i = addr.wrapping_sub(VIRT_REG_BASE) as usize;
        if i < N_VIRT_REGS { Some(self.virt_regs[i]) } else { None }
    }
}

// ─── HostFnSig ────────────────────────────────────────────────────────────────

/// Signature of a host function callable from TC2 via [`op::HOST_CALL`].
#[derive(Clone, Debug)]
pub struct HostFnSig {
    pub module:  alloc::string::String,
    pub name:    alloc::string::String,
    pub params:  u32,
    pub results: u32,
}

// ─── WasmDirectBackend ────────────────────────────────────────────────────────

pub struct WasmDirectBackend<'cb, 'ctx, Context = (), Err = String> {
    reactor:      Reactor<Context, Err, Function, LocalPool>,
    entry_points: Vec<u16>,
    /// Compact function indices reserved by `DirectBackend::prepare`.
    /// Empty preserves the legacy dense `ri()` numbering.
    function_indices: BTreeMap<DirectFunctionKey, u32>,
    tail_idx:     usize,

    layout:      LocalLayout,
    locals_mark: Mark,

    // Outer scratch (T, OFF, SCR0, SCR1)
    slot_t:    LocalSlot,
    slot_off:  LocalSlot,
    slot_scr0: LocalSlot,
    slot_scr1: LocalSlot,

    // Nested scratch (isolated from outer)
    slot_nested_t:    LocalSlot,
    slot_nested_off:  LocalSlot,
    slot_nested_scr0: LocalSlot,
    slot_nested_scr1: LocalSlot,

    // AGC register-file locals (shared between outer and nested)
    slot_reg_a:          LocalSlot,
    slot_reg_l:          LocalSlot,
    slot_reg_q:          LocalSlot,
    slot_reg_eb:         LocalSlot,
    slot_reg_fb:         LocalSlot,
    slot_reg_z:          LocalSlot,
    slot_reg_bb:         LocalSlot,
    slot_reg_tmp:        LocalSlot,
    slot_reg_extend:     LocalSlot,
    slot_reg_inhint:     LocalSlot,
    slot_reg_instr_word: LocalSlot,

    // Virtual registers (shared between outer and nested)
    slot_virt: [LocalSlot; N_VIRT_REGS],

    host_fns: Vec<HostFnSig>,
    traps:    TrapConfig<'cb, 'ctx, Context, Err>,
}

impl<'cb, 'ctx, Context, Err> WasmDirectBackend<'cb, 'ctx, Context, Err> {
    pub fn new(entry_points: Vec<u16>) -> Self {
        let mut s = Self {
            reactor:     Reactor::with_base_func_offset(NUM_IMPORTS),
            entry_points,
            function_indices: BTreeMap::new(),
            tail_idx:    0,
            layout:      LocalLayout::empty(),
            locals_mark: Mark { slot_count: 0, total_locals: 0 },
            slot_t:    LocalSlot::default(), slot_off:  LocalSlot::default(),
            slot_scr0: LocalSlot::default(), slot_scr1: LocalSlot::default(),
            slot_nested_t:    LocalSlot::default(), slot_nested_off:  LocalSlot::default(),
            slot_nested_scr0: LocalSlot::default(), slot_nested_scr1: LocalSlot::default(),
            slot_reg_a:          LocalSlot::default(),
            slot_reg_l:          LocalSlot::default(),
            slot_reg_q:          LocalSlot::default(),
            slot_reg_eb:         LocalSlot::default(),
            slot_reg_fb:         LocalSlot::default(),
            slot_reg_z:          LocalSlot::default(),
            slot_reg_bb:         LocalSlot::default(),
            slot_reg_tmp:        LocalSlot::default(),
            slot_reg_extend:     LocalSlot::default(),
            slot_reg_inhint:     LocalSlot::default(),
            slot_reg_instr_word: LocalSlot::default(),
            slot_virt: [LocalSlot::default(); N_VIRT_REGS],
            host_fns: Vec::new(),
            traps:    TrapConfig::new(),
        };
        s.setup_traps();
        s
    }

    /// Return the function index for a direct target. Before `prepare`, retain
    /// the legacy dense address/EXTEND numbering for compatibility.
    fn function_index(&self, addr: u16, extend: bool) -> u32 {
        planned_function_index(&self.function_indices, addr, extend)
    }

    pub fn add_host_fn(&mut self, sig: HostFnSig) -> u16 {
        let slot = self.host_fns.len() as u16;
        self.host_fns.push(sig);
        self.reactor.set_base_func_offset(NUM_IMPORTS + self.host_fns.len() as u32);
        slot
    }

    pub fn setup_traps(&mut self) {
        self.layout = LocalLayout::empty();
        self.traps.declare_params(CellIdx(0), &mut self.layout);
        self.locals_mark = self.layout.mark();
    }

    pub fn set_instruction_trap(
        &mut self,
        trap: &'cb mut (dyn speet_traps::InstructionTrap<Context, Err> + 'ctx),
    ) {
        self.traps.set_instruction_trap(trap);
        self.setup_traps();
    }

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

    fn prepare(&mut self, functions: &[DirectFunctionKey]) -> Result<(), Err> {
        assert_eq!(
            self.reactor.fn_count(),
            0,
            "direct functions must be reserved before feeding any body"
        );
        self.function_indices.clear();
        for (index, key) in functions.iter().copied().enumerate() {
            assert!(
                self.function_indices.insert(key, index as u32).is_none(),
                "direct function plan contains duplicate {key:?}"
            );
        }
        Ok(())
    }

    fn feed_instr(&mut self, ctx: &mut Context, instr: &DirectInstr) -> Result<(), Err> {
        if !self.function_indices.is_empty() {
            let key = DirectFunctionKey::new(instr.addr, instr.extend);
            assert_eq!(
                self.function_indices.get(&key),
                Some(&(self.reactor.fn_count() as u32)),
                "direct bodies must be fed in their declared plan order"
            );
        }
        // ── Per-function local layout ─────────────────────────────────────────
        self.layout.rewind(&self.locals_mark);

        // Outer scratch
        self.slot_t    = self.layout.append(1, ValType::I32);
        self.slot_off  = self.layout.append(1, ValType::I32);
        self.slot_scr0 = self.layout.append(1, ValType::I32);
        self.slot_scr1 = self.layout.append(1, ValType::I32);

        // Nested scratch (isolated)
        self.slot_nested_t    = self.layout.append(1, ValType::I32);
        self.slot_nested_off  = self.layout.append(1, ValType::I32);
        self.slot_nested_scr0 = self.layout.append(1, ValType::I32);
        self.slot_nested_scr1 = self.layout.append(1, ValType::I32);

        // AGC register-file locals (shared)
        self.slot_reg_a          = self.layout.append(1, ValType::I32);
        self.slot_reg_l          = self.layout.append(1, ValType::I32);
        self.slot_reg_q          = self.layout.append(1, ValType::I32);
        self.slot_reg_eb         = self.layout.append(1, ValType::I32);
        self.slot_reg_fb         = self.layout.append(1, ValType::I32);
        self.slot_reg_z          = self.layout.append(1, ValType::I32);
        self.slot_reg_bb         = self.layout.append(1, ValType::I32);
        self.slot_reg_tmp        = self.layout.append(1, ValType::I32);
        self.slot_reg_extend     = self.layout.append(1, ValType::I32);
        self.slot_reg_inhint     = self.layout.append(1, ValType::I32);
        self.slot_reg_instr_word = self.layout.append(1, ValType::I32);

        // Virtual registers (shared)
        for s in &mut self.slot_virt {
            *s = self.layout.append(1, ValType::I32);
        }

        self.traps.declare_locals(CellIdx(0), &mut self.layout);

        let fn_locals: alloc::vec::Vec<(u32, ValType)> =
            self.layout.iter_since(&self.locals_mark).collect();
        self.reactor.next_with(ctx, Function::new(fn_locals), 2)?;
        self.tail_idx = self.reactor.fn_count().saturating_sub(1);
        let tail_idx  = self.tail_idx;

        // ── Resolve local indices ─────────────────────────────────────────────
        let virt_regs: [u32; N_VIRT_REGS] =
            core::array::from_fn(|i| self.layout.base(self.slot_virt[i]));
        let host_fn_base = NUM_IMPORTS;

        let regs = RegisterLocals {
            a:          self.layout.base(self.slot_reg_a),
            l:          self.layout.base(self.slot_reg_l),
            q:          self.layout.base(self.slot_reg_q),
            eb:         self.layout.base(self.slot_reg_eb),
            fb:         self.layout.base(self.slot_reg_fb),
            z:          self.layout.base(self.slot_reg_z),
            bb:         self.layout.base(self.slot_reg_bb),
            tmp:        self.layout.base(self.slot_reg_tmp),
            extend:     self.layout.base(self.slot_reg_extend),
            inhint:     self.layout.base(self.slot_reg_inhint),
            instr_word: self.layout.base(self.slot_reg_instr_word),
        };

        let outer_ctx = EmitContext {
            li: LocalIndices {
                t:    self.layout.base(self.slot_t),
                off:  self.layout.base(self.slot_off),
                scr0: self.layout.base(self.slot_scr0),
                scr1: self.layout.base(self.slot_scr1),
            },
            regs,
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
            regs,      // shared
            virt_regs, // shared
            host_fn_base,
        };

        // ── Initialise register-file locals from linear memory ────────────────
        // Load all registers from their backing memory (AGC state from previous fn).
        // Z and INSTR_WORD are overridden immediately after.
        let r = &self.reactor;
        for (local_idx, mem_off) in reg_mem_pairs(&regs) {
            feed(r, tail_idx, ctx, Instruction::I32Const(0))?;
            feed(r, tail_idx, ctx, Instruction::I32Load16U(mem16(mem_off)))?;
            feed(r, tail_idx, ctx, Instruction::LocalSet(local_idx))?;
        }

        // Override Z = next_pc (PC has already advanced).
        let next_pc = instr.addr.wrapping_add(1) & 0x7FFF;
        feed(r, tail_idx, ctx, Instruction::I32Const(next_pc as i32))?;
        feed(r, tail_idx, ctx, Instruction::LocalSet(regs.z))?;

        // Override INSTR_WORD = raw word.
        feed(r, tail_idx, ctx, Instruction::I32Const(instr.raw_word as i32))?;
        feed(r, tail_idx, ctx, Instruction::LocalSet(regs.instr_word))?;

        // ── Instruction trap ──────────────────────────────────────────────────
        let speet_info = InstructionInfo {
            pc:    instr.addr as u64,
            len:   1,
            arch:  ArchTag::Other,
            class: classify_agc_insn(instr.instr_type),
        };
        let (reactor, traps, layout) = (&mut self.reactor, &mut self.traps, &self.layout);
        let action = traps.on_instruction(&speet_info, ctx, reactor, layout)?;

        if action == TrapAction::Skip {
            emit_direct_terminator(
                &mut self.reactor, self.tail_idx, ctx,
                instr, &mut self.traps, &self.layout, &outer_ctx, &self.function_indices,
            )?;
            return Ok(());
        }

        // ── Emit TC2 bytecode (outer namespace) ───────────────────────────────
        let mut outer_sink = FeedSink { reactor: &self.reactor, tail_idx };
        emit_bc_segment(&mut outer_sink, ctx, &instr.bytecode, 0, &outer_ctx)?;

        // ── Emit control-flow terminator ──────────────────────────────────────
        emit_direct_terminator(
            &mut self.reactor, self.tail_idx, ctx,
            instr, &mut self.traps, &self.layout, &outer_ctx, &self.function_indices,
        )?;

        let _ = nested_ctx;
        Ok(())
    }

    fn finish(self, ctx: &mut Context) -> Result<Vec<u8>, Err> {
        let n_funcs          = self.reactor.fn_count() as u32;
        let base_func_offset = self.reactor.base_func_offset();
        let entry_function_indices: Vec<_> = self
            .entry_points
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|entry| (entry, self.function_index(entry, false)))
            .collect();
        let mut functions    = self.reactor.into_fns();

        for f in &mut functions {
            f.instruction(&Instruction::End);
        }

        let mut module = Module::new();

        // Type section
        let mut types = TypeSection::new();
        types.ty().function([], []);
        types.ty().function([ValType::I32], [ValType::I32]);
        types.ty().function([ValType::I32, ValType::I32], []);

        let mut host_type_indices: Vec<u32> = Vec::new();
        for (i, hf) in self.host_fns.iter().enumerate() {
            let params:  alloc::vec::Vec<ValType> = (0..hf.params).map(|_| ValType::I32).collect();
            let results: alloc::vec::Vec<ValType> = (0..hf.results).map(|_| ValType::I32).collect();
            let existing = host_type_indices.iter().enumerate().find(|(_j, _)| {
                let hf2 = &self.host_fns[i];
                hf2.params == hf.params && hf2.results == hf.results
            });
            if let Some((_, &ty_idx)) = existing {
                host_type_indices.push(ty_idx);
            } else {
                let ty_idx = 3 + host_type_indices.len() as u32;
                types.ty().function(params, results);
                host_type_indices.push(ty_idx);
            }
        }
        module.section(&types);

        // Import section
        let mut imports = ImportSection::new();
        imports.import("env", "mem_read",  EntityType::Function(1));
        imports.import("env", "mem_write", EntityType::Function(2));
        imports.import("env", "chan_read", EntityType::Function(1));
        imports.import("env", "chan_write",EntityType::Function(2));
        for (i, hf) in self.host_fns.iter().enumerate() {
            imports.import(&hf.module, &hf.name, EntityType::Function(host_type_indices[i]));
        }
        module.section(&imports);

        // Function section (all type 0)
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
        for (entry, function_index) in entry_function_indices {
            exports.export(
                &format!("bb_{entry:05o}"),
                ExportKind::Func,
                function_index + base_func_offset,
            );
        }
        module.section(&exports);

        // Code section
        let mut code = CodeSection::new();
        for f in functions { code.function(&f); }
        module.section(&code);

        Ok(module.finish())
    }
}

// ─── AGC classification ───────────────────────────────────────────────────────

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

// ─── Flush / restore register-file locals ────────────────────────────────────

/// Write every register-local to its linear-memory backing store.
///
/// Called before any `FN_MEM_READ`/`FN_MEM_WRITE` call whose address is not
/// statically known, so the host sees a coherent register-file view.
fn flush_regs<Context, Err, S: WasmSink<Context, Err>>(
    sink:     &mut S,
    ctx:      &mut Context,
    emit_ctx: &EmitContext,
) -> Result<(), Err> {
    for (local_idx, mem_off) in reg_mem_pairs(&emit_ctx.regs) {
        wfeed(sink, ctx, Instruction::I32Const(0))?;
        wfeed(sink, ctx, Instruction::LocalGet(local_idx))?;
        wfeed(sink, ctx, Instruction::I32Store16(mem16(mem_off)))?;
    }
    Ok(())
}

/// Reload every register-local from linear memory.
///
/// Called after a host `FN_MEM_WRITE` call with an unknown address, to pick
/// up any register-mapped writes the host may have performed.
fn restore_regs<Context, Err, S: WasmSink<Context, Err>>(
    sink:     &mut S,
    ctx:      &mut Context,
    emit_ctx: &EmitContext,
) -> Result<(), Err> {
    for (local_idx, mem_off) in reg_mem_pairs(&emit_ctx.regs) {
        wfeed(sink, ctx, Instruction::I32Const(0))?;
        wfeed(sink, ctx, Instruction::I32Load16U(mem16(mem_off)))?;
        wfeed(sink, ctx, Instruction::LocalSet(local_idx))?;
    }
    Ok(())
}

/// Flush register-locals to memory via the direct-reactor path (in terminators).
fn flush_regs_direct<Context, Err>(
    reactor:   &Reactor<Context, Err, Function, LocalPool>,
    tail_idx:  usize,
    ctx:       &mut Context,
    emit_ctx:  &EmitContext,
) -> Result<(), Err> {
    for (local_idx, mem_off) in reg_mem_pairs(&emit_ctx.regs) {
        feed(reactor, tail_idx, ctx, Instruction::I32Const(0))?;
        feed(reactor, tail_idx, ctx, Instruction::LocalGet(local_idx))?;
        feed(reactor, tail_idx, ctx, Instruction::I32Store16(mem16(mem_off)))?;
    }
    Ok(())
}

// ─── fire_jmp ─────────────────────────────────────────────────────────────────

fn fire_jmp<'cb, 'ctx, Context, Err>(
    reactor:       &mut Reactor<Context, Err, Function, LocalPool>,
    tail_idx:      usize,
    ctx:           &mut Context,
    traps:         &mut TrapConfig<'cb, 'ctx, Context, Err>,
    layout:        &LocalLayout,
    emit_ctx:      &EmitContext,
    source_pc:     u16,
    kind:          JumpKind,
    target_addr:   u16,
    target_extend: bool,
    function_indices: &BTreeMap<DirectFunctionKey, u32>,
) -> Result<(), Err> {
    let jinfo = JumpInfo::direct(source_pc as u64, target_addr as u64, kind);
    let action = traps.on_jump(&jinfo, ctx, reactor, layout)?;
    if action == TrapAction::Continue {
        // Flush registers to linear memory before the return_call so the next
        // function's load-at-entry sees the current register state.
        flush_regs_direct(reactor, tail_idx, ctx, emit_ctx)?;
        reactor.jmp(
            tail_idx,
            ctx,
            FuncIdx(planned_function_index(function_indices, target_addr, target_extend)),
            0,
        )?;
    }
    Ok(())
}

// ─── emit_direct_terminator ───────────────────────────────────────────────────

fn emit_direct_terminator<'cb, 'ctx, Context, Err>(
    reactor:   &mut Reactor<Context, Err, Function, LocalPool>,
    tail_idx:  usize,
    ctx:       &mut Context,
    instr:     &DirectInstr,
    traps:     &mut TrapConfig<'cb, 'ctx, Context, Err>,
    layout:    &LocalLayout,
    emit_ctx:  &EmitContext,
    function_indices: &BTreeMap<DirectFunctionKey, u32>,
) -> Result<(), Err> {
    let next_addr = instr.addr.wrapping_add(1) & 0x7FFF;
    let base      = reactor.base_func_offset();

    match &instr.terminator {
        Terminator::FallThrough(_) => {
            if instr.instr_type == Some(InstrType::Extend) {
                fire_jmp(reactor, tail_idx, ctx, traps, layout, emit_ctx,
                    instr.addr, JumpKind::DirectJump, next_addr, true, function_indices)?;
            } else if instr.extend {
                fire_jmp(reactor, tail_idx, ctx, traps, layout, emit_ctx,
                    instr.addr, JumpKind::DirectJump, next_addr, false, function_indices)?;
            }
        }

        Terminator::Jump(target) => {
            fire_jmp(reactor, tail_idx, ctx, traps, layout, emit_ctx,
                instr.addr, JumpKind::DirectJump, *target, false, function_indices)?;
        }

        Terminator::CondBranch { taken, fallthru } => {
            // Read Z from its register-local (set by TC2 STORE15(Z)).
            feed(reactor, tail_idx, ctx, Instruction::LocalGet(emit_ctx.regs.z))?;
            feed(reactor, tail_idx, ctx, Instruction::I32Const(*taken as i32))?;
            feed(reactor, tail_idx, ctx, Instruction::I32Eq)?;
            feed(reactor, tail_idx, ctx, Instruction::If(BlockType::Empty))?;
            {
                let jinfo = JumpInfo::direct(instr.addr as u64, *taken as u64,
                    JumpKind::ConditionalBranch);
                let action = traps.on_jump(&jinfo, ctx, reactor, layout)?;
                if action == TrapAction::Continue {
                    flush_regs_direct(reactor, tail_idx, ctx, emit_ctx)?;
                    feed(reactor, tail_idx, ctx,
                        Instruction::ReturnCall(planned_function_index(function_indices, *taken, false) + base))?;
                }
            }
            feed(reactor, tail_idx, ctx, Instruction::End)?;
            fire_jmp(reactor, tail_idx, ctx, traps, layout, emit_ctx,
                instr.addr, JumpKind::ConditionalBranch, *fallthru, false, function_indices)?;
        }

        Terminator::CcsBranch(targets) => {
            feed(reactor, tail_idx, ctx, Instruction::LocalGet(emit_ctx.regs.z))?;
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
                        flush_regs_direct(reactor, tail_idx, ctx, emit_ctx)?;
                        feed(reactor, tail_idx, ctx,
                            Instruction::ReturnCall(planned_function_index(function_indices, t, false) + base))?;
                    }
                }
                feed(reactor, tail_idx, ctx, Instruction::End)?;
            }
            feed(reactor, tail_idx, ctx, Instruction::Unreachable)?;
        }

        Terminator::Indirect { possible_targets } => {
            feed(reactor, tail_idx, ctx, Instruction::LocalGet(emit_ctx.regs.z))?;
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
                        flush_regs_direct(reactor, tail_idx, ctx, emit_ctx)?;
                        feed(reactor, tail_idx, ctx,
                            Instruction::ReturnCall(planned_function_index(function_indices, t, false) + base))?;
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

// ─── Low-level helpers ────────────────────────────────────────────────────────

#[inline]
fn mem16(offset: u64) -> MemArg {
    MemArg { offset, align: 1, memory_index: 0 }
}

/// Reactor-direct instruction feed (for terminator emission).
#[inline]
fn feed<Context, Err>(
    reactor:  &Reactor<Context, Err, Function, LocalPool>,
    tail_idx: usize,
    ctx:      &mut Context,
    instr:    Instruction<'_>,
) -> Result<(), Err> {
    reactor.feed_to(tail_idx, ctx, &instr)
}

/// WasmSink-based instruction feed (for emit_bc_segment and friends).
#[inline]
fn wfeed<Context, Err, S: WasmSink<Context, Err>>(
    sink:  &mut S,
    ctx:   &mut Context,
    instr: Instruction<'_>,
) -> Result<(), Err> {
    sink.emit_wasm(ctx, &instr)
}

/// Map a TC2 address to its AGC register-local index, if it is one of the
/// fixed-register addresses (0x0000–0x0006, 0xFF00–0xFF03).
fn reg_local_for(addr: u16, regs: &RegisterLocals) -> Option<u32> {
    match addr {
        0x0000 => Some(regs.a),
        0x0001 => Some(regs.l),
        0x0002 => Some(regs.q),
        0x0003 => Some(regs.eb),
        0x0004 => Some(regs.fb),
        0x0005 => Some(regs.z),
        0x0006 => Some(regs.bb),
        0xFF00 => Some(regs.tmp),
        0xFF01 => Some(regs.extend),
        0xFF02 => Some(regs.inhint),
        0xFF03 => Some(regs.instr_word),
        _ => None,
    }
}

// ─── emit_load_known / emit_store_known ────────────────────────────────────────

fn emit_load_known<Context, Err, S: WasmSink<Context, Err>>(
    sink:     &mut S,
    ctx:      &mut Context,
    addr:     u16,
    emit_ctx: &EmitContext,
) -> Result<(), Err> {
    // AGC register-file → local.get
    if let Some(local_idx) = reg_local_for(addr, &emit_ctx.regs) {
        return wfeed(sink, ctx, Instruction::LocalGet(local_idx));
    }
    // Virtual register → local.get
    if let Some(local_idx) = emit_ctx.virt_reg(addr) {
        return wfeed(sink, ctx, Instruction::LocalGet(local_idx));
    }
    // ZERO (read-only constant 0)
    if addr == 0x0007 {
        return wfeed(sink, ctx, Instruction::I32Const(0));
    }
    // I/O channel
    if addr >= 0x8000 {
        wfeed(sink, ctx, Instruction::I32Const((addr - 0x8000) as i32))?;
        wfeed(sink, ctx, Instruction::Call(FN_CHAN_READ))?;
        wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
        return wfeed(sink, ctx, Instruction::I32And);
    }
    // AGC memory (not register-mapped)
    wfeed(sink, ctx, Instruction::I32Const(addr as i32))?;
    wfeed(sink, ctx, Instruction::Call(FN_MEM_READ))?;
    wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
    wfeed(sink, ctx, Instruction::I32And)
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
    // AGC register-file → local.set
    if let Some(local_idx) = reg_local_for(addr, &emit_ctx.regs) {
        return wfeed(sink, ctx, Instruction::LocalSet(local_idx));
    }
    // Virtual register → local.set
    if let Some(local_idx) = emit_ctx.virt_reg(addr) {
        return wfeed(sink, ctx, Instruction::LocalSet(local_idx));
    }
    // ZERO → drop (writes to ZERO are silently discarded)
    if addr == 0x0007 {
        return wfeed(sink, ctx, Instruction::Drop);
    }
    // I/O channel
    if addr >= 0x8000 {
        wfeed(sink, ctx, Instruction::LocalSet(emit_ctx.li.scr0))?;
        wfeed(sink, ctx, Instruction::I32Const((addr - 0x8000) as i32))?;
        wfeed(sink, ctx, Instruction::LocalGet(emit_ctx.li.scr0))?;
        return wfeed(sink, ctx, Instruction::Call(FN_CHAN_WRITE));
    }
    // AGC memory
    wfeed(sink, ctx, Instruction::LocalSet(emit_ctx.li.scr0))?;
    wfeed(sink, ctx, Instruction::I32Const(addr as i32))?;
    wfeed(sink, ctx, Instruction::LocalGet(emit_ctx.li.scr0))?;
    wfeed(sink, ctx, Instruction::Call(FN_MEM_WRITE))
}

// ─── emit_bc_segment — generic over WasmSink ─────────────────────────────────

/// Translate TC2 bytecode `code[start..]` to WASM, emitting via `sink`.
///
/// Generic over [`WasmSink`] so the same logic works with both the main
/// reactor path ([`FeedSink`]) and trap-callback paths
/// ([`speet_traps::TrapContext`]).
///
/// Uses [`ConstPeek`] (via `WasmSink`) to resolve `LOAD_OFF`, `STORE_OFF`,
/// `LOAD_IND`, and `STORE_IND` addresses at compile time when possible.
/// Falls back to flush/restore for runtime-determined addresses.
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

            // ── OFF-relative memory (with static folding) ─────────────────────

            op::LOAD_OFF => {
                if let Some(addr) = sink.peek_local_i32(li.off).map(|v| v as u16) {
                    // OFF is a compile-time constant — resolve directly.
                    emit_load_known(sink, ctx, addr, emit_ctx)?;
                } else {
                    // Dynamic address: flush, call host, no restore needed (read only).
                    flush_regs(sink, ctx, emit_ctx)?;
                    wfeed(sink, ctx, Instruction::LocalGet(li.off))?;
                    wfeed(sink, ctx, Instruction::Call(FN_MEM_READ))?;
                    wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                    wfeed(sink, ctx, Instruction::I32And)?;
                }
            }
            op::STORE_OFF => {
                if let Some(addr) = sink.peek_local_i32(li.off).map(|v| v as u16) {
                    emit_store_known(sink, ctx, addr, false, emit_ctx)?;
                } else {
                    flush_regs(sink, ctx, emit_ctx)?;
                    wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                    wfeed(sink, ctx, Instruction::LocalGet(li.off))?;
                    wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                    wfeed(sink, ctx, Instruction::Call(FN_MEM_WRITE))?;
                    restore_regs(sink, ctx, emit_ctx)?;
                }
            }
            op::LOAD_OFF1 => {
                if let Some(base_addr) = sink.peek_local_i32(li.off).map(|v| v as u16) {
                    let addr = base_addr.wrapping_add(1) & 0xFFFF;
                    emit_load_known(sink, ctx, addr, emit_ctx)?;
                } else {
                    flush_regs(sink, ctx, emit_ctx)?;
                    wfeed(sink, ctx, Instruction::LocalGet(li.off))?;
                    wfeed(sink, ctx, Instruction::I32Const(1))?;
                    wfeed(sink, ctx, Instruction::I32Add)?;
                    wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                    wfeed(sink, ctx, Instruction::I32And)?;
                    wfeed(sink, ctx, Instruction::Call(FN_MEM_READ))?;
                    wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                    wfeed(sink, ctx, Instruction::I32And)?;
                }
            }
            op::STORE_OFF1 => {
                if let Some(base_addr) = sink.peek_local_i32(li.off).map(|v| v as u16) {
                    let addr = base_addr.wrapping_add(1) & 0xFFFF;
                    emit_store_known(sink, ctx, addr, false, emit_ctx)?;
                } else {
                    flush_regs(sink, ctx, emit_ctx)?;
                    wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?;
                    wfeed(sink, ctx, Instruction::LocalGet(li.off))?;
                    wfeed(sink, ctx, Instruction::I32Const(1))?;
                    wfeed(sink, ctx, Instruction::I32Add)?;
                    wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                    wfeed(sink, ctx, Instruction::I32And)?;
                    wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?;
                    wfeed(sink, ctx, Instruction::Call(FN_MEM_WRITE))?;
                    restore_regs(sink, ctx, emit_ctx)?;
                }
            }


            // ── Channel (no register aliasing, no flush needed) ───────────────
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

            // ── Indirect memory (with static folding via ConstPeek) ───────────
            //
            // LOAD_IND: top of stack is the address.  peek_stack_i32(0) queries it.
            // If known, emit Drop to discard the deferred constant (which will be
            // elided by yecta's peephole), then call emit_load_known directly.
            op::LOAD_IND => {
                if let Some(addr) = sink.peek_stack_i32(0).map(|v| v as u16) {
                    // Address is a compile-time constant.  Drop it (elides the deferred
                    // constant via yecta's peephole), then load from the register-local.
                    wfeed(sink, ctx, Instruction::Drop)?;
                    emit_load_known(sink, ctx, addr, emit_ctx)?;
                } else {
                    // Dynamic: flush, call mem_read (which pops addr from stack), no restore.
                    flush_regs(sink, ctx, emit_ctx)?;
                    wfeed(sink, ctx, Instruction::Call(FN_MEM_READ))?;
                    wfeed(sink, ctx, Instruction::I32Const(0xFFFF))?;
                    wfeed(sink, ctx, Instruction::I32And)?;
                }
            }
            // STORE_IND: TC2 stack is [addr (top), val (below addr)].
            // peek_stack_i32(0) gives addr if it is a constant.
            op::STORE_IND => {
                if let Some(addr) = sink.peek_stack_i32(0).map(|v| v as u16) {
                    // Addr is known.  Drop it (elides deferred constant), then store.
                    wfeed(sink, ctx, Instruction::Drop)?;
                    emit_store_known(sink, ctx, addr, false, emit_ctx)?;
                } else {
                    // Dynamic: flush, reorder addr/val, call mem_write, restore.
                    flush_regs(sink, ctx, emit_ctx)?;
                    // Stack: [addr(top), val] — reorder to [addr, val] for Call.
                    wfeed(sink, ctx, Instruction::LocalSet(li.scr0))?; // scr0 = addr
                    wfeed(sink, ctx, Instruction::LocalSet(li.scr1))?; // scr1 = val
                    wfeed(sink, ctx, Instruction::LocalGet(li.scr0))?; // push addr
                    wfeed(sink, ctx, Instruction::LocalGet(li.scr1))?; // push val
                    wfeed(sink, ctx, Instruction::Call(FN_MEM_WRITE))?;
                    restore_regs(sink, ctx, emit_ctx)?;
                }
            }

            // ── Two-word: known-address LOAD / STORE ──────────────────────────
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

            // ── HOST_CALL ─────────────────────────────────────────────────────
            op::HOST_CALL => {
                let slot       = code[pc]; pc += 1;
                let packed     = code[pc]; pc += 1;
                let _n_args    = (packed >> 8) as u32;
                let _n_results = (packed & 0xFF) as u32;
                let wasm_fn    = emit_ctx.host_fn_base + slot as u32;
                wfeed(sink, ctx, Instruction::Call(wasm_fn))?;
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
