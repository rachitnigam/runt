use crate::{errors, test::Test, test_results};
use test_results::{TestResult, TestSuiteResult};

use errors::RuntError;
use futures::future;
use std::{path::PathBuf, time::Duration};

/// Configuration for a test suite.
#[derive(Debug)]
pub struct TestSuite {
    /// Name of this TestSuite
    pub name: String,
    /// Paths of input files.
    pub paths: Vec<PathBuf>,
    /// Command to execute. The pattern `{}` in this string is replaced with
    /// the matching path.
    pub cmd: String,
    /// Optional directory to store the generated .expect files.
    pub expect_dir: Option<PathBuf>,
    /// Optional timeout for the tests specified in seconds.
    /// Defaults to 120 seconds.
    pub timeout: Option<u64>,
}

impl TestSuite {
    /// Remove paths that match with the include filter.
    /// Filter is matched against the string `suite_name:path`.
    pub fn with_exclude_filter(
        mut self,
        exclude: Option<&regex::Regex>,
    ) -> Self {
        if let Some(ex) = exclude {
            // Matches the regexe to <suite-name>:<path>
            let name = self.name.clone();
            self.paths = self
                .paths
                .into_iter()
                .filter(|p| {
                    !ex.is_match(&(name.clone() + ":" + &p.to_string_lossy()))
                })
                .collect();
        }
        self
    }

    /// Remove paths that don't match with the include filter.
    /// Filter is matched against the string `suite_name:path`.
    pub fn with_include_filter(
        mut self,
        include: Option<&regex::Regex>,
    ) -> Self {
        if let Some(incl) = include {
            // Matches the regexe to <suite-name>:<path>
            let name = self.name.clone();
            self.paths = self
                .paths
                .into_iter()
                .filter(|p| {
                    incl.is_match(&(name.clone() + ":" + &p.to_string_lossy()))
                })
                .collect();
        }
        self
    }

    /// Dry run the test suite.
    /// Simply print out all the commands required to run the tests in this suite.
    pub fn dry_run(self) {
        use colored::*;
        let TestSuite {
            paths, cmd, name, ..
        } = self;
        // Skip test suite if there are no valid tests
        if paths.is_empty() {
            return;
        }

        let mut buf = String::with_capacity(500);

        buf.push_str(&format!("{} ({} tests)\n", name.bold(), paths.len()));
        paths.iter().for_each(|path| {
            let path_str = path.to_str().unwrap();
            buf.push_str(&format!(
                "  {} {}\n    {} {}",
                "⚬".blue(),
                path_str.blue(),
                "↳".blue(),
                cmd.replace("{}", path_str).replace('\\', "\\\\")
            ));
        });
        println!("{}", buf)
    }

    /// Execute the test suite and collect the results into a `TestSuiteResult`.
    pub async fn execute_test_suite(self) -> TestSuiteResult {
        use errors::RichResult;
        use errors::RichVec;

        let TestSuite {
            paths,
            name,
            cmd,
            expect_dir,
            timeout,
        } = self;

        // Create async tasks for all tests and get handle.
        let num_tests = paths.len();

        // spawn a thread for each command to run
        let handles = paths.into_iter().map(|path| {
            let test = Test::new(path, cmd.clone(), expect_dir.clone());
            // Get timeout or default to 120 seconds.
            let duration = Duration::from_secs(timeout.unwrap_or(120));
            tokio::spawn(test.execute_test(duration))
        });

        // Run all the tests in this suite and collect and errors.
        let resolved: Vec<Result<TestResult, RuntError>> =
            future::join_all(handles)
                .await
                .into_iter()
                .map(|rrr| {
                    rrr.map(|rr| rr.map_err(|err| RuntError(err.to_string())))
                        .map_err(|err| RuntError(err.to_string()))
                        // Collapse multiple levels of Results into one.
                        .collapse()
                })
                .collect();

        let (results, errors) = resolved.partition_results();

        TestSuiteResult::new(name.clone(), num_tests as i32, results, errors)
    }
}
