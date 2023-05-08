use std::collections::HashMap;

pub mod builder;

use crate::{blks::BlockDefinition, defs::FunctionBody as DefsFunctionBody};

use super::models::Signature;

pub struct Resolver {
    definitions: ResolverDefinitions,
    blocks: ResolverBlocks,
}

struct ResolverDefinitions {
    inner: HashMap<String, HashMap<Signature, DefsFunctionBody>>,
}

struct ResolverBlocks {
    inner: HashMap<String, BlockDefinition>,
}
