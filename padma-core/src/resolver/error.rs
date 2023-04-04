use thiserror::Error;

use crate::defs::models::Type;

use super::models::Signature;

// todo: error-stack :>
// todo: add more fields

#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("a returning expression cannot be used as a statement.")]
    ReturningExpressionAsStatement,

    #[error("too many arguments given")]
    TooManyArguments,
    #[error("too little arguments given")]
    TooLittleArguments,

    #[error("type mismatch, needed argument {:?}, given {:?}", required, given)]
    InvalidArgumentType {
        // These two fields are `None` when they are treated as regular statements.
        given: Option<Type>,
        required: Option<Type>,
    },

    #[error("cyclic dependency, a function could not recursively call itself")]
    CyclicDependency,
    #[error("block not found, the opcode {:?} is not defined", opcode)]
    BlockNotFound {
        opcode: String
    },
    #[error("definition not found, {:?} is not defined", signature)]
    DefinitionNotFound {
        signature: Signature
    },
    #[error("argument `@{}` not found", number)]
    ArgumentNotFound {
        number: usize
    },
    #[error("cannot access `@@`, function is not a method")]
    NotAMethod,
}