RUN Test (RUNT)
--------------

Runt is a lightweight, parallel snapshot testing framework. It aims to enable
snapshot testing with minimal configuration.

Runt is most useful when you have the following test setup:
- One command that needs to run on many input files.
- Test suites grouped by commands run on the files.
- Test outputs are sent to IO streams (stdout and stderr).

Runt is not useful when you want to do:
- Rich introspective testing of data structures.
- Test suites with complex setups, dependencies, and teardowns.

### Building

- Install [Rust][].
- Run `cargo build --release`. The `runt` executable is generated
  under `target/release/runt`.

### Example

- Runt has a minimal configuration example under cli-tools. The `runt.toml`
  file contains all the configuration and explanation for various options.

[rust]: https://www.rust-lang.org/tools/install
