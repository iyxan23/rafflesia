//! # Padma Resolver
//! 
//! Takes a parsed blks ([`crate::blks::BlockDefinitions`]) and parsed defs ([`crate::defs::models::Definitions`])
//! then combine them together to provide an API that allows a user to lookup
//! a signature, then receive a data structure that defines the block specifications
//! (things like opcode, args) where they could construct a block from those data.