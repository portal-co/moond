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
//!   InstrStream  (ir.rs)       DirectFunctionPlan + DirectInstr stream
//!        │   BTreeMap<u16, BasicBlock>     │  requested/reachable (addr, extend) pairs
//!        ▼                                     ▼
//!   Backend trait  (backend/mod.rs)    DirectBackend trait (backend/mod.rs)
//!        │                                     │
//!        └─► C backend  (backend/c.rs)         └─► WASM/yecta (backend/wasm.rs)
//! ```

extern crate alloc;

pub mod ir;
pub mod frontend;
pub mod backend;
pub mod slicer;

pub use ir::{BasicBlock, InstrRecord, InstrStream, Terminator};
pub use frontend::{
    DirectFunctionPlan, FrontendError, decode_direct, decode_stream, plan_direct_functions,
};
pub use backend::{Backend, DirectBackend, DirectFunctionKey, DirectInstr};

/// Feed exactly a previously discovered direct-function closure.
///
/// The backend sees the complete selected key set before the first body, so it
/// can reserve compact forward call indices. No instruction body outside the
/// plan is decoded or lowered here.
pub fn feed_direct_plan<B, Context>(
    backend: &mut B,
    context: &mut Context,
    memory: &[u16; 4096],
    plan: &DirectFunctionPlan,
) -> Result<(), B::Error>
where
    B: DirectBackend<Context>,
{
    backend.prepare(&plan.functions)?;
    for key in &plan.functions {
        let instruction = decode_direct(memory, key.addr, key.extend, &plan.indirect_targets);
        backend.feed_instr(context, &instruction)?;
    }
    Ok(())
}
