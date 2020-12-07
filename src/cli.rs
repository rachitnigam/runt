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

    #[structopt(short = "j", long = "jobs")]
    pub jobs_limit: Option<usize>,
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
