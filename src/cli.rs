//! Options of the command line interface.
use crate::errors;
use argh::FromArgs;
use std::path::{Path, PathBuf};

#[derive(FromArgs)]
/// Lightweight snapshot testing framework
pub struct Opts {
    /// test folder.
    #[argh(
        positional,
        from_str_fn(read_path),
        default = "Path::new(\".\").into()"
    )]
    pub dir: PathBuf,

    /// show diffs for each failing test.
    #[argh(switch, short = 'd')]
    pub diff: bool,

    /// update expect files for displayed tests. Use the --only flag to save
    /// failing or missing tests.
    #[argh(switch, short = 's')]
    pub save: bool,

    /// print out the commands to be run for each test case.
    /// Warning: Will probably generate a lot of text unless used with
    /// --include or --exclude
    #[argh(switch, short = 'n')]
    pub dry_run: bool,

    /// enable verbose printing
    #[argh(switch, short = 'v')]
    pub verbose: bool,

    /// also run tests which are normally skipped with .skip files
    #[argh(switch)]
    pub ignore_skip: bool,

    /// filter out the reported test results based on test status
    /// ("pass", "fail", "miss") or a regex for the test file path.
    /// Applied after running the tests.
    #[argh(option, short = 'o', long = "only")]
    pub post_filter: Option<OnlyOpt>,

    /// exclude matching tests using a regex on "<suite-name>:<path>" strings
    /// Applied before running tests.
    #[argh(option, short = 'x', long = "exclude")]
    pub exclude_filter: Option<String>,

    /// include matching tests using a regex on "<suite-name>:<path>" strings
    /// Applied before running tests.
    #[argh(option, short = 'i', long = "include")]
    pub include_filter: Option<String>,

    /// limit the number of jobs to run in parallel. Defaults to number of logical
    /// cpus.
    #[argh(option, short = 'j', long = "jobs")]
    pub jobs_limit: Option<usize>,

    /// maximum number of features that can be created for concurrent processing.
    /// Use a lower number if runt gives the "too many file handles" error.
    /// Defaults to 50.
    #[argh(option, long = "max-futures")]
    pub max_futures: Option<usize>,

    /// print the version of runt
    #[argh(switch, short = 'V')]
    pub version: bool,
}

fn read_path(path: &str) -> Result<PathBuf, String> {
    Ok(Path::new(path).into())
}

/// Possible values for the --only flag.
#[derive(Debug, PartialEq, Eq)]
pub enum OnlyOpt {
    /// Failing tests.
    Fail,
    /// Passing tests.
    Pass,
    /// Tests missing expect files.
    Missing,
}

impl std::str::FromStr for OnlyOpt {
    type Err = errors::RuntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fail" => Ok(OnlyOpt::Fail),
            "pass" => Ok(OnlyOpt::Pass),
            "miss" => Ok(OnlyOpt::Missing),
            _ => Err(errors::RuntError(
                "Unknown --only filter. Expected: fail, pass, miss".to_string(),
            )),
        }
    }
}
