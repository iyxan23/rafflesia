use std::fmt::Debug;
use swrs::api::block::{ArgumentBlockReturnType, ArgValue, Block, Blocks};
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
    OuterStatement, OuterStatements, UnaryOperator, VariableType
};

pub mod parser;
pub mod ast;
mod blocks;

#[cfg(test)]
mod tests;

// todo: add positions in AST
// todo: a custom result handling system similar to error-stack

/// Compiles a logic AST into blocks
pub fn compile_screen(statements: OuterStatements, attached_layout: &View)
    -> Result<LogicCompileResult, LogicCompileError> {

    let mut variables = LinkedHashMap::new();
    let mut list_variables = LinkedHashMap::new();
    let /* mut */ more_blocks = LinkedHashMap::new();
    let /* mut */ components = LinkedHashMap::new();
    let mut events = Vec::new();

    for outer_statement in statements.0 {
        match outer_statement {
            OuterStatement::SimpleVariableDeclaration { variable_type, identifier } => {
                variables.insert(
                    identifier.clone(),
                    Variable {
                        name: identifier,
                        r#type: variable_type_to_swrs(variable_type)
                    }
                );
            }

            OuterStatement::ComplexVariableDeclaration { variable_type, identifier } => {
                match variable_type {
                    ComplexVariableType::Map { .. } => {
                        // fixme: apparently you cant set types on map, perhaps we could add a some
                        //        kind of type safety layer on rafflesia so maps are "typed"
                        variables.insert(
                            identifier.clone(),
                            Variable {
                                name: identifier,
                                r#type: SWRSVariableType::HashMap
                            }
                        );
                    }

                    ComplexVariableType::List { inner_type } => {
                        // fixme: list maps :weary:
                        list_variables.insert(
                            identifier.clone(),
                            ListVariable {
                                name: identifier,
                                r#type: variable_type_to_swrs(inner_type)
                            }
                        );
                    }
                }
            }

            OuterStatement::ActivityEventListener { event_name, body } => {
                events.push(
                    Event {
                        name: event_name,
                        event_type: EventType::ActivityEvent,
                        code: compile_inner_statements(body)?
                    }
                );
            }

            OuterStatement::ViewEventListener { view_id, event_name, body } => {
                events.push(
                    Event {
                        name: event_name,
                        event_type: EventType::ViewEvent { id: view_id },
                        code: compile_inner_statements(body)?
                    }
                );
            }
        }
    }

    Ok(LogicCompileResult { variables, list_variables, more_blocks, components, events })
}

fn compile_inner_statements(stmts: InnerStatements) -> Result<Blocks, LogicCompileError> {
    let mut result = Vec::new();

    for statement in stmts.0 {
        match statement {
            InnerStatement::VariableAssignment(_) => {}
            InnerStatement::IfStatement(_) => {}
            InnerStatement::RepeatStatement(_) => {}
            InnerStatement::ForeverStatement(_) => {}
            InnerStatement::Break => result.push(blocks::r#break()),
            InnerStatement::Continue => result.push(blocks::r#continue()),
            InnerStatement::Expression(expr) => result.append(&mut compile_expression(expr)?),
        }
    }

    Ok(Blocks(result))
}

fn compile_expression(expr: Expression) -> Result<Vec<Block>, LogicCompileError> {
    let mut result = Vec::new();

    enum Value {
        Block(Block),
        Literal(Literal),
    }

    impl Value {
        fn to_num_arg(self) -> Result<ArgValue<f64>, LogicCompileError> {
            Ok(match self {
                Value::Block(block) => ArgValue::Block(block),
                Value::Literal(literal) => match literal {
                    Literal::Number(num) => ArgValue::Value(num),
                    Literal::Boolean(_) => Err(LogicCompileError::TypeError {
                        expected: ArgumentBlockReturnType::Number,
                        got: ArgumentBlockReturnType::Boolean
                    })?,
                    Literal::String(_) => Err(LogicCompileError::TypeError {
                        expected: ArgumentBlockReturnType::Number,
                        got: ArgumentBlockReturnType::String
                    })?
                }
            })
        }

        fn to_bool_arg(self) -> Result<ArgValue<bool>, LogicCompileError> {
            Ok(match self {
                Value::Block(block) => ArgValue::Block(block),
                Value::Literal(literal) => match literal {
                    Literal::Number(_) => Err(LogicCompileError::TypeError {
                        expected: ArgumentBlockReturnType::Boolean,
                        got: ArgumentBlockReturnType::Number
                    })?,
                    Literal::Boolean(bool) => ArgValue::Value(bool),
                    Literal::String(_) => Err(LogicCompileError::TypeError {
                        expected: ArgumentBlockReturnType::Boolean,
                        got: ArgumentBlockReturnType::String
                    })?
                }
            })
        }

        fn to_str_arg(self) -> Result<ArgValue<String>, LogicCompileError> {
            Ok(match self {
                Value::Block(block) => ArgValue::Block(block),
                Value::Literal(literal) => match literal {
                    Literal::Number(_) => Err(LogicCompileError::TypeError {
                        expected: ArgumentBlockReturnType::String,
                        got: ArgumentBlockReturnType::Number
                    })?,
                    Literal::Boolean(_) => Err(LogicCompileError::TypeError {
                        expected: ArgumentBlockReturnType::String,
                        got: ArgumentBlockReturnType::Boolean
                    })?,
                    Literal::String(str) => ArgValue::Value(str),
                }
            })
        }
    }

    fn compile_expr(expr: Expression) -> Result<Value, LogicCompileError> {
        Ok(match expr {
            Expression::BinOp { first, operator, second } => {
                let first = compile_expr(*first)?;
                let second = compile_expr(*second)?;

                let block = match operator {
                    BinaryOperator::Or =>
                        blocks::or(first.to_bool_arg()?, second.to_bool_arg()?),
                    BinaryOperator::And => todo!(),
                    BinaryOperator::LT => todo!(),
                    BinaryOperator::LTE => todo!(),
                    BinaryOperator::GT => todo!(),
                    BinaryOperator::GTE => todo!(),
                    BinaryOperator::EQ => todo!(),
                    BinaryOperator::Plus => todo!(),
                    BinaryOperator::Minus => todo!(),
                    BinaryOperator::Multiply => todo!(),
                    BinaryOperator::Divide => todo!(),
                    BinaryOperator::Power => todo!()
                };

                Value::Block(block)
            }

            Expression::UnaryOp { value, operator } => {
                let value = compile_expr(*value)?;
                Value::Block(match operator {
                    UnaryOperator::Not => blocks::not(value.to_bool_arg()?),
                    UnaryOperator::Minus => todo!(),
                    UnaryOperator::Plus => todo!()
                })
            }

            Expression::PrimaryExpression(prim) => {
                todo!()
            }

            Expression::Literal(literal) => Value::Literal(literal),
        })
    }

    Ok(result)
}

fn variable_type_to_swrs(var_type: VariableType) -> SWRSVariableType {
    match var_type {
        VariableType::Number => SWRSVariableType::Integer,
        VariableType::String => SWRSVariableType::String,
        VariableType::Boolean => SWRSVariableType::Boolean,
    }
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
    #[error("wrong type given")]
    TypeError {
        // todo: change to a simpler type lol
        expected: ArgumentBlockReturnType,
        got: ArgumentBlockReturnType,
    }
}
