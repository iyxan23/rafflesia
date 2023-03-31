//! # Padma Resolver
//! 
//! Takes a parsed blks ([`crate::blks::BlockDefinitions`]) and parsed defs ([`crate::defs::models::Definitions`])
//! then combine them together to provide an API that allows a user to lookup
//! a signature, then receive a data structure that defines the block specifications
//! (things like opcode, args) where they could construct a block from those data.

use std::borrow::Borrow;
use std::collections::HashMap;

use swrs::api::block::{BlockType, ArgumentBlockReturnType};

use super::defs;
use super::blks;

use self::error::ResolveError;
use self::models::DefinitionBlocks;
use self::models::{Definition, Block, Signature, Argument};

pub mod models;
pub mod error;

pub struct Resolver {
    blocks: HashMap<String, blks::BlockDefinition>,

    global_functions: HashMap<defs::FunctionSignature, defs::FunctionDefinition>,
    methods: HashMap<defs::Type, HashMap<defs::FunctionSignature, defs::FunctionDefinition>>,
}

impl Resolver {
    pub fn new(blks: Vec<blks::BlockDefinitions>, defs: Vec<defs::Definitions>) -> Self {
        let mut resolver = Self::default();

        for blk in blks { resolver.append_blks(blk); } 
        for def in defs { resolver.append_defs(def); }

        resolver
    }

    /// Appends block definitions ([`BlockDefinitions`]) into the struct.
    /// If a block with the same opcode gets added, it will be implicitly
    /// shadowed / overriden.
    pub fn append_blks(&mut self, blks: blks::BlockDefinitions) {
        let blocks = blks.0.into_iter()
            .map(|d| (d.opcode.clone(), d))
            .collect::<HashMap<_, _>>();

        self.blocks.extend(blocks);
    }

    /// Appens definitions ([`Definitions`]) into the struct.
    /// If a defintion has the same signature with with the existing ones,
    /// it will be implicitly shadowed / overriden.
    pub fn append_defs(&mut self, defs: defs::Definitions) {
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
        let mut cache = HashMap::new();

        // pop global functions overtime, add them to the cache

        // we start resolving the global functions, tho the order of methods and 
        // global functions shouldn't matter.
        while !self.global_functions.is_empty() {
            let key = self.global_functions.keys().next().unwrap().clone();
            // safety: .unwrap() is safe because the `while` block above
            //         makes sure that this block is run only when the hashmap
            //         is not empty

            let func = self.global_functions.remove(&key).unwrap();
            // safety: .unwrap() is safe because we got the key from the hashmap

            // convert defs' `FunctionSignature` into our own `Signature`
            // let signature = Self::create_signature(
            //     key.this, key.name, key.parameters,
            //     None
            // );

            // we then loop over the statements in the function
            // to convert them into actual blocks
            let mut blocks = func.statements
                .into_iter()
                .try_fold(vec![], |mut acc, dispatch| {
                    let result = self.resolve_dispatch(
                        &dispatch, &mut cache,

                        /* parent parameter types */ &key.parameters,
                        /* parent this type */ key.this.borrow().as_ref(),

                        /* required type: None, since this is a regular statement, the block doesn't need to return anything */
                        None
                    )?;
                    acc.extend(result);
                    Ok(acc)
                })?;
            
            // then convert the return dispatch (if it has one)
            let return_block = if let Some(return_statement) = func.return_statement {
                let mut return_blocks = self.resolve_dispatch(
                    &return_statement, &mut cache,

                    /* parent parameter types */ &key.parameters,
                    /* parent this type */ key.this.borrow().as_ref(),

                    /* required type: None, since this is a regular statement, the block doesn't need to return anything */
                    None
                )?;

                // since a dispatch could generate multiple blocks, a return block
                // can only be one. So we set the last generated block to be the
                // "return block" and let the rest to get appended to the whole
                // blocks vector as if they're regular statements.

                let last_block = return_blocks.pop().unwrap();
                // safety: .unwrap() is safe here because return_block is never empty
                //         a raw block call will always return its block
                //
                //         for the case of a function call, it wouldn't be possible since
                //         they require atleast one statement to "accomodate returning a value"
                // 
                //         -> if a function is empty and it has a return type, then it wouldn't
                //            be possible to be compiled

                blocks.extend(return_blocks);

                Some(last_block)
            } else { None };

            // now that we have all of our blocks and a signature to identify this function
            // we add them into our cache collection
            cache.insert(
                todo!("signature"),
                Definition {
                    blocks: DefinitionBlocks {
                        return_block, blocks
                    },
                    signature: todo!("signature"),
                }
            );
        }

        todo!()
    }

    fn resolve_dispatch(
        &mut self,
        dispatch: &defs::Dispatch,

        cache: &mut HashMap<Signature, Definition>,

        parent_parameter_types: &Vec<defs::Type>,
        parent_this_type: Option<&defs::Type>,
        parent_required_type: Option<&defs::Type>,
    ) -> Result<Vec<Block>, ResolveError> {
        self.resolve_dispatch_recursive(
            &dispatch,
            &mut vec![],
            cache,

            parent_parameter_types,
            parent_this_type,
            parent_required_type,
        )
    }

    fn resolve_dispatch_recursive(
        &mut self,
        block: &defs::Dispatch,

        visited: &mut Vec<Signature>, // <- to prevent cyclic dependency
        cache: &mut HashMap<Signature, Definition>,

        parent_parameter_types: &Vec<defs::Type>,
        parent_this_type: Option<&defs::Type>,
        parent_required_type: Option<&defs::Type>,
    ) -> Result<Vec<Block>, ResolveError> {
        Ok(match block.kind {
            defs::DispatchKind::RawBlock => {
                // take a block that matches the opcode
                let def = self.blocks.get(&block.identifier)
                    .ok_or_else(|| ResolveError::BlockNotFound { opcode: block.identifier.clone() })?
                    .clone();

                // resolve its arguments
                let arguments = block.arguments.iter()
                    .map::<Result<Argument, ResolveError>, _>(|arg| {
                        // convert defs' `BlockArgument` into our `Argument` type
                        // defs' `BlockArgument` has a `Dispatch` variant, whilst our type
                        // returns the block directly (`Block` variant)
                        Ok(match arg {
                            defs::BlockArgument::Dispatch(disp) => {
                                // fixme: 
                                //    what if `disp` generates multiple blocks
                                //    but we need to put it as an Argument? which takes
                                //    only one `Block`
                                todo!()
                            },
                            defs::BlockArgument::Argument { index } => Argument::Argument(*index),
                            defs::BlockArgument::Literal(lit) => Argument::Literal(lit.clone()),
                            defs::BlockArgument::This => Argument::This,
                        })
                    }).collect::<Result<Vec<Argument>, _>>()?;

                // only one block since it's a raw dispatch
                vec![Block {
                    opcode: def.opcode,
                    spec: def.spec,
                    arguments,

                    return_type: if let BlockType::Argument(arg) = def.block_type {
                        Some(match arg {
                            ArgumentBlockReturnType::Boolean => defs::Type::Boolean,
                            ArgumentBlockReturnType::String => defs::Type::String,
                            ArgumentBlockReturnType::Number => defs::Type::Number,
                            ArgumentBlockReturnType::View { type_name } =>
                                unimplemented!("implement view"),
                            ArgumentBlockReturnType::Component { type_name } =>
                                unimplemented!("implement component"),
                            ArgumentBlockReturnType::List { inner_type } =>
                                unimplemented!("implement list"),
                        })
                    } else { None }
                }]
            },
            defs::DispatchKind::FunctionDispatch => {
                // convert the given identifier, `this` value and arguments into FunctionSignature
                let signature = Self::create_signature(
                    // takes the block type of this, given that this is a BlockArgument
                    // we need to convert it into a defs::Type.
                    block.this.as_ref()
                        .map(|block_arg|
                            Self::def_block_argument_as_type(
                                block_arg.as_ref(),
                                parent_parameter_types, parent_this_type, parent_required_type
                            )
                        )
                        // flip `Option<Result<A, B>>` into `Result<Option<A>, B>`
                        .map_or(Ok(None), |v| v.map(Some))?,

                    block.identifier.clone(),

                    block.arguments.iter()
                        .map(|arg|
                            Self::def_block_argument_as_type(
                                arg,
                                parent_parameter_types, parent_this_type, parent_required_type
                            )
                        ).collect::<Result<Vec<_>, _>>()?,

                    parent_required_type
                );

                todo!()
            },
        })
    }

    /// Takes a [`Signature`], to then check the cache if there was a definition
    /// defined by it; if not then it will try to resolve it and return it as a
    /// substitute.
    fn get_compiled_or_resolve<'a>(
        &'a mut self,
        signature: Signature,

        visited: &mut Vec<Signature>,
        cache: &'a mut HashMap<Signature, Definition>,

        parent_parameter_types: &Vec<defs::Type>,
        parent_this_type: Option<&defs::Type>,
        parent_required_type: Option<&defs::Type>,
    ) -> Result<&'a Definition, ResolveError> {
        Ok(match cache.get(&signature) {
            None => {
                // get a block from the signature
                // let resolved_blocks = self.resolve_from_signature(
                //     &signature,
                //     visited, cache,

                //     parent_parameter_types,
                //     parent_this_type,
                //     parent_required_type
                // );

                todo!()
            },
            Some(def) => def,
        })
    }

    fn resolve_from_signature(
        &mut self,

        signature: &Signature,

        visited: &mut Vec<Signature>,
        cache: &mut HashMap<Signature, Definition>,

        parent_parameter_types: &Vec<defs::Type>,
        parent_this_type: Option<&defs::Type>,
        parent_required_type: Option<&defs::Type>,
    ) -> DefinitionBlocks {
        todo!()
    }

    // internal function to convert raw values given from defs's parser into a valid
    // [`FunctionSignature`] might want to move this into [`FunctionSignature`] itself
    fn create_signature(
        this: Option<defs::Type>,
        identifier: String,
        argument_types: Vec<defs::Type>,

        parent_required_type: Option<&defs::Type>,
    ) -> defs::FunctionSignature {
        defs::FunctionSignature {
            this: this,
            parameters: argument_types,
            name: identifier,
            return_type: parent_required_type.cloned()
        }
    }

    /// converts a [`defs::BlockArgument`] into [`defs::Type`]
    fn def_block_argument_as_type(
        arg: &defs::BlockArgument,

        parent_parameter_types: &Vec<defs::Type>,
        parent_this_type: Option<&defs::Type>,
        parent_required_type: Option<&defs::Type>,
    ) -> Result<defs::Type, ResolveError> {
        Ok(match arg {
            defs::BlockArgument::Dispatch(disp) => {
                todo!()
            },
            defs::BlockArgument::Argument { index } =>
                parent_parameter_types.get(*index as usize)
                    .ok_or_else(|| ResolveError::ArgumentNotFound { number: *index })?
                    .clone(),
            defs::BlockArgument::Literal(lit) =>
                Self::literal_to_type(lit),
            defs::BlockArgument::This =>
                parent_this_type
                    .ok_or_else(|| ResolveError::NotAMethod)?
                    .clone(),
        })
    }

    /// converts an [`&Argument`] into a [`defs::Type`]. Returns `None` if the argument tries to access
    /// `this` but it's none, or either it tries to use a block as an argument, but the block
    /// doesn't return anything.
    // todo: Result type, and probably move this into `TryFrom<Argument>`
    fn argument_to_type(
        arg: &Argument,

        parent_parameter_types: &Vec<defs::Type>,
        parent_this_type: Option<&defs::Type>,
    ) -> Option<defs::Type> {
        Some(match arg {
            Argument::Literal(lit) => Self::literal_to_type(lit),
            Argument::Block(block) => block.return_type.clone()?,
            Argument::Argument(arg) => parent_parameter_types.get(*arg as usize)?.clone(),
            Argument::This => parent_this_type?.clone(),
        })
    }

    fn literal_to_type(lit: &defs::Literal) -> defs::Type {
        match lit {
            defs::Literal::Boolean(_) => defs::Type::Boolean,
            defs::Literal::Number(_) => defs::Type::Number,
            defs::Literal::String(_) => defs::Type::String,
        }
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