use crate::{
    collation::collator_for_locale,
    comparer::{
        Comparer, DatetimeTextComparer, IpComparer, NetworkComparer, NumberedTextComparer,
        PathComparer, PathType, TextComparer,
    },
    error::CheckError,
    SortableLine,
};
use anyhow::Result;
use clap::ValueEnum;
use rayon::prelude::*;
use std::{cmp::Ordering, collections::HashMap, sync::Mutex};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum Strategy {
    Text,
    NumberedText,
    DatetimeText,
    Path,
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
}

pub(crate) struct Sorter {
    comparer: Box<dyn Comparer + Sync>,
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
            Strategy::Ip => Box::new(IpComparer::new()),
            Strategy::Network => Box::new(NetworkComparer::new()),
        };
        Ok(Self {
            comparer,
            unique,
            reverse,
        })
    }

    pub(crate) fn lines_are_sorted(&self, lines: &[SortableLine]) -> Result<bool> {
        let mut last_line: Option<&str> = None;

        let mut seen_lines: Option<HashMap<&str, usize>> = None;
        if self.unique {
            seen_lines = Some(HashMap::new());
        }

        for line in lines {
            if let Some(last_line) = last_line {
                if self.is_ordered(last_line, &line.line)? {
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
                        line2: line.line_number,
                        line: line.line.clone(),
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

    pub(crate) fn sort_lines(&self, mut lines: Vec<SortableLine>) -> Result<Vec<SortableLine>> {
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
}

#[cfg(test)]
mod test {
    use super::{Sorter, Strategy};
    use crate::SortableLine;
    use anyhow::Result;

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
