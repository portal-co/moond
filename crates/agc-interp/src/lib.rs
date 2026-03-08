//! # agc-interp — AGC Block-2 Interpreter (no_std)
//!
//! A data-driven interpreter for the Apollo Guidance Computer Block-2 ISA.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                     InstrSpecSet                        │
//! │  ┌──────────────┐   ┌────────────────────────────────┐  │
//! │  │  InstrSpec   │   │       Vec<SemOp>               │  │
//! │  │  · mnemonic  │   │  Parsed from text DSL or       │  │
//! │  │  · InstrType │   │  loaded from ref/block2/ .md   │  │
//! │  │  · AddrMode  │   └────────────────────────────────┘  │
//! │  │  · OpcodeFormat│                                     │
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

#![no_std]
extern crate alloc;

pub mod cpu;
pub mod decode;
pub mod exec;
pub mod interp;

// Re-exports from agc-isa
pub use agc_isa::{
    AddrMode, InstrSpec, InstrSpecSet, InstrType, OpcodeFormat,
    Cond, Dest, Expr, Flag, SemOp,
    parse_sem, sem_to_text, builtin_spec_set,
};

// Local re-exports
pub use cpu::Cpu;
pub use interp::Interpreter;
