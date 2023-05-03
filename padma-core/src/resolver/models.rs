use std::{collections::HashMap, rc::Rc};

use swrs::api::block::BlockContent;

use crate::defs::{Literal as DefsLiteral, Type as DefsType};

/// Signature of a definition.
///
/// Note: Bindings are meld as function / methods.
#[derive(Debug, Clone, PartialEq, Hash, Eq)] // <- Hash and Eq needed for HashMap
pub enum Signature {
    Function {
        name: String,
        parameters: Vec<DefsType>,
        return_type: Option<DefsType>,
    },
    Method {
        name: String,
        this: DefsType,
        parameters: Vec<DefsType>,
        return_type: Option<DefsType>,
    },
    // coming soon: Expression
}

#[derive(Debug, Clone, PartialEq)]
pub struct Definitions {
    pub blocks: Vec<Rc<Block>>,
    pub definitions: HashMap<Signature, (DefinitionBlocks, BlockReturn)>,
}

pub type DefinitionBlocks = Vec<Rc<Block>>;
pub type BlockReturn = Option<BlockArgument>;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub opcode: String,
    pub content: BlockContent,
    pub arguments: Vec<BlockArgument>,
    pub return_type: Option<DefsType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockArgument {
    Literal(DefsLiteral),
    Block(Rc<Block>),
    Argument(u32),
    This,
}
