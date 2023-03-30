use super::models::Type;

pub enum ResolveError {
    MultipleBlk,
    MultipleDef,
    
    TooManyArguments,
    TooLittleArguments,
    InvalidArgumentType {
        given: Type,
        required: Type,
    },
}