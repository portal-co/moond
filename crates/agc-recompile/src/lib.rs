//! # agc-recompile — AGC Block-2 → C recompiler
//!
//! ## Architecture
//!
//! ```text
//! AGC binary + entry points + indirect targets
//!        │
//!        ▼
//!   Frontend  (frontend.rs)
//!        │   recursive-descent decode, EXTEND-state tracking
//!        │   lowers each instruction to TC2 bytecode via agc-lower
//!        ▼
//!   InstrStream  (ir.rs)
//!        │   BTreeMap<u16, BasicBlock>  with Terminator annotations
//!        ▼
//!   Backend trait  (backend/mod.rs)
//!        │
//!        ├─► C backend  (backend/c.rs)   → C source string
//!        └─► (future backends share the same frontend)
//! ```
//!
//! ## Indirect branch targets
//!
//! The caller must supply a `BTreeSet<u16>` of all addresses that may be
//! reached via an indirect branch at runtime (e.g. TC targets stored in
//! erasable memory, NDX operands).  The frontend uses this set to:
//! 1. Seed additional decode roots so those blocks are included in the IR.
//! 2. Record the set on `InstrStream` so the C backend can emit a `switch`
//!    dispatch over the full reachable address space.

extern crate alloc;

pub mod ir;
pub mod frontend;
pub mod backend;

pub use ir::{BasicBlock, InstrRecord, InstrStream, Terminator};
pub use frontend::{FrontendError, decode_stream};
pub use backend::Backend;
