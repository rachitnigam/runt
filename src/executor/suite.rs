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

/// Wrapper struct to manage filtering paths in a [TestSuite].
struct PathStream<'a> {
    paths: Vec<PathBuf>,
    exclude: Option<&'a Regex>,
    include: Option<&'a Regex>,
}

impl<'a> PathStream<'a> {
    /// Remove paths that match the filter.
    /// Filter is matched against the string `suite_name:path`.
    pub fn with_exclude_filter(mut self, exclude: Option<&'a Regex>) -> Self {
        self.exclude = exclude;
        self
    }

    /// Include paths that match the filter.
    /// Filter is matched against the string `suite_name:path`.
    pub fn with_include_filter(mut self, include: Option<&'a Regex>) -> Self {
        self.include = include;
        self
    }

    /// Generate a collection of paths by running the include and exclude
    /// filters.
    pub fn into_paths(self, name: String) -> Vec<PathBuf> {
        let PathStream {
            paths,
            exclude,
            include,
        } = self;
        paths
            .into_iter()
            .filter(|p| {
                exclude
                    .map(|ex| {
                        !ex.is_match(
                            &(name.clone() + ":" + &p.to_string_lossy()),
                        )
                    })
                    .and_then(|accept| {
                        include.map(|inc| {
                            accept
                                && inc.is_match(
                                    &(name.clone()
                                        + ":"
                                        + &p.to_string_lossy()),
                                )
                        })
                    })
                    .unwrap_or(true)
            })
            .collect()
    }
}

impl From<Vec<PathBuf>> for PathStream<'_> {
    fn from(paths: Vec<PathBuf>) -> Self {
        Self {
            paths,
            exclude: None,
            include: None,
        }
    }
}
