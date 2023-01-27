use swrs::api::block::{BlockContentParseError, InvalidBlockType};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlksParseError {
    #[error("Unable to parse line {line}")]
    InvalidSyntax {
        line: usize,
        content: String
    },

    #[error("Invalid block spec on line {line}")]
    InvalidBlockSpec {
        line: usize,
        source: BlockContentParseError
    },

    #[error("Invalid block type on line {line}: {blk_type}")]
    InvalidBlockType {
        line: usize,
        blk_type: String,
        source: InvalidBlockType
    }
}