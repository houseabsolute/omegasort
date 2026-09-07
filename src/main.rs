extern crate alloc;

mod collation;
mod comparer;
mod error;
mod gitignore;
mod logging;
mod sorter;

use crate::{error::CheckError, gitignore::Grouper};
use anyhow::{anyhow, Context, Error, Result};
use clap::{CommandFactory, FromArgMatches, Parser};
use log::{debug, error};
use sorter::{Sorter, Strategy};
use std::{
    collections::hash_map::DefaultHasher,
    env::args_os,
    ffi::OsString,
    fs::{copy, File},
    hash::{Hash, Hasher},
    io::{stdout, BufRead, BufReader, BufWriter, Chain, Cursor, Read, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
use termimad::MadSkin;

const MAX_TERM_WIDTH: usize = 100;

#[derive(Parser)]
#[command(author, version, about)]
#[clap(max_term_width = MAX_TERM_WIDTH)]
#[clap(after_long_help = long_help())]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// The type of sorting to use.
    #[arg(short, long, value_enum)]
    sort: Strategy,
    /// The locale to use for sorting. If this is not specified the sorting is in codepoint order.
    #[arg(short, long, value_name = "CODE")]
    locale: Option<String>,
    /// Make the file contents unique, or check that they're unique when used with --check.
    #[arg(short, long)]
    unique: bool,
    /// A string that precedes comments. If this is set, comments starting
    /// with this string will be preserved and come before the same line in
    /// the sorted output. If the comment is preceded by an empty line, that
    /// empty line will also be preserved, unless the comment is the first
    /// thing in the file. If the --unique flag is also set then only the
    /// comment from the first instance of a repeated line will be
    /// preserved. If the --reverse flag is also set then only the last
    /// instance's comment will be preserved.
    #[arg(long, value_name = "PREFIX")]
    comment_prefix: Option<String>,
    /// Sort case-insensitively. Note that many locales always do this so if
    /// you specify a locale you may get case-insensitive output regardless of
    /// this flag.
    #[arg(short, long)]
    case_insensitive: bool,
    /// Sort in reverse order.
    #[arg(short, long)]
    reverse: bool,
    /// Parse paths as Windows paths for path sort.
    #[arg(long)]
    windows: bool,
    /// Modify the file in place instead of making a backup.
    #[arg(short, long, group = "output")]
    in_place: bool,
    /// Print the sorted output to stdout instead of making a new file.
    #[arg(long, group = "output")]
    stdout: bool,
    /// Check that the file is sorted instead of sorting it. If it is not
    /// sorted (or not unique if --unique is given) the exit status will be 1.
    #[arg(long, group = "output")]
    check: bool,
    /// The file to sort.
    file: PathBuf,
    /// Print debugging info while running.
    #[arg(long)]
    debug: bool,
}

fn main() {
    let status = match Cli::new_from_args(args_os()) {
        Ok(cli) => cli.run(),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<clap::Error>() {
                e.exit()
            } else {
                error!("{e}");
                42
            }
        }
    };
    std::process::exit(status);
}

/// The extended help for each sorting method is kept in `README.md` and pulled out of it here, so
/// that the two cannot drift apart. The markers are HTML comments, so they do not show up when
/// GitHub renders the file.
const SORTING_METHODS_START: &str = "<!-- sorting-methods -->";
const SORTING_METHODS_END: &str = "<!-- /sorting-methods -->";

fn long_help() -> String {
    const INTRO: &str = "There are a number of different sorting methods available.\n";

    let skin = MadSkin::default();
    let help = format!("{INTRO}\n{}", sorting_methods_from_readme());
    format!("{}", skin.text(&help, Some(MAX_TERM_WIDTH)))
}

/// Returns the part of `README.md` between the sorting-methods markers.
fn sorting_methods_from_readme() -> String {
    const README: &str = include_str!("../README.md");

    sorting_methods_from(README)
}

/// Returns the part of `readme` between the sorting-methods markers.
///
/// The README is baked in at compile time, so this cannot fail at runtime for a reason the tests
/// would not already have caught. A test checks that both markers are still there.
///
/// A Windows checkout can give the README `\r\n` line endings, so the start marker cannot include
/// a line ending and the text that comes back is normalized to `\n`.
fn sorting_methods_from(readme: &str) -> String {
    let start = readme
        .find(SORTING_METHODS_START)
        .expect("README.md has a sorting-methods start marker")
        + SORTING_METHODS_START.len();
    let end = readme[start..]
        .find(SORTING_METHODS_END)
        .expect("README.md has a sorting-methods end marker")
        + start;

    readme[start..end].trim().replace("\r\n", "\n")
}

impl Cli {
    fn new_from_args<I, T>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let command = Cli::command();
        Cli::from_arg_matches(&command.get_matches_from(args)).map_err(std::convert::Into::into)
    }

    fn run(&self) -> i32 {
        if let Err(e) = logging::init(self.debug) {
            error!("{e}");
            return 100;
        }

        if let Err(e) = self.validate_args() {
            error!("{e}");
            return 101;
        }

        if let Err(e) = self.execute() {
            error!("{e}");
            let status = match e.downcast::<CheckError>() {
                Ok(
                    CheckError::HasUnexpectedEmptyLines
                    | CheckError::NotSorted { .. }
                    | CheckError::NotUnique { .. },
                ) => 1,
                _ => 2,
            };
            return status;
        }

        0
    }

    fn validate_args(&self) -> Result<()> {
        if self.locale.is_some() && !self.sort.supports_locale() {
            return Err(anyhow!(
                "you cannot set a locale when sorting by {:?}",
                self.sort,
            ));
        }

        if self.windows && !self.sort.supports_path_type() {
            return Err(anyhow!(
                "you cannot pass the --windows flag when sorting {:?}",
                self.sort,
            ));
        }

        if self.reverse && !self.sort.supports_reverse() {
            return Err(anyhow!(
                "you cannot pass the --reverse flag when sorting {:?}, because reversing these files would change what they ignore",
                self.sort,
            ));
        }

        if self.comment_prefix.is_some() && self.sort.keeps_file_structure() {
            return Err(anyhow!(
                "you cannot set a comment prefix when sorting {:?}, because comments are part of the format and are always left where they are",
                self.sort,
            ));
        }

        if self.in_place && self.check {
            return Err(anyhow!("you cannot set both --in-place and --stdout"));
        }

        Ok(())
    }

    fn execute(&self) -> Result<()> {
        let sorter = Sorter::new(
            self.sort,
            self.locale.as_deref(),
            self.unique,
            self.case_insensitive,
            self.reverse,
            self.windows,
        )?;
        let contents = read_lines(&self.file, self.sort, self.comment_prefix.as_deref())?;
        if self.check {
            if contents.has_empty_lines {
                return Err(CheckError::HasUnexpectedEmptyLines.into());
            }
            if sorter.lines_are_sorted(&contents.lines)? {
                return Ok(());
            }
        }

        self.sort_lines(contents, &sorter)
    }

    fn sort_lines(&self, mut contents: FileContents, sorter: &Sorter) -> Result<()> {
        let orig_hash = if contents.has_empty_lines {
            None
        } else {
            Some(hash_lines(&contents.lines))
        };
        contents.lines = sorter.sort_lines(contents.lines)?;
        if !contents.has_empty_lines {
            let new_hash = hash_lines(&contents.lines);
            if orig_hash.unwrap() == new_hash && !self.stdout {
                debug!("file is already sorted");
                return Ok(());
            }
        }

        if self.stdout {
            return write_lines_to_writer(contents, &mut stdout());
        }

        if !self.in_place {
            let mut bak_file = self.file.clone();
            let ext = bak_file
                .extension()
                .map_or("", |e| e.to_str().unwrap_or(""));
            bak_file.set_extension(if ext.is_empty() {
                String::from("bak")
            } else {
                format!("{ext}.bak")
            });
            copy(&self.file, bak_file)?;
        }

        // If we don't make this in the same directory as the original file,
        // then the `persist` call later may fail because we may end up trying
        // to rename files across filesystems.
        let mut file = NamedTempFile::new_in(self.file.parent().unwrap())?;
        write_lines_to_writer(contents, &mut file)?;
        let temp_path = file.path().to_path_buf();
        file.persist(&self.file).with_context(|| {
            format!(
                "error renaming {} to {}",
                temp_path.display(),
                self.file.display(),
            )
        })?;

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct SortableLine {
    line_number: usize,
    line: String,
    comment: Option<Comment>,
    /// Lines are sorted within a group and never moved across one. Every strategy but gitignore
    /// puts the whole file in a single group.
    group: usize,
    kind: LineKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum LineKind {
    /// A line that takes part in sorting.
    Sortable,
    /// A line that stays exactly where it is, and that `--unique` never removes. Only gitignore
    /// files have these.
    Fence,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Comment {
    is_preceded_by_empty_line: bool,
    lines: Vec<String>,
}

impl SortableLine {
    // These are only used in tests.
    #[allow(dead_code)]
    fn from_number_and_str(from: (usize, &str)) -> Self {
        Self::for_test(from.0, from.1, 0, LineKind::Sortable)
    }

    #[allow(dead_code)]
    fn for_test(line_number: usize, line: &str, group: usize, kind: LineKind) -> Self {
        Self {
            line_number,
            line: line.to_string(),
            comment: None,
            group,
            kind,
        }
    }
}

/// A file's lines, plus the things about the file itself that have to survive sorting.
struct FileContents {
    lines: Vec<SortableLine>,
    has_empty_lines: bool,
    line_ending: &'static str,
    has_bom: bool,
}

fn read_lines<P: AsRef<Path>>(
    file: P,
    sort: Strategy,
    comment_prefix: Option<&str>,
) -> Result<FileContents> {
    let mut f = File::open(file.as_ref())?;
    let LineEndingChain {
        reader,
        line_ending,
        has_bom,
    } = determine_line_ending(&mut f)?;
    let (lines, has_empty_lines) = lines_from_reader(sort, comment_prefix, reader)?;
    Ok(FileContents {
        lines,
        has_empty_lines,
        line_ending,
        has_bom,
    })
}

fn lines_from_reader<R: Read>(
    sort: Strategy,
    comment_prefix: Option<&str>,
    read: R,
) -> Result<(Vec<SortableLine>, bool)> {
    if sort.keeps_file_structure() {
        // Nothing is dropped and nothing is an error, so there are never any unexpected empty lines
        // to report.
        return Ok((grouped_lines_from_reader(read)?, false));
    }

    let reader = BufReader::new(read);
    let mut lines = vec![];
    let mut comment: Option<Comment> = None;
    let mut last_line_was_empty = false;
    let mut has_empty_lines = false;

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() {
            last_line_was_empty = true;
            continue;
        }

        if comment_prefix.is_some() && line.trim().starts_with(comment_prefix.unwrap()) {
            if let Some(ref mut comment) = comment {
                comment.lines.push(line);
            } else {
                comment = Some(Comment {
                    lines: vec![line],
                    is_preceded_by_empty_line: last_line_was_empty,
                });
                last_line_was_empty = false;
            }
            continue;
        }

        // The last line was empty and this current line is not a comment.
        if last_line_was_empty {
            has_empty_lines = true;
        }

        lines.push(SortableLine {
            line_number: i + 1,
            line,
            comment,
            group: 0,
            kind: LineKind::Sortable,
        });
        last_line_was_empty = false;
        comment = None;
    }
    Ok((lines, has_empty_lines))
}

/// Reads a file whose own structure has to survive sorting. Every line is kept exactly as it was
/// read, including blank lines and comments, and each is tagged with the group it may be sorted
/// within.
fn grouped_lines_from_reader<R: Read>(read: R) -> Result<Vec<SortableLine>> {
    let reader = BufReader::new(read);
    let mut grouper = Grouper::default();
    let mut lines = vec![];

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let (group, kind) = grouper.next(&line);
        lines.push(SortableLine {
            line_number: i + 1,
            line,
            comment: None,
            group,
            kind,
        });
    }

    Ok(lines)
}

// Doing the uniqueness check here lets us avoid iterating over the lines yet
// another time while still avoiding rewriting an already sorted file.
fn hash_lines(lines: &[SortableLine]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for l in lines {
        l.hash(&mut hasher);
    }

    hasher.finish()
}

fn write_lines_to_writer<W: Write>(contents: FileContents, out: &mut W) -> Result<()> {
    let FileContents {
        lines,
        line_ending,
        has_bom,
        ..
    } = contents;
    let mut bw = BufWriter::new(out);
    // A file that came in with a byte order mark gets it back. So does one whose first line starts
    // with a mark of its own, even when the file had none, because reading takes a mark off byte 0
    // without asking whose it is. Writing such a line bare would hand its mark to the file, and the
    // next read would eat it, turning `<BOM>x` into `x`. Our own mark in front keeps the line's
    // where it belongs. Sorting alone cannot put that line first in a gitignore file, since a mark
    // makes it a fence, but `--unique` can drop every line above it.
    if has_bom || first_line_written(&lines).starts_with('\u{feff}') {
        bw.write_all(&UTF8_BOM)?;
    }
    for (i, l) in lines.into_iter().enumerate() {
        if let Some(comment) = l.comment {
            // If the comment is the first thing in the file we don't preserve its leading empty
            // line.
            if comment.is_preceded_by_empty_line && i != 0 {
                bw.write_all(line_ending.as_bytes())?;
            }
            for line in comment.lines {
                bw.write_all(line.as_bytes())?;
                bw.write_all(line_ending.as_bytes())?;
            }
        }
        bw.write_all(l.line.as_bytes())?;
        bw.write_all(line_ending.as_bytes())?;
    }

    Ok(())
}

/// The first text that lands in the file, which is the first line's comment block when it has one.
fn first_line_written(lines: &[SortableLine]) -> &str {
    let Some(first) = lines.first() else {
        return "";
    };
    first
        .comment
        .as_ref()
        .and_then(|c| c.lines.first())
        .unwrap_or(&first.line)
}

const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

const FIRST_CHUNK_SIZE: usize = 2048;

const LINE_ENDINGS: [&str; 3] = ["\r\n", "\n", "\r"];

/// A reader for the whole file, plus what reading its first bytes settled.
struct LineEndingChain<'a> {
    /// The bytes already read, chained in front of the rest of the file, minus any BOM.
    reader: Chain<Cursor<Vec<u8>>, &'a mut File>,
    line_ending: &'static str,
    has_bom: bool,
}

/// Reads just far enough to see how the file ends its lines, then hands back a reader for the whole
/// file along with the answer.
///
/// A BOM belongs to the file, not to its first line. Git skips one when it reads a gitignore file,
/// so `<BOM>!foo` is a negation to git and not a pattern for a file whose name starts with a
/// BOM. We take it off here and `write_lines_to_writer` puts it back, which keeps it at the front
/// of the file however the lines are reordered.
fn determine_line_ending(file: &mut File) -> Result<LineEndingChain<'_>> {
    let mut buf = [0; FIRST_CHUNK_SIZE];
    let read = file.read(&mut buf)?;

    let has_bom = buf[0..read].starts_with(&UTF8_BOM);
    let start = if has_bom { UTF8_BOM.len() } else { 0 };

    for le in LINE_ENDINGS {
        if buf_contains_str(le, &buf) {
            return Ok(LineEndingChain {
                reader: Cursor::new(Vec::from(&buf[start..read])).chain(file),
                line_ending: le,
                has_bom,
            });
        }
    }

    Err(could_not_determine_line_ending())
}

fn could_not_determine_line_ending() -> Error {
    anyhow!("could not determine line ending from first {FIRST_CHUNK_SIZE} bytes of file")
}

fn buf_contains_str(needle: &str, haystack: &[u8]) -> bool {
    if needle.len() == 1 {
        return haystack.contains(&needle.as_bytes()[0]);
    }

    for w in haystack.windows(needle.len()) {
        if w == needle.as_bytes() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod test {
    use crate::{CheckError, Cli};

    use super::{Comment, FileContents, LineEndingChain, LineKind, SortableLine};
    use crate::sorter::Strategy;
    use anyhow::Result;
    use std::{
        fs::{metadata, read_dir, read_to_string, write, File},
        io::{Read, Write},
        path::PathBuf,
    };
    use tempfile::tempdir;
    use test_log::test;

    const WITH_COMMENTS: &str = r"
foo
bar
# comment 1
baz

# comment 2
quux
";

    const WITH_REPEATED_LINES: &str = r"
# first foo
foo
bar

# first baz
baz

# second foo
foo
quux

# second baz
baz
";

    // The extended help is cut out of `README.md`, so a rename or a stray edit to either marker
    // would quietly leave `--help` with no sorting methods in it at all.
    #[test]
    fn sorting_methods_come_from_the_readme() {
        let methods = super::sorting_methods_from_readme();
        assert!(
            methods.starts_with("### Text (`--sort text`)"),
            "the section starts at the first sorting method",
        );
        assert!(
            methods.ends_with("This sorting method accepts the `--reverse` flag."),
            "the section ends with the last sorting method",
        );
        assert!(
            !methods.contains("## Linting and Tidying this Code"),
            "the section stops before the rest of the README",
        );
    }

    // A Windows checkout can hand us a README with `\r\n` line endings, which is what broke this
    // the first time around.
    #[test]
    fn sorting_methods_survive_crlf_line_endings() {
        let readme = concat!(
            "# omegasort\r\n",
            "\r\n",
            "<!-- sorting-methods -->\r\n",
            "\r\n",
            "### Text (`--sort text`)\r\n",
            "\r\n",
            "This sorts each line.\r\n",
            "\r\n",
            "<!-- /sorting-methods -->\r\n",
            "\r\n",
            "## Linting and Tidying this Code\r\n",
        );
        assert_eq!(
            super::sorting_methods_from(readme),
            "### Text (`--sort text`)\n\nThis sorts each line.",
        );
    }

    #[test]
    fn lines_from_reader() -> Result<()> {
        let lines = ["foo", "bar", "baz", "quux"]
            .map(|l| format!("{l}\n"))
            .join("");
        assert_eq!(
            super::lines_from_reader(Strategy::Text, None, lines.trim().as_bytes())?,
            (
                [(1, "foo"), (2, "bar"), (3, "baz"), (4, "quux")]
                    .into_iter()
                    .map(SortableLine::from_number_and_str)
                    .collect::<Vec<_>>(),
                false
            ),
        );

        let lines = ["foo", "", "bar", "", "baz", "quux"]
            .map(|l| format!("{l}\n"))
            .join("");
        assert_eq!(
            super::lines_from_reader(Strategy::Text, None, lines.trim().as_bytes())?,
            (
                [(1, "foo"), (3, "bar"), (5, "baz"), (6, "quux")]
                    .into_iter()
                    .map(SortableLine::from_number_and_str)
                    .collect::<Vec<_>>(),
                true,
            ),
            "empty lines are skipped",
        );

        assert_eq!(
            super::lines_from_reader(Strategy::Text, None, WITH_COMMENTS.trim_start().as_bytes())?,
            (
                vec![
                    SortableLine {
                        line_number: 1,
                        line: "foo".to_string(),
                        comment: None,
                        group: 0,
                        kind: LineKind::Sortable,
                    },
                    SortableLine {
                        line_number: 2,
                        line: "bar".to_string(),
                        comment: None,
                        group: 0,
                        kind: LineKind::Sortable,
                    },
                    SortableLine {
                        line_number: 3,
                        line: "# comment 1".to_string(),
                        comment: None,
                        group: 0,
                        kind: LineKind::Sortable,
                    },
                    SortableLine {
                        line_number: 4,
                        line: "baz".to_string(),
                        comment: None,
                        group: 0,
                        kind: LineKind::Sortable,
                    },
                    SortableLine {
                        line_number: 6,
                        line: "# comment 2".to_string(),
                        comment: None,
                        group: 0,
                        kind: LineKind::Sortable,
                    },
                    SortableLine {
                        line_number: 7,
                        line: "quux".to_string(),
                        comment: None,
                        group: 0,
                        kind: LineKind::Sortable,
                    },
                ],
                true,
            ),
        );

        assert_eq!(
            super::lines_from_reader(
                Strategy::Text,
                Some("#"),
                WITH_COMMENTS.trim_start().as_bytes()
            )?,
            (
                vec![
                    SortableLine {
                        line_number: 1,
                        line: "foo".to_string(),
                        comment: None,
                        group: 0,
                        kind: LineKind::Sortable,
                    },
                    SortableLine {
                        line_number: 2,
                        line: "bar".to_string(),
                        comment: None,
                        group: 0,
                        kind: LineKind::Sortable,
                    },
                    SortableLine {
                        line_number: 4,
                        line: "baz".to_string(),
                        comment: Some(Comment {
                            lines: vec!["# comment 1".to_string()],
                            is_preceded_by_empty_line: false,
                        }),
                        group: 0,
                        kind: LineKind::Sortable,
                    },
                    SortableLine {
                        line_number: 7,
                        line: "quux".to_string(),
                        comment: Some(Comment {
                            lines: vec!["# comment 2".to_string()],
                            is_preceded_by_empty_line: true,
                        }),
                        group: 0,
                        kind: LineKind::Sortable,
                    },
                ],
                false
            ),
        );

        Ok(())
    }

    /// These tests only vary the lines and the BOM, so the rest of a `FileContents` is filled in
    /// with values that do not affect what is written.
    fn contents(lines: Vec<SortableLine>, has_bom: bool) -> FileContents {
        FileContents {
            lines,
            has_empty_lines: false,
            line_ending: "\n",
            has_bom,
        }
    }

    #[test]
    fn write_lines_to_writer() -> Result<()> {
        struct TestCase<'a> {
            comment_marker: Option<&'static str>,
            input: &'a str,
            expect: &'a str,
        }
        let tests = [
            TestCase {
                comment_marker: Some("#"),
                input: WITH_COMMENTS.trim_start(),
                expect: WITH_COMMENTS.trim_start(),
            },
            TestCase {
                comment_marker: Some("#"),
                input: WITH_REPEATED_LINES.trim_start(),
                expect: WITH_REPEATED_LINES.trim_start(),
            },
        ];

        for t in tests {
            let mut buf = vec![];
            let (lines, _) =
                super::lines_from_reader(Strategy::Text, t.comment_marker, t.input.as_bytes())?;
            super::write_lines_to_writer(contents(lines, false), &mut buf)?;
            assert_eq!(unsafe { String::from_utf8_unchecked(buf) }, t.expect);
        }

        let mut buf = vec![];
        let (lines, _) = super::lines_from_reader(Strategy::Text, None, "a\nb\n".as_bytes())?;
        super::write_lines_to_writer(contents(lines, true), &mut buf)?;
        assert_eq!(
            unsafe { String::from_utf8_unchecked(buf) },
            "\u{feff}a\nb\n",
            "a file that had a BOM gets it back, in front of the first line",
        );

        let mut buf = vec![];
        let (lines, _) =
            super::lines_from_reader(Strategy::Text, None, "\u{feff}a\nb\n".as_bytes())?;
        super::write_lines_to_writer(contents(lines, false), &mut buf)?;
        assert_eq!(
            unsafe { String::from_utf8_unchecked(buf) },
            "\u{feff}\u{feff}a\nb\n",
            "a first line that starts with a BOM gets one written in front of it, so that \
             reading the file back does not take the line's own BOM for the file's",
        );

        let mut buf = vec![];
        let (lines, _) =
            super::lines_from_reader(Strategy::Text, None, "\u{feff}a\nb\n".as_bytes())?;
        super::write_lines_to_writer(contents(lines, true), &mut buf)?;
        assert_eq!(
            unsafe { String::from_utf8_unchecked(buf) },
            "\u{feff}\u{feff}a\nb\n",
            "one BOM for the file and one for the line, and no third one",
        );

        Ok(())
    }

    #[test]
    fn determine_line_ending() -> Result<()> {
        let mut long_str = "Lorem ipsum dolor sit amet".repeat(100);
        long_str.push('\n');
        // The third element is whether the file starts with a BOM, which `determine_line_ending`
        // reports so that it can be written back later.
        let tests: &[(&str, Result<&str>, bool)] = &[
            (
                "Lorem ipsum dolor sit amet\nconsectetur adipiscing elit",
                Ok("\n"),
                false,
            ),
            (
                "Lorem ipsum dolor sit amet\rconsectetur adipiscing elit",
                Ok("\r"),
                false,
            ),
            (
                "Lorem ipsum dolor sit amet\r\nconsectetur adipiscing elit",
                Ok("\r\n"),
                false,
            ),
            (
                "\u{feff}Lorem ipsum dolor sit amet\nconsectetur adipiscing elit",
                Ok("\n"),
                true,
            ),
            (
                "Lorem ipsum\u{feff} dolor sit amet\nconsectetur adipiscing elit",
                Ok("\n"),
                false,
            ),
            (
                "Lorem ipsum dolor sit amet\tconsectetur adipiscing elit",
                Err(super::could_not_determine_line_ending()),
                false,
            ),
            (
                long_str.as_str(),
                Err(super::could_not_determine_line_ending()),
                false,
            ),
        ];

        for t in tests {
            let dir = tempdir()?;
            let mut filename = dir.path().to_path_buf();
            filename.push("le-test");

            let mut file = File::create(&filename)?;
            write!(file, "{}", t.0)?;
            drop(file);

            let mut file = File::open(&filename)?;
            let res = super::determine_line_ending(&mut file);
            if let Ok(expect) = t.1 {
                let LineEndingChain {
                    mut reader,
                    line_ending,
                    has_bom,
                } = res?;
                assert_eq!(line_ending, expect, "line ending for {:?}", t.0);
                assert_eq!(has_bom, t.2, "BOM for {:?}", t.0);

                let mut rest = String::new();
                reader.read_to_string(&mut rest)?;
                assert_eq!(
                    rest,
                    t.0.strip_prefix('\u{feff}').unwrap_or(t.0),
                    "the reader hands back the file with any leading BOM taken off",
                );
            } else {
                assert!(res.is_err());
                assert_eq!(
                    res.map(|c| c.line_ending).unwrap_err().to_string(),
                    t.1.as_ref().unwrap_err().to_string(),
                );
            }
        }

        Ok(())
    }

    #[test]
    fn gitignore_rejects_flags_that_do_not_fit_the_format() {
        let validate = |extra: &[&str]| -> Result<()> {
            let mut args = vec![
                String::from("omegasort"),
                String::from("--sort"),
                String::from("gitignore"),
            ];
            args.extend(extra.iter().map(ToString::to_string));
            args.push(String::from("ignored.txt"));
            Cli::new_from_args(args)?.validate_args()
        };

        for extra in [
            vec!["--reverse"],
            vec!["--windows"],
            vec!["--comment-prefix", "#"],
        ] {
            assert!(
                validate(&extra).is_err(),
                "{extra:?} is rejected when sorting a gitignore file",
            );
        }

        for extra in [
            vec![],
            vec!["--unique"],
            vec!["--case-insensitive"],
            vec!["--locale", "en-US"],
        ] {
            assert!(
                validate(&extra).is_ok(),
                "{extra:?} is accepted when sorting a gitignore file",
            );
        }
    }

    #[test]
    fn a_bom_stays_at_the_front_of_the_file() -> Result<()> {
        let sorted = |strategy: &str, extra: &[&str], content: &str| -> Result<String> {
            let td = tempdir()?;
            let mut filename = td.path().to_path_buf();
            filename.push("input.txt");
            write(&filename, content)?;

            let mut args = vec![
                String::from("omegasort"),
                String::from("--sort"),
                String::from(strategy),
                String::from("--in-place"),
            ];
            args.extend(extra.iter().map(|a| String::from(*a)));
            args.push(filename.to_string_lossy().to_string());
            Cli::new_from_args(args)?.execute()?;

            Ok(read_to_string(filename)?)
        };

        assert_eq!(
            sorted("gitignore", &[], "\u{feff}zebra\napple\n")?,
            "\u{feff}apple\nzebra\n",
            "the BOM does not travel with the line it was in front of",
        );
        assert_eq!(
            sorted("text", &[], "\u{feff}zebra\napple\n")?,
            "\u{feff}apple\nzebra\n",
            "every sorting method leaves the BOM at the front, not just this one",
        );
        assert_eq!(
            sorted("gitignore", &[], "\u{feff}!foo\n!bar\nbaz\n")?,
            "\u{feff}!bar\n!foo\nbaz\n",
            "the first line is a negation, as it is to git, so it groups with the next one",
        );
        assert_eq!(
            sorted("gitignore", &[], "zb\nza\n\u{feff}x\nb\na\n")?,
            "za\nzb\n\u{feff}x\na\nb\n",
            "a BOM after the first line is part of the pattern, so that line stays where it is \
             and splits the run in two. Sorting it to the front would turn a `<BOM>!foo` into \
             the negation `!foo` and change what the file ignores.",
        );
        assert_eq!(
            sorted("gitignore", &["--unique"], "a\n\u{feff}x\na\n")?,
            "\u{feff}\u{feff}x\na\n",
            "--unique can drop every line above a BOM line and leave it first. It gets a BOM \
             written in front of it so that git still reads it as a pattern for a file whose \
             name starts with a mark, not as the pattern `x`.",
        );
        assert_eq!(
            sorted("text", &["--locale", "en-US"], "zzz\n\u{feff}aaa\n")?,
            "\u{feff}\u{feff}aaa\nzzz\n",
            "a collator can sort a BOM line to the front of a file that had no BOM, so this is \
             not only a gitignore problem. Without the extra BOM the line would lose its own.",
        );

        Ok(())
    }

    #[test]
    fn bak_file_by_default() -> Result<()> {
        let td = tempdir()?;
        let mut filename = td.path().to_path_buf();
        filename.push("input.txt");
        let orig_content = "foo\nbar\nbaz\n";
        write(&filename, orig_content)?;

        let cli = Cli::new_from_args([
            String::from("omegasort"),
            String::from("--sort"),
            String::from("text"),
            filename.to_string_lossy().to_string(),
        ])?;

        cli.execute()?;

        let mut new_filename = td.path().to_path_buf();
        new_filename.push("input.txt.bak");

        assert_eq!(read_to_string(new_filename)?, orig_content);
        assert_eq!(read_to_string(filename)?, "bar\nbaz\nfoo\n");

        Ok(())
    }

    #[test]
    fn do_not_rewrite_sorted_file() -> Result<()> {
        let td = tempdir()?;
        let mut filename = td.path().to_path_buf();
        filename.push("input.txt");
        write(&filename, "bar\nbaz\nfoo\n")?;

        let orig_meta = metadata(&filename)?;

        let cli = Cli::new_from_args([
            String::from("omegasort"),
            String::from("--sort"),
            String::from("text"),
            String::from("--in-place"),
            filename.to_string_lossy().to_string(),
        ])?;

        cli.execute()?;

        let new_meta = metadata(&filename)?;
        assert_eq!(orig_meta.modified()?, new_meta.modified()?);

        Ok(())
    }

    #[test]
    fn integration() -> Result<()> {
        let mut test_case_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_case_dir.push("./src/test-cases");

        let paths = read_dir(test_case_dir)?;
        let mut files = vec![];
        for path in paths {
            let path = path?.path();
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy() == "test" {
                    files.push(path);
                }
            }
        }

        files.sort();
        for file in files {
            run_one_integration_test(file)?;
        }

        Ok(())
    }

    fn run_one_integration_test(path: PathBuf) -> Result<()> {
        println!("{}", path.file_name().unwrap().to_string_lossy());

        let case = read_to_string(path)?.replace('\r', "");
        let mut elts = case.split("####\n");
        let mut args = vec![String::from("omegasort")];
        args.append(
            &mut elts
                .next()
                .unwrap()
                .trim()
                .split(' ')
                .map(String::from)
                .collect::<Vec<_>>(),
        );
        let expected_check_failure = elts.next().unwrap().trim();
        let input = elts.next().unwrap().trim_start();
        let expect = elts.next().unwrap().trim_start();

        let td = tempdir()?;
        let mut filename = td.path().to_path_buf();
        filename.push("input.txt");
        write(&filename, input)?;

        let mut check_args = args.clone();
        check_args.append(&mut vec![
            String::from("--check"),
            filename.to_string_lossy().to_string(),
        ]);

        let cli = Cli::new_from_args(check_args)?;
        let res = cli.execute();
        assert!(
            res.is_err(),
            "file is not sorted so --check should not pass",
        );
        let e = res.unwrap_err();
        let dc = e.downcast_ref::<CheckError>();
        assert!(dc.is_some(), "got a CheckError from execute: {e}");
        let check_error = dc.unwrap();
        match expected_check_failure {
            "HasUnexpectedEmptyLines" => assert!(
                matches!(check_error, CheckError::HasUnexpectedEmptyLines),
                "check_error ({check_error:?}) is a HasUnexpectedEmptyLines error"
            ),
            "NotSorted" => assert!(
                matches!(check_error, CheckError::NotSorted { .. }),
                "check_error ({check_error:?}) is a NotSorted error "
            ),
            "NotUnique" => assert!(
                matches!(check_error, CheckError::NotUnique { .. }),
                "check_error ({check_error:?}) is a NotUnique error from --check"
            ),
            _ => unreachable!(
                "unexpected expected_check_failure value in test file: {expected_check_failure}"
            ),
        }

        let mut sort_args = args.clone();
        sort_args.append(&mut vec![
            String::from("--in-place"),
            filename.to_string_lossy().to_string(),
        ]);
        let cli = Cli::new_from_args(sort_args)?;
        let res = cli.execute();
        assert!(res.is_ok(), "no error sorting file: {res:?}");

        assert_eq!(read_to_string(&filename)?, expect);

        // What the sorter produced has to pass the sorter's own check, and sorting it a second time
        // has to leave it alone. Without this a case can pass while `--check` still rejects the
        // output it asked for.
        let mut recheck_args = args.clone();
        recheck_args.append(&mut vec![
            String::from("--check"),
            filename.to_string_lossy().to_string(),
        ]);
        let res = Cli::new_from_args(recheck_args)?.execute();
        assert!(res.is_ok(), "sorted output passes --check: {res:?}");

        let mut resort_args = args;
        resort_args.append(&mut vec![
            String::from("--in-place"),
            filename.to_string_lossy().to_string(),
        ]);
        Cli::new_from_args(resort_args)?.execute()?;
        assert_eq!(
            read_to_string(&filename)?,
            expect,
            "sorting the output again does not change it",
        );

        Ok(())
    }
}
