mod diff;
mod errors;

use futures::future;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::process::Command;

use errors::RuntError;

#[derive(StructOpt, Debug)]
#[structopt(name = "runt", about = "Lightweight snapshot testing.")]
struct Opts {
    /// Files to process.
    #[structopt(name = "TEST_DIR", parse(from_os_str))]
    dir: PathBuf,

    /// Show diffs for each failing test.
    #[structopt(short, long)]
    diff: bool,

    /// Update expect files for each test (opens a dialog).
    #[structopt(short, long)]
    save: bool,
}

/// Store information related to one test.
#[derive(Debug)]
struct TestResult {
    /// Return status of the test.
    status: i32,

    /// STDOUT captured from the test.
    stdout: String,

    /// STRERR captured from the test.
    stderr: String,
}

impl TestResult {
    /// Format the output of the test into an expect string.
    /// An expect string is of the form:
    /// ---CODE---
    /// <exit code>
    /// ---STDOUT---
    /// <contents of STDOUT>
    /// ---STDERR---
    /// <contents of STDERR>
    fn to_expect_string(&self) -> String {
        let mut buf = String::new();
        buf.push_str("---CODE---");
        buf.push_str(format!("{}", self.status).as_str());
        buf.push_str("---STDOUT---");
        buf.push_str(self.stdout.as_str());
        buf.push_str("---STDERR---");
        buf.push_str(self.stderr.as_str());
        buf.to_string()
    }
}

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
        println!("{:#?}", pattern);
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
    // For each test suite, extract the glob patterns and run the tests.
    for suite in conf.tests {
        let rel_pats = suite
            .paths
            .iter()
            .map(|pattern| opts.dir.to_str().unwrap().to_owned() + "/" + pattern)
            .collect();
        // Run pattern on the relative path.
        let (paths, errors) = collect_globs(&rel_pats);
        let handles = paths
            .iter()
            .map(|path| {
                let mut cmd = construct_command(&suite.cmd, path);
                tokio::spawn(async move {
                    cmd.output()
                        .await
                        .map::<Result<TestResult, RuntError>, _>(|out| {
                            Ok(TestResult {
                                status: out.status.code().unwrap_or(-1),
                                stdout: String::from_utf8(out.stdout)?,
                                stderr: String::from_utf8(out.stderr)?,
                            })
                        })
                        .map_err(|err| RuntError(err.to_string()))
                })
            })
            .collect::<Vec<_>>();

        // Run all the tests in this suite and collect and errors.
        let results: Vec<Result<TestResult, RuntError>> =
            future::join_all(handles)
                .await
                .into_iter()
                .map(|res_of_res| {
                    res_of_res
                        .map_err(|err| RuntError(err.to_string()))
                        // Collapse multiple levels of Results into one.
                        .collapse()
                        .collapse()
                })
                .collect();

        println!("{:#?}", results);
    }
    Ok(())
}

fn run() -> Result<(), RuntError> {
    let opts = Opts::from_args();

    // Error if runt.toml doesn't exist.
    let conf_path = opts.dir.join("runt.toml");
    let contents = &fs::read_to_string(&conf_path).map_err(|err| {
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

    execute_all(conf, opts);

    Ok(())
}

fn main() {
    run();
}
