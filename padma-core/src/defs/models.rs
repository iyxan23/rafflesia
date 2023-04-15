use std::collections::HashMap;

/// Top-level structure, represents the whole file. Contains definitions
/// of a definition file.
#[derive(Debug, Clone, PartialEq)]
pub struct Definitions {
    pub global_functions: Vec<(FunctionDeclaration, FunctionBody)>,
    pub methods: HashMap<Type, Vec<(FunctionDeclaration, FunctionBody)>>,
    // todo: bindings and primitive exprs
    // pub bindings: Vec<(FunctionSignature, FunctionDefinition)>,
    // pub primitive_exprs: Vec<()>,
}

impl Default for Definitions {
    fn default() -> Self {
        Self {
            global_functions: Default::default(),
            methods: Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub this: Option<Type>,
    pub parameters: Vec<Type>,
    pub name: String,
    pub return_type: Option<Type>
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionBody {
    pub statements: Vec<Statement>,
}

// We give a strong differentiation between a statement and an expression.
//   A statement is one line of statement, it does not return / give out any value.
//   While expressions are the opposite.

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Block {
        opcode: String,
        arguments: Vec<Expression>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
        this: Option<Box<Expression>>,
    },
    Return {
        value: Expression
    },

    // later we'll have things like `repeat` or `if` at compile time :>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Block {
        opcode: String,
        arguments: Vec<Expression>,
        // return_type: Type,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
        this: Option<Box<Expression>>,
        // return_type: Type,
    },
    Literal(Literal),
    Argument(u32),
    This,
}

// having `return_type` to an expression sounds like a bad idea

// impl AsRef<Type> for Expression {
//     fn as_ref(&self) -> &Type {
//         match &self {
//             Expression::Block { return_type, .. } |
//             Expression::FunctionCall { return_type, .. } => return_type,
//             Expression::Literal(lit) => lit.as_ref(),
//         }
//     }
// }

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

impl AsRef<Type> for Literal {
    fn as_ref(&self) -> &Type {
        match self {
            Literal::Boolean(_) => &Type::Boolean,
            Literal::Number(_) => &Type::Number,
            Literal::String(_) => &Type::String,
        }
    }
}