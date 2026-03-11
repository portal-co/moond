//! Built-in ISA spec table with all instruction semantics pre-populated.

use alloc::vec;
use alloc::vec::Vec;

use crate::instr_type::InstrType;
use crate::sem_text::parse_sem;
use crate::semantics::SemOp;
use crate::spec::{AddrMode, InstrSpec, InstrSpecSet, OpcodeFormat};

/// Return the built-in ISA spec set with all semantics populated.
///
/// This is the authoritative fallback when no markdown directory is available.
pub fn builtin_spec_set() -> InstrSpecSet {
    use OpcodeFormat::*;

    macro_rules! spec {
        ($mn:expr, $ty:expr, $mode:expr, $fmt:expr, $opc:expr, $qtr:expr, $ext:expr, $desc:expr, $sem:expr) => {{
            let semantics: Option<Vec<SemOp>> = $sem
                .and_then(|text: &str| parse_sem(text).ok());
            InstrSpec {
                mnemonic: $mn.into(),
                instr_type: $ty,
                addr_mode: $mode,
                opcode_format: $fmt,
                opcode: $opc,
                quarter: $qtr,
                requires_extend: $ext,
                description: $desc.into(),
                source_file: None,
                semantics,
            }
        }};
    }

    let specs = vec![
        // ── Sequence changing ──────────────────────────────────────────────
        spec!("TC", InstrType::Tc, AddrMode::K, Whole3, 0o00, None, false,
              "Transfer Control to K",
              Some("set Q Z\nbranch operand")),

        spec!("TCF", InstrType::Tcf, AddrMode::F, Quarter5, 0o01, Some(2), false,
              "Transfer Control to Fixed F",
              Some("branch operand")),

        spec!("CCS", InstrType::Ccs, AddrMode::E, Quarter5, 0o01, Some(0), true,
              "Count, Compare, and Skip on E",
              Some("set A mem\n\
                    branch_if is_pos(A) pc_rel(0)\n\
                    branch_if is_plus_zero(A) pc_rel(1)\n\
                    branch_if is_neg(A) pc_rel(2)\n\
                    branch pc_rel(3)")),

        spec!("BZF", InstrType::Bzf, AddrMode::F, Quarter5, 0o16, Some(2), false,
              "Branch on Zero to Fixed F",
              Some("branch_if is_plus_zero(A) operand")),

        spec!("BZMF", InstrType::Bzmf, AddrMode::F, Quarter5, 0o12, Some(2), false,
              "Branch on Zero or Minus to Fixed F",
              Some("branch_if is_zero_or_neg(A) operand")),

        // ── Fetching and storing ───────────────────────────────────────────
        spec!("CA", InstrType::Ca, AddrMode::K, Whole3, 0o03, None, false,
              "Clear and Add K",
              Some("set A mem")),

        spec!("CS", InstrType::Cs, AddrMode::K, Whole3, 0o04, None, false,
              "Clear and Subtract K",
              Some("set A oc_neg(mem)")),

        spec!("DCA", InstrType::Dca, AddrMode::K, Quarter5, 0o13, None, true,
              "Double Clear and Add K",
              Some("set L mem\nset A mem_hi")),

        spec!("DCS", InstrType::Dcs, AddrMode::K, Quarter5, 0o14, None, true,
              "Double Clear and Subtract K",
              Some("set L oc_neg(mem)\nset A oc_neg(mem_hi)")),

        spec!("TS", InstrType::Ts, AddrMode::E, Quarter5, 0o05, Some(4), true,
              "Transfer to Storage E",
              Some("set tmp A\n\
                    set mem A\n\
                    set A sat(tmp)\n\
                    branch_if has_overflow(tmp) pc_rel(1)")),

        spec!("XCH", InstrType::Xch, AddrMode::E, Quarter5, 0o05, Some(5), true,
              "Exchange A and E",
              Some("set tmp mem\nset mem A\nset A tmp")),

        spec!("LXCH", InstrType::Lxch, AddrMode::E, Quarter5, 0o02, Some(2), true,
              "Exchange L and E",
              Some("set tmp mem\nset mem L\nset L tmp")),

        spec!("QXCH", InstrType::Qxch, AddrMode::E, Quarter5, 0o12, Some(2), true,
              "Exchange Q and E",
              Some("set tmp mem\nset mem Q\nset Q tmp")),

        spec!("DXCH", InstrType::Dxch, AddrMode::E, Quarter5, 0o05, Some(2), true,
              "Double Exchange A and E",
              Some("set tmp mem\n\
                    set mem A\n\
                    set A tmp\n\
                    set tmp mem_hi\n\
                    set mem_hi L\n\
                    set L tmp")),

        // ── Modifying ──────────────────────────────────────────────────────
        spec!("NDX", InstrType::Ndx, AddrMode::K, Quarter5, 0o05, Some(0), true,
              "Index next instruction",
              Some("set tmp mem\nset next_instr oc_add(deref(Z),tmp)")),

        // ── Arithmetic and logic ───────────────────────────────────────────
        spec!("AD", InstrType::Ad, AddrMode::K, Whole3, 0o06, None, false,
              "Add K to A",
              Some("set A oc_add(A,mem)")),

        spec!("SU", InstrType::Su, AddrMode::E, Quarter5, 0o16, Some(0), true,
              "Subtract E from A",
              Some("set A oc_sub(A,mem)")),

        spec!("MP", InstrType::Mp, AddrMode::K, Quarter5, 0o17, None, true,
              "Multiply A by K",
              Some("set L mul_lo(A,mem)\nset A mul_hi(A,mem)")),

        spec!("DV", InstrType::Dv, AddrMode::E, Quarter5, 0o11, Some(0), true,
              "Divide A,L by E",
              Some("set tmp A\nset A div_q(tmp,L,mem)\nset L div_r(tmp,L,mem)")),

        spec!("ADS", InstrType::Ads, AddrMode::E, Quarter5, 0o02, Some(6), true,
              "Add to Storage E",
              Some("set tmp mem\nset mem oc_add(A,tmp)\nset A mem")),

        spec!("DAS", InstrType::Das, AddrMode::E, Quarter5, 0o02, Some(0), true,
              "Double Add to Storage E",
              Some("set tmp mem\n\
                    set mem oc_add(L,tmp)\n\
                    set mem_hi dp_add_hi(A,L,mem_hi,tmp)\n\
                    set A 0")),

        spec!("INCR", InstrType::Incr, AddrMode::E, Quarter5, 0o02, Some(4), true,
              "Increment E",
              Some("set mem oc_add(mem,1)")),

        spec!("AUG", InstrType::Aug, AddrMode::E, Quarter5, 0o12, Some(4), true,
              "Augment E toward ±max",
              Some("set mem aug(mem)")),

        spec!("DIM", InstrType::Dim, AddrMode::E, Quarter5, 0o12, Some(6), true,
              "Diminish E toward zero",
              Some("set mem dim(mem)")),

        spec!("MSU", InstrType::Msu, AddrMode::E, Quarter5, 0o12, Some(0), true,
              "Modular Subtract E from A",
              Some("set A oc_sub(A,mem)")),

        spec!("MSK", InstrType::Msk, AddrMode::K, Whole3, 0o07, None, false,
              "Mask A with K",
              Some("set A and(A,mem)")),

        // ── Channel I/O ────────────────────────────────────────────────────
        spec!("READ",  InstrType::Read,  AddrMode::H, Channel6, 0o10, Some(0), true,
              "Read channel H into A",
              Some("set A chan")),

        spec!("WRITE", InstrType::Write, AddrMode::H, Channel6, 0o10, Some(1), true,
              "Write A to channel H",
              Some("set chan A")),

        spec!("RAND",  InstrType::Rand,  AddrMode::H, Channel6, 0o10, Some(2), true,
              "Read channel AND A → A",
              Some("set A and(A,chan)")),

        spec!("WAND",  InstrType::Wand,  AddrMode::H, Channel6, 0o10, Some(3), true,
              "Write A AND channel → channel",
              Some("set chan and(A,chan)")),

        spec!("ROR",   InstrType::Ror,   AddrMode::H, Channel6, 0o10, Some(4), true,
              "Read channel OR A → A",
              Some("set A or(A,chan)")),

        spec!("WOR",   InstrType::Wor,   AddrMode::H, Channel6, 0o10, Some(5), true,
              "Write A OR channel → channel",
              Some("set chan or(A,chan)")),

        spec!("RXOR",  InstrType::Rxor,  AddrMode::H, Channel6, 0o10, Some(6), true,
              "Read channel XOR A → A",
              Some("set A xor(A,chan)")),

        // ── Special ────────────────────────────────────────────────────────
        spec!("EXTEND", InstrType::Extend, AddrMode::None, Special, 0o00, None, false,
              "Enable extracode for next instruction",
              Some("set_flag extend")),

        spec!("INHINT", InstrType::Inhint, AddrMode::None, Special, 0o00, None, false,
              "Inhibit interrupt",
              Some("set_flag inhint")),

        spec!("RELINT", InstrType::Relint, AddrMode::None, Special, 0o00, None, false,
              "Release interrupt inhibit",
              Some("clear_flag inhint")),

        spec!("RESUME", InstrType::Resume, AddrMode::None, Special, 0o05, Some(0), false,
              "Resume interrupted program",
              Some("branch mem_at(0o16)")),

        spec!("GO", InstrType::Go, AddrMode::None, Special, 0o00, None, false,
              "Restart at 04000",
              Some("branch 0o4000")),

        spec!("RUPT", InstrType::Rupt, AddrMode::None, Special, 0o10, None, false,
              "Interrupt (save state, vector to handler)",
              Some("set mem_at(0o17) deref(Z)\n\
                    set mem_at(0o16) Z\n\
                    branch mem_at(0o4)")),

        // ── Counter (hardware-triggered) ───────────────────────────────────
        spec!("PINC",  InstrType::Pinc,  AddrMode::C, Hardware, 0, None, false,
              "Plus Increment Counter C",
              Some("set mem oc_add(mem,1)")),

        spec!("MINC",  InstrType::Minc,  AddrMode::C, Hardware, 0, None, false,
              "Minus Increment Counter C",
              Some("set mem oc_sub(mem,1)")),

        spec!("DINC",  InstrType::Dinc,  AddrMode::C, Hardware, 0, None, false,
              "Diminish Increment Counter C",
              Some("set mem dim(mem)")),

        spec!("PCDU",  InstrType::Pcdu,  AddrMode::C, Hardware, 0, None, false,
              "Plus Counter Down-Up C",
              Some("set mem oc_add(mem,1)")),

        spec!("MCDU",  InstrType::Mcdu,  AddrMode::C, Hardware, 0, None, false,
              "Minus Counter Down-Up C",
              Some("set mem oc_sub(mem,1)")),

        spec!("SHINC", InstrType::Shinc, AddrMode::C, Hardware, 0, None, false,
              "Shift Increment C",
              Some("set mem oc_add(mem,1)")),

        spec!("SHANC", InstrType::Shanc, AddrMode::C, Hardware, 0, None, false,
              "Shift and Add Increment C",
              Some("set mem oc_add(mem,1)")),

        // ── Peripheral / GSE ───────────────────────────────────────────────
        spec!("TCSAJ",  InstrType::Tcsaj,  AddrMode::K, Whole3,   0o00, None, false,
              "Transfer Control to Specified Address K",
              None::<&str>),

        spec!("FETCH",  InstrType::Fetch,  AddrMode::K, Hardware, 0, None, false,
              "Fetch K (display on GSE)",
              None::<&str>),

        spec!("STORE",  InstrType::Store,  AddrMode::E, Hardware, 0, None, true,
              "Store E (load from GSE)",
              None::<&str>),

        spec!("INOTRD", InstrType::Inotrd, AddrMode::H, Hardware, 0, None, false,
              "I/O Not Read H (display on GSE)",
              None::<&str>),

        spec!("INOTLD", InstrType::Inotld, AddrMode::H, Hardware, 0, None, false,
              "I/O Not Load H (load from GSE)",
              None::<&str>),
    ];

    InstrSpecSet::new(specs)
}
