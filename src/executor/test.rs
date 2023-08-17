use super::{results, suite};
use crate::errors::RuntError;
use std::{fs, path::PathBuf, time::Duration};
use tokio::{process::Command, time};

/// Configuration of a test to be executed.
pub struct Test {
    /// Path of the test to be run.
    pub path: PathBuf,
    /// Command to be executed for the test.
    pub cmd: String,
    /// Directory to save/check the expect results for.
    /// If set to `None`, defaults to the directory containing `Path`.
    pub expect_dir: Option<PathBuf>,
    /// Test suite with which this Test is associated.
    /// The mapping from the test suite
    pub test_suite: suite::Id,
    /// Timeout for this test.
    pub timeout: Duration,
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

    /// Path of the skip file
    pub fn skip_file(&self) -> PathBuf {
        self.get_base().with_extension("skip")
    }

    /// Construct a command to run by replacing all occurances of `{}` with that
    /// matching path.
    fn construct_command(&self) -> Command {
        let concrete_command =
            self.cmd.replace("{}", self.path.to_str().unwrap());
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(concrete_command);
        cmd.kill_on_drop(true);
        cmd
    }

    /// Create a task to asynchronously execute this test. We use
    /// std library fs::* and command::* so that there is a 1-to-1
    /// correspondence between tokio threads and spawned processes.
    /// This lets us control the number of parallel running processes.
    pub async fn execute_test(self, ignore_skip: bool) -> Result<results::Test, RuntError> {
        let skip_path = self.skip_file();
        if skip_path.exists() && !ignore_skip {
            return Ok(results::Test {
                path: self.path,
                expect_path: skip_path,
                state: results::State::Skip,
                saved: false,
                test_suite: self.test_suite,
            });
        }

        let expect_path = self.expect_file();

        let mut cmd = self.construct_command();

        match time::timeout(self.timeout, cmd.output()).await {
            Err(_) => Ok(results::Test {
                path: self.path,
                expect_path,
                state: results::State::Timeout,
                saved: false,
                test_suite: self.test_suite,
            }),
            Ok(res) => {
                let out = res.map_err(|err| {
                    RuntError(format!(
                        "{}: {}",
                        self.path.to_str().unwrap(),
                        err
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
                            results::State::Correct
                        } else {
                            results::State::Mismatch(
                                expect_string.clone(),
                                contents,
                            )
                        }
                    })
                    .unwrap_or(results::State::Missing(expect_string));

                Ok(results::Test {
                    path: self.path,
                    expect_path,
                    state,
                    saved: false,
                    test_suite: self.test_suite,
                })
            }
        }
    }
}
