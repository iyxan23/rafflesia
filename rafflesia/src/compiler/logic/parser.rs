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
    lex.start();
    let mut statements = OuterStatements(vec![]);

    loop {
        // skip any newlines
        while let Some(_) = lex.expect_failsafe(Token::Newline) {}

        // check if we've reached the end (no more tokens)
        if let None = lex.peek() { break; }

        statements.0.push(outer_statement(lex)?);
    }

    lex.success();
    Ok(statements)
}

fn outer_statement(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    lex.start();

    let res = match lex.expect_peek_multiple_choices(
        // expects a type, complex type, or an event identifier
        vec![Token::NumberType, Token::StringType, Token::BooleanType, Token::MapType,
             Token::ListType, Token::Identifier]
    )? {
        TokenWrapperOwned { token: Token::Identifier, .. } => outer_event_definition(lex),
        TokenWrapperOwned {
            token: Token::NumberType | Token::StringType | Token::BooleanType,
            ..
        } => outer_variable_declaration(lex),
        TokenWrapperOwned {
            token: Token::MapType | Token::ListType,
            ..
        } => outer_complex_variable_declaration(lex),
        _ => unreachable!()
    };

    lex.success();
    res
}

fn outer_variable_declaration(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    Ok(todo!())
}

fn outer_complex_variable_declaration(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    Ok(todo!())
}

fn outer_event_definition(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    Ok(todo!())
}