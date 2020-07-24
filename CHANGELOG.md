Runt Changelog
==============

0.2.7
-----
- Internal: Use `buffered_unordered` to limit two parallel test suite runs at
  a time.
- Diff mode: Trim lines that were not changed to 80 characters.

0.2.6
-----
- Asynchronous test printing: Print out test suites as they finish instead
  of waiting on all test suites.
- Remove support for regex based `--only` filters. Pre-filters `--include`
  and `--exclude` subsume them.

0.2.5
-----

- Include and exclude regexes match on `<suite-name>:<path>` strings.
  - To select a test suite, simply do `runt -i "suite name"`
  - To select a path, simply do `runt -i "path"`
  - To select a path for a test suite, do `runt -i "suite name:path"`

0.2.4
-----
- Implement "pre-filters"
  - `--include`: Only run tests that match given regex.
  - `--exclude`: Exclude tests that match given regex.
- Modification to test suite name printing: When all tests from a test suite
  are suppressed, don't print the name.
- Code reorganization.

0.2.3
-----
- Bug fix: Print out the right `runt` command when runt configuration version does not match.

0.2.2
-----
- Add `ver` and `expect_dir` configuration options to runt.toml.

0.2.1
-----
- CLI uses "." as the default directory to find `runt.toml`.
- Use distinct symbols to show test states.
- Remove `name` field from the configuration.

0.2.0
-----
- Execute test suites in parallel. If certain test suites take longer to run,
  they will not block the execution of other test suites.

0.1.4
-----
- Suppress reporting when there are not tests of a certain category (fail,
  miss, or correct).

0.1.3
-----

- Change the expect string format to be:
  ```
  <STDOUT>
  ---CODE---
  <exit code>
  ---STDERR---
  <stderr>
  ```
  and suppress stderr when its empty and code when its zero.

0.1.2
-----
- Execute all commands in the directory where `runt.toml` resides.
- Print out test suite name and the total number of test states.
- Return code is the number of failing tests.

0.1.1
-----
- Fix help display for --only flag to say `miss` instead of `missing`.

0.1.0
-----
Initial release.
