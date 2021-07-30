//! Module to generate diffs when the test result and the contents of the
//! expect file do not match.
mod diff;

pub use diff::gen_diff;
