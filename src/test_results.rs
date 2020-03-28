/// Track the state of TestResult.
#[derive(Debug, PartialEq)]
pub enum TestState {
    /// The .expect file is missing.
    Missing,
    /// The comparison succeeded.
    Correct,
    /// The comparison failed. Contains the formatted diff string.
    Diff(String),
}

/// Store information related to one test.
#[derive(Debug)]
pub struct TestResult {
    /// Path of the test
    pub path: std::path::PathBuf,

    /// Return status of the test.
    pub status: i32,

    /// STDOUT captured from the test.
    pub stdout: String,

    /// STRERR captured from the test.
    pub stderr: String,

    /// Result of comparison
    pub state: TestState,
}

/// Format the output of the test into an expect string.
/// An expect string is of the form:
/// ---CODE---
/// <exit code>
/// ---STDOUT---
/// <contents of STDOUT>
/// ---STDERR---
/// <contents of STDERR>
pub fn to_expect_string(
    status: &i32,
    stdout: &String,
    stderr: &String,
) -> String {
    let mut buf = String::new();
    buf.push_str("---CODE---\n");
    buf.push_str(format!("{}", status).as_str());
    buf.push('\n');

    buf.push_str("---STDOUT---\n");
    buf.push_str(stdout.as_str());

    buf.push_str("---STDERR---\n");
    buf.push_str(stderr.as_str());

    buf.to_string()
}

impl TestResult {
    pub fn report_str(&self, show_diff: bool) -> String {
        use colored::*;
        use TestState as TS;
        let path_str = self.path.to_str().unwrap();
        let status = match &self.state {
            TS::Missing => ("⚬ missing - ".to_owned() + path_str).yellow(),
            TS::Diff(..) => ("⚬ failed - ".to_owned() + path_str).red(),
            TS::Correct => ("⚬ ok - ".to_owned() + path_str).green(),
        }.to_string();
        status.to_string()
    }
}
