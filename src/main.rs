mod cli;
mod diff;
mod errors;
mod test_results;

use cli::Opts;
use futures::future;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use test_results::{TestResult, TestState, TestSuiteResult};
use tokio::process::Command;

use errors::RuntError;

/// Configuration for a single runt run.
#[derive(Debug, Deserialize)]
struct Config {
    /// Name of this runt configuration
    name: String,
    /// Tests suites for this runt configuration
    tests: Vec<TestSuite>,
}

/// Configuration for a test suite.
#[derive(Debug, Deserialize)]
struct TestSuite {
    /// Name of this TestSuite
    name: String,
    /// Paths of input files.
    paths: Vec<String>,
    /// Command to execute. The pattern `{}` in this string is replaced with
    /// the matching path.
    cmd: String,
}

/// Transform a list of glob patterns into matching paths and list of errors.
fn collect_globs<'a>(patterns: &Vec<String>) -> (Vec<PathBuf>, Vec<RuntError>) {
    // Generate list of all inputs using a globs and collect any errors.
    let mut matching_paths: Vec<PathBuf> = Vec::new();
    let mut errors: Vec<RuntError> = Vec::new();
    for pattern in patterns {
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
    let concrete_command = cmd.clone().replace("{}", path.to_str().unwrap());
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(concrete_command);
    cmd
}

#[tokio::main]
async fn execute_all(conf: Config, opts: Opts) -> Result<(), RuntError> {
    use errors::RichResult;
    for suite in conf.tests {
        let rel_pats = suite
            .paths
            .iter()
            .map(|pattern| {
                opts.dir.to_str().unwrap().to_owned() + "/" + pattern
            })
            .collect();

        // Run pattern on the relative path.
        let (paths, glob_errors) = collect_globs(&rel_pats);
        let num_tests = paths.len();
        let mut handles = Vec::with_capacity(num_tests);
        // For each test suite, extract the glob patterns and run the tests.
        // XXX(rachit): Slower test suites will block other tests from running.
        for path in paths {
            let mut cmd = construct_command(&suite.cmd, &path);
            let handle: tokio::task::JoinHandle<Result<_, RuntError>> =
                tokio::spawn(async move {
                    let out = cmd.output().await?;
                    let status = out.status.code().unwrap_or(-1);
                    let stdout = String::from_utf8(out.stdout)?;
                    let stderr = String::from_utf8(out.stderr)?;

                    // Generate expected string
                    let expect_string = test_results::to_expect_string(
                        &status, &stdout, &stderr,
                    );
                    // Open expect file for comparison.
                    let expect_path = path.as_path().with_extension("expect");
                    let state = tokio::fs::read_to_string(expect_path)
                        .await
                        .map(|contents| {
                            if contents == expect_string {
                                TestState::Correct
                            } else {
                                TestState::Mismatch(contents)
                            }
                        })
                        .unwrap_or(TestState::Missing);

                    return Ok(TestResult {
                        path,
                        status,
                        stdout,
                        stderr,
                        state: state,
                    });
                });
            handles.push(handle);
        }

        let glob_errors_to_chain = glob_errors
            .into_iter()
            .map(Err)
            .collect::<Vec<Result<TestResult, RuntError>>>();

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

        TestSuiteResult(suite.name, resolved)
            .only_results(&opts.only)
            .print_test_suite_results(&opts, num_tests);
    }
    Ok(())
}

fn run() -> Result<(), RuntError> {
    let opts = Opts::from_args();

    // Error if runt.toml doesn't exist.
    let conf_path = opts.dir.join("runt.toml");
    let contents = &fs::read_to_string(&conf_path).map_err(|_| {
        RuntError(format!(
            "{} is missing. Runt expects a directory with a runt.toml file.",
            conf_path.to_str().unwrap()
        ))
    })?;

    let conf: Config = toml::from_str(contents).map_err(|err| {
        RuntError(format!(
            "Failed to parse {}: {}",
            conf_path.to_str().unwrap(),
            err.to_string()
        ))
    })?;

    execute_all(conf, opts)
}

fn main() {
    match run() {
        Err(RuntError(msg)) => println!("error: {}", msg),
        Ok(..) => (),
    }
}
