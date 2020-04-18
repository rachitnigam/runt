mod cli;
mod config;
mod diff;
mod errors;
mod test_results;
mod test_suite;

use cli::Opts;
use config::Config;
use errors::RuntError;
use futures::future;
use structopt::StructOpt;
use test_results::{TestState, TestSuiteResult};
use test_suite::TestSuite;

#[tokio::main]
async fn execute_all(
    suites: Vec<TestSuite>,
) -> Vec<Result<TestSuiteResult, RuntError>> {
    let test_suite_tasks = suites
        .into_iter()
        .map(|suite| tokio::spawn(suite.execute_test_suite()));

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

            let mut results = res.only_results(&opts.post_filter);
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

    let Config { ver, suite_confs } =
        toml::from_str(contents).map_err(|err| {
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

    // Run all the test suites.
    let all_results = execute_all(
        suite_confs
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<_>>(),
    );

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
