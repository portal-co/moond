//! # agc-lower — AGC Block-2 → TC2 16-bit stack bytecode lowering
//!
//! Translates AGC [`SemOp`] sequences into a compact TC2 (two's-complement)
//! 16-bit stack bytecode that can be executed by [`vm::BytecodeVm`].
//!
//! ## VM Architecture
//!
//! - Memory: 65 536 × 16-bit words (16-bit-byte, ready for memory tagging).
//! - Registers: `T` (main temporary), `OFF` (immediate offset / operand).
//! - AGC registers identity-mapped to VM memory (AGC addr K → VM addr K).
//! - Non-addressable AGC `tmp` → `0xFF00`, flags at `0xFF01`/`0xFF02`.
//! - I/O channels mapped to `0x8000 + channel_number`.
//! - Stack: unbounded operand stack (separate from VM memory).
//!
//! ## OC → TC2 Lowering
//!
//! Ones-complement operations (`OcAdd`, `OcSub`, `OcNeg`, `OcAug`, `OcDim`,
//! `OcSat`, `MulHi`, `MulLo`, `DivQ`, `DivR`, `DpAddHi`) are emitted as
//! dedicated VM opcodes that call the same OC arithmetic routines used by the
//! reference interpreter, ensuring bit-exact results.

extern crate alloc;

pub mod bytecode;
pub mod lower;
pub mod vm;

pub use lower::lower_sem;
pub use vm::BytecodeVm;
