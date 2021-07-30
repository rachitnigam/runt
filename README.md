# RUN Test (RUNT) &emsp; [![latest]][crate]

[latest]: https://img.shields.io/crates/v/runt.svg
[crate]: https://crates.io/crates/runt

Runt is a lightweight, concurrent, and parallel snapshot testing framework. 
It aims to enable snapshot testing with minimal configuration.

![](static/runt.gif)

Install the latest version of `runt` using:
```
cargo install runt
```

Runt is most useful when you have the following test setup:
- One command that needs to run on many input files.
- Test suites grouped by commands run on the files.
- Test outputs are sent to IO streams (stdout and stderr).
- Test and test suites are unordered.

Runt is not useful when you want to do:
- Rich introspective testing of data structures.
- Test suites with complex setups, dependencies, and teardowns.

Snapshot testing with runt is extremely flexible. For example, the tests
under `runt-cli-test` test the outputs of the runt CLI.

### Building & Developing

- Install [Rust][].
- Run `cargo build --release`. The `runt` executable is generated
  under `target/release/runt`.
- Runt is tested using `runt`. Run `runt runt-cli-test` to test runt.

### Configuration

Runt is configured using a single `runt.toml` file:

```toml
# Version of runt to be used with this configuration.
ver = "0.3.1"

# Configuration for each test suite. File paths are relative to the folder
# containing runt.toml.
[[tests]]
# Optional name for this test suite.
name = "Cat tests"
# Test paths can be globs or exact.
paths = [ "cat-test/*.txt" ]
# Command to run on each test file. {} is replaced with input name.
cmd = "sleep 1; cat {}"
# (Optional) Directory to store the generated .expect files.
expect_dir = "cat-out/"
# (Optional) Timeout for tests in seconds. Defaults to 1200 seconds.
timeout = 1200

[[tests]]
name = "Ls test"
paths = [ "ls-test/input.txt" ]
cmd = "sleep 2; cat {} | ls"

[[tests]]
name = "Error test"
paths = ["error-test/input.txt"]
cmd = "sleep 3; echo error message 1>&2 && exit 1"

[[tests]]
name = "Timeout test"
cmd = """
sleep 100
"""
paths = ["timeout-test/input.txt"]
timeout = 2 # Timeout of two seconds
```

Run `runt <dir>` to execute all the tests. `<dir>` defaults to the current
directory.

### Options

**Showing diffs**: By default, runt does not show diffs between the new output
and the expect file. Use `--diff` to show the diffs.

**Saving changes**: The `--save` flag overwrites the expect files to save the
updated outputs.

**Suppress specific outputs**: The `--only` flag can be used to focus on only
failing, missing, or correct tests. It composes with the diff and save flags.


### Example

- Runt has a minimal configuration example under cli-tools. The `runt.toml`
  file contains all the configuration and explanation for various options.

### Troubleshooting

- **When executing a large test suite, I get `Too many open files (os error 24)`.**
  Runt tries to spawn as many processes in parallel as possible and might hit
  the system limit on open file descriptors. Use `ulimit -n 4096` to increase
  the number of file descriptors that can be opened at the same time.

### Other options

- **[Turnt][]** is a testing framework that allows for more
  complex snapshot comparisons. It's particularly powerful when you have
  several intermediate files you'd like to compare. `runt` forgoes the
  flexibility of turnt for faster execution and built-in output diffing.
- **[insta][]** enables snapshot testing of inline rust programs. Useful when
  the testing intrinsic structure of Rust programs. `runt` operators on
  arbitrary shell commands which enables testing CLI programs.
- **[jest][]** is a JavaScript snapshot testing framework that allow
  formulation of complex expectation queries.

[rust]: https://www.rust-lang.org/tools/install
[turnt]: https://github.com/cucapra/turnt
[insta]: https://docs.rs/insta/0.15.0/insta/
[jest]: https://jestjs.io/
