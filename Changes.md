## 0.1.3 - 2023-11-04

- When sorting a file with repeated lines with a `--locale`, the sorting order was not always
  consistent, and the `--unique` flag could leave duplicates behind.
- As of this release there are no longer binaries built for MIPS on Linux. These targets have been
  demoted to tier 3 support by the Rust compiler.

## 0.1.2 - 2023-06-04

- Sorting is now done in parallel using [rayon](https://docs.rs/rayon/latest/rayon/). This
  significantly speeds up sorting of large files. In my experiments, a c. 100MB file went from 12s
  to 8s. A c. 1GB file went from 2m40s to 1m40s.

## 0.1.1 - 2023-05-20

- Fixed the release orchestration so releases are only built with stable Rust.

## 0.1.0 - pulled

- I rewrote it in Rust (RIIR).
- Added the ability to preserve comments in files when sorting. Use the `--comment-prefix` option
  for this.
- Fixed the handling of empty lines when running in `--check` mode. Fixes GH #3.

## 0.0.7 - 2022-11-12

- Added Darwin arm64 and Windows arm64 builds.

## 0.0.6 - 2022-11-06

- When a file is already sorted, `omegasort` will no longer write to the file. This means that the
  file's last modification time will not change in this case. Fixes #3.

## 0.0.5 - 2021-03-27

- Added a `--unique` flag. This can also be used with `--check` to check that a file is both sorted
  and unique.
- Always close temp files before moving them. On Windows attempting to move an open file causes an
  error.

## 0.0.4 - 2020-12-29

- Fix handling of errors during initialization. These sorts of error could lead to a confusing panic
  instead of showing the actual error message.
- Handle the case where stdout is not connected to the terminal. Previously this caused an error
  during initialization.
- Replace file renaming with copying to handle the case where the temp file we sort into and the
  original file are not on the same partition.
- Fix bug where sorting wasn't stable in the presence of two case-insensitively identical lines (and
  possibly other similar scenarios).

## 0.0.3 - 2020-12-27

- Fix terminal width check. It was using the height as the width. In addition, it now makes the text
  width 90 characters if the terminal is wider than that.

## 0.0.2 - 2019-09-27

- The `--check` flag was not implemented and now it is.

## 0.0.1 - 2019-08-27

- First release upon an unsuspecting world.
