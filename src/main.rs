mod cli;
mod diff;
mod errors;
mod test_results;
mod config;

use cli::{OnlyOpt, Opts};
use config::{TestSuite};
use errors::RuntError;
use serde::Deserialize;
use futures::future;
use structopt::StructOpt;
use test_results::{TestState, TestSuiteResult};

/// Configuration for a single runt run.
/// Tests suites for this runt configuration
#[derive(Debug, Deserialize)]
struct Config {
    /// Version of the runt tool this configuration is compatible with.
    ver: String,
    /// Test suite configurations.
    tests: Vec<TestSuite>,
}

#[tokio::main]
async fn execute_all(
    suites: Vec<TestSuite>,
    pre_filter: Option<&regex::Regex>,
) -> Vec<Result<TestSuiteResult, RuntError>> {
    let test_suite_tasks = suites
        .into_iter()
        .map(|suite| tokio::spawn(suite.execute_test_suite(&pre_filter)));

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
        return Err(RuntError(format!("Runt version mismatch. Configuration requires: {}, tool version: {}.\nRun `cargo install runt` to get the latest version of runt.", ver, env!("CARGO_PKG_VERSION"))));
    }

    // Switch to directory containing runt.toml.
    std::env::set_current_dir(&opts.dir)?;

    let pre_filter = match opts.only {
        Some(OnlyOpt::Matches(ref reg)) => Some(reg),
        _ => None
    };

    // Run all the test suites.
    let all_results = execute_all(tests, pre_filter);

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
