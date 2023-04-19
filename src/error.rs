use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum CheckError {
    #[error(r#"the given file contains empty lines not preceded by a comment"#)]
    HasUnexpectedEmptyLines,
    #[error(r#"the given file is not sorted - found "{first:}" before "{second:}""#)]
    NotSorted { first: String, second: String },
    #[error(
        r#"the given file contains non-unique lines at {line1:} and {line2:} containing "{line:}""#
    )]
    NotUnique {
        line1: usize,
        line2: usize,
        line: String,
    },
}
