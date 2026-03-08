//! # agc-interp — AGC Block-2 Interpreter
//!
//! A data-driven interpreter for the Apollo Guidance Computer Block-2 ISA.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                     InstrSpecSet                        │
//! │  ┌──────────────┐   ┌────────────────────────────────┐  │
//! │  │  InstrSpec   │   │       InstrSemantics           │  │
//! │  │  · mnemonic  │   │  (Vec<SemOp>)                  │  │
//! │  │  · InstrType │   │  Parsed from text DSL or       │  │
//! │  │  · AddrMode  │   │  loaded from ref/block2/ .md   │  │
//! │  │  · OpcodeFormat│  │  files via SpecParser          │  │
//! │  │  · opcode/qtr│   └────────────────────────────────┘  │
//! │  └──────────────┘                                       │
//! └───────────────────────────────┬─────────────────────────┘
//!                                 │ drives
//!       ┌──────────┐    ┌─────────▼──────────┐
//!       │   Cpu    │◄───│    Interpreter     │
//!       │ registers│    │ fetch → decode →   │
//!       │ memory   │    │ eval SemOp         │
//!       │ channels │    └────────────────────┘
//!       └──────────┘
//! ```
//!
//! ## Quick start
//!
//! ```no_run
//! use agc_interp::{Interpreter, InstrSpecSet};
//! use agc_interp::spec_parser::SpecParser;
//!
//! // Option A: built-in specs (no files needed)
//! let mut interp = Interpreter::with_builtin_specs();
//!
//! // Option B: load from reference docs
//! let specs = SpecParser::load("ref/block2").unwrap();
//! let mut interp = Interpreter::new(specs);
//!
//! // Load a program: CA 0o100 at address 0o4000
//! interp.cpu_mut().mem_write(0o4000, 0o_030100); // CA opcode=3, addr=0o100
//! interp.cpu_mut().mem_write(0o100, 42);
//!
//! interp.step().unwrap();
//! assert_eq!(interp.cpu().a, 42);
//! ```
//!
//! ## Instruction semantics DSL
//!
//! Each instruction's behaviour is stored as a `Vec<SemOp>` and can be
//! round-tripped through a compact text DSL:
//!
//! ```
//! use agc_interp::semantics::{parse_sem, sem_to_text};
//!
//! let tc_sem = "set Q Z\nbranch operand";
//! let ops = parse_sem(tc_sem).unwrap();
//! assert_eq!(sem_to_text(&ops), tc_sem);
//! ```

pub mod cpu;
pub mod decode;
pub mod exec;
pub mod interp;
pub mod semantics;
pub mod spec;
pub mod spec_parser;

// ── Re-exports ─────────────────────────────────────────────────────────────────

pub use cpu::Cpu;
pub use interp::Interpreter;
pub use spec::{AddrMode, InstrSpec, InstrSpecSet, InstrType, OpcodeFormat};
pub use semantics::{Dest, Expr, SemOp, Cond, Flag, parse_sem, sem_to_text};
