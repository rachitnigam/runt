use crate::test_suite::TestSuite;
use serde::Deserialize;
use std::path::PathBuf;

/// Configuration for a single runt run.
/// Tests suites for this runt configuration
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Version of the runt tool this configuration is compatible with.
    pub ver: String,
    /// Test suite configurations.
    pub tests: Vec<SuiteConfig>,
}

/// Configuration for a test suite.
#[derive(Debug, Deserialize)]
pub struct SuiteConfig {
    /// Name of this TestSuite
    pub name: String,
    /// Paths of input files.
    pub paths: Vec<String>,
    /// Command to execute. The pattern `{}` in this string is replaced with
    /// the matching path.
    pub cmd: String,
    /// Optional directory to store the generated .expect files.
    pub expect_dir: Option<PathBuf>,
}

impl From<SuiteConfig> for TestSuite {
    /// Transform a list of glob patterns into matching paths and
    /// list of errors.
    fn from(conf: SuiteConfig) -> Self {
        // Arcane sorcery
        let all_paths = conf.paths
            .into_iter()
            .map(|pattern| glob::glob(&pattern))
            .collect::<Result<Vec<_>, glob::PatternError>>()
            .expect("Glob pattern error")
            .into_iter()
            .flat_map(|paths| paths.collect::<Vec<_>>())
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to read globbed path");

        TestSuite {
            name: conf.name,
            paths: all_paths,
            cmd: conf.cmd,
            expect_dir: conf.expect_dir,
        }
    }
}
