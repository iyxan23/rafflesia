use std::collections::HashMap;

/// Top-level structure, represents the whole file. Contains definitions
/// of a definition file.
#[derive(Debug, Clone, PartialEq)]
pub struct Definitions {
    pub global_functions: Vec<(FunctionDeclaration, FunctionBody)>,
    pub methods: HashMap<Type, Vec<(FunctionDeclaration, FunctionBody)>>,
    pub bindings: Vec<(BindingDeclaration, BindingBody)>,
    // todo: bindings and primitive exprs
    // pub primitive_exprs: Vec<()>,
}

impl Default for Definitions {
    fn default() -> Self {
        Self {
            global_functions: Default::default(),
            methods: Default::default(),
            bindings: Default::default(),
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

#[derive(Debug, Clone, PartialEq)]
pub struct BindingDeclaration {
    pub this: Option<Type>,
 
    // a binding optionally takes parameters. If there is no parameter
    // specified, the resolver will implicitly take its parameters
    // depending on the block's / function's parameters.
    pub parameters: Option<Vec<Type>>,

    pub name: String,
    pub return_type: Option<Type>
}

#[derive(Debug, Clone, PartialEq)]
pub enum BindingBody {
    Block {
        opcode: String,

        // None if arguments are implicit
        arguments: Option<Vec<Expression>>
    },
    FunctionCall {
        name: String,

        // None if arguments are implicit
        arguments: Option<Vec<Expression>>
    },

    // note for resolver:
    //   for a "smarter" move, the resolver should check if `this` is a
    //   StaticVariable of StaticVariable::This. And if it is and
    //   arguments is None, we should only pass in `@0`+ as variables
    //   instead of `@@`. (see docs/padma-notation.md)
    MethodCall {
        name: String,
        this: Expression,

        arguments: Option<Vec<Expression>>
    },
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
    },
    MethodCall {
        name: String,
        arguments: Vec<Expression>,
        this: Box<Expression>,
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
        // return_type: Type,
    },
    MethodCall {
        name: String,
        arguments: Vec<Expression>,
        this: Box<Expression>,
        // return_type: Type,
    },

    // things like literals, arguments, and `this`
    StaticVariable(StaticVariable),
}

/// Anything that is other than dispatching blocks.
/// Things like literals, argument, and `this`.
#[derive(Debug, Clone, PartialEq)]
pub enum StaticVariable {
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