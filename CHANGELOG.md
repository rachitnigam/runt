Runt Changelog
==============

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
