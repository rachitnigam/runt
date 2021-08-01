# RUN Tests (RUNT) &emsp; [![latest]][crate] [![latest-docs]][docs]

Runt is a lightweight, concurrent, and parallel snapshot testing framework
that requires minimal configuration.
Checkout the [documentation][docs] for explaination of various features.

Here is an example of `runt` in action:
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
- Runt is tested using `runt`. Run `runt cli-test` to test runt.

### Example

View the [example configuration][conf] for the tests in `cli-tests`.
To run the tests, run `runt cli-tests`

### Alternatives

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
[latest-docs]: https://docs.rs/runt/badge.svg
[docs]: https://docs.rs/runt/0.3.1/runt/
[latest]: https://img.shields.io/crates/v/runt.svg
[crate]: https://crates.io/crates/runt
[conf]: https://github.com/rachitnigam/runt/blob/master/cli-test/runt.toml
