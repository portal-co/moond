//! Backend abstraction for the AGC recompiler.

pub mod c;

use crate::ir::InstrStream;

/// A backend converts an [`InstrStream`] into a target representation.
///
/// The associated types allow backends to return whatever output format they
/// need (e.g. `String` for source-code backends, `Vec<u8>` for binary backends).
pub trait Backend {
    type Output;
    type Error: core::fmt::Display;

    fn emit(&self, stream: &InstrStream) -> Result<Self::Output, Self::Error>;
}
