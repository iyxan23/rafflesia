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

#[cfg(test)]
mod tests;

pub struct Resolver {
    blocks: HashMap<String, blks::BlockDefinition>,

    // These definitions are raw, they will eventually get popped one-by-one
    // until everything it's all empty.
    global_functions: HashMap<defs::FunctionSignature, defs::FunctionDefinition>,
    methods: HashMap<defs::Type, HashMap<defs::FunctionSignature, defs::FunctionDefinition>>,
}

impl Resolver {

    /// Creates a new instance of [`Resolver`] with the provided block [`blks::BlockDefinitions`] and
    /// defs [`defs::Definitions`].
    /// 
    /// Note: If there are multiple definitions of the same opcode provided in `blks`, they will
    /// implicitly get overriden by a definition located on a higher index. This also applies for
    /// defs but the same function signature.
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
    /// will return a [`HashMap<Signature, Definition>`] which contains
    /// each of definitions' blocks representation.
    pub fn resolve(mut self) -> Result<HashMap<Signature, Definition>, ResolveError> {
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

            // we then resolve its signature and definitions using `resolve_definitions()`
            let (signature, definition) =
                self.resolve_definitions(
                    &mut cache,
                    key, func,

                    &mut vec![],

                    /* required type: None, since this is a regular statement, the block doesn't need to return anything */
                    None
                )?;

            cache.insert(signature, definition);
        }

        // also do it for methods
        while !self.methods.is_empty() {
            let typ = self.methods.keys().next().unwrap().clone();
            // safety: .unwrap() is safe because the `while` block above
            //         makes sure that this block is run only when the hashmap
            //         is not empty

            while !self.methods.get(&typ).unwrap().is_empty() {
                let type_methods = self.methods.get_mut(&typ).unwrap();
                let key = type_methods.keys().next().unwrap().clone();
                // safety: .unwrap() is safe, see comments above
                
                let func = type_methods.remove(&key).unwrap();
                // safety: .unwrap() is safe because we got the key from the hashmap

                // we then resolve its signature and definitions using `resolve_definitions()`
                let (signature, definition) =
                    self.resolve_definitions(
                        &mut cache,
                        key, func,

                        &mut vec![],

                        None
                    )?;

                cache.insert(signature, definition);
            }
        
            // after we've done with that type we remove it
            self.methods.remove(&typ);
        }

        Ok(cache)
    }

    // Resolves a `FunctionSignature` and `FunctionDefinition`
    fn resolve_definitions(
        &mut self,
        cache: &mut HashMap<Signature, Definition>,

        defs_signature: defs::FunctionSignature,
        defs_definition: defs::FunctionDefinition,

        visited: &mut Vec<Signature>,

        // None if the function acts as a statement,
        // Some if it's used as an argument or needed to return a value
        required_type: Option<&defs::Type>,
    ) -> Result<(Signature, Definition), ResolveError> {
        // we then loop over the statements in the function
        // to convert them into actual blocks
        let mut blocks = defs_definition.statements
            .into_iter()
            .try_fold(vec![], |mut acc, dispatch| {
                let (blocks, return_block) = self.resolve_dispatch(
                    &dispatch, visited, cache,

                    /* parent parameter types */ &defs_signature.parameters,
                    /* parent this type */ defs_signature.this.borrow().as_ref(),

                    /* required type: None, since this is a regular statement, the block doesn't need to return anything */
                    None
                )?;

                // we shouldn't have any return block since we set the `parent_required_type`
                // into None.
                if return_block.is_some() {
                    return Err(ResolveError::ReturningExpressionAsStatement);
                }

                acc.extend(blocks);
                Ok(acc)
            })?;
        
        // then convert the return dispatch (if it has one)
        let return_block = if let Some(return_statement) = defs_definition.return_statement {
            let (blocks_result, return_block) = self.resolve_dispatch(
                &return_statement, visited, cache,

                /* parent parameter types */ &defs_signature.parameters,
                /* parent this type */ defs_signature.this.borrow().as_ref(),

                required_type
            )?;

            blocks.extend(blocks_result);

            return_block
        } else { None };

        // convert defs' `FunctionSignature` to our `Signature::Function`
        let signature = Signature::Function {
            name: defs_signature.name, this: defs_signature.this,
            ret_type: defs_signature.return_type, parameter_types: defs_signature.parameters
        };

        // now that we have all of our blocks and a signature to identify this function
        // we return it
        Ok((
            signature.clone(),
            Definition {
                blocks: DefinitionBlocks {
                    return_block, blocks
                },
                signature
            }
        ))
    }

    // todo: return borrowed values instead (`Result<(&Vec<Block>, &Option<Block>), ResolveError>`)
    // to reduce clone() calls :s - or maybe use Rc<T>?? hmmm
    // then somehow bind it into `cache`, since those values belong to the `cache` HashMap.
    fn resolve_dispatch(
        &mut self,
        dispatch: &defs::Dispatch,

        // this `visited` parameter is used to prevent cyclic dependency,
        // as we traverse down into calls of functions, we check on each
        // function resolving if we have already visited it before, if it
        // does, then that function has already been visited before.
        //
        // after the parsing of a function's statements, we pop the function
        // from the visited list.
        visited: &mut Vec<Signature>,
        cache: &mut HashMap<Signature, Definition>,

        parent_parameter_types: &Vec<defs::Type>,
        parent_this_type: Option<&defs::Type>,

        // `resolve_dispatch` doesn't check if the dispatch given coerces 
        // to this type vvv
        parent_required_type: Option<&defs::Type>,
    ) -> Result<(Vec<Block>, Option<Block>), ResolveError> {
        Ok(match dispatch.kind {
            defs::DispatchKind::RawBlock => {
                // take a block that matches the opcode
                let def = self.blocks.get(&dispatch.identifier)
                    .ok_or_else(|| ResolveError::BlockNotFound { opcode: dispatch.identifier.clone() })?
                    .clone();

                let def_args = def.spec.get_args();
                
                if def_args.len() > dispatch.arguments.len() {
                    return Err(ResolveError::TooManyArguments);
                }

                let mut result = vec![];

                // resolve its arguments
                let arguments = def_args.into_iter()
                    .enumerate()
                    .map::<Result<Argument, ResolveError>, _>(|(idx, arg)| {
                        let spec_type = Self::swrs_arg_to_type(arg);
                        let Some(given_argument) = dispatch.arguments.get(idx) else {
                            return Err(ResolveError::TooLittleArguments);
                        };

                        let given_argument_type = self.def_block_argument_as_type(
                            given_argument,
                            visited, cache,
                            parent_parameter_types, parent_this_type,
                            Some(&spec_type)
                        )?;

                        // check if the two matches
                        if spec_type != given_argument_type {
                            return Err(ResolveError::InvalidArgumentType {
                                given: Some(given_argument_type),
                                required: Some(spec_type)
                            });
                        }

                        // convert defs' `BlockArgument` into our `Argument` type
                        // defs' `BlockArgument` has a `Dispatch` variant, whilst our type
                        // returns the block directly (`Block` variant)
                        Ok(match given_argument {
                            defs::BlockArgument::Dispatch(disp) => {
                                // this is a dispatch to either a raw block or a function call
                                let (blocks, return_block) = self.resolve_dispatch(
                                    disp,
                                    visited, cache,

                                    parent_parameter_types, parent_this_type,
                                    Some(&given_argument_type)
                                )?;

                                // make sure that `return_block` actually returns something
                                let Some(return_block) = return_block else {
                                    return Err(ResolveError::InvalidArgumentType {
                                        given: None, required: Some(given_argument_type)
                                    });
                                };

                                // check if it's the same type
                                if let Some(return_type) = &return_block.return_type {
                                    if return_type != &given_argument_type {
                                        return Err(ResolveError::InvalidArgumentType {
                                            given: return_block.return_type.clone(),
                                            required: Some(given_argument_type)
                                        });
                                    }
                                } else {
                                    return Err(ResolveError::InvalidArgumentType {
                                        given: None,
                                        required: Some(given_argument_type)
                                    });
                                }

                                result.extend(blocks);

                                Argument::Block(return_block)
                            },

                            // these types are guaranteed to be correct, since we've done the
                            // checking a few lines above
                            defs::BlockArgument::Literal(lit) => Argument::Literal(lit.clone()),
                            defs::BlockArgument::Argument { index } => Argument::Argument(*index),
                            defs::BlockArgument::This => Argument::This,
                        })
                    }).collect::<Result<Vec<Argument>, _>>()?;

                let block_type = Self::def_block_type_as_type(&def.block_type);

                // `resolve_dispatch` doesn't care about whether the block type coerces
                // to the parent's required type

                // let parent_required_type = parent_required_type.cloned();

                // // check if this raw block matches with the `parent_required_type`
                // if block_type != parent_required_type {
                //     // doesn't match! incorrect argument type passed
                //     return Err(ResolveError::InvalidArgumentType {
                //         given: block_type,
                //         required: parent_required_type
                //     });
                // }

                // if it has a block type, it must be a returning block
                // if not then add to the result and set None for the return block
                if let Some(_) = block_type {
                    (result, Some(Block {
                        opcode: def.opcode,
                        spec: def.spec,
                        arguments,
                        return_type: block_type
                    }))
                } else {
                    result.push(Block {
                        opcode: def.opcode,
                        spec: def.spec,
                        arguments,
                        return_type: block_type
                    });

                    (result, None)
                }
            },
            defs::DispatchKind::FunctionDispatch => {
                // convert this dispatch's defs' signature to our own Signature
                let this_type = dispatch.this.as_ref()
                        .map(|block_arg|
                            self.def_block_argument_as_type(
                                block_arg.as_ref(),
                                visited, cache,
                                parent_parameter_types, parent_this_type, parent_required_type
                            )
                        )
                        // flip `Option<Result<A, B>>` into `Result<Option<A>, B>`
                        .map_or(Ok(None), |v| v.map(Some))?;

                let parameter_types = dispatch.arguments.iter()
                        .map(|arg|
                            self.def_block_argument_as_type(
                                arg,
                                visited, cache,
                                parent_parameter_types, parent_this_type, parent_required_type
                            )
                        ).collect::<Result<Vec<_>, _>>()?;

                let defs_signature = defs::FunctionSignature {
                    this: this_type.clone(),
                    parameters: parameter_types.clone(),
                    name: dispatch.identifier.clone(),
                    return_type: parent_required_type.cloned(),
                };
                
                let signature = Signature::Function {
                    name: dispatch.identifier.clone(),
                    this: this_type.clone(),
                    ret_type: parent_required_type.cloned(),
                    parameter_types,
                };

                // check if we already cached this function
                if let Some(definition) = cache.get(&signature) {
                    return Ok(
                        (definition.blocks.blocks.clone(), definition.blocks.return_block.clone())
                    );
                }

                // find a function that matches that signature
                // if we have a `this` type, we must search in the `.methods`,
                // otherwise `.global_functions`
                let defs_func = if let Some(typ) = this_type {
                    // get the methods list of the associated type, and then get the method
                    // with the matching signature
                    let Some(defs_method) = self.methods.get_mut(&typ)
                        .map(|methods| methods.remove(&defs_signature))
                        .flatten() else {
                            return Err(ResolveError::DefinitionNotFound { signature });
                        };
                    defs_method
                } else {
                    // having no `this` type indicates that this is a global function
                    let Some(defs_func) = self.global_functions
                        .remove(&defs_signature) else {
                            return Err(ResolveError::DefinitionNotFound { signature });
                        };

                    defs_func
                };

                // insert ourselves into visited
                visited.push(signature);

                // then we resolve its definitions
                let (signature, definition) = self.resolve_definitions(
                    cache,
                    defs_signature, defs_func,
                    visited,
                    parent_required_type
                )?;

                // pop ourselves out since we're done
                visited.pop();

                // get our blocks first
                let blocks = definition.blocks.blocks.clone();
                let return_block = definition.blocks.return_block.clone();

                // add this function to our cache
                cache.insert(signature, definition); 

                (blocks, return_block)
            },
        })
    }

    /// Infer a [`defs::Type`] from swrs' [`BlockType`]. `None` when the block type
    /// is not an argument block (either a regular block or a control block).
    fn def_block_type_as_type(block_type: &BlockType) -> Option<defs::Type> {
        if let BlockType::Argument(arg) = block_type {
            Some(match arg {
                ArgumentBlockReturnType::Boolean => defs::Type::Boolean,
                ArgumentBlockReturnType::String => defs::Type::String,
                ArgumentBlockReturnType::Number => defs::Type::Number,
                ArgumentBlockReturnType::View { .. } =>
                    unimplemented!("implement view"),
                ArgumentBlockReturnType::Component { .. } =>
                    unimplemented!("implement component"),
                ArgumentBlockReturnType::List { .. } =>
                    unimplemented!("implement list"),
            })
        } else { None }
    }

    /// Converts a [`defs::BlockArgument`] into [`defs::Type`].
    /// Since [`defs::BlockArgument`] can be [`defs::BlockArgument::Dispatch`], it's necessary
    /// for this function to resolve the function.
    fn def_block_argument_as_type(
        &mut self,
        arg: &defs::BlockArgument,

        visited: &mut Vec<Signature>,
        cache: &mut HashMap<Signature, Definition>,

        parent_parameter_types: &Vec<defs::Type>,
        parent_this_type: Option<&defs::Type>,
        parent_required_type: Option<&defs::Type>,
    ) -> Result<defs::Type, ResolveError> {
        Ok(match arg {
            defs::BlockArgument::Dispatch(disp) => {
                // since this argument is a dispatch, we must resolve it first
                // to know it's return value
                
                let (_, return_block) = self.resolve_dispatch(
                    disp,
                    visited, cache,
                    parent_parameter_types, parent_this_type, parent_required_type
                )?;

                let Some(return_block) = return_block else {
                    return Err(ResolveError::InvalidArgumentType {
                        given: return_block
                            .map(|block| block.return_type)
                            .flatten(),
                        required: parent_required_type.cloned()
                    });
                };

                // we must return a type, `def_block_argument_as_type` cannot be
                // changed to return `Option<defs::Type>`. since as from it's name
                // "def_block_ARGUMENT_as_type".
                return_block.return_type
                    .ok_or_else(|| ResolveError::InvalidArgumentType {
                        given: None, required: parent_required_type.cloned()
                    })?
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

    fn literal_to_type(lit: &defs::Literal) -> defs::Type {
        match lit {
            defs::Literal::Boolean(_) => defs::Type::Boolean,
            defs::Literal::Number(_) => defs::Type::Number,
            defs::Literal::String(_) => defs::Type::String,
        }
    }

    fn swrs_arg_to_type(arg: &swrs::api::block::Argument) -> defs::Type {
        match arg {
            swrs::api::block::Argument::String { .. } => defs::Type::String,
            swrs::api::block::Argument::Number { .. } => defs::Type::Number,
            swrs::api::block::Argument::Boolean { .. } => defs::Type::Boolean,
            swrs::api::block::Argument::Menu { .. } => todo!("how to get view/component/map/list types from menu?"),
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