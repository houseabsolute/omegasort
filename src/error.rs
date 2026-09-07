use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum CheckError {
    #[error(r#"the given file contains empty lines not preceded by a comment"#)]
    HasUnexpectedEmptyLines,
    #[error(r#"the given file is not sorted - found "{first:}" before "{second:}""#)]
    NotSorted { first: String, second: String },
    // Each line is quoted next to its own number because the two are not always spelled the
    // same. When sorting a gitignore file `foo` and `**/foo` are the same pattern, and naming only
    // one of them would point at a line that does not contain it.
    #[error(
        r#"the given file contains non-unique lines at {line1:} ("{text1:}") and {line2:} ("{text2:}")"#
    )]
    NotUnique {
        line1: usize,
        text1: String,
        line2: usize,
        text2: String,
    },
}
