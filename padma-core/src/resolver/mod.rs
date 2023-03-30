//! # Padma Resolver
//! 
//! Takes a parsed blks ([`crate::blks::BlockDefinitions`]) and parsed defs ([`crate::defs::models::Definitions`])
//! then combine them together to provide an API that allows a user to lookup
//! a signature, then receive a data structure that defines the block specifications
//! (things like opcode, args) where they could construct a block from those data.

use std::collections::HashMap;

use crate::defs::models::BlockArgument;
use crate::defs::models::{Dispatch, DispatchKind, FunctionDefinition, FunctionSignature, Type};
use crate::resolver::models::Argument;

use self::error::ResolveError;
use self::models::{Definition, Block};

use super::blks::BlockDefinitions;
use super::blks::BlockDefinition;
use super::defs::models::Definitions;

pub mod models;
pub mod error;

pub struct Resolver {
    blocks: HashMap<String, BlockDefinition>,

    global_functions: HashMap<FunctionSignature, FunctionDefinition>,
    methods: HashMap<Type, HashMap<FunctionSignature, FunctionDefinition>>,
}

impl Resolver {
    pub fn new(blks: Vec<BlockDefinitions>, defs: Vec<Definitions>) -> Self {
        let mut resolver = Self::default();

        for blk in blks { resolver.append_blks(blk); } 
        for def in defs { resolver.append_defs(def); }

        resolver
    }

    /// Appends block definitions ([`BlockDefinitions`]) into the struct.
    /// If a block with the same opcode gets added, it will be implicitly
    /// shadowed / overriden.
    pub fn append_blks(&mut self, blks: BlockDefinitions) {
        let blocks = blks.0.into_iter()
            .map(|d| (d.opcode.clone(), d))
            .collect::<HashMap<_, _>>();

        self.blocks.extend(blocks);
    }

    /// Appens definitions ([`Definitions`]) into the struct.
    /// If a defintion has the same signature with with the existing ones,
    /// it will be implicitly shadowed / overriden.
    pub fn append_defs(&mut self, defs: Definitions) {
        let global_functions = defs.global_functions.into_iter()
            .map(|gf| (gf.0, gf.1))
            .collect::<HashMap<_, _>>();
        
        let methods = defs.methods.into_iter()
            .map(|gf| (gf.0, gf.1.into_iter().collect::<HashMap<_, _>>()))
            .collect::<Vec<_>>();

        self.global_functions.extend(global_functions);

        // merging `methods` is a tiny bit trickier
        for (typ, hmap) in methods {
            if let Some(map) = self.methods.get_mut(&typ) {
                map.extend(hmap);
            } else {
                self.methods.insert(typ, hmap);
            }
        }
    }

    /// Resolves all the definitions with the blocks previously given,
    /// will return a [`Definition`] which contains each of definitions'
    /// blocks representation.
    pub fn resolve(mut self) -> Result<Definition, ResolveError> {
        // gogogogo

        // pop global functions overtime, add them to the cache

        // let mut global_functions_cache = HashMap::new();
        // let mut methods_cache = HashMap::new();

        // we start resolving the global functions, tho the order of methods and 
        // global functions shouldn't matter.
        while !self.global_functions.is_empty() {
            let key = self.global_functions.keys().next().unwrap().clone();
            // safety: .unwrap() is safe because the `while` block above
            //         makes sure that this block is run only when the hashmap
            //         is not empty

            let func = self.global_functions.remove(&key).unwrap();
            // safety: .unwrap() is safe because we got the key from the hashmap

            let blocks = func.blocks
                .into_iter()
                .map(|block| self.resolve_block(&block))
                .collect::<Result<Vec<Block>, _>>()?;
        }

        for (sign, def) in self.global_functions.clone() {
            // we start by trying to resolve its blocks
            for block in &def.blocks {
                self.resolve_block(block)?;
            }
        }

        todo!()
    }

    fn resolve_block(&mut self, block: &Dispatch, root_signature: FunctionSignature) -> Result<Block, ResolveError> {
        self.resolve_block_recursive(&block, root_signature, vec![])
    }

    fn resolve_block_recursive(
        &mut self,
        block: &Dispatch,

        root_signature: FunctionSignature,
        visited_functions: Vec<FunctionSignature> // <- to prevent cyclic dependency
    ) -> Result<Block, ResolveError> {
        Ok(match block.kind {
            DispatchKind::RawBlock => {
                // take a block that matches the opcode
                let def = self.blocks.get(&block.identifier)
                    .ok_or_else(|| ResolveError::BlockNotFound)?
                    .clone();

                // resolve its arguments
                let arguments = block.arguments.iter()
                    .map::<Result<Argument, ResolveError>, _>(|arg| {
                        // convert defs' `BlockArgument` into our `Argument` type
                        // defs' `BlockArgument` has a `Dispatch` variant, whilst our type
                        // returns the block directly (`Block` variant)
                        Ok(match arg {
                            BlockArgument::Dispatch(disp) => {
                                Argument::Block(self.resolve_block_recursive(
                                    disp, root_signature, visited_functions
                                )?)
                            },
                            BlockArgument::Argument { index } => Argument::Argument(*index),
                            BlockArgument::Literal(lit) => Argument::Literal(lit.clone()),
                            BlockArgument::This => Argument::This,
                        })
                    }).collect::<Result<Vec<Argument>, _>>()?;

                Block {
                    opcode: def.opcode,
                    spec: def.spec,
                    arguments,
                }
            },
            DispatchKind::FunctionDispatch => {
                // smart way to prevent cyclic dependency lol
                todo!()
            },
        })
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self { 
            blocks: Default::default(),
            global_functions: Default::default(),
            methods: Default::default()
        }
    }
}