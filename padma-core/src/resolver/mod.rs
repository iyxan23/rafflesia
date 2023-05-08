//! # Padma Resolver
//!
//! The code that processes parsed defs ([`crate::defs::Definitions`]), matching
//! and combining them with parsed blks ([`crate::blks::BlockDefinitions`]), to generate
//! a single data structure of which every definitions (functions, methods) are flattened
//! into blocks which could then be emitted as compiler output.

mod models;
mod resolver;
#[cfg(test)]
mod tests;

pub use models::*;
pub use resolver::*;
