use crate::cli;
use crate::errors::RuntError;
use std::path::Path;
use std::path::PathBuf;

/// Track the state of TestResult.
#[derive(Debug, PartialEq)]
pub enum TestState {
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

    /// Return status of the test.
    pub status: i32,

    /// STDOUT captured from the test.
    pub stdout: String,

    /// STRERR captured from the test.
    pub stderr: String,

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
            TS::Correct => Ok(()),
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
pub struct TestSuiteResult(
    pub String,          // Name of the test suite.
    pub i32,             // Number of matching paths.
    pub Vec<TestResult>, // TestResult for successfully executed tests.
    pub Vec<RuntError>,  // Errors while running this suite.
);

impl TestSuiteResult {
    /// Filter out the test suite results using the test statuses.
    pub fn only_results(mut self, only: &Option<cli::OnlyOpt>) -> Self {
        use cli::OnlyOpt as O;
        use TestState as TS;
        self.2.retain(|el| {
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
    ) -> (String, i32, i32, i32) {
        use colored::*;
        let TestSuiteResult(name, num_tests, results, errors) = self;

        let mut buf = String::with_capacity(500);
        let (mut pass, mut fail, mut miss) = (0, 0, 0);

        if !results.is_empty() {
            buf.push_str(&format!("{} ({} tests)\n", name.bold(), num_tests));
            results.iter().for_each(|info| {
                buf.push_str(&format!("  {}\n", info.report_str(opts.diff)));
                match info.state {
                    TestState::Correct => pass += 1,
                    TestState::Missing(..) => miss += 1,
                    TestState::Mismatch(..) => fail += 1,
                }
            });
        }
        if !errors.is_empty() {
            buf.push_str(&format!("  {}\n", "runt errors".red()));
            errors.iter().for_each(|info| {
                buf.push_str(&format!("    {}\n", info.to_string().red()))
            });
        }
        (buf, pass, fail, miss)
    }

    /// Save results from this TestSuite.
    pub fn save_all(&mut self) -> &mut Self {
        let TestSuiteResult(_, _, results, _) = self;
        for result in results {
            if let Err(e) = result.save_results() {
                self.3.push(e);
            }
        }
        self
    }
}

/// Format the output of the test into an expect string.
/// An expect string is of the form:
/// <contents of STDOUT>
/// ---CODE---
/// <exit code>
/// ---STDERR---
/// <contents of STDERR>
pub fn to_expect_string(status: i32, stdout: &str, stderr: &str) -> String {
    let mut buf = String::new();
    if !stdout.is_empty() {
        buf.push_str(stdout);
    }

    if status != 0 {
        buf.push_str("---CODE---\n");
        buf.push_str(format!("{}", status).as_str());
        buf.push('\n');
    }

    if !stderr.is_empty() {
        buf.push_str("---STDERR---\n");
        buf.push_str(stderr);
    }

    buf
}

/// Path of the expect file.
pub fn expect_file(expect_dir: Option<PathBuf>, path: &Path) -> PathBuf {
    expect_dir
        .map(|base| base.join(path.file_name().unwrap()))
        .unwrap_or_else(|| path.to_path_buf())
        .as_path()
        .with_extension("expect")
}
