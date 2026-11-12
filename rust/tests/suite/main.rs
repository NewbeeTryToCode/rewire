//! Port of tests/test_rewire.py: the Rust binary must behave like the Python reference.
//!
//! The suite is split by subject, one module per area, with the fixtures they share in
//! [`support`]. A test belongs in the module named after the behaviour it pins.

mod cli;
mod consolidation;
mod nodes;
mod recall;
mod security;
mod snapshot;
mod support;
mod sync;
mod transcripts;
mod trust;
