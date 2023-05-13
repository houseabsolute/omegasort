extern crate alloc;

mod collation;
mod comparer;
mod error;
mod logging;
mod sorter;

use crate::error::CheckError;
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
        Err(e) => match e.downcast_ref::<clap::Error>() {
            Some(e) => e.exit(),
            _ => {
                error!("{e}");
                42
            }
        },
    };
    std::process::exit(status);
}

fn long_help() -> String {
    const HELP: &str = r#"
There are a number of different sorting methods available.

## Text (`--sort text`)

This sorts each line of the file as text without any special parsing. The exact sorting is determined by the `--locale`, `--case-insensitive`, and `--reverse` flags.

## Numbered Text (`--sort numbered-text`)

This assumes that each line of the file starts with a numeric value, optionally followed by non-numeric text.

Lines should not have any leading space before the number. The number can either be an integer (including 0) or a simple float (no scientific notation).

The lines will be sorted numerically first. If two lines have the same number they will be sorted by text as above.

Lines without numbers always sort after lines with numbers.

This sorting method accepts the `--locale`, `--case-insensitive`, and `--reverse` flags.

## Datetime (`--sort datetime-text`)

This sorting method assumes that each line starts with a date or datetime, without any space in it. That means that a string with both a date *and* a time needs to be in a format like "2019-08-27T19:13:16".

Lines should not have any leading space before the datetime.

This sorting method accepts the `--locale`, `--case-insensitive`, and `--reverse` flags.

## Path (`--sort path`)

Each line is treated as a path.

The paths are sorted by the following rules:

* Absolute paths come before relative.
* Paths are sorted by depth before sorting by the path content, so /z comes before /a/a.
* If you pass the `--windows` flag, then paths with drive letters or UNC names are sorted based on that prefix first. Paths with drive letters or UNC names sort before paths without them.

This sorting method accepts the `--locale`, `--case-insensitive`, and `--reverse` flags in addition to the `--windows` flag.

## IP (`--sort ip`)

This method assumes that each line is an IPv4 or IPv6 address (not a network).

The sorting method is the same as if each line were the corresponding integer for the address. IPv4 addresses always sort before IPv6 addresses.

This sorting method accepts the `--reverse` flag.

## Network (`--sort network`)

This method assumes that each line is an IPv4 or IPv6 network in CIDR notation.

If there are two networks with the same base address they are sorted with the larger network first (so 1.1.1.0/24 comes before 1.1.1.0/28). IPv4 networks always sort before IPv6 networks.

This sorting method accepts the `--reverse` flag.
"#;

    let skin = MadSkin::default();
    format!("{}", skin.text(HELP, Some(MAX_TERM_WIDTH)))
}

impl Cli {
    fn new_from_args<I, T>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let command = Cli::command();
        Cli::from_arg_matches(&command.get_matches_from(args)).map_err(|e| e.into())
    }

    fn run(&self) -> i32 {
        if let Err(e) = logging::init(self.debug) {
            error!("{}", e);
            return 100;
        }

        if let Err(e) = self.validate_args() {
            error!("{}", e);
            return 101;
        }

        if let Err(e) = self.execute() {
            error!("{}", e);
            let status = match e.downcast::<CheckError>() {
                Ok(CheckError::HasUnexpectedEmptyLines { .. } | CheckError::NotSorted { .. })
                | Ok(CheckError::NotUnique { .. }) => 1,
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
        let (lines, has_empty_lines, line_ending) =
            read_lines(&self.file, self.comment_prefix.as_deref())?;
        if self.check {
            if has_empty_lines {
                return Err(CheckError::HasUnexpectedEmptyLines.into());
            }
            if sorter.lines_are_sorted(&lines)? {
                return Ok(());
            }
        }

        self.sort_lines(lines, has_empty_lines, line_ending, sorter)
    }

    fn sort_lines(
        &self,
        lines: Vec<SortableLine>,
        has_empty_lines: bool,
        line_ending: &'static str,
        sorter: Sorter,
    ) -> Result<()> {
        let orig_hash = if has_empty_lines {
            None
        } else {
            Some(hash_lines(&lines))
        };
        let lines = sorter.sort_lines(lines)?;
        if !has_empty_lines {
            let new_hash = hash_lines(&lines);
            if orig_hash.unwrap() == new_hash && !self.stdout {
                debug!("file is already sorted");
                return Ok(());
            }
        }

        if self.stdout {
            return write_lines_to_writer(lines, line_ending, &mut stdout());
        }

        if !self.in_place {
            let mut bak_file = self.file.clone();
            let ext = bak_file
                .extension()
                .map(|e| e.to_str().unwrap_or(""))
                .unwrap_or("");
            bak_file.set_extension(if ext.is_empty() {
                String::from("bak")
            } else {
                format!("{ext}.bak")
            });
            copy(&self.file, bak_file)?;
        }

        // If we don't make this in the same directory as the original file,
        // then the `persist` call later may fail because we can rename files
        // across filesystems.
        let mut file = NamedTempFile::new_in(self.file.parent().unwrap())?;
        write_lines_to_writer(lines, line_ending, &mut file)?;
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
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Comment {
    is_preceded_by_empty_line: bool,
    lines: Vec<String>,
}

impl SortableLine {
    // This is only used in tests.
    #[allow(dead_code)]
    fn from_number_and_str(from: (usize, &str)) -> Self {
        Self {
            line_number: from.0,
            line: from.1.to_string(),
            comment: None,
        }
    }
}

fn read_lines<P: AsRef<Path>>(
    file: P,
    comment_prefix: Option<&str>,
) -> Result<(Vec<SortableLine>, bool, &'static str)> {
    let mut f = File::open(file.as_ref())?;
    let (read, line_ending) = determine_line_ending(&mut f)?;
    let (lines, has_empty_lines) = lines_from_reader(comment_prefix, read)?;
    Ok((lines, has_empty_lines, line_ending))
}

fn lines_from_reader<R: Read>(
    comment_prefix: Option<&str>,
    read: R,
) -> Result<(Vec<SortableLine>, bool)> {
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
            match comment {
                Some(ref mut comment) => comment.lines.push(line),
                None => {
                    comment = Some(Comment {
                        lines: vec![line],
                        is_preceded_by_empty_line: last_line_was_empty,
                    });
                    last_line_was_empty = false;
                }
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
        });
        last_line_was_empty = false;
        comment = None;
    }
    Ok((lines, has_empty_lines))
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

fn write_lines_to_writer<W: Write>(
    lines: Vec<SortableLine>,
    line_ending: &'static str,
    out: &mut W,
) -> Result<()> {
    let mut bw = BufWriter::new(out);
    for (i, l) in lines.into_iter().enumerate() {
        if let Some(comment) = l.comment {
            // If the comment is the first thing in the file we don't preserve
            // its leading empty line.
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

const FIRST_CHUNK_SIZE: usize = 2048;

const LINE_ENDINGS: [&str; 3] = ["\r\n", "\n", "\r"];

type LineEndingChain<'a> = (Chain<Cursor<Vec<u8>>, &'a mut File>, &'static str);

fn determine_line_ending(file: &mut File) -> Result<LineEndingChain> {
    let mut buf = [0; FIRST_CHUNK_SIZE];
    let read = file.read(&mut buf)?;

    for le in LINE_ENDINGS {
        if buf_contains_str(le, &buf) {
            return Ok((Cursor::new(Vec::from(&buf[0..read])).chain(file), le));
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

    use super::{Comment, SortableLine};
    use anyhow::Result;
    use std::{
        fs::{metadata, read_dir, read_to_string, write, File},
        io::Write,
        path::PathBuf,
    };
    use tempfile::tempdir;

    const WITH_COMMENTS: &str = r#"
foo
bar
# comment 1
baz

# comment 2
quux
"#;

    const WITH_REPEATED_LINES: &str = r#"
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
"#;

    #[test]
    fn lines_from_reader() -> Result<()> {
        let lines = ["foo", "bar", "baz", "quux"]
            .map(|l| format!("{l}\n"))
            .join("");
        assert_eq!(
            super::lines_from_reader(None, lines.trim().as_bytes())?,
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
            super::lines_from_reader(None, lines.trim().as_bytes())?,
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
            super::lines_from_reader(None, WITH_COMMENTS.trim_start().as_bytes())?,
            (
                vec![
                    SortableLine {
                        line_number: 1,
                        line: "foo".to_string(),
                        comment: None,
                    },
                    SortableLine {
                        line_number: 2,
                        line: "bar".to_string(),
                        comment: None,
                    },
                    SortableLine {
                        line_number: 3,
                        line: "# comment 1".to_string(),
                        comment: None,
                    },
                    SortableLine {
                        line_number: 4,
                        line: "baz".to_string(),
                        comment: None,
                    },
                    SortableLine {
                        line_number: 6,
                        line: "# comment 2".to_string(),
                        comment: None,
                    },
                    SortableLine {
                        line_number: 7,
                        line: "quux".to_string(),
                        comment: None,
                    },
                ],
                true,
            ),
        );

        assert_eq!(
            super::lines_from_reader(Some("#"), WITH_COMMENTS.trim_start().as_bytes())?,
            (
                vec![
                    SortableLine {
                        line_number: 1,
                        line: "foo".to_string(),
                        comment: None,
                    },
                    SortableLine {
                        line_number: 2,
                        line: "bar".to_string(),
                        comment: None,
                    },
                    SortableLine {
                        line_number: 4,
                        line: "baz".to_string(),
                        comment: Some(Comment {
                            lines: vec!["# comment 1".to_string()],
                            is_preceded_by_empty_line: false,
                        }),
                    },
                    SortableLine {
                        line_number: 7,
                        line: "quux".to_string(),
                        comment: Some(Comment {
                            lines: vec!["# comment 2".to_string()],
                            is_preceded_by_empty_line: true,
                        }),
                    },
                ],
                false
            ),
        );

        Ok(())
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
            let (lines, _) = super::lines_from_reader(t.comment_marker, t.input.as_bytes())?;
            super::write_lines_to_writer(lines, "\n", &mut buf)?;
            assert_eq!(unsafe { String::from_utf8_unchecked(buf) }, t.expect);
        }

        Ok(())
    }

    #[test]
    fn determine_line_ending() -> Result<()> {
        let mut long_str = "Lorem ipsum dolor sit amet".repeat(100);
        long_str.push('\n');
        let tests: &[(&str, Result<&str>)] = &[
            (
                "Lorem ipsum dolor sit amet\nconsectetur adipiscing elit",
                Ok("\n"),
            ),
            (
                "Lorem ipsum dolor sit amet\rconsectetur adipiscing elit",
                Ok("\r"),
            ),
            (
                "Lorem ipsum dolor sit amet\r\nconsectetur adipiscing elit",
                Ok("\r\n"),
            ),
            (
                "Lorem ipsum dolor sit amet\tconsectetur adipiscing elit",
                Err(super::could_not_determine_line_ending()),
            ),
            (
                long_str.as_str(),
                Err(super::could_not_determine_line_ending()),
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
            let le = super::determine_line_ending(&mut file).map(|le| le.1);
            if let Ok(expect) = t.1 {
                assert_eq!(le?, expect);
            } else {
                assert!(le.is_err());
                assert_eq!(
                    le.unwrap_err().to_string(),
                    t.1.as_ref().unwrap_err().to_string(),
                );
            }
        }

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

    // This test fails on mip64 Linux for some reason. The error is:
    //
    //    thread 'test::do_not_rewrite_sorted_file' panicked at 'assertion failed: tv_nsec >= 0 && tv_nsec < NSEC_PER_SEC as i64', library/std/src/sys/unix/time.rs:77:9
    #[test]
    #[cfg(not(all(
        target_os = "linux",
        any(target_arch = "mips64", target_arch = "mips64el")
    )))]
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
                matches!(check_error, CheckError::HasUnexpectedEmptyLines { .. }),
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

        assert_eq!(read_to_string(filename)?, expect);

        Ok(())
    }
}
