#[derive(Debug, Clone, PartialEq)]
pub struct Definitions {
    pub global_functions: Vec<(FunctionSignature, FunctionDefinition)>,
    pub methods: Vec<(Type, Vec<(FunctionSignature, FunctionDefinition)>)>,
    // todo: bindings and primitive exprs
    // pub bindings: Vec<(FunctionSignature, FunctionDefinition)>,
    // pub primitive_exprs: Vec<()>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub this: Option<Type>,
    pub parameters: Vec<Type>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub blocks: Vec<BlockDispatch>
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockDispatch {
    pub opcode: String,
    pub arguments: Vec<BlockArgument>
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockArgument {
    Argument { index: u32 },
    Literal(Literal),
    This
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Boolean,
    Number,
    String,
    // View,
    // Component,
    // List,
    // Map,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Boolean(bool),
    Number(u64),
    String(String),
    // View(),
    // Component(),
    // List(),
    // Map(),
}