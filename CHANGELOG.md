Runt Changelog
==============

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
