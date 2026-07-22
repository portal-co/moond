//! AGC binary → [`InstrStream`] recursive-descent frontend.
//!
//! ## Algorithm
//!
//! 1. Start a worklist from `entry_points ∪ indirect_targets`.
//! 2. Pop an `(address, extend_state)` pair, decode the instruction at that
//!    address, and continue sequentially until a terminating instruction.
//! 3. When an EXTEND word is encountered it is folded into the same block
//!    and the next instruction is decoded with `extend = true`.
//! 4. All discovered branch targets are pushed onto the worklist.
//! 5. Already-decoded addresses are skipped.

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::string::ToString;
use alloc::vec::Vec;

use agc_interp::decode::decode;
use agc_isa::{builtin_spec_set, InstrType, SemOp};
use agc_lower::lower_sem;

use crate::backend::{DirectFunctionKey, DirectInstr};
use crate::ir::{BasicBlock, InstrRecord, InstrStream, Terminator};
use crate::slicer::slice_next_instr;

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FrontendError {
    pub address: u16,
    pub message: alloc::string::String,
}

impl core::fmt::Display for FrontendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "frontend error at {:#07o}: {}",
            self.address, self.message
        )
    }
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Deterministic, conservatively reachable direct-backend function closure.
///
/// A direct function is identified by both address and EXTEND state. The plan
/// contains only metadata and decoded terminators; it does not feed or emit a
/// backend body.
#[derive(Debug, Clone)]
pub struct DirectFunctionPlan {
    pub functions: Vec<DirectFunctionKey>,
    pub indirect_targets: BTreeSet<u16>,
}

/// Discover the direct-backend closure from requested entry and indirect roots.
///
/// Discovery is breadth-first and deterministic. Dynamic/indirect instructions
/// conservatively add every caller-supplied indirect target. Callers can pass
/// this plan to `feed_direct_plan` to reserve indices and emit only this
/// closure rather than all `4096 × 2` possible instruction states.
pub fn plan_direct_functions(
    memory: &[u16; 4096],
    entry_points: &[u16],
    indirect_targets: BTreeSet<u16>,
) -> DirectFunctionPlan {
    let mut pending = VecDeque::new();
    let mut seen = BTreeSet::new();
    for address in entry_points.iter().chain(indirect_targets.iter()) {
        let key = DirectFunctionKey::new(*address & 0x0FFF, false);
        if seen.insert(key) {
            pending.push_back(key);
        }
    }

    let mut functions = Vec::new();
    while let Some(key) = pending.pop_front() {
        let instr = decode_direct(memory, key.addr, key.extend, &indirect_targets);
        let next = key.addr.wrapping_add(1) & 0x0FFF;
        let mut add = |address: u16, extend: bool| {
            let next_key = DirectFunctionKey::new(address & 0x0FFF, extend);
            if seen.insert(next_key) {
                pending.push_back(next_key);
            }
        };
        match &instr.terminator {
            Terminator::FallThrough(_) => {
                // EXTEND affects exactly the immediately following word.
                add(next, instr.instr_type == Some(InstrType::Extend));
            }
            Terminator::Jump(target) => add(*target, false),
            Terminator::CondBranch { taken, fallthru } => {
                add(*taken, false);
                add(*fallthru, false);
            }
            Terminator::CcsBranch(targets) => {
                for target in targets {
                    add(*target, false);
                }
            }
            Terminator::Indirect { possible_targets } => {
                for target in possible_targets {
                    add(*target, false);
                }
            }
            Terminator::Halt => {}
        }
        functions.push(key);
    }

    DirectFunctionPlan {
        functions,
        indirect_targets,
    }
}

/// Decode an AGC memory image into an [`InstrStream`].
///
/// # Parameters
/// - `memory`: full 12-bit AGC address space (indices 0–4095); each slot is
///   the raw 15-bit word stored at that address.
/// - `entry_points`: addresses from which to begin decoding.
/// - `indirect_targets`: all addresses that may be reached via indirect
///   branches at runtime.  These become additional decode roots **and** are
///   recorded on the returned stream for backends that need the full set.
pub fn decode_stream(
    memory: &[u16; 4096],
    entry_points: &[u16],
    indirect_targets: BTreeSet<u16>,
) -> Result<InstrStream, FrontendError> {
    let specs = builtin_spec_set();

    // Addresses whose basic block has already been fully built.
    let mut done: BTreeSet<u16> = BTreeSet::new();
    // Addresses that are basic-block headers (entry points + branch targets).
    let mut headers: BTreeSet<u16> = BTreeSet::new();
    // Worklist: (block_start_address, extend_state_at_block_start).
    let mut worklist: VecDeque<(u16, bool)> = VecDeque::new();

    let mut blocks: BTreeMap<u16, BasicBlock> = BTreeMap::new();

    // Seed from entry points and indirect targets.
    for &addr in entry_points.iter().chain(indirect_targets.iter()) {
        if headers.insert(addr) {
            worklist.push_back((addr, false));
        }
    }

    while let Some((start, start_extend)) = worklist.pop_front() {
        if done.contains(&start) {
            continue;
        }

        let mut instrs: Vec<InstrRecord> = Vec::new();
        let mut pc: u16 = start;
        let mut extend: bool = start_extend;
        // Set by NDX constant-folding: the modified raw word for the next
        // instruction decode iteration (mirrors the EXTEND absorb pattern).
        let mut next_instr_override: Option<u16> = None;
        let terminator;

        let mut visited = BTreeSet::new();

        'block: loop {
            // If we reach an address that is a header (other than our own start)
            // and we've already decoded it, treat this as a fall-through.
            if pc != start && headers.contains(&pc) {
                terminator = Terminator::FallThrough(pc);
                break 'block;
            }

            // Use the NDX override if set, otherwise read directly from memory.
            let raw_word = next_instr_override
                .take()
                .unwrap_or_else(|| memory[pc as usize & 0x0FFF] & 0x7FFF);

            let decoded = decode(raw_word, extend, &specs).map_err(|e| FrontendError {
                address: pc,
                message: e.to_string().into(),
            })?;

            let was_extended = extend;
            let instr_type = decoded.spec.instr_type;
            let mnemonic = decoded.spec.mnemonic.clone();
            let operand = decoded.address;

            // Semantics from spec (empty slice if not populated).
            let ops: Vec<SemOp> = decoded.spec.semantics.clone().unwrap_or_default();

            let next_pc = (pc + 1) & 0x7FFF;

            // Lowered TC2 bytecode for this instruction.
            let bytecode = lower_sem(operand, &ops);

            instrs.push(InstrRecord {
                pc,
                raw_word,
                was_extended,
                instr_type,
                mnemonic,
                operand,
                ops: ops.clone(),
                bytecode,
            });

            // After an EXTEND instruction, decode the next word as extracode
            // within the same block.  EXTEND itself does not terminate a block.
            if instr_type == InstrType::Extend {
                extend = true;
                pc = next_pc;
                continue 'block;
            }

            // NDX constant-folding: if the slicer can determine the modified
            // next-instruction word at compile time, absorb NDX into the block
            // (like EXTEND) so the next instruction is decoded with the override.
            if instr_type == InstrType::Ndx {
                let bytecode = &instrs.last().expect("just pushed").bytecode;
                if let Some(modified_word) = slice_next_instr(bytecode, memory, next_pc) {
                    next_instr_override = Some(modified_word);
                    pc = next_pc;
                    continue 'block;
                }
                // Slicer returned None — fall through to make_indirect below.
            }

            // Reset extend state (for any potential future loop continuation).
            #[allow(unused_assignments)]
            {
                extend = false;
            }

            // Determine block terminator from instruction type and semantics.
            let term = classify_terminator(
                instr_type,
                &ops,
                pc,
                next_pc,
                operand,
                memory,
                &indirect_targets,
            );

            match term {
                Terminator::FallThrough(addr) | Terminator::Jump(addr)
                    if !visited.contains(&addr) =>
                {
                    visited.insert(addr);
                    pc = addr;
                }
                term => {
                    // Push all successor addresses onto the worklist as new headers.
                    for succ in term.successors() {
                        if headers.insert(succ) {
                            worklist.push_back((succ, false));
                        }
                    }

                    terminator = term;
                    break 'block;
                }
            }
        }

        done.insert(start);
        blocks.insert(
            start,
            BasicBlock {
                label: start,
                instrs,
                terminator,
            },
        );
    }

    Ok(InstrStream {
        blocks,
        entry_points: entry_points.to_vec(),
        indirect_targets,
    })
}

// ─── Terminator classification ────────────────────────────────────────────────

pub(crate) fn classify_terminator(
    instr_type: InstrType,
    ops: &[SemOp],
    pc: u16,
    next_pc: u16,
    operand: u16,
    memory: &[u16; 4096],
    indirect_targets: &BTreeSet<u16>,
) -> Terminator {
    // Collect all Branch / BranchIf targets appearing in SemOps.
    let mut branch_targets: Vec<Option<u16>> = Vec::new();
    let mut has_branch = false;
    let mut has_branch_if = false;

    for op in ops {
        match op {
            SemOp::Branch(expr) => {
                has_branch = true;
                branch_targets.push(static_branch_target(expr, pc, next_pc, operand, memory));
            }
            SemOp::BranchIf(_, expr) => {
                has_branch_if = true;
                branch_targets.push(static_branch_target(expr, pc, next_pc, operand, memory));
            }
            _ => {}
        }
    }

    // CCS: 4 consecutive BranchIf ops, each targeting next_pc+0..3.
    if instr_type == InstrType::Ccs {
        let t: Vec<u16> = branch_targets.iter().filter_map(|t| *t).collect();
        if t.len() == 4 {
            return Terminator::CcsBranch([t[0], t[1], t[2], t[3]]);
        }
        // Fallback: treat as indirect if we couldn't resolve targets.
        return make_indirect(indirect_targets);
    }

    // Unconditional branch with a known static target.
    if has_branch && !has_branch_if {
        if let Some(Some(target)) = branch_targets.first() {
            return Terminator::Jump(*target);
        }
        // Dynamic target — resolve against TC target (mem[operand] for TC).
        if instr_type == InstrType::Tc {
            let tc_target = memory[operand as usize & 0x0FFF] & 0x7FFF;
            return Terminator::Jump(tc_target);
        }
        return make_indirect(indirect_targets);
    }

    // Single conditional branch.
    if has_branch_if && !has_branch {
        if let Some(Some(taken)) = branch_targets.first() {
            return Terminator::CondBranch {
                taken: *taken,
                fallthru: next_pc,
            };
        }
        return make_indirect(indirect_targets);
    }

    // Indirect instructions that we cannot statically resolve.
    if matches!(
        instr_type,
        InstrType::Resume | InstrType::Rupt | InstrType::Go
    ) {
        return make_indirect(indirect_targets);
    }

    // NDX (Index) — indirect unless we give up.
    if instr_type == InstrType::Ndx {
        return make_indirect(indirect_targets);
    }

    // Default: fall through to next sequential address.
    Terminator::FallThrough(next_pc)
}

/// Try to resolve a branch target expression to a concrete address at decode time.
fn static_branch_target(
    expr: &agc_isa::Expr,
    pc: u16,
    next_pc: u16,
    operand: u16,
    memory: &[u16; 4096],
) -> Option<u16> {
    use agc_isa::Expr;
    match expr {
        Expr::Lit(n) => Some(*n & 0x7FFF),
        Expr::PcRel(n) => Some((next_pc as i32 + *n as i32) as u16 & 0x7FFF),
        Expr::Z => Some(next_pc),
        // Branch to mem[operand] — resolvable if operand is in fixed or erasable.
        Expr::Mem => Some(memory[operand as usize & 0x0FFF] & 0x7FFF),
        Expr::MemAt(a) => Some(memory[*a as usize & 0x0FFF] & 0x7FFF),
        // Operand field itself used as target (e.g. TCF uses fixed F address).
        Expr::Operand => Some(operand & 0x7FFF),
        _ => {
            let _ = pc; // suppress unused warning
            None
        }
    }
}

fn make_indirect(indirect_targets: &BTreeSet<u16>) -> Terminator {
    Terminator::Indirect {
        possible_targets: indirect_targets.iter().copied().collect(),
    }
}

// ─── Direct backend helper ────────────────────────────────────────────────────

/// Decode one AGC instruction at `addr` in the given `extend` state and
/// return a [`DirectInstr`] ready to feed into a [`crate::backend::DirectBackend`].
///
/// Called once per `(addr, extend)` pair by callers of direct backends.
/// The AGC address space is 12-bit (0x000–0xFFF); `addr` is silently masked.
///
/// On decode failure (e.g. an extracode opcode with no spec entry) the
/// returned record has `instr_type = None`, empty `bytecode`, and
/// `terminator = Terminator::Halt`.
pub fn decode_direct(
    memory: &[u16; 4096],
    addr: u16,
    extend: bool,
    indirect_targets: &BTreeSet<u16>,
) -> DirectInstr {
    let specs = builtin_spec_set();
    let raw_word = memory[addr as usize & 0x0FFF] & 0x7FFF;
    let next_pc = addr.wrapping_add(1) & 0x7FFF;

    match decode(raw_word, extend, &specs) {
        Ok(decoded) => {
            let instr_type = decoded.spec.instr_type;
            let operand = decoded.address;
            let ops: Vec<SemOp> = decoded.spec.semantics.clone().unwrap_or_default();
            let bytecode = lower_sem(operand, &ops);
            let terminator = classify_terminator(
                instr_type,
                &ops,
                addr,
                next_pc,
                operand,
                memory,
                indirect_targets,
            );
            DirectInstr {
                addr,
                extend,
                raw_word,
                instr_type: Some(instr_type),
                operand,
                bytecode,
                terminator,
            }
        }
        Err(_) => DirectInstr {
            addr,
            extend,
            raw_word,
            instr_type: None,
            operand: 0,
            bytecode: alloc::vec![],
            terminator: Terminator::Halt,
        },
    }
}
