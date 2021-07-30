use runt::{
    cli, errors,
    executor::{self, suite},
    picker::toml::Config,
};

use cli::Opts;
use errors::RuntError;
use regex::Regex;
use structopt::StructOpt;
use tokio::runtime;

fn run() -> Result<i32, RuntError> {
    let opts = Opts::from_args();
    let Config { tests, .. } = Config::from_path(&opts.dir)?;

    // Get the include and exclude regexes.
    let include = opts
        .include_filter
        .as_ref()
        .map(|reg| Regex::new(reg).expect("Invalid --include regex"));

    let exclude = opts
        .exclude_filter
        .as_ref()
        .map(|reg| Regex::new(reg).expect("Invalid --exclude regex"));

    // Switch to directory containing runt.toml.
    std::env::set_current_dir(&opts.dir)?;

    let suites: Vec<suite::Suite> = tests
        .into_iter()
        .map(|c| {
            suite::Suite::from(c)
                .with_filters(include.as_ref(), exclude.as_ref())
        })
        .collect();

    let ctx = executor::Context::from(suites, opts.max_futures.unwrap_or(50));
    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(opts.jobs_limit.unwrap_or_else(num_cpus::get))
        .build()
        .unwrap();

    // Run all the test suites.
    runtime.block_on(ctx.flat_summary(&opts))
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
