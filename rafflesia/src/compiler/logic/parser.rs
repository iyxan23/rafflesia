use anyhow::Result;
use crate::compiler::parser::{error, LexerWrapper, TokenWrapperOwned};
use logos::Logos;
use super::ast::*;

pub fn parse_logic(raw: &str) -> Result<OuterStatements> {
    Ok(todo!())
}

#[derive(Logos, PartialEq, Debug, Clone)]
enum Token {

    // arithmetic operations
    #[token("**")] Pow,
    #[token("*")] Mult,
    #[token("/")] Div,
    #[token("+")] Plus,
    #[token("-")] Minus,

    // delimiter
    #[token(",")] Comma,
    #[token("\n")] Newline,

    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token("{")] LBrace,
    #[token("}")] RBrace,

    #[token(".")] DOT,

    // boolean operators
    #[token("!")] Not,
    #[token("==")] DEQ,
    #[token("=")] EQ,
    #[token("<")] LT,
    #[token("<=")] LTE,
    #[token(">")] GT,
    #[token(">=")] GTE,

    #[token("&&")] And,
    #[token("||")] Or,

    // types
    #[token("number")] NumberType,
    #[token("string")] StringType,
    #[token("boolean")] BooleanType,
    #[token("map")] MapType,
    #[token("list")] ListType,

    // compound statements
    #[token("if")] If,
    #[token("else")] Else,
    #[token("repeat")] Repeat,
    #[token("forever")] Forever,

    // simple statements
    #[token("break")] Break,
    #[token("continue")] Continue,

    // literals
    #[token("true")] True,
    #[token("false")] False,

    #[regex(r#""([^"]|\\")*""#)] String,
    #[regex("[0-9]+(?:\\.[0-9]+)")] Number,

    #[regex(r"[a-zA-Z][a-zA-Z0-9]*")] Identifier,

    #[error]
    #[regex(r"[ \t]+", logos::skip)] // whitespace
    #[regex(r"//[^\n]*\n?", logos::skip)] // comment
    Error
}

pub type LogicParseError = error::ParseError<Token, TokenWrapperOwned<Token>>;
pub type LogicParseResult<T> = Result<T, LogicParseError>;
type Lexer<'a> = LexerWrapper<'a, Token>;

fn outer_statements(lex: &mut Lexer) -> LogicParseResult<OuterStatements> {
    todo!()
}