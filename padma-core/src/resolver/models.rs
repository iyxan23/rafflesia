use swrs::api::block::BlockContent;

pub struct Definition {
    pub blocks: DefinitionBlocks,
    pub signature: Signature,
}

pub enum Signature {
    Function {
        name: String,
        this: Option<Type>,
        ret_type: Option<Type>,
        arguments: Vec<Type>,
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

pub struct DefinitionBlocks {
    pub return_block: Option<Block>,
    pub blocks: Vec<Block>,
}

pub struct Block {
    pub opcode: String,
    pub spec: BlockContent,
    pub arguments: Vec<Argument>,
}

pub enum Argument {
    Literal(crate::defs::models::Literal),
    Block(Block),
    Argument(u32),
    This,
}

pub enum Type {
    Boolean,
    Number,
    String,
    // View,
    // Component,
    // List,
    // Map,
}