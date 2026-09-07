//! Support for sorting gitignore files.
//!
//! Gitignore uses the *last* pattern matching a path to decide whether that path is ignored, so
//! moving lines around can change what a file ignores.
//!
//! To make that impossible, a gitignore file is broken into groups and lines are only ever sorted
//! within a group. A blank line, a comment, a line starting with a byte order mark, and every
//! switch between `!` and non-`!` is a fence, and no line ever crosses one. So a group is a run of
//! consecutive patterns that all have the same polarity and none of which start with a BOM.
//!
//! That is safe no matter what the patterns are. Git's verdict for a path is the polarity of the
//! last line matching it. Any permutation of a run breaks down into swaps of adjacent lines inside
//! it, and swapping two adjacent lines of the same polarity cannot change which polarity ends up
//! last. The argument never mentions globs, `**`, or character classes. The only things it needs to
//! know about a line are whether it starts with `!` and whether it starts with a BOM. The BOM
//! matters because git skips one at the very start of a file, so `<BOM>!foo` is the negation `!foo`
//! on the first line and a pattern for a name beginning with a BOM anywhere else.
//!
//! This does mean the sort stops at boundaries the author drew, so a file with several
//! comment-headed blocks is tidied block by block rather than turned into one list. That is the
//! point. Those boundaries usually carry meaning, and merging across them is exactly what would
//! change behavior.

use crate::{LineKind, SortableLine};
use std::collections::HashMap;

/// A single gitignore pattern, broken up into the syntactic markers that decorate it and the path
/// text those markers apply to.
//
// These are four independent markers that a pattern either carries or does not, so there is nothing
// to turn into a state machine or an enum.
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct GitignorePattern<'a> {
    pub(crate) negated: bool,
    pub(crate) anchored: bool,
    pub(crate) dir_only: bool,
    pub(crate) double_star: bool,
    pub(crate) path: &'a str,
}

/// Trims trailing spaces from a line the way git does.
///
/// Git drops the last run of ASCII spaces from every gitignore line, so `foo ` is the pattern
/// `foo`. Only spaces count, not tabs or other whitespace. A `\` escapes whatever follows it, so
/// `foo\ ` really is a pattern for a file called `foo `, and the space it escapes ends the run.
///
/// This is `trim_trailing_spaces` in git's `dir.c`.
fn trim_trailing_spaces(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut last_space = None;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b' ' => {
                if last_space.is_none() {
                    last_space = Some(i);
                }
            }
            b'\\' => {
                // Whatever comes next is part of the pattern, so any run of spaces ends here. A
                // trailing `\` with nothing after it leaves the line alone.
                i += 1;
                if i >= bytes.len() {
                    return line;
                }
                last_space = None;
            }
            _ => last_space = None,
        }
        i += 1;
    }

    // Only a byte equal to `b' '` is ever used as the split point, and no continuation byte of a
    // multi-byte character can equal it, so this is always on a character boundary.
    match last_space {
        Some(i) => &line[..i],
        None => line,
    }
}

impl<'a> GitignorePattern<'a> {
    pub(crate) fn new(line: &'a str) -> Self {
        let mut line = trim_trailing_spaces(line);

        // A leading `\` escapes the `!` or `#` that follows it, so such a line is not a negation
        // and we sort it by the literal character.
        let negated = if line.starts_with(r"\!") || line.starts_with(r"\#") {
            line = &line[1..];
            false
        } else if let Some(r) = line.strip_prefix('!') {
            line = r;
            true
        } else {
            false
        };

        let mut anchored = match line.strip_prefix('/') {
            Some(r) if !r.is_empty() => {
                line = r;
                true
            }
            _ => false,
        };

        let dir_only = match line.strip_suffix('/') {
            Some(r) if !r.is_empty() => {
                line = r;
                true
            }
            _ => false,
        };

        // A leading `**/` means "at any depth", so it is not an anchor even when it was written as
        // `/**/`. Git treats `/**/a/b` and `**/a/b` as the same pattern, and both differ from
        // `a/b`.
        let double_star = if line.starts_with("**/") {
            anchored = false;
            // Git documents `**/foo` as meaning the same thing as `foo`, so we drop the prefix and
            // sort the two together. That only holds when nothing else is left, since `**/a/b`
            // matches `b` in any `a` directory while `a/b` is anchored to the top.
            match line.strip_prefix("**/") {
                Some(r) if !r.is_empty() && !r.contains('/') => {
                    line = r;
                    true
                }
                _ => false,
            }
        } else {
            false
        };

        Self {
            negated,
            anchored,
            dir_only,
            double_star,
            path: line,
        }
    }
}

/// Returns a key identifying the pattern for the sake of `--unique`.
///
/// Two lines share a key only when they are the same pattern spelled two ways, which in practice
/// means `foo`, `**/foo`, and `/**/foo`.
pub(crate) fn unique_key(line: &str) -> String {
    let pattern = GitignorePattern::new(line);
    // The markers have to stay in the key. Otherwise `\!foo`, which is the literal file `!foo`,
    // would collide with `!foo`, which negates `foo`.
    format!(
        "{}\0{}\0{}\0{}",
        pattern.negated, pattern.anchored, pattern.dir_only, pattern.path,
    )
}

/// Assigns each line of a gitignore file to a group of lines that may be sorted among themselves.
/// Feed it the lines of the file in order.
#[derive(Default)]
pub(crate) struct Grouper {
    group: usize,
    /// The polarity of the run being built, if one is open.
    open_run: Option<bool>,
}

impl Grouper {
    pub(crate) fn next(&mut self, line: &str) -> (usize, LineKind) {
        // Blank lines and comments are fences. Each gets a group to itself so that it cannot move,
        // and it closes whatever run was open. Note that the `#` has to be in the first column,
        // since ` #foo` is a pattern for a file named `#foo` in a directory. A line of nothing but
        // spaces is blank to git, which strips them, but a line of tabs is not, so that one is a
        // pattern like any other.
        //
        // A line starting with a byte order mark is a fence too, for a different reason. Git skips
        // one mark at the very start of a file, so what such a line means depends on where it sits:
        // `<BOM>!foo` is the negation `!foo` on the first line and a pattern for a file whose name
        // begins with a mark anywhere else. The file's own mark is taken off before the lines get
        // here, so any that is left is an interior one, and moving it to the front would change
        // what it means.
        if trim_trailing_spaces(line).is_empty()
            || line.starts_with('#')
            || line.starts_with('\u{feff}')
        {
            self.open_run = None;
            self.group += 1;
            return (self.group, LineKind::Fence);
        }

        // A run continues only while the polarity stays the same. Crossing from a pattern to a
        // negation, or back, is the one move that can change what the file ignores, so it starts a
        // new group.
        let negated = GitignorePattern::new(line).negated;
        if self.open_run != Some(negated) {
            self.open_run = Some(negated);
            self.group += 1;
        }

        (self.group, LineKind::Sortable)
    }
}

/// Removes duplicate patterns, keeping the *last* copy of each.
///
/// Keeping the last one is not a matter of taste, it is what makes this safe.  A line matches
/// exactly what a later copy of itself matches, so the earlier copy is never the last line matching
/// any path and dropping it cannot change a verdict. Dropping the later copy could: given `foo`,
/// `!foo`, `foo` the file ignores `foo`, but keeping only the first `foo` does not.
///
/// Fences are never removed, so two blank lines stay two blank lines.
///
/// The groups are recomputed on the way out. Removing a line can empty a whole group, which leaves
/// the runs that were on either side of it next to each other. Those two runs have the same
/// polarity, so putting them together is safe, and doing it now is what makes sorting an already
/// sorted file a no-op. If we left it, the merge would happen on the next run instead and the file
/// would keep changing.
pub(crate) fn dedup_keeping_last(lines: Vec<SortableLine>) -> Vec<SortableLine> {
    let mut last_seen = HashMap::new();
    for (i, line) in lines.iter().enumerate() {
        if line.kind == LineKind::Sortable {
            last_seen.insert(unique_key(&line.line), i);
        }
    }

    let mut grouper = Grouper::default();
    lines
        .into_iter()
        .enumerate()
        .filter(|(i, line)| {
            line.kind == LineKind::Fence || last_seen.get(&unique_key(&line.line)) == Some(i)
        })
        .map(|(_, mut line)| {
            let (group, kind) = grouper.next(&line.line);
            line.group = group;
            line.kind = kind;
            line
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::{dedup_keeping_last, trim_trailing_spaces, unique_key, GitignorePattern, Grouper};
    use crate::{LineKind, SortableLine};
    use test_log::test;

    fn groups(lines: &[&str]) -> Vec<usize> {
        let mut grouper = Grouper::default();
        lines.iter().map(|l| grouper.next(l).0).collect()
    }

    /// Turns group numbers into letters so that a test can say what the shape of the file is
    /// without caring which numbers were handed out.
    fn shape(lines: &[&str]) -> String {
        let groups = groups(lines);
        let mut seen = vec![];
        groups
            .into_iter()
            .map(|g| {
                let i = seen.iter().position(|s| *s == g).unwrap_or_else(|| {
                    seen.push(g);
                    seen.len() - 1
                });
                char::from(b'a' + u8::try_from(i).unwrap())
            })
            .collect()
    }

    #[test]
    fn a_run_of_one_polarity_is_one_group() {
        assert_eq!(shape(&["a", "b", "c"]), "aaa", "a flat file is one group");
        assert_eq!(
            shape(&["a", "!b", "c"]),
            "abc",
            "switching polarity twice makes three groups",
        );
        assert_eq!(
            shape(&["a", "!b", "!c", "d"]),
            "abbc",
            "consecutive negations share a group",
        );
    }

    #[test]
    fn blank_lines_and_comments_are_fences() {
        assert_eq!(
            shape(&["a", "", "b"]),
            "abc",
            "a blank line splits a run in two",
        );
        assert_eq!(
            shape(&["a", "# c", "b"]),
            "abc",
            "a comment splits a run in two",
        );
        assert_eq!(
            shape(&["a", "   ", "b"]),
            "abc",
            "a whitespace-only line is blank, since git strips trailing spaces",
        );
        assert_eq!(
            shape(&["a", "\t", "b"]),
            "aaa",
            "a tab-only line is a pattern, since git only strips spaces",
        );
        assert_eq!(
            shape(&[" #a", " #b"]),
            "aa",
            "a `#` that is not in the first column is part of a pattern",
        );
        assert_eq!(
            shape(&["#a", "#b"]),
            "ab",
            "each comment is a fence of its own, so comments never reorder",
        );
    }

    #[test]
    fn which_lines_are_fences() {
        let mut grouper = Grouper::default();
        let kinds = ["a", "", "# c", "!d", "   ", r"\!e", " #f", "\u{feff}g"]
            .into_iter()
            .map(|l| grouper.next(l).1)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                LineKind::Sortable,
                LineKind::Fence,
                LineKind::Fence,
                LineKind::Sortable,
                LineKind::Fence,
                LineKind::Sortable,
                LineKind::Sortable,
                LineKind::Fence,
            ],
        );
    }

    #[test]
    fn a_line_starting_with_a_byte_order_mark_is_a_fence() {
        assert_eq!(
            shape(&["a", "\u{feff}b", "c"]),
            "abc",
            "the line cannot move, so the run around it is split in two",
        );
        assert_eq!(
            shape(&["\u{feff}a", "\u{feff}b"]),
            "ab",
            "neither of them can move, so they do not sort against each other",
        );
    }

    #[test]
    fn an_escaped_bang_is_not_a_polarity_switch() {
        assert_eq!(
            shape(&["a", r"\!b", "c"]),
            "aaa",
            r"`\!b` is the file `!b`, not a negation",
        );
    }

    fn deduped(lines: &[&str]) -> Vec<String> {
        let mut grouper = Grouper::default();
        let lines = lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let (group, kind) = grouper.next(l);
                SortableLine::for_test(i + 1, l, group, kind)
            })
            .collect::<Vec<_>>();
        dedup_keeping_last(lines)
            .into_iter()
            .map(|l| l.line)
            .collect()
    }

    fn deduped_shape(lines: &[&str]) -> String {
        let mut grouper = Grouper::default();
        let lines = lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let (group, kind) = grouper.next(l);
                SortableLine::for_test(i + 1, l, group, kind)
            })
            .collect::<Vec<_>>();
        let mut seen = vec![];
        dedup_keeping_last(lines)
            .into_iter()
            .map(|l| {
                let i = seen.iter().position(|s| *s == l.group).unwrap_or_else(|| {
                    seen.push(l.group);
                    seen.len() - 1
                });
                char::from(b'a' + u8::try_from(i).unwrap())
            })
            .collect()
    }

    #[test]
    fn dedup_keeps_the_last_copy() {
        assert_eq!(
            deduped(&["foo", "!foo", "foo"]),
            ["!foo", "foo"],
            "the surviving `foo` is the one that was last, so it still wins",
        );
        assert_eq!(
            deduped(&["foo", "**/foo", "/**/foo"]),
            ["/**/foo"],
            "the three spellings of one pattern are duplicates",
        );
        assert_eq!(
            deduped(&["foo", r"\!foo", "!foo"]),
            ["foo", r"\!foo", "!foo"],
            "the markers are part of the key, so these are three patterns",
        );
        assert_eq!(
            deduped(&["a", "", "", "a"]),
            ["", "", "a"],
            "blank lines are never removed as duplicates of each other",
        );
    }

    #[test]
    fn dedup_merges_the_runs_around_a_group_it_empties() {
        assert_eq!(
            deduped(&["a", "!x", "b", "!x"]),
            ["a", "b", "!x"],
            "the first `!x` goes, so the two patterns end up next to each other",
        );
        assert_eq!(
            deduped_shape(&["a", "!x", "b", "!x"]),
            "aab",
            "and they are one group, so a second sort would not move them again",
        );
        assert_eq!(
            deduped_shape(&["a", "!x", "b", "!x", "", "c"]),
            "aabcd",
            "an emptied group does not merge runs that a fence still separates",
        );
    }

    #[test]
    fn trailing_spaces_are_trimmed_like_git_does() {
        let cases = [
            ("foo", "foo"),
            ("foo ", "foo"),
            ("foo   ", "foo"),
            ("a b", "a b"),
            ("a b  ", "a b"),
            ("   ", ""),
            // Only spaces are stripped, so this line is a pattern for a file whose name is a tab.
            ("\t", "\t"),
            (" \t ", " \t"),
            // A `\` escapes the space after it, so that space is part of the pattern and only the
            // unescaped one after it goes.
            (r"foo\ ", r"foo\ "),
            (r"foo\  ", r"foo\ "),
            // A `\` with nothing after it leaves the whole line alone, which is what git does
            // rather than trimming past the end.
            (r"\", r"\"),
            (r"a \", r"a \"),
        ];
        for (line, expect) in cases {
            assert_eq!(trim_trailing_spaces(line), expect, "trimmed `{line}`");
        }
    }

    #[test]
    fn pattern_parsing() {
        let cases = [
            // line, negated, anchored, dir_only, double_star, path
            ("foo", false, false, false, false, "foo"),
            ("!foo", true, false, false, false, "foo"),
            ("/foo", false, true, false, false, "foo"),
            ("foo/", false, false, true, false, "foo"),
            ("!/foo/", true, true, true, false, "foo"),
            (r"\!foo", false, false, false, false, "!foo"),
            (r"\#foo", false, false, false, false, "#foo"),
            ("/", false, false, false, false, "/"),
            ("**/foo", false, false, false, true, "foo"),
            // A `/**/` prefix is "at any depth", not an anchor, so this is the same pattern as
            // `**/foo` and as `foo`.
            ("/**/foo", false, false, false, true, "foo"),
            ("/**/foo/", false, false, true, true, "foo"),
            // With more path left the prefix stays, since `**/a/b` matches `b` in any `a` directory
            // while `a/b` does not.
            ("**/a/b", false, false, false, false, "**/a/b"),
            ("/**/a/b", false, false, false, false, "**/a/b"),
            ("a/b", false, false, false, false, "a/b"),
            ("/**", false, true, false, false, "**"),
            // Git strips the trailing spaces before it does anything else, so these are the same
            // patterns as the ones without them.
            ("foo  ", false, false, false, false, "foo"),
            ("!/foo/ ", true, true, true, false, "foo"),
            (r"foo\ ", false, false, false, false, r"foo\ "),
        ];
        for (line, negated, anchored, dir_only, double_star, path) in cases {
            let pattern = GitignorePattern::new(line);
            assert_eq!(pattern.negated, negated, "negated for `{line}`");
            assert_eq!(pattern.anchored, anchored, "anchored for `{line}`");
            assert_eq!(pattern.dir_only, dir_only, "dir_only for `{line}`");
            assert_eq!(pattern.double_star, double_star, "double_star for `{line}`");
            assert_eq!(pattern.path, path, "path for `{line}`");
        }
    }

    #[test]
    fn spellings_of_one_pattern_share_a_unique_key() {
        for same in [
            ["foo", "**/foo"],
            ["foo", "/**/foo"],
            ["**/a/b", "/**/a/b"],
            ["!foo", "!**/foo"],
        ] {
            assert_eq!(
                unique_key(same[0]),
                unique_key(same[1]),
                "`{}` and `{}` are the same pattern",
                same[0],
                same[1],
            );
        }

        for same_with_spaces in [["foo", "foo  "], ["!/foo/", "!/foo/ "]] {
            assert_eq!(
                unique_key(same_with_spaces[0]),
                unique_key(same_with_spaces[1]),
                "`{}` and `{}` are the same pattern, since git strips the trailing spaces",
                same_with_spaces[0],
                same_with_spaces[1],
            );
        }

        for different in [
            ["foo", "/foo"],
            // Trimming the spaces first is what keeps these apart. Untrimmed, both would look like
            // a negation of the path ` `.
            ["! ", "!**/ "],
            [r"foo\ ", "foo"],
            ["foo", "foo/"],
            ["foo", "!foo"],
            [r"\!foo", "!foo"],
            ["a/b", "**/a/b"],
        ] {
            assert_ne!(
                unique_key(different[0]),
                unique_key(different[1]),
                "`{}` and `{}` are different patterns",
                different[0],
                different[1],
            );
        }
    }
}
