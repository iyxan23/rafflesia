//! # Padma Resolver
//! 
//! The code that processes parsed defs ([`crate::defs::Definitions`]), matching
//! and combining them with parsed blks ([`crate::blks::BlockDefinitions`]), to generate
//! a single data structure of which every definitions (functions, methods) are flattened
//! into blocks which could then be emitted as compiler output.

mod models;
use std::{collections::{BTreeMap, HashMap}, rc::Rc};

pub use models::*;
use swrs::api::block::{
    Argument as SWRSArgument, BlockType as SWRSBlockType,
    ArgumentBlockReturnType as SWRSArgumentBlockReturnType
};

use crate::{
    blks::{BlockDefinitions, BlockDefinition},
    defs::{
        FunctionDeclaration as DefsFunctionDeclaration, FunctionBody as DefsFunctionBody,
        BindingDeclaration as DefsBindingDeclaration, BindingBody as DefsBindingBody,
        Type as DefsType, Statement as DefsStatement, Expression as DefsExpression,
        StaticVariable as DefsStaticVariable
    }
};

use super::defs::Definitions as DefsDefinitions;

pub struct ResolverPayload {
    definitions: BTreeMap<u32, DefsDefinitions>,
    blocks: BTreeMap<u32, BlockDefinitions>,
}

impl ResolverPayload {
    pub fn new(definitions: BTreeMap<u32, DefsDefinitions>, blocks: BTreeMap<u32, BlockDefinitions>) -> Self {
        Self { definitions, blocks }
    }

    pub fn add_definitions(&mut self, def: DefsDefinitions, priority: u32) {
        self.definitions.insert(priority, def);
    }

    pub fn add_blocks(&mut self, blocks: BlockDefinitions, priority: u32) {
        self.blocks.insert(priority, blocks);
    }

    pub fn flatten(self) -> Resolver {
        let (flattened_definitions, flattened_bindings) = self
            // since BTreeMap is ordered, `into_iter()` iterates in an ascending order
            // which is exactly what we want. Higher priorities gets placed last (overrides anyone).
            .definitions.into_iter()
            .fold(
                (HashMap::<String, HashMap<Signature, DefsFunctionBody>>::new(),
                HashMap::<DefsBindingDeclaration, DefsBindingBody>::new()),
                |(mut definitions, mut bindings), (_prio, defs)| {
                    for (dec, body) in defs.global_functions {
                        if let Some(defs) = definitions.get_mut(&dec.name) {
                            defs.insert(Signature::Function {
                                name: dec.name,
                                parameters: dec.parameters,
                                return_type: dec.return_type,
                            }, body);
                        } else {
                            let mut hmap = HashMap::new();
                            hmap.insert(Signature::Function {
                                name: dec.name.clone(),
                                parameters: dec.parameters,
                                return_type: dec.return_type,
                            }, body);

                            definitions.insert(dec.name, hmap);
                        }
                    }

                    for (typ, methods) in defs.methods {
                        for (dec, body) in methods {
                            assert_eq!(Some(&typ), dec.this.as_ref());

                            if let Some(defs) = definitions.get_mut(&dec.name) {
                                defs.insert(Signature::Method {
                                    this: typ.clone(),
                                    name: dec.name,
                                    parameters: dec.parameters,
                                    return_type: dec.return_type,
                                }, body);
                            } else {
                                let mut hmap = HashMap::new();
                                hmap.insert(Signature::Method {
                                    this: typ.clone(),
                                    name: dec.name.clone(),
                                    parameters: dec.parameters,
                                    return_type: dec.return_type,
                                }, body);

                                definitions.insert(dec.name, hmap);
                            }
                        }
                    }

                    bindings.extend(defs.bindings);

                    (definitions, bindings)
                });

        Resolver {
            definitions: flattened_definitions,
            bindings: flattened_bindings,

            blocks: self
                .blocks.into_iter()
                .fold(Default::default(), |mut acc, (_prio, blks)| {
                    blks.0.into_iter()
                        .for_each(|blk| { acc.insert(blk.opcode.clone(), blk); });

                    acc
                }),
        }
    }
}

pub struct Resolver {
    definitions: HashMap<String, HashMap<Signature, DefsFunctionBody>>,
    bindings: HashMap<DefsBindingDeclaration, DefsBindingBody>,

    blocks: HashMap<String, BlockDefinition>,
}

impl Resolver {
    pub fn resolve(mut self) -> Definitions {
        let mut result = Definitions {
            blocks: vec![], definitions: HashMap::new(),
        };

        // first we resolve bindings, which might resolve some other definitions.
        self.resolve_bindings(&mut result);

        // second, we'll loop over definitions and resolve each of them.
        let mut seen_functions = vec![];

        let mut resolved_definitions = HashMap::new();

        while let Some((_name, mut overloads))
            = self.definitions
                .keys().next().cloned()
                .and_then(|key| self.definitions.remove_entry(&key)) {

            while let Some((signature, body))
                    = overloads.keys().next().cloned()
                    .and_then(|key| overloads.remove_entry(&key)) {

                let (this_type, args_types) = match &signature {
                    Signature::Function { parameters, .. } => (None, parameters),
                    Signature::Method { this, parameters, .. } => (Some(this), parameters),
                };

                let resolved = self.resolve_function_body(
                    body, &mut seen_functions, &mut resolved_definitions, None,
                    this_type, args_types
                );

                result.definitions.insert(signature, resolved);
            }
        }

        // finally we all got the definitions resolved!
        result
    }

    /// Turns bindings into funcitons/methods and inserts them into the given
    /// mutable borrow of [`Definitions`].
    fn resolve_bindings(&mut self, result: &mut Definitions) {
        // while let Some((declaration, body)) = {
        //     let next = self.bindings
        //         .keys().next()
        //         .cloned();

        //     if let Some(next) = next {
        //         self.bindings.remove_entry(&next)
        //     } else { None }
        // } {
        //     if let Some(this) = declaration.this {
        //         // a method

        //         let (function_body_expr, parameter_types) =
        //             convert_to_expression(body, &self.blocks);
                
        //         if let Some(declared_params) = &declaration.parameters {
        //             // verify if both the inferred type and the declared parameters matches.
        //             // (only match the ones the inferred type, any other extra declared parameters can be ignored)
        //             if declared_params[..parameter_types.len()].eq(&parameter_types) {
        //                 // todo: return an err
        //                 panic!("declared parameters doesn't share the same inferred types");
        //             }
        //         }

        //         let function_declaration = DefsFunctionDeclaration {
        //             this: Some(this), 
        //             name: declaration.name,
        //             return_type: declaration.return_type,

        //             parameters: declaration
        //                 .parameters
        //                 .unwrap_or(parameter_types),
        //         };

        //         // infers types from the body
        //         //
        //         // so for example, we give a block arguments of (@0, @1) and the block's 
        //         // spec provides that it has two of those parameters, use those types.
        //         //
        //         // also check for "ambiguity", where an argument might be used in another
        //         // nested block / function call, where they are used as a different type!
        //         // that'll throw out an error.
        //         fn infer_arg_types(expr: DefsBindingBody) -> Vec<DefsType> {
        //             match expr {
        //                 DefsBindingBody::Block { opcode, arguments } => {

        //                 },
        //                 DefsBindingBody::FunctionCall { name, arguments } => {

        //                 },
        //                 DefsBindingBody::MethodCall { name, this, arguments } => {
                            
        //                 },
        //             }
        //         }

        //         let function_body = DefsFunctionBody {
        //             statements: vec![if declaration.return_type.is_some() {
        //                 DefsStatement::Return { value: convert_to_expression(body) }
        //             } else {
                        
        //                     // safety: .unwrap() is safe because the function conver_to_expression
        //                     //         never constructs a `DefsExpression::StaticVariable`, which
        //                     //         is the only reason the TryInto<DefsStatement> of
        //                     //         DefsExpression impl will fail.
        //                     .try_into().unwrap()
        //             }],
        //         };

        //         fn convert_to_expression(
        //             body: DefsBindingBody,
        //             blocks: &HashMap<String, BlockDefinition>,
        //         ) -> (DefsExpression, Vec<DefsType>) {
        //             match body {
        //                 DefsBindingBody::Block { opcode, arguments } => {
        //                     DefsExpression::Block {
        //                         opcode, arguments: arguments.unwrap_or_else(|| {
        //                             // infer types from this block
        //                             blocks.get(opcode)
        //                         })
        //                     }
        //                 },
        //                 DefsBindingBody::FunctionCall { name, arguments } => todo!(),
        //                 DefsBindingBody::MethodCall { name, this, arguments } => todo!(),
        //             }
        //         }

        //         self.methods
        //             .entry(&this)
        //             .and_modify(|hmap| )
        //             .or_insert_with(|| )
        //     } else {
        //         // a function
        //     }
        // }
        todo!()
    }

    // NOTE:
    //   DefsStatement -> DefinitionBlocks
    //   DefsExpression -> BlockArgument
    //
    // Statement and expressions are treated differently here. A statement may not
    // be an expression, because expression things like literals, this, or arguments
    // can't be an expression, they can only be passed into other blocks things;
    // exactly what a BlockArgument is.
    // 
    // For instance, there are two types of blocks: A block statement and an expression
    // block (returning block). An expression block cannot be a statement, and vice versa.
    // It is separated exactly because of how the block system in sketchware works.
    //
    // Here, functions are like macros (they are); it can be a function statement or
    // function expression. The problem here is that, a function can contain many blocks
    // inside it. For a function statement, this is not a problem; we only need to copy
    // paste it one-by-one. But a problem arises on function expressions.
    //
    // Statement blocks / statement functions are possible inside a function expression.
    // Since we can't just pack all of them into "one expression"; what we do is that we
    // take the last block of the function expression / the return value
    // 
    //    | func my_func() {
    //    |   #blk;
    //    |   #blk;
    // => |   < #returning;
    //    | }
    /// 
    // we then put that as the argument of where the function expression is called.
    //
    //    | #lorem(  my_func() )
    //    :        vvvvvvvvvvvv
    //    | #lorem( #returning )
    //
    // and copy-paste it's "previous" blocks.
    //
    // => + #blk;
    // => + #blk;
    //    | #lorem( #returning )
    //
    // if there are multiple arguments, we just add them by the order.
    //
    //   | #lorem(my_func(), other_func())
    //   : vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
    //   :
    //   + #blk; <-- my_func
    //   + #blk; <-- my_func
    //   + #other_blk; <- other_func
    //   + #other_blk; <- other_func
    //   :
    //   | #lorem(#returning, #other_returning)
    //
    // ==> PROBLEM: if there are multiple functions, a block in the second argument
    //              might change some global state that results in the first argument
    //              being changed.
    //
    //   => we can use variables, but that's a complex feature that we'll later implement :>

    /// Resolves the given [`DefsFunctionBody`]. It will try to retrieve the functions
    /// through the given lookup argument and inserts it into `&mut self` (but the
    /// result of this function won't, it will only be returned).
    fn resolve_function_body(
        &mut self,
        body: DefsFunctionBody,
        seen_functions: &mut Vec<Signature>,
        cache: &mut HashMap<String, HashMap<Signature, (DefinitionBlocks, Option<BlockArgument>)>>,

        return_type: Option<DefsType>,

        this_type: Option<&DefsType>,
        args_types: &Vec<DefsType>,
    ) -> (DefinitionBlocks, Option<BlockArgument>) {
        let mut result = DefinitionBlocks::new();
        let mut return_value = None;

        for stmt in body.statements {
            match stmt {
                DefsStatement::Block { opcode, arguments } => {
                    let block = self.blocks.get(&opcode)
                        // todo: error propagation
                        .expect("todo: error: no block with opcode {} found");

                    // make sure the block's return type matches with the needed return type
                    if swrs_block_type_to_type(&block.block_type).is_some() {
                        // todo: propagate errors
                        panic!("error: argument block as a statement, a statement must not return a value.");
                    }

                    // make sure that the arguments are given as defined from the block spec's args
                    let spec_args: Vec<&SWRSArgument> = block.spec.get_args();
                    if spec_args.len() != arguments.len() {
                        // todo: error propagation
                        panic!("invalid arguments given");
                    }

                    let arguments =
                        spec_args.into_iter()
                            .zip(arguments.into_iter())
                            .map(|(spec_arg, expr)| (swrs_arg_to_type(spec_arg), expr))
                            .collect::<Vec<_>>();

                    result.push(Rc::new(Block {
                        opcode, content: block.spec.clone(),
                        arguments: arguments.into_iter()
                            .map(|(typ, expr)| {
                                let overloads = self.resolve_expression(
                                    expr, seen_functions, cache,
                                    this_type, args_types,
                                );

                                // result.extend(blocks);
                                // value
                                todo!("find overload that has the right typ")
                            }).collect(),

                        return_type: None,
                    }));
                },
                DefsStatement::FunctionCall { name, arguments } => { 
                    // let arguments = arguments.into_iter()
                    //     .map(|expr|
                    //         self.resolve_expression(
                    //             expr, seen_functions, cache,
                    //             this_type, args_types
                    //         ))
                    //     .collect::<Vec<_>>();

                    // let signature = Signature::Function {
                    //     name,
                    //     parameters: arguments.iter()
                    //         .map(|arg|
                    //             block_argument_as_type(&arg, this_type, args_types))
                    //         .collect::<Vec<_>>(),

                    //     return_type: None,
                    // };
                    todo!("update with the new method of indexing a definition with the name")
                },
                DefsStatement::MethodCall { name, arguments, this } => {
                    todo!()
                },
                DefsStatement::Return { value } => {
                    let Some(return_type) = return_type
                        else {
                            // todo: error propagation
                            panic!("no return type, given a return statement")
                        };

                    let overloads =
                        self.resolve_expression(
                            value, seen_functions, cache,
                            this_type, args_types
                        );

                    // find for the right return type
                    let Some((res_blocks, res_return_block)) = overloads.into_iter()
                        .find(|(_blocks, block_arg)|
                            block_argument_as_type(block_arg, this_type, args_types) == Some(&return_type)
                        ) else {
                            // todo: error propagation
                            panic!("err: expression doesn't return type .., required by function signature")
                        };

                    result.extend(res_blocks);
                    return_value = Some(res_return_block);

                    // skip any other statements, we're done here
                    break;
                },
            }
        }

        (result, return_value)
    }

    /// The underlying implementation of resolve_expression or resolve_function_body to resolve
    /// functions or methods from raw parts (its name and its arguments). The function will
    /// generate a signature for it and generates its flattened blocks--which gets added into
    /// the cache--and returned.
    fn resolve_from_raw_function(
        &mut self,
        name: String,
        arguments: Vec<DefsExpression>,
        this_expr: Option<DefsExpression>,

        // todo: seen functions isnt being checked
        seen_functions: &mut Vec<Signature>,
        cache: &mut HashMap<String, HashMap<Signature, (DefinitionBlocks, Option<BlockArgument>)>>,

        this_type: Option<&DefsType>,
        args_types: &Vec<DefsType>,
    ) -> Vec<(DefinitionBlocks, Option<BlockArgument>)> {
        // a vector of vectors, yikes.
        // the outer vector are each of the arguments
        // and the inner vector contains the possible overloads of that argument
        let arguments = arguments
            .into_iter().map(|expr| {
                self.resolve_expression(
                    expr, seen_functions, cache, this_type, args_types
                )
            }).collect::<Vec<Vec<_>>>();

        // check the cache
        if let Some(cached_funcs) = cache.get(&name) {
            // the function must be here, if defined properly. Since on every new name we
            // enconter, we will always resolve every signature of that function.

            // what we're going to do here is to find the correct combination
            // of arguments' types that matches any signature that has types,
            // and get their blocks
            let possible_parameters = cached_funcs.keys()
                .into_iter()
                .map(|signature| match signature {
                    Signature::Function { parameters, .. } |
                    Signature::Method { parameters, .. } => parameters,
                })
                .collect::<Vec<_>>();

            let matched_arguments = arguments.into_iter()
                .enumerate()
                .map(|(arg_idx, combinations)| {
                    let Some((blocks, block_arg, _typ)) = combinations.into_iter()
                        .map(|(blocks, block_arg)| 
                            (blocks, block_argument_as_type(&block_arg, this_type, args_types).cloned(), block_arg))
                        .find_map(|(blocks, arg_type, block_arg)| {
                            // todo: error propagation
                            let arg_type = arg_type.expect("err: argument .. doesn't return a value");

                            // fixme: this matches with the wrong args types
                            args_types.get(arg_idx).map(|typ|
                                (typ == &arg_type)
                                    .then(|| (blocks, block_arg, arg_type))
                            ).flatten()
                        }) else {
                            // todo: error propagation
                            panic!("err: argument .. doesn't return type .. needed for arg_type ..");
                        };

                    (blocks, block_arg)
                })
                .collect::<Vec<_>>();
        }

        let overloads = self.definitions.remove(&name)
            // todo: error propagation
            .expect("err: no such function named ...")
            .into_iter()
            .map(|(sign, body)| {
                // retrieve parameter types, return types and this type from the signature
                // of the funtion
                let (args_types, return_type, this_type) = match &sign {
                    Signature::Function { parameters, return_type, .. }
                        => (parameters, return_type, None),
                    Signature::Method { this, parameters, return_type, .. }
                        => (parameters, return_type, Some(this)),
                };

                // fixme: i feel like `sign` could be moved without cloning here
                (sign.clone(), self.resolve_function_body(
                    body, seen_functions, cache,
                    return_type.clone(), this_type, args_types
                ))
            })
            .collect::<Vec<_>>();
        
        // insert em to the cache
        let mut overloads_cache = HashMap::new();

        let result = overloads.into_iter()
            .map(|(sign, resolved)| {
                overloads_cache.insert(sign, resolved.clone()); resolved
            })
            .collect();

        cache.insert(name, overloads_cache);

        result
    }

    // returns a vector possible overloads of the same expression, expressions
    // that might be the same, but are returning different types.
    fn resolve_expression(
        &mut self,
        expr: DefsExpression,
        seen_functions: &mut Vec<Signature>,
        cache: &mut HashMap<String, HashMap<Signature, (DefinitionBlocks, Option<BlockArgument>)>>,

        this_type: Option<&DefsType>,
        args_types: &Vec<DefsType>,
    ) -> Vec<(DefinitionBlocks, BlockArgument)> {
        match expr {
            DefsExpression::Block { opcode, arguments } => {
                let block = self.blocks.get(&opcode)
                    // todo: error propagation
                    .expect("todo: error: no block with opcode {} found");

                // make sure the block's return type matches with the needed return type
                let Some(block_type) = swrs_block_type_to_type(&block.block_type)
                    else {
                        // todo: propagate errors
                        panic!("error: required type .. but block doesnt return anything; use this as a statement?");
                    };

                // if block_type != return_type {
                //     // todo: error propagation
                //     panic!("error: block returns .. but required type ..")
                // }

                // make sure that the arguments are given as defined from the block spec's args
                let spec_args = block.spec.get_args();
                if spec_args.len() != arguments.len() {
                    // todo: error propagation
                    panic!("invalid arguments given")
                }

                let arguments =
                    spec_args.into_iter()
                        .zip(arguments.into_iter())
                        .map(|(spec_arg, expr)| (swrs_arg_to_type(spec_arg), expr))
                        .collect::<Vec<_>>();

                let mut result = vec![];
                let ret_val = BlockArgument::Block(Rc::new(Block {
                    opcode, content: block.spec.clone(),
                    arguments: arguments.into_iter()
                        .map(|(typ, expr)| {
                            let overloads = self.resolve_expression(
                                expr, seen_functions, cache,
                                this_type, args_types,
                            );

                            // find the right overload that matches the type
                            let Some((blocks, value)) = 
                                overloads
                                    .into_iter()
                                    .find(|(_blocks, arg)|
                                        block_argument_as_type(arg, this_type, args_types)
                                            .and_then(|arg| Some(arg == &typ))
                                            .unwrap_or(false))
                                    else {
                                        // todo: error propagation
                                        panic!("no matching overload for this expression")
                                    };

                            result.extend(blocks);

                            value
                        }).collect(),

                    return_type: Some(block_type)
                }));

                vec![(result, ret_val)]
            },

            DefsExpression::FunctionCall { name, arguments } => {
                let overloads = self.resolve_from_raw_function(
                    name, arguments, None, seen_functions,
                    cache, this_type, args_types
                ).into_iter()
                    // filter any overloads that doesn't return anything
                    // expressions **must** return something
                    .filter_map(|(blocks, arg)|
                        arg.and_then(|arg| Some((blocks, arg))))
                    .collect::<Vec<_>>();

                if overloads.len() == 0 {
                    // todo: error propagation
                    panic!("any of this function with name .. definitions doesnt return any value");
                }

                overloads
            },

            DefsExpression::MethodCall { name, arguments, this } => {
                let overloads = self.resolve_from_raw_function(
                    name, arguments, None, seen_functions,
                    cache, this_type, args_types
                ).into_iter()
                    // filter any overloads that doesn't return anything
                    // expressions **must** return something
                    .filter_map(|(blocks, arg)|
                        arg.and_then(|arg| Some((blocks, arg))))
                    .collect::<Vec<_>>();
                
                if overloads.len() == 0 {
                    // todo: error propagation
                    panic!("any of this function with name .. definitions doesnt return any value");
                }

                overloads
            },

            DefsExpression::StaticVariable(static_var)
                => vec![(vec![], match static_var {
                    DefsStaticVariable::Literal(lit) => BlockArgument::Literal(lit),
                    DefsStaticVariable::Argument(arg) => BlockArgument::Argument(arg),
                    DefsStaticVariable::This => BlockArgument::This,
                })],
        }
    }
}

fn apply_block_arguments(
    block: &mut Block,
    arguments: &Vec<(Vec<Rc<Block>>, BlockArgument)>,
    the_this: Option<&BlockArgument>,
    result_blocks: &mut Vec<Rc<Block>>,
) {
    block.arguments.iter_mut()
        .for_each(|arg|
            apply_block_argument_arguments(arg, arguments, the_this, result_blocks)
        );
}

fn apply_block_argument_arguments(
    arg: &mut BlockArgument,
    arguments: &Vec<(Vec<Rc<Block>>, BlockArgument)>,
    the_this: Option<&BlockArgument>,
    result_blocks: &mut Vec<Rc<Block>>,
) {
    match arg {
        BlockArgument::Argument(arg_no) => {
            let (blocks, ret_block) = arguments.get(*arg_no as usize)
                // safety: .unwrap is safe here because the arguments variable
                //         has been checked thoroughly by the codes above
                .unwrap().clone();

            *arg = ret_block;
            result_blocks.extend(blocks);
        }
        BlockArgument::This => {
            // todo: error propagation
            *arg = the_this.expect("`this` not provided").clone();
        }
        BlockArgument::Block(blk) => {
            // recursively apply arguments on nested blocks
            let blk_ref = Rc::make_mut(blk);

            apply_block_arguments(
                blk_ref, arguments, the_this, result_blocks
            );

            *arg = BlockArgument::Block(Rc::clone(blk));
        }
        _ => (),
    }
}
fn block_argument_as_type<'a, 'v: 'a>(
    block_arg: &'a BlockArgument,

    this_type: Option<&'a DefsType>,
    args_types: &'v Vec<DefsType>
) -> Option<&'a DefsType> {
    match block_arg {
        BlockArgument::Literal(lit) => Some(lit.as_ref()),
        BlockArgument::Block(blk) => blk.as_ref().return_type.as_ref(),
        BlockArgument::Argument(arg) => args_types.get(*arg as usize),
        BlockArgument::This => this_type,
    }
}

// returns None if the given block type is a regular type
fn swrs_block_type_to_type(arg: &SWRSBlockType) -> Option<DefsType> {
    Some(match arg {
        SWRSBlockType::Argument(SWRSArgumentBlockReturnType::Boolean) => DefsType::Boolean,
        SWRSBlockType::Argument(SWRSArgumentBlockReturnType::Number) => DefsType::Boolean,
        SWRSBlockType::Argument(SWRSArgumentBlockReturnType::String) => DefsType::Boolean,
        
        SWRSBlockType::Argument(SWRSArgumentBlockReturnType::Component { .. }) |
        SWRSBlockType::Argument(SWRSArgumentBlockReturnType::List { ..  }) |
        SWRSBlockType::Argument(SWRSArgumentBlockReturnType::View { ..  })
            => todo!("types other than primitive types"),

        SWRSBlockType::Regular | SWRSBlockType::Control(_) => None?,
    })
}
fn swrs_arg_to_type(arg: &SWRSArgument) -> DefsType {
    match arg {
        SWRSArgument::String { .. } => DefsType::String,
        SWRSArgument::Number { .. } => DefsType::Number,
        SWRSArgument::Boolean { .. } => DefsType::Boolean,

        SWRSArgument::Menu { name, .. }
            => todo!("types other than primitive types"),
    }
}
