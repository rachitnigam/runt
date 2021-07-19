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

/// Execute the runt configuration and generate results.
/* async fn execute_all(
    suites: Vec<TestSuite>,
    incl_reg: Option<Regex>,
    excl_reg: Option<Regex>,
    opts: cli::Opts,
) -> Result<i32, errors::RuntError> {
    use colored::*;

    // spawn as many suite managers in parallel as possible.
    // the number of actual worker tasks is still limited to
    // `opts.job_limit`.
    let num_suites = suites.len();
    let mut test_suite_tasks = stream::iter(suites)
        .map(|suite| {
            // Add filters to each test suite.
            let filtered = suite
                .with_include_filter(incl_reg.as_ref())
                .with_exclude_filter(excl_reg.as_ref());
            tokio::spawn(filtered.execute_test_suite())
        })
        .buffered(num_suites);

    // Collect summary statistics while printing this test suite.
    let (mut pass, mut fail, mut miss, mut timeout) = (0, 0, 0, 0);
    // Buffered writing for stdout.
    let stdout = io::stdout();
    let mut handle = AllowStdIo::new(BufWriter::new(stdout));

    while let Some(res) = test_suite_tasks.next().await {
        let mut results = res?.only_results(&opts.post_filter);
        // Save if needed.
        if opts.save {
            results.save_all();
        }
        let (buf, p, f, m, t) = results.test_suite_results(&opts);
        // Write the strings
        handle.write_all(buf.as_bytes()).await?;
        handle.flush().await?;
        // Update the statistics.
        pass += p;
        fail += f + t;
        miss += m;
        timeout += t;
    }

    println!();
    if miss != 0 {
        println!("{}", &format!("{} missing", miss).yellow().bold())
    }
    if fail != 0 {
        print!("{}", &format!("{} failing", fail).red().bold());
        if timeout == 0 {
            println!();
        } else {
            println!("{}", &format!(" ({} timeouts)", timeout).red().bold());
        }
    }
    if pass != 0 {
        println!("{}", &format!("{} passing", pass).green().bold());
    }
    Ok(fail)
} */

/// Print out the commands to be run to execute the test suites.
/* fn dry_run(
    suites: Vec<TestSuite>,
    incl_reg: Option<Regex>,
    excl_reg: Option<Regex>,
) -> Result<i32, RuntError> {
    suites.into_iter().for_each(|suite| {
        // Add filters to each test suite.
        suite
            .with_include_filter(incl_reg.as_ref())
            .with_exclude_filter(excl_reg.as_ref())
            .dry_run();
    });

    Ok(0)
} */

fn run() -> Result<i32, RuntError> {
    let opts = Opts::from_args();
    let Config { tests, .. } = Config::from_path(&opts.dir)?;

    // Get the include and exclude regexes.
    let include = opts
        .include_filter
        .as_ref()
        .map(|reg| Regex::new(&reg).expect("Invalid --include regex"));

    let exclude = opts
        .exclude_filter
        .as_ref()
        .map(|reg| Regex::new(&reg).expect("Invalid --exclude regex"));

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
