//! # Padma Resolver
//! 
//! Takes a parsed blks ([`crate::blks::BlockDefinitions`]) and parsed defs ([`crate::defs::models::Definitions`])
//! then combine them together to provide an API that allows a user to lookup
//! a signature, then receive a data structure that defines the block specifications
//! (things like opcode, args) where they could construct a block from those data.

use std::collections::HashMap;

use crate::defs::models::DispatchKind;

use self::error::ResolveError;
use self::models::Definition;

use super::blks::BlockDefinitions;
use super::blks::BlockDefinition;
use super::defs::models::Definitions;

pub mod models;
pub mod error;

// warning: multiple defs or blks that has the same name / signature will
// be implicitly overriden / shadowed
pub fn resolve(
    blks: Vec<BlockDefinitions>,
    defs: Vec<Definitions>,
) -> Result<Definition, ResolveError> {
    // we firstly flatten these blks and defs

    // then we turn it into a map that that resolves from opcode: String -> BlockDefinition
    let blocks = blks.into_iter()
        .map(|d| d.0)
        .flatten()
        .map(|d| (d.opcode.clone(), d))
        .collect::<HashMap<_, _>>();

    let definitions = defs.into_iter()
        .fold(Definitions { global_functions: vec![], methods: vec![] },
            |acc, d|
            Definitions { 
                global_functions: acc.global_functions.into_iter()
                             .chain(d.global_functions).collect(),
                methods: acc.methods.into_iter()
                    .chain(d.methods).collect(),
            }
        );

    let global_functions = definitions.global_functions.into_iter()
        .map(|f| (f.0, f.1))
        .collect::<HashMap<_, _>>();
    
    let methods = definitions.methods.into_iter()
        .map(|f| (f.0, f.1))
        .collect::<HashMap<_, _>>();

    // and we start doing it
    for (sign, def) in global_functions {
        // we start by trying to resolve its blocks
        for block in def.blocks {
            match block.kind {
                DispatchKind::RawBlock => {
                    // todo: parse the raw block got from global_functions
                },
                DispatchKind::FunctionDispatch => {
                    // john we have a problem here, what if we have not parsed this function?
                },
            }
        }
    }

    todo!()
}