/// Top-level structure, represents the whole file. Contains definitions
/// of a definition file.
#[derive(Debug, Clone, PartialEq)]
pub struct Definitions {
    pub global_functions: Vec<(FunctionSignature, FunctionDefinition)>,
    pub methods: Vec<(Type, Vec<(FunctionSignature, FunctionDefinition)>)>,
    // todo: bindings and primitive exprs
    // pub bindings: Vec<(FunctionSignature, FunctionDefinition)>,
    // pub primitive_exprs: Vec<()>,
}

/// Represents a signature of a defined function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)] // <- needed for it to be HashMap keys
pub struct FunctionSignature {
    pub this: Option<Type>,
    pub parameters: Vec<Type>,
    pub name: String,
    pub return_type: Option<Type>
}

/// The acutal function's implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub statements: Vec<Dispatch>,
    pub return_statement: Option<Dispatch>,
}

/// A dispatch, may be a raw block or a function call from the same
/// defs or other defs file that gets merged together.
#[derive(Debug, Clone, PartialEq)]
pub struct Dispatch {
    pub kind: DispatchKind,
    pub identifier: String,
    pub arguments: Vec<BlockArgument>,
    
    // only exists if `DispatchKind` is `FunctionDispatch` this field
    // is used to represent a method call of a defined function
    pub this: Option<Box<BlockArgument>>,
}

/// A dispatch, may be a raw block or a function call from the same
/// defs or other defs file that gets merged together.
#[derive(Debug, Clone, PartialEq)]
pub enum DispatchKind {
    RawBlock,
    FunctionDispatch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockArgument {
    Dispatch(Dispatch),
    Argument { index: usize },
    Literal(Literal),
    This,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
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