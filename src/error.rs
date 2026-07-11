use std::ops::Range;

#[derive(Debug, Clone, Default, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct ParseError(pub String);

impl ParseError {
    pub fn from_range(range: Range<usize>) -> Self {
        Self(format!("Invalid token at byte {}", range.start))
    }

    pub fn at(msg: &str, span: Range<usize>, input: &str) -> Self {
        let snippet = &input[span.clone()];
        Self(format!("{msg} near '{snippet}' at byte {}", span.start))
    }

    pub fn at_eof(msg: &str, input: &str) -> Self {
        if input.is_empty() {
            return Self(msg.to_string());
        }

        let n = input.len();
        let snippet = &input[n.saturating_sub(5)..];
        Self(format!("{msg} but reached end of input near '{snippet}'"))
    }
}
