use super::{results, suite, Test};
use crate::{
    cli,
    errors::{self, RuntError},
};
use futures::{
    io::{AllowStdIo, AsyncWriteExt},
    stream, StreamExt,
};

/// An executor manages the execution of a list of tests.
pub struct Executor {
    /// Test configurations to be executed.
    tests: Vec<Test>,
    /// Maximum number of futures that can be created.
    max_futures: usize,
}
impl Executor {
    /// Execute the test suites and generate test results in any order.
    /// It is the job of the consumer of this method to collect the results and
    /// display them in the desired manner (grouped by test suite or order of
    /// completion)
    pub fn execute_all(
        self,
    ) -> impl stream::Stream<Item = Result<results::Test, RuntError>> {
        stream::iter(self.tests.into_iter().map(|test| test.execute_test()))
            .buffer_unordered(self.max_futures)
    }
}

/// An execution context manage the mapping between test suites and test, asynchronously executes
/// tests, collects results, and streams out results as appropriate.
pub struct Context {
    /// Configurations for suites to be executed.
    configs: Vec<suite::Config>,
    /// Test configurations to be executed.
    pub exec: Executor,
}

impl Context {
    /// Construct a new [Context] using suites and a maximum number of futures
    /// allowed to run concurrently.
    pub fn from(suites: Vec<suite::Suite>, max_futures: usize) -> Self {
        let mut configs = Vec::with_capacity(suites.len());
        let mut tests = Vec::with_capacity(suites.len());
        for (idx, suite) in suites.into_iter().enumerate() {
            let suite::Suite { config, paths } = suite;
            tests.extend(paths.into_iter().map(|path| Test {
                path,
                cmd: config.cmd.clone(),
                expect_dir: config.expect_dir.clone(),
                test_suite: idx as u64,
                timeout: config.timeout,
            }));
            configs.push(config);
        }
        Context {
            exec: Executor { tests, max_futures },
            configs,
        }
    }

    /// Generate a formatted string representing the current statistics
    fn summary_string(
        remaining: u64,
        miss: u64,
        timeout: u64,
        fail: u64,
        pass: u64,
    ) -> String {
        use colored::*;

        format!(
            " {} {} / {} {} / {} {} / {} {}",
            pass.to_string().green().bold(),
            &"passing".green().bold(),
            (fail + timeout).to_string().red().bold(),
            &"failing".red().bold(),
            miss.to_string().yellow().bold(),
            &"missing".yellow().bold(),
            remaining.to_string().dimmed().bold(),
            &"remaining".dimmed().bold(),
        )
    }

    /// Generates a summary of the test results that streams the test results
    /// without grouping them with test suites.
    /// Immediately generates the output of the test as soon as they become
    /// available.
    pub async fn flat_summary(
        self,
        opts: &cli::Opts,
    ) -> Result<i32, errors::RuntError> {
        let (mut miss, mut timeout, mut fail, mut pass, mut remaining) =
            (0, 0, 0, 0, self.exec.tests.len() as u64);
        let mut tasks = self.exec.execute_all();
        let stdout_buf = std::io::BufWriter::new(std::io::stdout());
        let mut handle = AllowStdIo::new(stdout_buf);

        // Initial summary printing to give user feedback that runt has started.
        let report = Self::summary_string(remaining, miss, timeout, fail, pass);
        handle.write_all(report.as_bytes()).await?;
        handle.flush().await?;

        while let Some(result) = tasks.next().await {
            let mut res = result?;

            // Save the result if needed
            if res.should_save(opts) {
                res.save_results().await?;
            }

            // Update summary
            match &res.state {
                results::State::Correct => {
                    pass += 1;
                }
                results::State::Mismatch(..) => {
                    fail += 1;
                }
                results::State::Timeout => {
                    timeout += 1;
                }
                results::State::Missing(..) => {
                    miss += 1;
                }
            }
            remaining -= 1;

            // Clear the current line to print the updating counter.
            handle.write_all("\r\x1B[K".as_bytes()).await?;

            // Print test information if needed.
            if res.should_print(opts) {
                let suite_name = &self.configs[res.test_suite as usize].name;
                handle
                    .write_all(
                        res.report_str(Some(suite_name), opts.diff).as_bytes(),
                    )
                    .await?;
                handle.write("\n".as_bytes()).await?;
            }

            // Print out the current summary
            let report =
                Self::summary_string(remaining, miss, timeout, fail, pass);
            handle.write_all(report.as_bytes()).await?;
            handle.flush().await?;
        }
        println!();

        Ok((timeout + fail) as i32)
    }
}
