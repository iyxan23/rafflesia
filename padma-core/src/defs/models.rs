use std::collections::HashMap;

// todo: add spans on these!

/// Top-level structure, represents the whole file. Contains definitions
/// of a definition file.
#[derive(Debug, Clone, PartialEq)]
pub struct Definitions {
    pub global_functions: HashMap<FunctionDeclaration, FunctionBody>,
    pub methods: HashMap<Type, HashMap<FunctionDeclaration, FunctionBody>>,
    pub bindings: HashMap<BindingDeclaration, BindingBody>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)] // <- needed for HashMap
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)] // <- needed for hashmap
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

impl Into<Expression> for Statement {
    /// Tries to convert an [`Statement`] to be a [`Expression`]. This conversion
    /// should and is only done for statements that returns a value. It is not
    /// supposed to be used to convert regular statements into expressions
    /// as both of them are different.
    /// 
    /// Any variants are converted as-is, with a little exception of the [`Statement::Return`]
    /// variant, which is converted implicitly by taking it's value as an expression
    /// and using that as a result.
    fn into(self) -> Expression {
        match self {
            Statement::Block { opcode, arguments } => 
                Expression::Block { opcode, arguments },
            Statement::FunctionCall { name, arguments } =>
                Expression::FunctionCall { name, arguments },
            Statement::MethodCall { name, arguments, this } =>
                Expression::MethodCall { name, arguments, this },

            Statement::Return { value } => value,
        }
    }
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

impl TryInto<Statement> for Expression {
    type Error = StaticVariable;

    /// Tries to convert an [`Expression`] to be a [`Statement`]. This conversion
    /// should and is only done for expressions that was previously thought to be
    /// a statement. It is not supposed to be used to convert regular expressions
    /// into statements as both of them are different.
    /// 
    /// This will only succeed if the given [`Expression`] is anything other
    /// than [`Expression::StaticVariable`], since its not possible for 
    fn try_into(self) -> Result<Statement, Self::Error> {
        Ok(match self {
            Expression::Block { opcode, arguments } => 
                Statement::Block { opcode, arguments },
            Expression::FunctionCall { name, arguments } =>
                Statement::FunctionCall { name, arguments },
            Expression::MethodCall { name, arguments, this } =>
                Statement::MethodCall { name, arguments, this },

            Expression::StaticVariable(v) => Err(v)?,
        })
    }
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