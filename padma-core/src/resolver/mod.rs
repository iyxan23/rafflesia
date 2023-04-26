//! # Padma Resolver
//! 
//! The code that processes parsed defs ([`crate::defs::Definitions`]), matching
//! and combining them with parsed blks ([`crate::blks::BlockDefinitions`]), to generate
//! a single data structure of which every definitions (functions, methods) are flattened
//! into blocks which could then be emitted as compiler output.

mod models;
use std::{collections::{BTreeMap, HashMap}, rc::Rc, ops::Deref, borrow::Borrow, marker::PhantomData};

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
                (HashMap::<Signature, DefsFunctionBody>::new(),
                HashMap::<DefsBindingDeclaration, DefsBindingBody>::new()),
                |(mut definitions, mut bindings), (_prio, defs)| {
                    for (dec, body) in defs.global_functions {
                        definitions.insert(Signature::Function {
                            name: dec.name,
                            parameters: dec.parameters,
                            return_type: dec.return_type,
                        }, body);
                    }

                    for (typ, methods) in defs.methods {
                        for (dec, body) in methods {
                            assert_eq!(Some(&typ), dec.this.as_ref());

                            definitions.insert(Signature::Method {
                                name: dec.name,
                                this: typ.clone(),
                                parameters: dec.parameters,
                                return_type: dec.return_type,
                            }, body);
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
    definitions: HashMap<Signature, DefsFunctionBody>,
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
    /// 
    /// The return of this function is a tuple of:
    ///  - `.0` statement blocks.
    ///  - `.1` the block used as a return value.
    fn resolve_function_body(
        &mut self,
        body: DefsFunctionBody,
        seen_functions: &mut Vec<Signature>,
        cache: &mut HashMap<Signature, (DefinitionBlocks, Option<BlockArgument>)>,

        return_type: Option<DefsType>,

        this_type: Option<&DefsType>,
        args_types: &Vec<DefsType>,
    ) -> (DefinitionBlocks, Option<BlockArgument>) {
        let mut blocks = DefinitionBlocks::new();
        let mut return_block = None;

        for stmt in body.statements {
            match stmt {
                DefsStatement::Block { opcode, arguments } => {
                    todo!()
                },
                DefsStatement::FunctionCall { name, arguments } => {
                    todo!()
                },
                DefsStatement::MethodCall { name, arguments, this } => {
                    todo!()
                },
                DefsStatement::Return { value } => {
                    let Some(return_value) = return_type
                        else {
                            // todo: error propagation
                            panic!("no return type, given a return statement")
                        };

                    let (res_blocks, res_return_block) =
                        self.resolve_expression(
                            value, seen_functions, cache, return_value,
                            this_type, args_types
                        );

                    blocks.extend(res_blocks);
                    return_block = Some(res_return_block);

                    // skip any other statements, we're done here
                    break;
                },
            }
        }

        (blocks, return_block)
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

        seen_functions: &mut Vec<Signature>,
        cache: &mut HashMap<Signature, (DefinitionBlocks, Option<BlockArgument>)>,

        // needed type
        return_type: DefsType,

        this_type: Option<&DefsType>,
        args_types: &Vec<DefsType>,
    ) -> (DefinitionBlocks, Option<BlockArgument>) {
        let mut result = DefinitionBlocks::new();
        let mut return_value: Option<BlockArgument> = None;

        // convert these into a signature
        //   but first we compile their arguments first
        let arguments = arguments.into_iter()
            .map(|expr| self.resolve_expression(
                expr, seen_functions, cache, return_type.clone(),
                this_type, args_types
            ))
            .collect::<Vec<_>>();
                
        let func_args_types = arguments.iter()
                .map(|(_blocks, arg)|
                    block_argument_as_type(arg, this_type, args_types)
                    // todo: error propagation
                        .expect("err: arg doesn't return any value").clone())
                .collect::<Vec<DefsType>>();
        
        // compile `this` if this is a method
        let this = this_expr.map(|this| self.resolve_expression(
            this, seen_functions, cache, return_type.clone(),
            this_type, args_types
        ));

        let this_type = this.as_ref().map(|(_resolved_this, resolved_this_ret_block)| block_argument_as_type(
            &resolved_this_ret_block, this_type, args_types
            // todo: error propagation
        ).expect("err: expr `this` doesn't return any value").clone());

        let signature = if let Some(this_type) = &this_type {
            Signature::Method {
                name, return_type: Some(return_type.clone()),
                parameters: func_args_types.clone(),
                this: this_type.clone()
            }
        } else {
            Signature::Function {
                name, return_type: Some(return_type.clone()),
                parameters: func_args_types.clone(),
            }
        };

        // after we've made the signature, check if this method was already resolved before
        // so check the cache.
        if let Some((cached_blocks, cached_ret_block)) = cache.get(&signature) {
            // let's use those
            let mut cached_ret_block = cached_ret_block
                .as_ref().expect("err: function doesn't return a type").clone();

            // apply the arguments to the blocks and add this and arguments' blocks
            for block_rc in 
                this.as_ref()
                    .map(|t| t.0.as_slice())
                    .unwrap_or_else(|| &[]).iter()
                    .chain(cached_blocks.iter()) {

                let mut block = block_rc.as_ref().clone();

                apply_block_arguments(
                    &mut block, &arguments,
                    this.as_ref().map(|t| &t.1), &mut result
                );

                result.push(Rc::new(block));
            }

            apply_block_argument_arguments(
                &mut cached_ret_block, &arguments,
                this.as_ref().map(|t| &t.1), &mut result
            );

            return_value = Some(cached_ret_block);

        } else { 
            let signature = signature;

            // we need to resolve this by ourselves
            // we first add this to the seen functions
            // then search for its function body.
            seen_functions.push(signature.clone());

            let body = self.definitions.remove(&signature)
                // todo: error propagation
                .expect("err: no func with signature {} found");

            // resolve its statements then
            let (resolved_blocks, resolved_return_block) =
                self.resolve_function_body(
                    body, seen_functions, cache, Some(return_type),
                    this_type.as_ref(), &func_args_types
                );

            let mut resolved_return_block =
                resolved_return_block.expect("err: function doesn't return a type");

            // we pop ourselves from the seen_functions
            assert_eq!(seen_functions.pop().as_ref(), Some(&signature));

            // finalize by appending this blocks and each arguments' blocks
            for block_rc in 
                this.as_ref()
                    .map(|t| t.0.as_slice())
                    .unwrap_or_else(|| &[]).iter()
                    .chain(resolved_blocks.iter()) {

                let mut block = block_rc.as_ref().clone();

                apply_block_arguments(
                    &mut block, &arguments, 
                    this.as_ref().map(|t| &t.1), &mut result
                );

                result.push(Rc::new(block));
            }

            apply_block_argument_arguments(
                &mut resolved_return_block, &arguments,
                this.as_ref().map(|t| &t.1), &mut result
            );

            return_value = Some(resolved_return_block);

            // don't forget to insert these to the cache
            cache.insert(signature, (resolved_blocks, return_value.clone()));
        }
    
        (result, return_value)
    }

    fn resolve_expression(
        &mut self,
        expr: DefsExpression,
        seen_functions: &mut Vec<Signature>,
        cache: &mut HashMap<Signature, (DefinitionBlocks, Option<BlockArgument>)>,

        // needed type
        return_type: DefsType,

        this_type: Option<&DefsType>,
        args_types: &Vec<DefsType>,
    ) -> (DefinitionBlocks, BlockArgument) {
        let mut result = DefinitionBlocks::new();
        let return_value;

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

                if block_type != return_type {
                    // todo: error propagation
                    panic!("error: block returns .. but required type ..")
                }

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

                return_value = BlockArgument::Block(Rc::new(Block {
                    opcode, content: block.spec.clone(),
                    arguments: arguments.into_iter()
                        .map(|(typ, expr)| {
                            let (blocks, value) = self.resolve_expression(
                                expr, seen_functions, cache, typ,
                                this_type, args_types,
                            );

                            result.extend(blocks);

                            value
                        }).collect(),

                    return_type: Some(return_type),
                }))
            },

            DefsExpression::FunctionCall { name, arguments } => {
                let (blocks, ret_block) = self.resolve_from_raw_function(
                    name, arguments, None, seen_functions,
                    cache, return_type, this_type, args_types
                );

                result.extend(blocks);
                return_value = ret_block.expect("function doesnt return a value");
            },

            DefsExpression::MethodCall { name, arguments, this } => {
                let (blocks, ret_block) = self.resolve_from_raw_function(
                    name, arguments, Some(*this), seen_functions,
                    cache, return_type, this_type, args_types
                );

                result.extend(blocks);
                return_value = ret_block.expect("method doesnt return a value");
            },

            DefsExpression::StaticVariable(static_var)
                => return_value = match static_var {
                    DefsStaticVariable::Literal(lit) => BlockArgument::Literal(lit),
                    DefsStaticVariable::Argument(arg) => BlockArgument::Argument(arg),
                    DefsStaticVariable::This => BlockArgument::This,
                },
        }

        (result, return_value)
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