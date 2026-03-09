//! # agc-recompile — AGC Block-2 → C / WASM recompiler
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
//!   InstrStream  (ir.rs)           DirectInstr stream (frontend::decode_direct)
//!        │   BTreeMap<u16, BasicBlock>         │  one per (addr, extend) pair
//!        ▼                                     ▼
//!   Backend trait  (backend/mod.rs)    DirectBackend trait (backend/mod.rs)
//!        │                                     │
//!        └─► C backend  (backend/c.rs)         └─► WASM/yecta (backend/wasm.rs)
//! ```

extern crate alloc;

pub mod ir;
pub mod frontend;
pub mod backend;

pub use ir::{BasicBlock, InstrRecord, InstrStream, Terminator};
pub use frontend::{FrontendError, decode_stream, decode_direct};
pub use backend::{Backend, DirectBackend, DirectInstr};
