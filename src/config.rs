use crate::errors;
use crate::test_results;

use errors::RuntError;
use futures::future;
use serde::Deserialize;
use std::path::PathBuf;
use test_results::{TestResult, TestState, TestSuiteResult};
use tokio::process::Command;

/// Configuration for a test suite.
#[derive(Debug, Deserialize)]
pub struct TestSuite {
    /// Name of this TestSuite
    pub name: String,
    /// Paths of input files.
    pub paths: Vec<String>,
    /// Command to execute. The pattern `{}` in this string is replaced with
    /// the matching path.
    pub cmd: String,
    /// Optional directory to store the generated .expect files.
    pub expect_dir: Option<PathBuf>,
}

/// Transform a list of glob patterns into matching paths and list of errors.
fn collect_globs(patterns: &[String]) -> (Vec<PathBuf>, Vec<RuntError>) {
    // Generate list of all inputs using a globs and collect any errors.
    let mut matching_paths: Vec<PathBuf> = Vec::new();
    let mut errors: Vec<RuntError> = Vec::new();
    for pattern in patterns {
        // If the glob patter is a concrete path, skip it
        let path = PathBuf::from(pattern);
        if path.is_file() {
            matching_paths.push(path);
            continue;
        }
        let glob_res = glob::glob(&pattern);
        // The glob can either succeed for fail.
        match glob_res {
            // If the glob pattern succeeded, collect errors and matching paths.
            Ok(paths) => {
                for maybe_path in paths {
                    maybe_path.map_or_else(
                        // Format error messages and collect them.
                        |pat_err| {
                            errors.push(RuntError(format!(
                                "{} matches but failed to read file: {}",
                                pattern,
                                pat_err.to_string()
                            )))
                        },
                        // Collect matching paths
                        |path| matching_paths.push(path),
                    )
                }
            }
            // If the glob failed, collect the error messages.
            Err(err) => errors.push(RuntError(format!(
                "Invalid glob pattern: {}",
                err.to_string()
            ))),
        }
    }

    (matching_paths, errors)
}

/// Construct a command to run by replacing all occurances of `{}` with that
/// matching path.
fn construct_command(cmd: &str, path: &PathBuf) -> Command {
    let concrete_command = cmd.replace("{}", path.to_str().unwrap());
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(concrete_command);
    cmd
}

/// Create a task to asynchronously execute this test.
async fn execute_test(
    mut cmd: Command,
    path: PathBuf,
    expect_dir: Option<PathBuf>,
) -> Result<TestResult, RuntError> {
    let out = cmd.output().await.map_err(|err| {
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
    let state = tokio::fs::read_to_string(expect_path.clone())
        .await
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
    pub async fn execute_test_suite(
        self,
        pre_filter: &Option<&regex::Regex>,
    ) -> TestSuiteResult {
        use errors::RichResult;
        use errors::RichVec;

        // For each test suite, extract the glob patterns and run the tests.
        let (paths, glob_errors) = collect_globs(&self.paths);
        let glob_errors_to_chain = glob_errors
            .into_iter()
            .map(Err)
            .collect::<Vec<Result<TestResult, RuntError>>>();

        // Create async tasks for all tests and get handle.
        let num_tests = paths.len();
        let handles = paths.into_iter().map(|path| {
            let cmd = construct_command(&self.cmd, &path);
            tokio::spawn(execute_test(cmd, path, self.expect_dir.clone()))
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
                .chain(glob_errors_to_chain)
                .collect();

        let (results, errors) = resolved.partition_results();

        TestSuiteResult(self.name.clone(), num_tests as i32, results, errors)
    }
}
