//! An executor is responsible for executing the test configurations and generating results.

mod context;
pub mod results;
pub mod suite;
mod test;

pub use context::Context;
pub use test::Test;
