use crate::cli;
use crate::errors::RuntError;
use std::path::PathBuf;

/// Track the state of TestResult.
#[derive(Debug, PartialEq)]
pub enum TestState {
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
pub struct TestResult {
    /// Path of the test
    pub path: PathBuf,

    /// Location of the expect string.
    pub expect_path: PathBuf,

    /// Result of comparison
    pub state: TestState,

    /// The results of this structure were saved.
    pub saved: bool,
}

impl TestResult {
    /// Save the results of the test suite into the expect file.
    pub fn save_results(&mut self) -> Result<(), RuntError> {
        use std::fs;
        use TestState as TS;
        match &self.state {
            TS::Correct | TS::Timeout => Ok(()),
            TS::Missing(expect) | TS::Mismatch(expect, _) => {
                self.saved = true;
                fs::write(&self.expect_path, expect).map_err(|err| {
                    RuntError(format!(
                        "{}: {}.",
                        self.expect_path.to_str().unwrap(),
                        err
                    ))
                })
            }
        }
    }

    /// Generate colorized string to report the results of this test.
    pub fn report_str(&self, show_diff: bool) -> String {
        use crate::diff;
        use colored::*;
        use TestState as TS;

        let mut buf = String::new();
        let path_str = self.path.to_str().unwrap();
        match &self.state {
            TS::Missing(expect_string) => {
                buf.push_str(&"? miss - ".yellow().to_string());
                buf.push_str(&path_str.yellow().to_string());
                if self.saved {
                    buf.push_str(&" (saved)".dimmed().to_string());
                }
                if show_diff {
                    let diff = diff::gen_diff(&"".to_string(), &expect_string);
                    buf.push('\n');
                    buf.push_str(&diff);
                }
            }
            TS::Timeout => {
                buf.push_str(&"✗ timeout - ".red().to_string());
                buf.push_str(&path_str.red().to_string());
            }
            TS::Correct => {
                buf.push_str(&"✓ pass - ".green().to_string());
                buf.push_str(&path_str.green().to_string());
            }
            TS::Mismatch(expect_string, contents) => {
                buf.push_str(&"✗ fail - ".red().to_string());
                buf.push_str(&path_str.red().to_string());
                if self.saved {
                    buf.push_str(&" (saved)".dimmed().to_string());
                }
                if show_diff {
                    let diff = diff::gen_diff(&contents, &expect_string);
                    buf.push('\n');
                    buf.push_str(&diff);
                }
            }
        };
        buf
    }
}

/// Result of running a TestSuite.
pub struct TestSuiteResult {
    // Name of the test suite.
    pub name: String,
    // Number of matching paths.
    pub num_tests: i32,
    // TestResult for successfully executed tests.
    pub results: Vec<TestResult>,
    // Errors while running this suite.
    pub errors: Vec<RuntError>,
}

impl TestSuiteResult {
    /// Construct a new instance of TestSuiteResult
    pub fn new(
        name: String,
        num_tests: i32,
        results: Vec<TestResult>,
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
        use TestState as TS;
        self.results.retain(|el| {
            if let (Some(only), TestResult { state, .. }) = (only, el) {
                return match (only, state) {
                    (O::Fail, TS::Mismatch(..)) => true,
                    (O::Pass, TS::Correct) => true,
                    (O::Missing, TS::Missing(..)) => true,
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
        let TestSuiteResult {
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
                buf.push_str(&format!("  {}\n", info.report_str(opts.diff)));
                match info.state {
                    TestState::Correct => pass += 1,
                    TestState::Missing(..) => miss += 1,
                    TestState::Mismatch(..) => fail += 1,
                    TestState::Timeout => timeout += 1,
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

    /// Save results from this TestSuite.
    pub fn save_all(&mut self) -> &mut Self {
        let TestSuiteResult { results, .. } = self;
        for result in results {
            if let Err(e) = result.save_results() {
                self.errors.push(e);
            }
        }
        self
    }
}
