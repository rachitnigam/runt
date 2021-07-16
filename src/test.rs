use std::{fs, path::PathBuf, time::Duration};
use tokio::{process::Command, time};

use crate::{
    errors::RuntError,
    test_results::{TestResult, TestState},
};

/// Struct that defines the state of a test to be run.
pub struct Test {
    /// Path of the test to be run.
    path: PathBuf,
    /// Command to be executed for the test.
    cmd: String,
    /// Directory to save/check the expect results for.
    /// If set to `None`, defaults to the directory containing `Path`.
    expect_dir: Option<PathBuf>,
}

impl Test {
    /// Format the output of the test into an expect string.
    /// An expect string is of the form:
    /// <contents of STDOUT>
    /// ---CODE---
    /// <exit code>
    /// ---STDERR---
    /// <contents of STDERR>
    pub fn format_expect_string(
        status: i32,
        stdout: &str,
        stderr: &str,
    ) -> String {
        let mut buf = String::new();
        if !stdout.is_empty() {
            buf.push_str(stdout);
        }

        if status != 0 {
            buf.push_str("---CODE---\n");
            buf.push_str(format!("{}", status).as_str());
            buf.push('\n');
        }

        if !stderr.is_empty() {
            buf.push_str("---STDERR---\n");
            buf.push_str(stderr);
        }

        buf
    }

    /// Construct a new instance of Test.
    pub fn new(
        path: PathBuf,
        cmd: String,
        expect_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            path,
            cmd,
            expect_dir,
        }
    }

    fn get_base(&self) -> PathBuf {
        self.expect_dir
            .clone()
            .map(|base| base.join(self.path.file_name().unwrap()))
            .unwrap_or_else(|| self.path.clone())
            .as_path()
            .to_path_buf()
    }

    /// Path of the expect file.
    pub fn expect_file(&self) -> PathBuf {
        self.get_base().with_extension("expect")
    }

    /// Construct a command to run by replacing all occurances of `{}` with that
    /// matching path.
    fn construct_command(&self) -> Command {
        let concrete_command =
            self.cmd.replace("{}", self.path.to_str().unwrap());
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(concrete_command);
        cmd
    }

    /// Create a task to asynchronously execute this test. We use
    /// std library fs::* and command::* so that there is a 1-to-1
    /// correspondence between tokio threads and spawned processes.
    /// This lets us control the number of parallel running processes.
    pub async fn execute_test(
        self,
        timeout: Duration,
    ) -> Result<TestResult, RuntError> {
        let expect_path = self.expect_file();

        let mut cmd = self.construct_command();

        match time::timeout(timeout, cmd.output()).await {
            Err(_) => Ok(TestResult {
                path: self.path,
                expect_path,
                state: TestState::Timeout,
                saved: false,
            }),
            Ok(res) => {
                let out = res.map_err(|err| {
                    RuntError(format!(
                        "{}: {}",
                        self.path.to_str().unwrap(),
                        err.to_string()
                    ))
                })?;

                let status = out.status.code().unwrap_or(-1);
                let stdout = String::from_utf8(out.stdout)?;
                let stderr = String::from_utf8(out.stderr)?;

                // Generate expected string
                let expect_string =
                    Self::format_expect_string(status, &stdout, &stderr);

                // Open expect file for comparison.
                let state = fs::read_to_string(expect_path.clone())
                    .map(|contents| {
                        if contents == expect_string {
                            TestState::Correct
                        } else {
                            TestState::Mismatch(expect_string.clone(), contents)
                        }
                    })
                    .unwrap_or(TestState::Missing(expect_string));

                Ok(TestResult {
                    path: self.path,
                    expect_path,
                    state,
                    saved: false,
                })
            }
        }
    }
}
