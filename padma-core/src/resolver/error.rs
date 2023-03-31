use thiserror::Error;

use crate::defs::models::Type;

use super::models::Signature;

// todo: error-stack :>

#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("too many arguments given")]
    TooManyArguments,
    #[error("too little arguments given")]
    TooLittleArguments,
    #[error("type mismatch, needed argument {:?}, given {:?}", required, given)]
    InvalidArgumentType {
        given: Type,
        required: Type,
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
        number: u32
    },
    #[error("cannot access `@@`, function is not a method")]
    NotAMethod,
    #[error("function {} does not return any value, type {:?} required", func_name, required)]
    FunctionDoesNotReturn {
        func_name: String,
        required: Type
    },
}