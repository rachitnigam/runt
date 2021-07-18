use crate::errors;
use std::path::PathBuf;
use structopt::StructOpt;

/// Options for the CLI.
#[derive(StructOpt, Debug)]
#[structopt(name = "runt")]
#[structopt(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = "Lightweight snapshot testing.",
)]
pub struct Opts {
    /// Test folder.
    #[structopt(name = "TEST_DIR", parse(from_os_str), default_value = ".")]
    pub dir: PathBuf,

    /// Show diffs for each failing test.
    #[structopt(short, long)]
    pub diff: bool,

    /// Update expect files for displayed tests. Use the --only flag to save
    /// failing or missing tests.
    #[structopt(short, long)]
    pub save: bool,

    /// Print out the commands to be run for each test case.
    /// Warning: Will probably generate a lot of text unless used with
    /// --include or --exclude
    #[structopt(short = "n", long)]
    pub dry_run: bool,

    /// Enable verbose printing
    #[structopt(short = "v", long)]
    pub verbose: bool,

    /// Filter out the reported test results based on test status
    /// ("pass", "fail", "miss") or a regex for the test file path.
    /// Applied after running the tests.
    #[structopt(short = "o", long = "only")]
    pub post_filter: Option<OnlyOpt>,

    /// Exclude matching tests using a regex on "<suite-name>:<path>" strings
    /// Applied before running tests.
    #[structopt(short = "x", long = "exclude")]
    pub exclude_filter: Option<String>,

    /// Include matching tests using a regex on "<suite-name>:<path>" strings
    /// Applied before running tests.
    #[structopt(short = "i", long = "include")]
    pub include_filter: Option<String>,

    /// Limit the number of jobs to run in parallel. Defaults to number of logical
    /// cpus.
    #[structopt(short = "j", long = "jobs")]
    pub jobs_limit: Option<usize>,

    /// Maximum number of features that can be created for concurrent processing.
    /// Use a lower number if runt gives the "too many file handles" error.
    /// Defaults to 50.
    #[structopt(long = "max-futures")]
    pub max_futures: Option<usize>,
}

/// Possible values for the --only flag.
#[derive(Debug)]
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
