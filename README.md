# What is this?

Omegasort is a text file sorting tool that aims to be the last sorting tool you'll need.

I wrote this because I like to keep various types of files in a sorted order (`.gitignore` files,
lists of spelling stopwords, etc.) and I wanted a tool I could call as part of my commit hooks and
CI (using [precious](https://github.com/houseabsolute/precious)).

## Installation

There are several ways to install this tool.

### Use ubi

Install my [universal binary installer (ubi)](https://github.com/houseabsolute/ubi) tool and you can
use it to download `omegasort` and many other tools.

```
$> ubi --project houseabsolute/omegasort --in ~/bin
```

### Binary Releases

You can grab a binary release from the
[releases page](https://github.com/houseabsolute/omegasort/releases). Untar the tarball and put the
executable it contains somewhere in your path and you're good to go.

## usage: `omegasort [<flags>] [<file>]`

### Flags:

| Short | Long                      | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ----- | ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-s`  | `--sort <SORT>`           | The type of sorting to use. See below for options.                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `-l`  | `--locale <LOCALE>`       | The locale to use for sorting. If this is not specified the sorting is in codepoint order.                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `-u`  | `--unique`                | Make the file contents unique, or check that they're unique when used with `--check`.                                                                                                                                                                                                                                                                                                                                                                                                                                 |
|       | `--comment-prefix PREFIX` | A string that precedes comments. If this is set, comments starting with this string will be preserved and come before the same line in the sorted output. If the comment is preceded by an empty line, that empty line will also be preserved, unless the comment is the first thing in the file. If the `--unique` flag is also set then only the comment from the first instance of a repeated line will be preserved. If the `--reverse flag` is also set then only the last instance's comment will be preserved. |
| `-c`  | `--case-insensitive`      | Sort case-insensitively. Note that many locales always do this so if you specify a locale you may get case-insensitive output regardless of this flag.                                                                                                                                                                                                                                                                                                                                                                |
| `-r`  | `--reverse`               | Sort in reverse order.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
|       | `--windows`               | Parse paths as Windows paths for path sort.                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `-i`  | `--in-place`              | Modify the file in place instead of making a backup.                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
|       | `--stdout`                | Print the sorted output to stdout instead of making a new file.                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
|       | `--check`                 | Check that the file is sorted instead of sorting it. If it is not sorted the exit status will be 1.                                                                                                                                                                                                                                                                                                                                                                                                                   |
|       | `--debug`                 | Print out debugging info while running.                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `-h`  |                           | Show help summary.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
|       | `--help`                  | Show extended help with details about each sorting type.                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `-V`  | `--version`               | Show application version.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |

### Positional Args:

- `[<file>]` The file to sort.

## Sorting Options:

- `text` - sort the file as text according to the specified locale
- `numbered-text` - sort the file assuming that each line starts with a numeric prefix, then fall
  back to sorting by text according to the specified locale
- `datetime-text` - sort the file assuming that each line starts with a date or datetime prefix,
  then fall back to sorting by text according to the specified locale
- `path` - sort the file assuming that each line is a path, sorted so that deeper paths come after
  shorter
- `ip` - sort the file assuming that each line is an IP address
- `network` - sort the file assuming that each line is a network in CIDR form

### Text (`--sort text`)

This sorts each line of the file as text without any special parsing. The exact sorting is
determined by the `--locale`, `--case-insensitive`, and `--reverse` flags.

### Numbered Text (`--sort numbered-text`)

This assumes that each line of the file starts with a numeric value, optionally followed by
non-numeric text.

Lines should not have any leading space before the number. The number can either be an integer
(including 0) or a simple float (no scientific notation).

The lines will be sorted numerically first. If two lines have the same number they will be sorted by
text as above.

Lines without numbers always sort after lines with numbers.

This sorting method accepts the `--locale`, `--case-insensitive`, and `--reverse` flags.

### Datetime (`--sort datetime-text`)

This sorting method assumes that each line starts with a date or datetime, without any space in it.
That means that a string with both a date _and_ a time needs to be in a format like
"2019-08-27T19:13:16".

Lines should not have any leading space before the datetime.

This sorting method accepts the `--locale`, `--case-insensitive`, and `--reverse` flags.

### Path (`--sort path`)

Each line is treated as a path.

The paths are sorted by the following rules:

- Absolute paths come before relative.
- Paths are sorted by depth before sorting by the path content, so /z comes before /a/a.
- If you pass the `--windows` flag, then paths with drive letters or UNC names are sorted based on
  that prefix first. Paths with drive letters or UNC names sort before paths without them.

This sorting method accepts the `--locale`, `--case-insensitive`, and `--reverse` flags in addition
to the `--windows` flag.

### IP (`--sort ip`)

This method assumes that each line is an IPv4 or IPv6 address (not a network).

The sorting method is the same as if each line were the corresponding integer for the address. IPv4
addresses always sort before IPv6 addresses.

This sorting method accepts the `--reverse` flag.

### Network (`--sort network`)

This method assumes that each line is an IPv4 or IPv6 network in CIDR notation.

If there are two networks with the same base address they are sorted with the larger network first
(so 1.1.1.0/24 comes before 1.1.1.0/28). IPv4 networks always sort before IPv6 networks.

This sorting method accepts the `--reverse` flag.

## Linting and Tidying this Code

The code in this repo is linted and tidied with
[`precious`](https://github.com/houseabsolute/precious). This repo contains a `mise.toml` file.
[Mise](https://mise.jdx.dev/) is a tool for managing dev tools with per-repo configuration. You can
install `mise` and use it to run `precious` as follows:

```
# Installs mise
curl https://mise.run | sh
# Installs precious and other dev tools
mise install
```

Once this is done, you can run `precious` via `mise`:

```
# Lints all code
mise exec -- precious lint -a
# Tidies all code
mise exec -- precious tidy -a
```

If you want to use `mise` for other projects, see [its documentation](https://mise.jdx.dev/) for
more details on how you can configure your shell to always activate `mise`.
