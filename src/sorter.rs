use crate::{
    collation::collator_for_locale,
    comparer::{
        Comparer, DatetimeTextComparer, GitignoreComparer, IpComparer, NetworkComparer,
        NumberedTextComparer, PathComparer, PathType, TextComparer,
    },
    error::CheckError,
    gitignore::{dedup_keeping_last, unique_key},
    LineKind, SortableLine,
};
use anyhow::Result;
use clap::ValueEnum;
use rayon::prelude::*;
use std::{
    cmp::Ordering,
    collections::{hash_map::Entry, HashMap},
    sync::Mutex,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum Strategy {
    Text,
    NumberedText,
    DatetimeText,
    Path,
    Gitignore,
    Ip,
    Network,
}

impl Strategy {
    pub(crate) fn supports_locale(self) -> bool {
        !matches!(self, Strategy::Ip | Strategy::Network)
    }

    pub(crate) fn supports_path_type(self) -> bool {
        matches!(self, Strategy::Path)
    }

    /// Whether sorting in reverse makes any sense. It does not for gitignore.  Putting the groups
    /// back in the opposite order would change what the file ignores, and reversing only within
    /// each group gives an order that is neither the file backwards nor the sort backwards.
    pub(crate) fn supports_reverse(self) -> bool {
        !matches!(self, Strategy::Gitignore)
    }

    /// Whether the file has a structure of its own that sorting must not destroy. A gitignore file
    /// does: its blank lines, comments, and the boundaries between patterns and negations all carry
    /// meaning.
    pub(crate) fn keeps_file_structure(self) -> bool {
        matches!(self, Strategy::Gitignore)
    }
}

pub(crate) struct Sorter {
    comparer: Box<dyn Comparer + Sync>,
    // Whether lines carry a group that they may not be sorted out of. See `crate::gitignore` for
    // what the groups are and why they exist.
    grouped: bool,
    unique: bool,
    reverse: bool,
}

impl Sorter {
    #[allow(clippy::fn_params_excessive_bools)]
    pub(crate) fn new(
        strategy: Strategy,
        locale_name: Option<&str>,
        unique: bool,
        case_insensitive: bool,
        reverse: bool,
        windows: bool,
    ) -> Result<Sorter> {
        let collator = if let Some(locale_name) = locale_name {
            Some(collator_for_locale(locale_name, case_insensitive)?)
        } else {
            None
        };
        let comparer: Box<dyn Comparer + Sync> = match strategy {
            Strategy::Text => Box::new(TextComparer::new(collator, case_insensitive)),
            Strategy::NumberedText => {
                Box::new(NumberedTextComparer::new(collator, case_insensitive))
            }
            Strategy::DatetimeText => {
                Box::new(DatetimeTextComparer::new(collator, case_insensitive))
            }
            Strategy::Path => Box::new(PathComparer::new(
                collator,
                case_insensitive,
                if windows {
                    PathType::Windows
                } else {
                    PathType::Unix
                },
            )),
            Strategy::Gitignore => Box::new(GitignoreComparer::new(collator, case_insensitive)),
            Strategy::Ip => Box::new(IpComparer::new()),
            Strategy::Network => Box::new(NetworkComparer::new()),
        };
        Ok(Self {
            comparer,
            grouped: strategy.keeps_file_structure(),
            unique,
            reverse,
        })
    }

    pub(crate) fn lines_are_sorted(&self, lines: &[SortableLine]) -> Result<bool> {
        if self.grouped {
            return self.grouped_lines_are_sorted(lines);
        }

        let mut last_line: Option<&str> = None;

        let mut seen_lines: Option<HashMap<&str, usize>> = None;
        if self.unique {
            seen_lines = Some(HashMap::new());
        }

        for line in lines {
            if let Some(last_line) = last_line {
                if !self.is_ordered(last_line, &line.line)? {
                    return Err(CheckError::NotSorted {
                        first: last_line.to_string(),
                        second: line.line.clone(),
                    }
                    .into());
                }
            }

            if self.unique {
                let seen_lines = seen_lines.as_mut().unwrap();
                if let Some(seen) = seen_lines.get(line.line.as_str()) {
                    return Err(CheckError::NotUnique {
                        line1: *seen,
                        text1: line.line.clone(),
                        line2: line.line_number,
                        text2: line.line.clone(),
                    }
                    .into());
                }
                seen_lines.insert(&line.line, line.line_number);
            }

            last_line = Some(&line.line);
        }

        Ok(true)
    }

    fn is_ordered(&self, str1: &str, str2: &str) -> Result<bool> {
        self.comparer.is_ordered(str1, str2, self.reverse)
    }

    /// Checks a grouped file by running the very sort it would run and comparing the result to what
    /// is already there.
    ///
    /// A separate "is this in order?" test would be a second opinion about the sort, and the two
    /// could disagree. That is how `--check` ends up passing on a file that sorting then changes,
    /// so there is only one implementation here and the check uses it.
    fn grouped_lines_are_sorted(&self, lines: &[SortableLine]) -> Result<bool> {
        if self.unique {
            if let Some((first, second)) = Self::first_duplicate(lines) {
                return Err(CheckError::NotUnique {
                    line1: lines[first].line_number,
                    text1: lines[first].line.clone(),
                    line2: lines[second].line_number,
                    text2: lines[second].line.clone(),
                }
                .into());
            }
        }

        // With no duplicates to remove the sort is a permutation, so the two are the same length
        // and can be compared line by line.
        let sorted = self.sort_lines(lines.to_vec())?;
        if let Some(i) = (0..lines.len()).find(|i| lines[*i].line != sorted[*i].line) {
            return Err(CheckError::NotSorted {
                first: lines[i].line.clone(),
                second: sorted[i].line.clone(),
            }
            .into());
        }

        Ok(true)
    }

    /// Finds the first line that repeats an earlier one, as the pair of their positions.
    fn first_duplicate(lines: &[SortableLine]) -> Option<(usize, usize)> {
        let mut seen = HashMap::new();
        for (i, line) in lines.iter().enumerate() {
            if line.kind == LineKind::Fence {
                continue;
            }
            match seen.entry(unique_key(&line.line)) {
                Entry::Occupied(e) => return Some((*e.get(), i)),
                Entry::Vacant(e) => {
                    e.insert(i);
                }
            }
        }

        None
    }

    pub(crate) fn sort_lines(&self, mut lines: Vec<SortableLine>) -> Result<Vec<SortableLine>> {
        if self.grouped {
            return self.sort_grouped_lines(lines);
        }

        let res = Mutex::new(Ok(()));
        lines.par_sort_by(|a, b| match self.comparer.cmp(&a.line, &b.line) {
            Ok(o) => o,
            Err(e) => {
                // If there are multiple errors, only the last one will be
                // visible, but that's fine.
                *res.lock().unwrap() = Err(e);
                Ordering::Less
            }
        });
        // The first `?` is for the `MutexGuard` and the second is for the
        // underlying `Result`.
        res.into_inner()??;

        if self.reverse {
            lines.reverse();
        }

        if self.unique {
            lines.dedup_by(|a, b| a.line == b.line);
        }

        Ok(lines)
    }

    /// Sorts within each group, leaving the groups themselves in the order they appeared in the
    /// file. Fences each have a group to themselves, so they do not move at all.
    fn sort_grouped_lines(&self, mut lines: Vec<SortableLine>) -> Result<Vec<SortableLine>> {
        // This has to happen before the sort, because which copy of a repeated line survives
        // depends on where they were, not on where they end up. See `dedup_keeping_last`.
        if self.unique {
            lines = dedup_keeping_last(lines);
        }

        let res = Mutex::new(Ok(()));
        lines.par_sort_by(|a, b| {
            if a.group != b.group {
                return a.group.cmp(&b.group);
            }
            match self.comparer.cmp(&a.line, &b.line) {
                Ok(o) => o,
                Err(e) => {
                    // If there are multiple errors, only the last one will be visible, but that's
                    // fine.
                    *res.lock().unwrap() = Err(e);
                    Ordering::Less
                }
            }
        });
        res.into_inner()??;

        Ok(lines)
    }
}

#[cfg(test)]
mod test {
    use super::{Sorter, Strategy};
    use crate::{gitignore::Grouper, SortableLine};
    use anyhow::Result;
    use test_log::test;

    fn gitignore_lines(lines: &[&str]) -> Vec<SortableLine> {
        let mut grouper = Grouper::default();
        lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let (group, kind) = grouper.next(l);
                SortableLine::for_test(i + 1, l, group, kind)
            })
            .collect()
    }

    fn gitignore_sorted(lines: &[&str], unique: bool) -> Result<Vec<String>> {
        let sorter = Sorter::new(Strategy::Gitignore, None, unique, false, false, false)?;
        Ok(sorter
            .sort_lines(gitignore_lines(lines))?
            .into_iter()
            .map(|l| l.line)
            .collect())
    }

    #[test]
    fn gitignore_sorts_within_groups_only() -> Result<()> {
        assert_eq!(
            gitignore_sorted(
                &["# c", "/target/", "*.log", "", "!b.log", "!a.log", "zzz"],
                false
            )?,
            ["# c", "*.log", "/target/", "", "!a.log", "!b.log", "zzz"],
            "each run is sorted and the fences stay put",
        );

        Ok(())
    }

    #[test]
    fn gitignore_unique_keeps_the_last_copy() -> Result<()> {
        assert_eq!(
            gitignore_sorted(&["*.log", "!keep.log", "*.log", "zzz"], true)?,
            ["!keep.log", "*.log", "zzz"],
            "the `*.log` that survives is the one that came after the negation",
        );

        Ok(())
    }

    #[test]
    fn gitignore_unique_moves_a_blank_line_to_the_top() -> Result<()> {
        assert_eq!(
            gitignore_sorted(&["foo", "", "bar", "foo"], true)?,
            ["", "bar", "foo"],
            "the only line before the blank line went away, so the blank line is first now",
        );

        Ok(())
    }

    #[test]
    fn gitignore_unique_names_both_spellings_of_a_repeated_pattern() -> Result<()> {
        let sorter = Sorter::new(Strategy::Gitignore, None, true, false, false, false)?;
        let lines = gitignore_lines(&["# Deps", "node_modules", "!keep", "**/node_modules"]);
        let err = sorter.lines_are_sorted(&lines).unwrap_err();
        assert_eq!(
            err.to_string(),
            r#"the given file contains non-unique lines at 2 ("node_modules") and 4 ("**/node_modules")"#,
            "the message quotes each line as it is written, since the two are not spelled the same",
        );

        Ok(())
    }

    #[test]
    fn gitignore_unique_does_not_count_repeated_fences_as_duplicates() -> Result<()> {
        let sorter = Sorter::new(Strategy::Gitignore, None, true, false, false, false)?;
        let lines = gitignore_lines(&["# Deps", "a", "", "# Deps", "b", ""]);
        assert!(
            sorter.lines_are_sorted(&lines)?,
            "the same comment can head two blocks, and blank lines repeat all the time",
        );

        Ok(())
    }

    #[test]
    fn sort_lines() -> Result<()> {
        let sorter = Sorter::new(Strategy::Text, None, false, false, false, false)?;
        let lines = ["foo", "bar", "quux", "baz"]
            .into_iter()
            .enumerate()
            .map(|l| (l.0 + 1, l.1))
            .map(SortableLine::from_number_and_str)
            .collect::<Vec<_>>();
        let sorted_lines = sorter.sort_lines(lines.clone())?;

        let mut expect = [(2, "bar"), (4, "baz"), (1, "foo"), (3, "quux")]
            .into_iter()
            .map(SortableLine::from_number_and_str)
            .collect::<Vec<_>>();
        assert_eq!(sorted_lines, expect, "got expected ascending sorting");

        let sorter = Sorter::new(Strategy::Text, None, false, false, true, false)?;
        let sorted_lines = sorter.sort_lines(lines)?;
        expect.reverse();
        assert_eq!(sorted_lines, expect, "got expected descending sorting");

        Ok(())
    }
}
