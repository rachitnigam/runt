use crate::{cli, errors::RuntError, printer};
use std::path::PathBuf;
use tokio::fs;

use super::suite;

/// Track the state of TestResult.
#[derive(Debug, PartialEq)]
pub enum State {
    /// The test timed out.
    Timeout,
    /// The comparison succeeded.
    Correct,
    /// The .expect file is missing. Contains the generated expectation string.
    Missing(String),
    /// The comparison failed. Contains the the generated expectation string
    /// and the contents of the expect file.
    Mismatch(
        String, // Generated expect string.
        String, // Contents of the expect file.
    ),
}

/// Store information related to one test.
#[derive(Debug)]
pub struct Test {
    /// Path of the test
    pub path: PathBuf,
    /// Location of the expect string.
    pub expect_path: PathBuf,
    /// Result of comparison
    pub state: State,
    /// The results of this structure were saved.
    pub saved: bool,
    /// Id for the test suite that owns this test.
    pub test_suite: suite::Id,
}

impl Test {
    /// Save the results of the test suite into the expect file.
    pub async fn save_results(&mut self) -> Result<(), RuntError> {
        match &self.state {
            State::Correct | State::Timeout => Ok(()),
            State::Missing(expect) | State::Mismatch(expect, _) => {
                self.saved = true;
                fs::write(&self.expect_path, expect).await.map_err(|err| {
                    RuntError(format!(
                        "{}: {}.",
                        self.expect_path.to_str().unwrap(),
                        err
                    ))
                })
            }
        }
    }

    fn with_only_opt(&self, only: &cli::OnlyOpt) -> bool {
        use cli::OnlyOpt as O;
        match (only, &self.state) {
            (O::Fail, State::Mismatch(..)) => true,
            (O::Pass, State::Correct) => true,
            (O::Missing, State::Missing(..)) => true,
            (O::Fail, _) | (O::Pass, _) | (O::Missing, _) => false,
        }
    }

    pub fn should_save(&self, opts: &cli::Opts) -> bool {
        if !opts.save {
            return false;
        }

        if let Some(only) = &opts.post_filter {
            return self.with_only_opt(only);
        }

        true
    }

    /// Returns true if this test should be printed with the current options.
    pub fn should_print(&self, opts: &cli::Opts) -> bool {
        // Print everything if verbose mode is enabled
        if opts.verbose {
            return true;
        }

        // Selectively print things if post_filter is enabled.
        if let Some(only) = &opts.post_filter {
            return self.with_only_opt(only);
        }
        // Otherwise just print failing and missing tests
        !matches!(self.state, State::Correct)
    }

    /// Generate colorized string to report the results of this test.
    pub fn report_str(
        &self,
        suite: Option<&String>,
        show_diff: bool,
    ) -> String {
        use colored::*;

        let mut buf = String::new();
        let path_str = self.path.to_str().unwrap();
        match &self.state {
            State::Missing(expect_string) => {
                buf.push_str(&"? ".yellow().to_string());
                suite.into_iter().for_each(|suite_name| {
                    buf.push_str(&suite_name.bold().yellow().to_string());
                    buf.push_str(&":".yellow().to_string())
                });
                buf.push_str(&path_str.yellow().to_string());
                if self.saved {
                    buf.push_str(&" (saved)".dimmed().to_string());
                }
                if show_diff {
                    let diff =
                        printer::gen_diff(&"".to_string(), &expect_string);
                    buf.push('\n');
                    buf.push_str(&diff);
                }
            }
            State::Timeout => {
                buf.push_str(&"✗ ".red().to_string());
                suite.into_iter().for_each(|suite_name| {
                    buf.push_str(&suite_name.bold().red().to_string());
                    buf.push_str(&":".red().to_string())
                });
                buf.push_str(&path_str.red().to_string());
                buf.push_str(&" (timeout)".dimmed().to_string());
            }
            State::Correct => {
                buf.push_str(&"✓ ".green().to_string());
                suite.into_iter().for_each(|suite_name| {
                    buf.push_str(&suite_name.bold().green().to_string());
                    buf.push_str(&":".green().to_string())
                });
                buf.push_str(&path_str.green().to_string());
            }
            State::Mismatch(expect_string, contents) => {
                buf.push_str(&"✗ ".red().to_string());
                suite.into_iter().for_each(|suite_name| {
                    buf.push_str(&suite_name.bold().red().to_string());
                    buf.push_str(&":".red().to_string())
                });
                buf.push_str(&path_str.red().to_string());
                if self.saved {
                    buf.push_str(&" (saved)".dimmed().to_string());
                }
                if show_diff {
                    let diff = printer::gen_diff(&contents, &expect_string);
                    buf.push('\n');
                    buf.push_str(&diff);
                }
            }
        };
        buf
    }
}

/// Result of running a TestSuite.
pub struct Suite {
    // Name of the test suite.
    pub name: String,
    // Number of matching paths.
    pub num_tests: i32,
    // TestResult for successfully executed tests.
    pub results: Vec<Test>,
    // Errors while running this suite.
    pub errors: Vec<RuntError>,
}

impl Suite {
    /// Construct a new instance of TestSuiteResult
    pub fn new(
        name: String,
        num_tests: i32,
        results: Vec<Test>,
        errors: Vec<RuntError>,
    ) -> Self {
        Self {
            name,
            num_tests,
            results,
            errors,
        }
    }

    /// Filter out the test suite results using the test statuses.
    pub fn only_results(mut self, only: &Option<cli::OnlyOpt>) -> Self {
        use cli::OnlyOpt as O;
        self.results.retain(|el| {
            if let (Some(only), Test { state, .. }) = (only, el) {
                return match (only, state) {
                    (O::Fail, State::Mismatch(..)) => true,
                    (O::Pass, State::Correct) => true,
                    (O::Missing, State::Missing(..)) => true,
                    (O::Fail, _) | (O::Pass, _) | (O::Missing, _) => false,
                };
            }
            true
        });
        self
    }

    /// Print the results of running this test suite.
    pub fn test_suite_results(
        &self,
        opts: &cli::Opts,
    ) -> (String, i32, i32, i32, i32) {
        use colored::*;
        let Suite {
            name,
            num_tests,
            results,
            errors,
        } = self;

        let mut buf = String::with_capacity(500);
        let (mut pass, mut fail, mut miss, mut timeout) = (0, 0, 0, 0);

        if !results.is_empty() {
            buf.push_str(&format!("{} ({} tests)\n", name.bold(), num_tests));
            results.iter().for_each(|info| {
                buf.push_str(&format!(
                    "  {}\n",
                    info.report_str(None, opts.diff)
                ));
                match info.state {
                    State::Correct => pass += 1,
                    State::Missing(..) => miss += 1,
                    State::Mismatch(..) => fail += 1,
                    State::Timeout => timeout += 1,
                }
            });
        }
        if !errors.is_empty() {
            buf.push_str(&format!("  {}\n", "runt errors".red()));
            errors.iter().for_each(|info| {
                buf.push_str(&format!("    {}\n", info.to_string().red()))
            });
        }
        (buf, pass, fail, miss, timeout)
    }

    // Save results from this TestSuite.
    /* pub fn save_all(&mut self) -> &mut Self {
        let Suite { results, .. } = self;
        for result in results {
            if let Err(e) = result.save_results() {
                self.errors.push(e);
            }
        }
        self
    } */
}
