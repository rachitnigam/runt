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

/// Track the status of an executing test suite and print output to stdout.
pub struct Status {
    pub miss: u64,
    pub pass: u64,
    pub remain: u64,
    pub skip: u64,
    pub fail: u64,
    pub timeout: u64,
    /// Handle to the output
    handle: AllowStdIo<std::io::BufWriter<std::io::Stdout>>,
    istty: bool,
}

impl Status {
    /// Instantiate a test suite with total number of tests
    pub fn new(total: u64) -> Self {
        let handle =
            AllowStdIo::new(std::io::BufWriter::new(std::io::stdout()));
        let istty = atty::is(atty::Stream::Stdout);
        Self {
            remain: total,
            miss: 0,
            pass: 0,
            skip: 0,
            fail: 0,
            timeout: 0,
            handle,
            istty,
        }
    }

    /// Generate a summary string for the current state.
    fn summary(&self) -> String {
        use colored::*;

        format!(
            " {} {} / {} {} / {} {} / {} {} / {} {}",
            self.pass.to_string().green().bold(),
            &"passing".green().bold(),
            (self.fail + self.timeout).to_string().red().bold(),
            &"failing".red().bold(),
            self.miss.to_string().yellow().bold(),
            &"missing".yellow().bold(),
            self.skip.to_string().yellow().dimmed().bold(),
            &"skipped".yellow().dimmed().bold(),
            self.remain.to_string().dimmed().bold(),
            &"remaining".dimmed().bold(),
        )
    }

    /// Stream out the current summary to the output if possible.
    #[inline]
    pub async fn stream_summary(&mut self) -> Result<(), errors::RuntError> {
        if self.istty {
            self.print_summary().await?;
        }
        Ok(())
    }

    /// Print the current status
    #[inline]
    pub async fn print_summary(&mut self) -> Result<(), errors::RuntError> {
        self.handle.write_all(self.summary().as_bytes()).await?;
        self.handle.flush().await?;
        Ok(())
    }

    /// Clear the current output if possible.
    #[inline]
    pub async fn clear(&mut self) -> Result<(), errors::RuntError> {
        if self.istty {
            self.handle.write_all("\r\x1B[K".as_bytes()).await?;
        }
        Ok(())
    }

    /// Print a message to the output handler.
    pub async fn print<S: AsRef<[u8]>>(
        &mut self,
        msg: S,
    ) -> Result<(), errors::RuntError> {
        self.handle.write_all(msg.as_ref()).await?;
        self.handle.write_all("\n".as_ref()).await?;
        self.handle.flush().await?;
        Ok(())
    }

    /// Update the status given the [State] of a test.
    pub fn update(&mut self, state: &results::State) {
        match state {
            results::State::Skip => {
                self.skip += 1;
            }
            results::State::Correct => {
                self.pass += 1;
            }
            results::State::Mismatch(..) => {
                self.fail += 1;
            }
            results::State::Timeout => {
                self.timeout += 1;
            }
            results::State::Missing(..) => {
                self.miss += 1;
            }
        }
        self.remain -= 1;
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

    /// Generates a streaming summary of the test results.
    pub async fn execute_and_summarize(
        self,
        opts: &cli::Opts,
    ) -> Result<i32, errors::RuntError> {
        let mut st = Status::new(self.exec.tests.len() as u64);
        let mut tasks = self.exec.execute_all();

        // Initial summary printing to give user feedback that runt has started.
        st.stream_summary().await?;

        while let Some(result) = tasks.next().await {
            let mut res = result?;

            // Save the result if needed
            if res.should_save(opts) {
                res.save_results().await?;
            }

            // Update summary
            st.update(&res.state);

            // Clear the current line to print the updating counter.
            st.clear().await?;

            // Print test information if needed.
            if res.should_print(opts) {
                let suite_name = &self.configs[res.test_suite as usize].name;
                st.print(res.report_str(Some(suite_name), opts.diff))
                    .await?;
            }

            // Print out the current summary
            st.stream_summary().await?;
        }

        // Print the final summary
        st.clear().await?;
        st.print_summary().await?;
        println!();

        match opts.post_filter {
            Some(cli::OnlyOpt::Fail) => Ok((st.fail + st.timeout) as i32),
            Some(cli::OnlyOpt::Missing) => Ok((st.miss) as i32),
            Some(cli::OnlyOpt::Pass) => Ok(0),
            None => Ok((st.fail + st.timeout + st.miss) as i32)
        }
    }
}
