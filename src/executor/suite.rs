use std::{path::PathBuf, time::Duration};

use regex::Regex;

/// Type for mapping test suite objects.
pub type Id = u64;

pub struct Config {
    /// Name of this TestSuite
    pub name: String,
    /// Command to execute. The pattern `{}` in this string is replaced with
    /// the matching path.
    pub cmd: String,
    /// Optional directory to store the generated .expect files.
    pub expect_dir: Option<PathBuf>,
    /// Optional timeout for the tests specified in seconds.
    /// Defaults to 120 seconds.
    pub timeout: Duration,
}

/// Defines a test suite which is a collection of test paths, command, and other
/// configurations.
pub struct Suite {
    /// Paths of input files.
    pub paths: Vec<PathBuf>,
    /// Configuration for the [TestSuite].
    pub config: Config,
}

impl Suite {
    pub fn with_filters<'a>(
        mut self,
        include: Option<&'a Regex>,
        exclude: Option<&'a Regex>,
    ) -> Self {
        let name = &self.config.name;
        self.paths.retain(|path| {
            let path_str = path.to_string_lossy().clone();
            path_str.to_string().insert_str(0, name);
            include.map(|incl| incl.is_match(&path_str)).unwrap_or(true)
                && exclude
                    .map(|excl| !excl.is_match(&path_str))
                    .unwrap_or(true)
        });
        self
    }
}
