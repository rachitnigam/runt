mod cli;
mod config;
mod diff;
mod errors;
mod test_results;
mod test_suite;

use cli::Opts;
use config::Config;
use errors::RuntError;
use futures::io::{AllowStdIo, AsyncWriteExt};
use futures::{stream::FuturesUnordered, stream::StreamExt};
use regex::Regex;
use std::io::{self, BufWriter};
use structopt::StructOpt;
use test_suite::TestSuite;

/// Execute the runt configuration and generate results.
#[tokio::main]
async fn execute_all(
    suites: Vec<TestSuite>,
    incl_reg: Option<Regex>,
    excl_reg: Option<Regex>,
    opts: cli::Opts,
) -> Result<i32, errors::RuntError> {
    use colored::*;

    let test_suite_tasks = suites
        .into_iter()
        .map(|suite| {
            // Add filters to each test suite.
            let filtered = suite
                .with_include_filter(incl_reg.as_ref())
                .with_exclude_filter(excl_reg.as_ref());
            tokio::spawn(filtered.execute_test_suite())
        })
        .collect::<FuturesUnordered<_>>();

    // Collect summary statistics while printing this test suite.
    let (mut pass, mut fail, mut miss): (i32, i32, i32) = (0, 0, 0);
    let mut task = test_suite_tasks.into_future();
    // Buffered writing for stdout.
    let stdout = io::stdout();
    let mut handle = AllowStdIo::new(BufWriter::new(stdout));
    loop {
        match task.await {
            (None, _) => break,
            (Some(res), nxt) => {
                let mut results = res?.only_results(&opts.post_filter);
                // Save if needed.
                if opts.save {
                    results.save_all();
                }
                let (buf, p, f, m) = results.test_suite_results(&opts);
                // Write the strings
                handle.write_all(buf.as_bytes()).await?;
                handle.flush().await?;
                // Update the statistics.
                pass += p;
                fail += f;
                miss += m;
                task = nxt.into_future();
            }
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
    Ok(fail)
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

    // Get the include and exclude regexes.
    let incl_reg: Option<Regex> = opts
        .include_filter
        .as_ref()
        .map(|reg| Regex::new(&reg).expect("Invalid --include regex"));

    let excl_reg = opts
        .exclude_filter
        .as_ref()
        .map(|reg| Regex::new(&reg).expect("Invalid --exclude regex"));

    // Switch to directory containing runt.toml.
    std::env::set_current_dir(&opts.dir)?;

    // Run all the test suites.
    execute_all(
        tests.into_iter().map(|c| c.into()).collect::<Vec<_>>(),
        incl_reg,
        excl_reg,
        opts,
    )
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
