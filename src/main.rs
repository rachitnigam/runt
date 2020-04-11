mod cli;
mod diff;
mod errors;
mod test_results;

use cli::Opts;
use futures::future;
use serde::Deserialize;
use std::path::PathBuf;
use structopt::StructOpt;
use test_results::{TestResult, TestState, TestSuiteResult};
use tokio::process::Command;

use errors::RuntError;

/// Configuration for a single runt run.
/// Tests suites for this runt configuration
#[derive(Debug, Deserialize)]
struct Config {
    ver: String,
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
    /// Optional directory to store the generated .expect files.
    expect_dir: Option<PathBuf>,
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

async fn execute_test_suite(suite: TestSuite) -> TestSuiteResult {
    use errors::RichResult;
    use errors::RichVec;

    // For each test suite, extract the glob patterns and run the tests.
    let (paths, glob_errors) = collect_globs(&suite.paths);
    let glob_errors_to_chain = glob_errors
        .into_iter()
        .map(Err)
        .collect::<Vec<Result<TestResult, RuntError>>>();

    // Create async tasks for all tests and get handle.
    let num_tests = paths.len();
    let handles = paths.into_iter().map(|path| {
        let cmd = construct_command(&suite.cmd, &path);
        tokio::spawn(execute_test(cmd, path, suite.expect_dir.clone()))
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

    TestSuiteResult(suite.name.clone(), num_tests as i32, results, errors)
}

#[tokio::main]
async fn execute_all(
    suites: Vec<TestSuite>,
) -> Vec<Result<TestSuiteResult, RuntError>> {
    let test_suite_tasks = suites
        .into_iter()
        .map(|suite| tokio::spawn(execute_test_suite(suite)));

    future::join_all(test_suite_tasks)
        .await
        .into_iter()
        .map(|res| res.map_err(|err| RuntError(err.to_string())))
        .collect()
}

fn summarize_all_results(
    opts: &Opts,
    all_results: Vec<Result<TestSuiteResult, RuntError>>,
) -> i32 {
    use colored::*;

    // Collect summary statistics while printing this test suite.
    let (mut pass, mut fail, mut miss) = (0, 0, 0);
    for suite_res in all_results {
        if let Ok(res) = suite_res {
            res.2.iter().for_each(|res| match res.state {
                TestState::Correct => pass += 1,
                TestState::Missing(..) => miss += 1,
                TestState::Mismatch(..) => fail += 1,
            });

            let mut results = res.only_results(&opts.only);
            if opts.save {
                results.save_all();
            }
            results.print_test_suite_results(&opts);
        } else if let Err(err) = suite_res {
            println!("Failed to execute test suite: {}", err);
        }
    }

    println!();
    if miss != 0 {
        println!("{}", &format!("{} missing", miss).yellow().bold())
    }
    if fail != 0 {
        println!("{}", &format!("{} failing", fail).red().bold());
    }
    if pass != 0 {
        println!("{}", &format!("{} passing", pass).green().bold());
    }
    fail
}

fn run() -> Result<i32, RuntError> {
    let opts = Opts::from_args();

    // Error if runt.toml doesn't exist.
    let conf_path = opts.dir.join("runt.toml");
    let contents = &std::fs::read_to_string(&conf_path).map_err(|_| {
        RuntError(format!(
            "{} is missing. Runt expects a directory with a runt.toml file.",
            conf_path.to_str().unwrap()
        ))
    })?;

    let Config { ver, tests } = toml::from_str(contents).map_err(|err| {
        RuntError(format!(
            "Failed to parse {}: {}",
            conf_path.to_str().unwrap(),
            err.to_string()
        ))
    })?;

    // Check if the current `runt` matches the version specified in
    // the configuration.
    if env!("CARGO_PKG_VERSION") != ver {
        return Err(RuntError(format!("Runt version mismatch. Configuration requires: {}, tool version: {}.\nRun `cargo update runt` to get the latest version of runt.", ver, env!("CARGO_PKG_VERSION"))))
    }

    // Switch to directory containing runt.toml.
    std::env::set_current_dir(&opts.dir)?;

    // Run all the test suites.
    let all_results = execute_all(tests);

    // Summarize all the results.
    Ok(summarize_all_results(&opts, all_results))
}

fn main() {
    std::process::exit(match run() {
        Err(RuntError(msg)) => {
            println!("error: {}", msg);
            1
        }
        Ok(failed_tests) => failed_tests,
    })
}
