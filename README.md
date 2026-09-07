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

| Short | Long                      | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ----- | ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-s`  | `--sort <SORT>`           | The type of sorting to use. See below for options.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `-l`  | `--locale <LOCALE>`       | The locale to use for sorting. If this is not specified the sorting is in codepoint order.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `-u`  | `--unique`                | Make the file contents unique, or check that they're unique when used with `--check`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
|       | `--comment-prefix PREFIX` | A string that precedes comments. If this is set, comments starting with this string will be preserved and come before the same line in the sorted output. If the comment is preceded by an empty line, that empty line will also be preserved, unless the comment is the first thing in the file. If the `--unique` flag is also set then only the comment from the first instance of a repeated line will be preserved. If the `--reverse flag` is also set then only the last instance's comment will be preserved. Not accepted with `--sort gitignore`, which always handles `#` comments. |
| `-c`  | `--case-insensitive`      | Sort case-insensitively. Note that many locales always do this so if you specify a locale you may get case-insensitive output regardless of this flag.                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `-r`  | `--reverse`               | Sort in reverse order. Not accepted with `--sort gitignore`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
|       | `--windows`               | Parse paths as Windows paths for `--sort path`. Not accepted with `--sort gitignore`, where `/` is always the separator.                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `-i`  | `--in-place`              | Modify the file in place instead of making a backup.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
|       | `--stdout`                | Print the sorted output to stdout instead of making a new file.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
|       | `--check`                 | Check that the file is sorted instead of sorting it. If it is not sorted the exit status will be 1.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
|       | `--debug`                 | Print out debugging info while running.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `-h`  |                           | Show help summary.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
|       | `--help`                  | Show extended help with details about each sorting type.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `-V`  | `--version`               | Show application version.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |

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
- `gitignore` - sort the file assuming that each line is a gitignore pattern, sorting within the
  blocks the file is already divided into so that what it ignores cannot change
- `ip` - sort the file assuming that each line is an IP address
- `network` - sort the file assuming that each line is a network in CIDR form

<!-- sorting-methods -->

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

### Gitignore (`--sort gitignore`)

Each line is treated as a gitignore pattern.

Git uses the _last_ pattern matching a path to decide whether that path is ignored, so moving one
line past another can change what a file ignores. To make that impossible, this sorting method
breaks a file into blocks and lines are only ever sorted within a block. A blank line, a comment, a
line starting with a byte order mark, and every switch between a line starting with and without `!`
marks the boundary between blocks. A line is never moved across a boundary.

A run of consecutive `!` lines is a new block, and these lines are sorted within the block. These
blocks are left in place relative to other blocks.

Sorting a `.gitignore` file this way never changes what it ignores, whatever the patterns are. It
also means the sort stops at the boundaries the author drew, so a file organized into blocks is
tidied block by block instead of being flattened into one list.

Within a block, patterns are sorted as text by these rules:

- The leading `!`, any leading `/`, and any trailing `/` are ignored when comparing. This means that
  `foo`, `/foo`, `foo/`, and `!foo` each sort where the name alone would, instead of being scattered
  by their punctuation.
- A leading `**/` or `/**/` is ignored too, but only when a single name follows it, since git
  documents `**/foo` as meaning the same thing as `foo`. A pattern like `**/a/b` keeps its prefix,
  because it matches `b` in any `a` directory while `a/b` is anchored to the top of the tree.
- Spaces at the end of a line are ignored, since git strips them, so a pattern written with spaces
  after it sorts where it would without them. A space escaped with a `\` is part of the pattern and
  is kept.
- If two patterns are otherwise equal, anchored patterns (those starting with `/`) come first, then
  directory-only patterns (those ending with `/`), then patterns without a `**/` prefix.
- A leading `\!` or `\#` is an escape, so those lines are sorted by the literal `!` or `#` and are
  not treated as negations.

So this file:

```gitignore
# Build output
/target/
*.log

# Keep the checked-in log
!important.log

node_modules
/node_modules
vendor/*
!vendor/keep-me
```

sorts to this:

```gitignore
# Build output
*.log
/target/

# Keep the checked-in log
!important.log

/node_modules
node_modules
vendor/*
!vendor/keep-me
```

Blank lines and comments are kept where they were, so the `--comment-prefix` flag is not accepted
with this sorting method. A `#` in the first column is already a comment in this format, and
comments never move.

A negation which was already doing nothing goes on doing nothing, since fixing it would mean
changing what the file ignores. Use `git check-ignore -v` to find out which pattern really decides
whether to ignore a given path.

With `--unique`, two lines count as the same pattern when git would read them the same way, so
`foo`, `**/foo`, `/**/foo`, and `foo` followed by a space are all one pattern. The copy that is kept
is the _last_ one. Keeping an earlier copy could change what the file ignores, since a later copy
may be overriding a negation between them.

Removing lines can change which lines may be sorted together. If the only negation between two runs
of patterns goes away, those runs become one block and are sorted together. And if the only pattern
before a blank line goes away, that blank line ends up at the top of the file. Neither of these
changes what the file ignores, but both show up in the diff.

This sorting method accepts the `--locale` and `--case-insensitive` flags. It does not accept the
`--windows` flag, since gitignore patterns always use `/` as the separator.

It does not accept the `--reverse` flag either. There's no meaningful way to reverse the sorting
order of a `.gitignore` file.

This sorting method can also be used for `.npmignore` and `.dockerignore` files, but those differ
from `.gitignore` files in some ways. Both npm and Docker trim the whitespace around a pattern
before they look for the `!`, and git does not. So a line that puts a space before the `!` is a
negation to npm and Docker. This method follows git, so it sorts that line in among the plain
patterns. This will move negations for those files, which could cause them to not be applied as
expected. If you want to use this sorting method for those files, make sure you don't have any
negation lines that start with whitespace.

Only use `--unique` on files that really do follow git's rules, such as `.npmignore`. A bare `foo`
in a `.dockerignore` file matches only at the top of the tree, so two lines that `omegasort` would
call one pattern can be two different patterns to Docker.

A line starting with a byte order mark stays where it is. Git skips one BOM at the very start of a
file, so `<BOM>!foo` is the negation `!foo` on the first line and a pattern for a file whose name
begins with a mark anywhere else. A mark at the start of the file is left at the front, in front of
whichever line ends up first. If a line carrying its own mark ends up first in a file that had none,
a mark is written in front of it, so that reading the file back does not take the line's mark for
the file's. Note that this case is somewhat pathological, since it doesn't make sense to use a BOM
byte in a regular ignore pattern most of the time!

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

<!-- /sorting-methods -->

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
