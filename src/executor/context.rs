use super::{results, suite, Test};
use crate::errors::{self, RuntError};
use futures::{
    io::{AllowStdIo, AsyncWriteExt},
    stream, StreamExt,
};

/// An executor manages the execution of a list of tests.
pub struct Executor {
    /// Test configurations to be executed.
    tests: Vec<Test>,
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
            .buffer_unordered(10)
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
    /// Generates a summary that groups together tests of each test suite.
    /// Necessarily blocks till all results in a test suite become available
    /// before outputing the results.
    pub fn test_suite_summary(self) {
        todo!()
    }

    /// Generates a summary of the test results that streams the test results
    /// without grouping them with test suites.
    /// Immediately generates the output of the test as soon as they become
    /// available.
    pub async fn flat_summary(self) -> Result<i32, errors::RuntError> {
        let mut tasks = self.exec.execute_all();
        let stdout_buf = std::io::BufWriter::new(std::io::stdout());
        let mut handle = AllowStdIo::new(stdout_buf);

        while let Some(res) = tasks.next().await {
            let buf = res?.report_str(false) + "\n";
            handle.write_all(buf.as_bytes()).await?;
            handle.flush().await?;
        }

        Ok(0)
    }
}

/// Construct a Context from a list of Suites.
impl From<Vec<suite::Suite>> for Context {
    fn from(suites: Vec<suite::Suite>) -> Self {
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
            exec: Executor { tests },
            configs,
        }
    }
}
