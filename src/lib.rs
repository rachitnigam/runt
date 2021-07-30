//! Runt is a blazing fast, concurrent, and parallel snapshot testing framework.
//!
//! Snapshot testing involves running a command and testing its output against
//! an already known "golden" output file.
//! A runt test suite is defined using a `runt.toml` file.
//!
//! ## Installation
//!
//! To install the `runt` binary, simply run:
//! ```
//! cargo install runt
//! ```
//!
//! ## Testing Model
//! Runt's organizes tests using test suites.
//! At the minimum, a test suite needs to be specify the input file paths as
//! well as a command that is run for each input file.
//! For example, the following defines a test suite named "Cat tests" that
//! runs the command `cat {}` on every file `.txt` file in the directory
//! `cat-tests/`.
//! The `{}` is replaced by the path of the input file.
//! ```
//! [[tests]]
//! # Name for this test suite.
//! name = "Cat tests"
//! # Test paths can be globs or exact.
//! paths = [ "cat-test/*.txt" ]
//! # Command to run on each test file. {} is replaced with input name.
//! cmd = "cat {}"
//! # (Optional) Directory to store the generated .expect files.
//! expect_dir = "cat-out/"
//! # (Optional) Timeout for tests in seconds. Defaults to 1200 seconds.
//! timeout = 120
//! ```
//!
//! ## Running a Test Suite
//! Runt's command line interface is used to run and interact with a Runt
//! test suite.
//!
//! For example, we can run [Runt's own test suite][runt-suite].
//! From the directory containing the `runt.toml` file, run `runt`.
//! Runt will generate a summary by running all the test suites. It will also
//! print out the paths of the missing and the failing test suites:
//! ```text
//! ? Cat tests:cat-test/a.txt
//! ✗ Cat tests:cat-test/b.txt
//! ✗ Timeout test:timeout-test/input.txt (timeout)
//! ? Ls test:ls-test/input.txt
//!   1 passing / 2 failing / 2 missing
//! ```
//!
//! According to Runt, we have 2 failing and 2 missing tests.
//!
//! ## Filters
//!
//! A complete runt configuration might have hundreds of tests. Runt provides
//! two kinds of filter operations to select a subset of the test configuration.
//!   - Pre-filters: The `--include` and `--exclude` flags can be used to select
//!     tests names that match (or don't match) a particular regex. The regexes
//!     are matched against the string `<suite-name>:<path>`.
//!   - Post-filters: The `--only` flag can be used to print out the names of
//!     tests with a specific exit condition (failing or missing). This is
//!     particularly useful in conjuction with the `--diff` and `--save` flags.
//!
//!
//! For example, to view only the tests that failed, we can invoke:
//! ```bash
//! runt -o fail
//! ```
//! To which, `runt` will respond with:
//! ```text
//! ✗ Cat tests:cat-test/b.txt
//! ✗ Timeout test:timeout-test/input.txt (timeout)
//!   1 passing / 2 failing / 2 missing
//! ```
//! Note that Runt is still running all the tests. It simply suppressing the
//! output from the missing tests.
//!
//! If we only want to run the tests from the test suite `Cat tests`, we can
//! use the `-i` flag:
//! ```bash
//! runt -i 'Cat tests'
//! ```
//! To which, `runt` reports:
//! ```test
//! ✗ Cat tests:cat-test/b.txt
//! ? Cat tests:cat-test/a.txt
//!   0 passing / 1 failing / 1 missing
//! ```
//! Note that runt reports 0 passing tests because it is not running the test
//! suite with the previously passing tests.
//!
//! ## Viewing diffs and Saving .expect Files
//!
//! Under the hood, `runt` uses `.expect` files to test the outputs of running
//! a command.
//! The `expect_dir` option specifies the directory which contains the `.expect`
//! file (defaults to the directory containing the test path).
//! For example, we can view the `.expect` files for the "Cat tests" suite
//! under `cat-out`.
//!
//! Runt is also capable of showing diffs for failing and missing tests using
//! the `-d` flag.
//! For example, we can run:
//! ```bash
//! runt -i 'Cat tests' -d
//! ```
//! Runt generates diffs for both the missing test and the failing tests.
//!
//! In order to bless the diff as correct, we can use the `-s` flag to save
//! the output.
//! ```bash
//! runt -i 'Cat tests' -d -s
//! ```
//!
//! Both the `-d` and `-s` flags work with the filtering flags.
//!
//! ## Timeouts
//!
//! Test suites can require a default timeout for each individual test.
//! When left unspecified, Runt will use 20 minutes as the default.
//!
//! [runt-suite]: https://github.com/rachitnigam/runt/tree/master/cli-test
pub mod cli;
pub mod errors;
pub mod executor;
pub mod picker;
pub mod printer;
