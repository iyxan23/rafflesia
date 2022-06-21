use swrs::api::block::{Block, Blocks};
use swrs::api::component::ComponentKind;
use swrs::api::screen::{EventType, MoreBlock};
use swrs::api::screen::Event;
use swrs::api::view::View;
use swrs::parser::logic::list_variable::ListVariable;
use swrs::parser::logic::variable::{Variable, VariableType as SWRSVariableType};
use swrs::LinkedHashMap;
use thiserror::Error;

use crate::compiler::logic::ast::{ComplexVariableType, InnerStatement, InnerStatements, OuterStatement, OuterStatements, VariableType};

pub mod parser;
pub mod ast;

#[cfg(test)]
mod tests;

/// Compiles a logic AST into
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
    todo!()
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

}