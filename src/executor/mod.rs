//! An executor is responsible for executing the test configurations and generating results.

pub mod results;
pub mod suite;
mod test;
mod context;

pub use test::Test;
pub use context::Context;
