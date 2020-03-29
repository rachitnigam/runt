use crate::errors;
use std::path::PathBuf;
use structopt::StructOpt;

/// Options for the CLI.
#[derive(StructOpt, Debug)]
#[structopt(name = "runt", about = "Lightweight snapshot testing.")]
pub struct Opts {
    /// Test folder.
    #[structopt(name = "TEST_DIR", parse(from_os_str))]
    pub dir: PathBuf,

    /// Show diffs for each failing test.
    #[structopt(short, long)]
    pub diff: bool,

    /// Update expect files for displayed tests. Use the --only flag to save
    /// failing or missing tests.
    #[structopt(short, long)]
    pub save: bool,

    /// Only display tests from a specific class.
    #[structopt(short, long)]
    pub only: Option<OnlyOpt>,
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
                "Must be one of fail, pass, missing.".to_string(),
            )),
        }
    }
}
