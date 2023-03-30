use thiserror::Error;

use crate::defs::models::Type;

#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("too many arguments given")]
    TooManyArguments,
    #[error("")]
    TooLittleArguments,
    #[error("")]
    InvalidArgumentType {
        given: Type,
        required: Type,
    },

    #[error("")]
    CyclicDependency,
    #[error("")]
    BlockNotFound,
}