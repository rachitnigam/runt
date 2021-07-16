use crate::errors;
use crate::test_results;

use errors::RuntError;
use futures::future;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;
use std::{fs, process::Command};
use test_results::{TestResult, TestState, TestSuiteResult};

/// Configuration for a test suite.
#[derive(Debug, Deserialize)]
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
}

/// Construct a command to run by replacing all occurances of `{}` with that
/// matching path.
fn construct_command(cmd: &str, path: &Path) -> Command {
    let concrete_command = cmd.replace("{}", path.to_str().unwrap());
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(concrete_command);
    cmd
}

/// Create a task to asynchronously execute this test. We use
/// std library fs::* and command::* so that there is a 1-to-1
/// correspondence between tokio threads and spawned processes.
/// This lets us control the number of parallel running processes.
async fn execute_test(
    mut cmd: Command,
    path: PathBuf,
    expect_dir: Option<PathBuf>,
) -> Result<TestResult, RuntError> {
    let out = cmd.output().map_err(|err| {
        RuntError(format!("{}: {}", path.to_str().unwrap(), err.to_string()))
    })?;

    let status = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8(out.stdout)?;
    let stderr = String::from_utf8(out.stderr)?;

    // Generate expected string
    let expect_string =
        test_results::to_expect_string(status, &stdout, &stderr);
    // Open expect file for comparison.
    let expect_path = test_results::expect_file(expect_dir, &path);
    let state = fs::read_to_string(expect_path.clone())
        .map(|contents| {
            if contents == expect_string {
                TestState::Correct
            } else {
                TestState::Mismatch(expect_string.clone(), contents)
            }
        })
        .unwrap_or(TestState::Missing(expect_string));

    Ok(TestResult {
        path,
        expect_path,
        status,
        stdout,
        stderr,
        state,
        saved: false,
    })
}

impl TestSuite {
    /// Remove paths that match with the include filter.
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

    /// Execute the test suite and collect the results into a `TestSuiteResult`.
    pub async fn execute_test_suite(self) -> TestSuiteResult {
        use errors::RichResult;
        use errors::RichVec;

        let TestSuite {
            paths,
            name,
            cmd,
            expect_dir,
        } = self;

        // Create async tasks for all tests and get handle.
        let num_tests = paths.len();

        // spawn a thread for each command to run
        let handles = paths.into_iter().map(|path| {
            let cmd = construct_command(&cmd, &path);
            tokio::spawn(execute_test(cmd, path, expect_dir.clone()))
        });

        // Run all the tests in this suite and collect and errors.
        let resolved: Vec<Result<TestResult, RuntError>> =
            future::join_all(handles)
                .await
                .into_iter()
                .map(|res_of_res| {
                    res_of_res
                        .map_err(|err| RuntError(err.to_string()))
                        // Collapse multiple levels of Results into one.
                        .collapse()
                })
                .collect();

        let (results, errors) = resolved.partition_results();

        TestSuiteResult(name.clone(), num_tests as i32, results, errors)
    }
}
