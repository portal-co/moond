//! Backend abstraction for the AGC recompiler.
//!
//! ## Two backend families
//!
//! ### Block-based (`Backend`)
//! Consumes a fully-decoded [`InstrStream`] (basic-block IR with liveness
//! already computed by the frontend).  The C backend lives here.
//!
//! ### Direct (`DirectBackend`)
//! Receives pre-parsed instructions one-at-a-time in a fixed order: all 4096
//! AGC addresses × 2 EXTEND states.  The yecta WASM backend lives here.
//!
//! `DirectBackend<Context>` is generic over a user-supplied context type that
//! is threaded through every call.  The default `Context = ()` preserves
//! backward compatibility.

pub mod c;
pub mod wasm;

use alloc::vec::Vec;

use agc_isa::InstrType;

use crate::ir::{InstrStream, Terminator};

// ─── Block-based backend ──────────────────────────────────────────────────────

/// A backend that consumes a fully-decoded [`InstrStream`].
pub trait Backend {
    type Output;
    type Error: core::fmt::Display;

    fn emit(&mut self, stream: &InstrStream) -> Result<Self::Output, Self::Error>;
}

// ─── Direct backend ───────────────────────────────────────────────────────────

/// One pre-parsed AGC instruction in a specific EXTEND state, ready for a
/// direct backend to consume without further decoding.
///
/// Instructions must be fed to a [`DirectBackend`] in strict address order:
/// `(addr=0, ext=false)`, `(addr=0, ext=true)`, `(addr=1, ext=false)`, …
#[derive(Clone)]
pub struct DirectInstr {
    /// AGC 12-bit address.
    pub addr: u16,
    /// EXTEND state when this instruction was decoded.
    pub extend: bool,
    /// Raw 15-bit word at this address (used for the INSTR_WORD register).
    pub raw_word: u16,
    /// Decoded instruction type; `None` if the word could not be decoded in
    /// this EXTEND state.
    pub instr_type: Option<InstrType>,
    /// Effective operand / address field extracted from the instruction word.
    pub operand: u16,
    /// Pre-lowered TC2 16-bit stack bytecode for this instruction.
    pub bytecode: Vec<u16>,
    /// How control leaves this instruction.
    pub terminator: Terminator,
}

/// A streaming direct backend generic over a user context type.
///
/// Instructions MUST be fed in the order described on [`DirectInstr`].
/// After all 4096 × 2 = 8192 instructions have been fed, call [`finish`] to
/// obtain the compiled output.
///
/// [`finish`]: DirectBackend::finish
///
/// # Type parameter
///
/// `Context` is an arbitrary user-defined value threaded through every call
/// (default `()`).  Architecture recompilers use it to pass trap-callback
/// state; the default unit context requires no changes at call sites that
/// already pass `&mut ()`.
pub trait DirectBackend<Context = ()> {
    type Output;
    type Error: core::fmt::Display;

    fn feed_instr(
        &mut self,
        ctx: &mut Context,
        instr: &DirectInstr,
    ) -> Result<(), Self::Error>;

    fn finish(self, ctx: &mut Context) -> Result<Self::Output, Self::Error>;
}
