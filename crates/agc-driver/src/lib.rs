//! # agc-driver — File I/O driver for loading AGC ISA specs from markdown.
//!
//! Reads `.md` files from `ref/block2/`, looks for `## Semantics` sections
//! with ` ```agc-sem` code blocks, and overlays parsed semantics onto the
//! built-in spec set.

pub mod loader;

pub use loader::{load_from_dir, LoadError};
