use buffered_lexer::{error, SpannedTokenOwned};

#[derive(Debug, Clone, PartialEq)]
pub enum DefsParseError {
    ParseError(error::ParseError<super::Token, SpannedTokenOwned<super::Token>>),
}

impl From<error::ParseError<super::Token, SpannedTokenOwned<super::Token>>> for DefsParseError {
    fn from(value: error::ParseError<super::Token, SpannedTokenOwned<super::Token>>) -> Self {
        Self::ParseError(value)
    }
}