//! # agc-isa — AGC Block-2 ISA specification (no_std)
//!
//! Contains ISA spec types, semantics DSL types, text parser, and builtin spec table.
//! Designed for use in no_std environments (embedded, codegen, etc.).

#![no_std]
extern crate alloc;

pub mod instr_type;
pub mod semantics;
pub mod spec;
pub mod sem_text;
pub mod builtin;

// Re-exports
pub use instr_type::InstrType;
pub use semantics::{Cond, Dest, Expr, Flag, SemOp};
pub use spec::{AddrMode, InstrSpec, InstrSpecSet, OpcodeFormat};
pub use sem_text::{parse_sem, sem_to_text};
pub use builtin::builtin_spec_set;
