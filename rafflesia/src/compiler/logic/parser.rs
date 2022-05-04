use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "compiler/logic/grammar.pest"]
pub struct LogicParser;