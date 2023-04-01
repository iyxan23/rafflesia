use swrs::api::block::BlockContent;

use crate::defs::models::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub blocks: DefinitionBlocks,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)] // <- Eq and Hash are required to be as keys in a HashMap
pub enum Signature {
    Function {
        name: String,
        this: Option<Type>,
        ret_type: Option<Type>,
        parameter_types: Vec<Type>,
    },
    // todo: Expression(Expression)
}

// todo:
// pub enum Expression {
//     Arithmetic {
//         operator: Operator,
//         left: Argument,
//         right: Argument,
//     },
//     // ...
// }

#[derive(Debug, Clone, PartialEq)]
pub struct DefinitionBlocks {
    // return block can only be one
    //
    // in an event where a function call is used as a return value and it
    // generates multiple other blocks inside it. The last block is used
    // as the `return_block` and the others will be appended into `blocks`.
    pub return_block: Option<Block>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub opcode: String,
    pub spec: BlockContent,
    pub arguments: Vec<Argument>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    Literal(crate::defs::models::Literal),

    // note: for an argument of a function that might return multiple blocks
    //       the last / returning block is set as the argument, and the rest
    //       are appended with the resulting blocks list.
    Block(Block),

    Argument(u32),
    This,
}