use swrs::api::block::{BlockCategory, BlockContent, BlockType};

pub mod error;

pub struct BlockDefinitions(pub Vec<BlockDefinition>);

pub struct BlockDefinition {
    block_type: BlockType,
    category: BlockCategory,
    opcode: String,
    spec: BlockContent,
}

/// Parses a `.blks` file from a `&str`
pub fn parse(data: &str) -> Result<BlockDefinitions, error::BlksParseError> {
    todo!()
}