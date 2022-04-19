//! The default picker for runt test suites that gathers tests to run from a
//! runt.toml file.
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{errors, executor::suite};

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
    /// Optional timeout
    pub timeout: Option<u64>,
}

impl Config {
    /// Create a configuration by reading a `runt.toml` file.
    /// Ensures that the version number specified in the `runt.toml` matches
    /// the version of the installed `runt` binary.
    pub fn from_path(conf_dir: &Path) -> Result<Self, errors::RuntError> {
        // Error if runt.toml doesn't exist.
        let conf_path = conf_dir.join("runt.toml");
        let contents = &std::fs::read_to_string(&conf_path).map_err(|_| {
            errors::RuntError(format!(
            "{} is missing. Runt expects a directory with a runt.toml file.",
            conf_path.to_str().unwrap()
        ))
        })?;

        let conf: Config = toml::from_str(contents).map_err(|err| {
            errors::RuntError(format!(
                "Failed to parse {}: {}",
                conf_path.to_str().unwrap(),
                err
            ))
        })?;

        // Check if the current `runt` matches the version specified in
        // the configuration.
        if env!("CARGO_PKG_VERSION") != conf.ver {
            return Err(errors::RuntError(format!("Runt version mismatch. Configuration requires: {}, tool version: {}.\nRun `cargo install runt` to get the latest version of runt.", conf.ver, env!("CARGO_PKG_VERSION"))));
        }

        Ok(conf)
    }
}

impl From<SuiteConfig> for suite::Suite {
    /// Transform a list of glob patterns into matching paths and
    /// list of errors.
    fn from(conf: SuiteConfig) -> Self {
        // Arcane sorcery
        let all_paths = conf
            .paths
            .into_iter()
            .map(|pattern| glob::glob(&pattern))
            .collect::<Result<Vec<_>, glob::PatternError>>()
            .expect("Glob pattern error")
            .into_iter()
            .flat_map(|paths| paths.collect::<Vec<_>>())
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to read globbed path");

        suite::Suite {
            paths: all_paths,
            config: suite::Config {
                name: conf.name,
                cmd: conf.cmd,
                expect_dir: conf.expect_dir,
                timeout: Duration::from_secs(conf.timeout.unwrap_or(1200)),
            },
        }
    }
}
