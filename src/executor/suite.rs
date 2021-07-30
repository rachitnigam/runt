//! A Runt test suite configuration.
use std::{path::PathBuf, time::Duration};

use regex::Regex;

/// Type for mapping test suite objects.
pub type Id = u64;

/// Configuration for a test suite.
pub struct Config {
    /// Name of this TestSuite
    pub name: String,
    /// Command to execute. The pattern `{}` in this string is replaced with
    /// the matching path.
    pub cmd: String,
    /// Optional directory to store the generated .expect files.
    pub expect_dir: Option<PathBuf>,
    /// Optional timeout for the tests specified in seconds.
    /// Defaults to 1200 seconds.
    pub timeout: Duration,
}

/// Defines a test suite which is a collection of test paths, command, and other
/// configurations.
pub struct Suite {
    /// Paths of input files.
    pub paths: Vec<PathBuf>,
    pub config: Config,
}

impl Suite {
    /// Filters the tests in this test suite using the regexes.
    /// Matches the regexes against `<suite-name>:<test-name>`.
    pub fn with_filters<'a>(
        mut self,
        include: Option<&'a Regex>,
        exclude: Option<&'a Regex>,
    ) -> Self {
        let name = self.config.name.clone() + ":";
        self.paths.retain(|path| {
            let path_str = name.clone() + &path.to_string_lossy();
            include.map(|incl| incl.is_match(&path_str)).unwrap_or(true)
                && exclude
                    .map(|excl| !excl.is_match(&path_str))
                    .unwrap_or(true)
        });
        self
    }
}
