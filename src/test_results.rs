use crate::cli;
use crate::errors::RuntError;

/// Track the state of TestResult.
#[derive(Debug, PartialEq)]
pub enum TestState {
    /// The .expect file is missing.
    Missing,
    /// The comparison succeeded.
    Correct,
    /// The comparison failed. Contains the contents of the expect string.
    Mismatch(String),
}

/// Store information related to one test.
#[derive(Debug)]
pub struct TestResult {
    /// Path of the test
    pub path: std::path::PathBuf,

    /// Return status of the test.
    pub status: i32,

    /// STDOUT captured from the test.
    pub stdout: String,

    /// STRERR captured from the test.
    pub stderr: String,

    /// Result of comparison
    pub state: TestState,
}

/// Result of running a TestSuite.
pub struct TestSuiteResult(pub String, pub Vec<Result<TestResult, RuntError>>);

/// Format the output of the test into an expect string.
/// An expect string is of the form:
/// ---CODE---
/// <exit code>
/// ---STDOUT---
/// <contents of STDOUT>
/// ---STDERR---
/// <contents of STDERR>
pub fn to_expect_string(
    status: &i32,
    stdout: &String,
    stderr: &String,
) -> String {
    let mut buf = String::new();
    buf.push_str("---CODE---\n");
    buf.push_str(format!("{}", status).as_str());
    buf.push('\n');

    buf.push_str("---STDOUT---\n");
    buf.push_str(stdout.as_str());

    buf.push_str("---STDERR---\n");
    buf.push_str(stderr.as_str());

    buf.to_string()
}

impl TestResult {
    pub fn report_str(&self, show_diff: bool) -> String {
        use crate::diff;
        use colored::*;
        use TestState as TS;

        let mut buf = String::new();
        let path_str = self.path.to_str().unwrap();
        match &self.state {
            TS::Missing => {
                buf.push_str(&"⚬ miss - ".yellow().to_string());
                buf.push_str(&path_str.yellow().to_string());
                if show_diff {
                    let updated = to_expect_string(
                        &self.status,
                        &self.stdout,
                        &self.stderr,
                    );
                    let diff = diff::gen_diff("", &updated);
                    buf.push_str("\n");
                    buf.push_str(&diff);
                }
            }
            TS::Correct => {
                buf.push_str(&"⚬ pass - ".green().to_string());
                buf.push_str(&path_str.green().to_string());
            }
            TS::Mismatch(contents) => {
                buf.push_str(&"⚬ fail - ".red().to_string());
                buf.push_str(&path_str.red().to_string());
                if show_diff {
                    let updated = to_expect_string(
                        &self.status,
                        &self.stdout,
                        &self.stderr,
                    );
                    let diff = diff::gen_diff(&contents, &updated);
                    buf.push_str("\n");
                    buf.push_str(&diff);
                }
            }
        };
        buf.to_string()
    }
}

impl TestSuiteResult {
    pub fn only_results(mut self, only: &Option<cli::OnlyOpt>) -> Self {
        use cli::OnlyOpt as O;
        use TestState as TS;
        self.1.retain(|el| {
            if let (Some(only), Ok(TestResult { state, .. })) = (only, el) {
                return match (only, state) {
                    (O::Fail, TS::Mismatch(..)) => true,
                    (O::Pass, TS::Correct) => true,
                    (O::Missing, TS::Missing) => true,
                    _ => false
                }
            }
            true
        });
        self
    }

    /// Print the results of running this test suite.
    pub fn print_test_suite_results(
        self: TestSuiteResult,
        opts: &cli::Opts,
        num_tests: usize,
    ) {
        use colored::*;
        let TestSuiteResult(name, resolved) = self;

        // Summarize all the results
        let (results, errors): (Vec<_>, Vec<_>) =
            resolved.into_iter().partition(|el| el.is_ok());

        println!("{} ({} tests)", name.bold(), num_tests);
        results
            .into_iter()
            .map(Result::unwrap)
            .for_each(|info| println!("  {}", info.report_str(opts.diff)));

        // Report internal errors if any happened while executing this suite.
        let err_rep: Vec<RuntError> =
            errors.into_iter().map(Result::unwrap_err).collect();
        if !err_rep.is_empty() {
            println!("  {}", "runt errors".red());
            err_rep
                .into_iter()
                .for_each(|info| println!("    {}", info.to_string().red()))
        }
        ()
    }
}
