use std::fmt::Debug;
use swrs::api::block::{ArgumentBlockReturnType, ArgValue, Block, Blocks, BlockType, ListItem};
use swrs::api::component::ComponentKind;
use swrs::api::screen::{EventType, MoreBlock};
use swrs::api::screen::Event;
use swrs::api::view::View;
use swrs::parser::logic::list_variable::ListVariable;
use swrs::parser::logic::variable::{Variable, VariableType as SWRSVariableType};
use swrs::LinkedHashMap;
use thiserror::Error;

use crate::compiler::logic::ast::{
    BinaryOperator, ComplexVariableType, Expression, InnerStatement, InnerStatements, Literal,
    OuterStatement, OuterStatements, PrimaryExpression, UnaryOperator, VariableAssignment,
    VariableType
};
use crate::compiler::logic::blocks::types::{ComplexType, Definitions, PrimitiveType, Type};

pub mod parser;
pub mod ast;
mod blocks;

#[cfg(test)]
mod tests;

// todo: add positions in AST
// todo: a custom result handling system similar to error-stack

/// Compiles a logic AST into blocks
pub fn compile_logic(statements: OuterStatements, attached_layout: &View)
    -> Result<LogicCompileResult, LogicCompileError> {

    let mut definitions = Definitions::new(attached_layout);
    let /* mut */ more_blocks = LinkedHashMap::new();
    let /* mut */ components = LinkedHashMap::new();
    let mut events = Vec::new();

    for outer_statement in statements.0 {
        match outer_statement {
            OuterStatement::SimpleVariableDeclaration { variable_type, identifier } => {
                definitions.add_variable(
                    identifier,
                    match variable_type {
                        VariableType::Number => Type::Primitive(PrimitiveType::Number),
                        VariableType::String => Type::Primitive(PrimitiveType::String),
                        VariableType::Boolean => Type::Primitive(PrimitiveType::Boolean),
                    }
                );
            }

            OuterStatement::ComplexVariableDeclaration { variable_type, identifier } => {
                // fixme: apparently you cant set types on map, perhaps we could add a some
                //        kind of type safety layer on rafflesia so maps are "typed"

                // fixme: list maps :weary:

                definitions.add_variable(
                    identifier,
                    complex_variable_type_to_type(variable_type)
                );
            }

            OuterStatement::ActivityEventListener { event_name, body } => {
                events.push(
                    (Event {
                        name: event_name,
                        event_type: EventType::ActivityEvent,
                        code: Blocks::new() // will be compiled later
                    }, body)
                );
            }

            OuterStatement::ViewEventListener { view_id, event_name, body } => {
                events.push(
                    (Event {
                        name: event_name,
                        event_type: EventType::ViewEvent { id: view_id },
                        code: Blocks::new() // will be compiled later
                    }, body)
                );
            }
        }
    }

    // compile the events' blocks now that we have access to all of the variables
    let events = events.into_iter().map(|(event, body)| {
        Ok(Event {
            name: event.name,
            event_type: event.event_type,
            code: compile_inner_statements(body, &definitions)?
        })
    }).collect::<Result<_, _>>()?;

    let (variables, list_variables)
        = definitions.deconstruct();

    Ok(LogicCompileResult { variables, list_variables, more_blocks, components, events })
}

fn compile_inner_statements(
    stmts: InnerStatements,
    definitions: &Definitions,
) -> Result<Blocks, LogicCompileError> {

    let mut result = Vec::new();

    for statement in stmts.0 {
        match statement {
            InnerStatement::VariableAssignment(var_assign) => {
                let var = definitions.get_var(&var_assign.identifier)
                    .ok_or_else(|| LogicCompileError::VariableDoesntExist {
                        identifier: var_assign.identifier.clone()
                    })?;

                // only primitive types can be assigned to a value
                // todo: maybe allow complex types as well? that'd be a cool feature
                let var_type = match var {
                    Type::Primitive(primitive_type) => primitive_type,
                    _ => return Err(LogicCompileError::UnAssignableVariable {
                        identifier: var_assign.identifier.clone(),
                        variable_type: Type::Void
                    })
                };

                let value = compile_expression(var_assign.value, &definitions)?;

                result.push(match var_type {
                    PrimitiveType::Boolean =>
                        blocks::set_var_boolean(var_assign.identifier, value.to_bool_arg()?),
                    PrimitiveType::Number =>
                        blocks::set_var_int(var_assign.identifier, value.to_num_arg()?),
                    PrimitiveType::String =>
                        blocks::set_var_string(var_assign.identifier, value.to_str_arg()?),
                });
            }

            InnerStatement::IfStatement(if_stmt) => {
                let condition = compile_expression(
                    if_stmt.condition, &definitions
                )?.to_bool_arg()?;

                let body = compile_inner_statements(if_stmt.body, &definitions)?;
                let else_body = if_stmt.else_body
                    .map(|else_body| compile_inner_statements(else_body, &definitions))
                    .transpose()?;

                result.push(match else_body {
                    None => blocks::r#if(condition, body),
                    Some(else_body) => blocks::if_else(condition, body, else_body)
                });
            }

            InnerStatement::RepeatStatement(repeat_stmt) => {
                let value = compile_expression(
                    repeat_stmt.condition, &definitions
                )?.to_num_arg()?;

                let body = compile_inner_statements(repeat_stmt.body, &definitions)?;

                result.push(blocks::repeat(value, body));
            }

            InnerStatement::ForeverStatement(forever_stmt) => {
                let body = compile_inner_statements(forever_stmt.body, &definitions)?;

                result.push(blocks::forever(body));
            }

            InnerStatement::Break => result.push(blocks::r#break()),
            InnerStatement::Continue => result.push(blocks::r#continue()),
            InnerStatement::Expression(expr) =>
                result.push(
                    compile_expression(expr, &definitions)?
                        .expect_block()?
                ),
        }
    }

    Ok(Blocks(result))
}

// the return value of [`compile_expression`], can either be a regular block, an argument block or
// a literal
enum ExprValue {
    // a regular freestanding block, has a type of BlockType::Regular
    Block(Block),
    // an argument block that can't be a statement and could only be an argument of a block, has a
    // type of BlockType::Argument(_)
    ArgBlock(Block),
    // a literal value
    Literal(Literal),
}

impl ExprValue {
    fn to_num_arg(self) -> Result<ArgValue<f64>, LogicCompileError> {
        Ok(match self {
            ExprValue::Block(block) => return Err(LogicCompileError::RegularBlockAsArg {
                block, expected_arg_type: Type::Primitive(PrimitiveType::Number)
            }),
            ExprValue::ArgBlock(block) => {
                match block.block_type {
                    BlockType::Argument(ArgumentBlockReturnType::Number) => ArgValue::Block(block),
                    other => Err(LogicCompileError::TypeError {
                        expected: Type::Primitive(PrimitiveType::Number),

                        // expect n panic: it's an error if ArgBlock is given a block type other
                        //                 than arg blocks
                        got: if let BlockType::Argument(arg) = other {
                            Type::from_arg_block(&arg)
                                .expect(&*format!("Invalid block argument: {:?}", arg))
                        } else {
                            panic!("ArgBlock() is given a block with type {:?}", other)
                        }
                    })?
                }
            },
            ExprValue::Literal(literal) => match literal {
                Literal::Number(num) => ArgValue::Value(num),
                Literal::Boolean(_) => Err(LogicCompileError::TypeError {
                    expected: Type::Primitive(PrimitiveType::Number),
                    got: Type::Primitive(PrimitiveType::Boolean)
                })?,
                Literal::String(_) => Err(LogicCompileError::TypeError {
                    expected: Type::Primitive(PrimitiveType::Number),
                    got: Type::Primitive(PrimitiveType::String)
                })?
            },
        })
    }

    fn to_bool_arg(self) -> Result<ArgValue<bool>, LogicCompileError> {
        Ok(match self {
            ExprValue::Block(block) => return Err(LogicCompileError::RegularBlockAsArg {
                block, expected_arg_type: Type::Primitive(PrimitiveType::Boolean)
            }),
            ExprValue::ArgBlock(block) => {
                match block.block_type {
                    BlockType::Argument(ArgumentBlockReturnType::Boolean) => ArgValue::Block(block),
                    other => Err(LogicCompileError::TypeError {
                        expected: Type::Primitive(PrimitiveType::Boolean),

                        // expect n panic: it's an error if ArgBlock is given a block type other
                        //                 than arg blocks
                        got: if let BlockType::Argument(arg) = other {
                            Type::from_arg_block(&arg)
                                .expect(&*format!("Invalid block argument: {:?}", arg))
                        } else {
                            panic!("ArgBlock() is given a block with type {:?}", other)
                        }
                    })?
                }
            },
            ExprValue::Literal(literal) => match literal {
                Literal::Number(_) => Err(LogicCompileError::TypeError {
                    expected: Type::Primitive(PrimitiveType::Boolean),
                    got: Type::Primitive(PrimitiveType::Number)
                })?,
                Literal::Boolean(bool) => ArgValue::Value(bool),
                Literal::String(_) => Err(LogicCompileError::TypeError {
                    expected: Type::Primitive(PrimitiveType::Boolean),
                    got: Type::Primitive(PrimitiveType::Number)
                })?
            }
        })
    }

    fn to_str_arg(self) -> Result<ArgValue<String>, LogicCompileError> {
        Ok(match self {
            ExprValue::Block(block) => return Err(LogicCompileError::RegularBlockAsArg {
                block, expected_arg_type: Type::Primitive(PrimitiveType::String)
            }),
            ExprValue::ArgBlock(block) => {
                match block.block_type {
                    BlockType::Argument(ArgumentBlockReturnType::String) => ArgValue::Block(block),
                    other => Err(LogicCompileError::TypeError {
                        expected: Type::Primitive(PrimitiveType::String),

                        // expect n panic: it's an error if ArgBlock is given a block type other
                        //                 than arg blocks
                        got: if let BlockType::Argument(arg) = other {
                            Type::from_arg_block(&arg)
                                .expect(&*format!("Invalid block argument: {:?}", arg))
                        } else {
                            panic!("ArgBlock() is given a block with type {:?}", other)
                        }
                    })?
                }
            },
            ExprValue::Literal(literal) => match literal {
                Literal::Number(_) => Err(LogicCompileError::TypeError {
                    expected: Type::Primitive(PrimitiveType::String),
                    got: Type::Primitive(PrimitiveType::Number),
                })?,
                Literal::Boolean(_) => Err(LogicCompileError::TypeError {
                    expected: Type::Primitive(PrimitiveType::String),
                    got: Type::Primitive(PrimitiveType::Boolean)
                })?,
                Literal::String(str) => ArgValue::Value(str),
            }
        })
    }

    // expects a block, otherwise return an `Err(LogicCompileError::DanglingLiteral)`
    fn expect_block(self) -> Result<Block, LogicCompileError> {
        match self {
            ExprValue::Block(block) => Ok(block),
            ExprValue::ArgBlock(block) => Err(LogicCompileError::DanglingArgBlock { block }),
            ExprValue::Literal(literal) => Err(LogicCompileError::DanglingLiteral { literal })
        }
    }

    /// turns a [`Block`] to [`ExprValue`] (either to [`ExprValue::Block`] or [`ExprValue::ArgBlock`])
    /// depending on the `block_type` of [`Block`]
    fn from_block(block: Block) -> Self {
        match &block.block_type {
            BlockType::Regular => ExprValue::Block(block),
            BlockType::Argument(_) => ExprValue::ArgBlock(block),
            BlockType::Control(_) => ExprValue::Block(block),
        }
    }
}

fn compile_expression(
    expr: Expression,
    definitions: &Definitions
) -> Result<ExprValue, LogicCompileError> {
    Ok(match expr {
        Expression::BinOp { first, operator, second } => {
            let first = compile_expression(*first, &definitions)?;
            let second = compile_expression(*second, &definitions)?;

            let block = match operator {
                BinaryOperator::Or       => blocks::or(first.to_bool_arg()?, second.to_bool_arg()?),
                BinaryOperator::And      => blocks::and(first.to_bool_arg()?, second.to_bool_arg()?),
                BinaryOperator::LT       => blocks::lt(first.to_num_arg()?, second.to_num_arg()?),
                BinaryOperator::LTE      => blocks::lte(first.to_num_arg()?, second.to_num_arg()?),
                BinaryOperator::GT       => blocks::gt(first.to_num_arg()?, second.to_num_arg()?),
                BinaryOperator::GTE      => blocks::gte(first.to_num_arg()?, second.to_num_arg()?),
                BinaryOperator::EQ       => blocks::eq(first.to_num_arg()?, second.to_num_arg()?),
                BinaryOperator::Plus     => blocks::plus(first.to_num_arg()?, second.to_num_arg()?),
                BinaryOperator::Minus    => blocks::minus(first.to_num_arg()?, second.to_num_arg()?),
                BinaryOperator::Multiply => blocks::multiply(first.to_num_arg()?, second.to_num_arg()?),
                BinaryOperator::Divide   => blocks::divide(first.to_num_arg()?, second.to_num_arg()?),
                BinaryOperator::Power    => blocks::power(first.to_num_arg()?, second.to_num_arg()?)
            };

            ExprValue::ArgBlock(block)
        }

        Expression::UnaryOp { value, operator } => {
            let value = compile_expression(*value, &definitions)?;

            ExprValue::ArgBlock(match operator {
                UnaryOperator::Not => blocks::not(value.to_bool_arg()?),
                UnaryOperator::Minus => blocks::minus_unary(value.to_num_arg()?),
                UnaryOperator::Plus => blocks::minus_unary(value.to_num_arg()?),
            })
        }

        Expression::PrimaryExpression(prim) => {
            match prim {
                PrimaryExpression::Index { from, index } => {
                    todo!("create a type system")
                }

                PrimaryExpression::VariableAccess { from, name } => {
                    if let Some(from) = from {
                        // let val = compile_expression(*from, &definitions)?;
                        todo!("fields lol")
                    } else {
                        let var = definitions.get_var(&name)
                            .ok_or_else(|| LogicCompileError::VariableDoesntExist {
                                identifier: name.clone()
                            })?;

                        ExprValue::ArgBlock(blocks::get_var(name, var.to_arg_block_type()))
                    }
                }

                PrimaryExpression::Call { from, arguments } => {
                    todo!("create a type system")
                }
            }
        }

        Expression::Literal(literal) => ExprValue::Literal(literal),
    })
}

fn variable_type_to_type(typ: VariableType) -> Type {
    Type::Primitive(match typ {
        VariableType::Number => PrimitiveType::Number,
        VariableType::String => PrimitiveType::String,
        VariableType::Boolean => PrimitiveType::Boolean,
    })
}

fn complex_variable_type_to_type(typ: ComplexVariableType) -> Type {
    Type::Complex(match typ {
        ComplexVariableType::Map { .. } => ComplexType::Map,
        ComplexVariableType::List { inner_type } => ComplexType::List {
            inner_type: if let Type::Primitive(primitive_type) =
                variable_type_to_type(inner_type) { primitive_type } else { unreachable!() }
        }
    })
}

#[derive(Debug)]
pub struct LogicCompileResult {
    pub variables: LinkedHashMap<String, Variable>,
    pub list_variables: LinkedHashMap<String, ListVariable>,
    pub more_blocks: LinkedHashMap<String, MoreBlock>,
    pub components: LinkedHashMap<String, ComponentKind>,
    pub events: Vec<Event>,
}

#[derive(Debug, Error)]
pub enum LogicCompileError {
    #[error("wrong type given, expected {expected:?} got {got:?}")]
    TypeError {
        // todo: change to a simpler type lol
        expected: Type,
        got: Type,
    },

    #[error("tries to define variable {var_name} with type {var_type:?}, but another variable with \
    the same name already exists with the type {existing_var_type:?}")]
    VariableAlreadyExists {
        var_name: String,
        var_type: Type,
        existing_var_type: Type
    },

    #[error("variable {identifier} doesn't exist")]
    VariableDoesntExist {
        identifier: String,
    },

    #[error("variable {identifier} with type {variable_type:?} can't be assigned to a value")]
    UnAssignableVariable {
        identifier: String,
        variable_type: Type
    },

    #[error("regular block can't be used as a argument. expected an arg block with type {expected_arg_type:?}")]
    RegularBlockAsArg {
        block: Block,
        expected_arg_type: Type
    },

    #[error("dangling argument block as a statement")]
    DanglingArgBlock {
        block: Block
    },

    #[error("dangling literal as a statement")]
    DanglingLiteral {
        literal: Literal
    }
}
