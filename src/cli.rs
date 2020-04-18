use crate::errors;
use regex::Regex;
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

    /// Post filters for test status.
    #[structopt(short="o", long="only,status")]
    pub post_filter: Option<OnlyOpt>,

    /// Exclude matching tests.
    #[structopt(short="x", long="exclude")]
    pub exclude_filter: Regex,

    /// Include matching tests.
    #[structopt(short="i", long="include")]
    pub include_filter: Regex,
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
    /// Filter tests to run by matching a regex.
    Matches(Regex),
}

impl std::str::FromStr for OnlyOpt {
    type Err = errors::RuntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fail" => Ok(OnlyOpt::Fail),
            "pass" => Ok(OnlyOpt::Pass),
            "miss" => Ok(OnlyOpt::Missing),
            matches => Regex::new(matches)
                .map_err(|err| errors::RuntError(err.to_string()))
                .map(|reg| OnlyOpt::Matches(reg)),
        }
    }
}
