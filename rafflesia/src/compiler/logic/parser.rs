use anyhow::Result;
use crate::compiler::parser::{error, LexerWrapper, TokenWrapperOwned};
use logos::Logos;
use super::ast::*;

pub fn parse_logic(raw: &str) -> Result<OuterStatements> {
    Ok(todo!())
}

#[derive(Logos, PartialEq, Debug, Clone)]
pub enum Token {

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

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")] Identifier,

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

fn simple_variable_type(lex: &mut Lexer) -> LogicParseResult<VariableType> {
    lex.start();

    let res = match lex.expect_multiple_choices(
        vec![Token::NumberType, Token::StringType, Token::BooleanType]
    )? {
        TokenWrapperOwned { token: Token::NumberType, .. } => VariableType::Number,
        TokenWrapperOwned { token: Token::StringType, .. } => VariableType::String,
        TokenWrapperOwned { token: Token::BooleanType, .. } => VariableType::Boolean,
        _ => unreachable!()
    };

    lex.success();
    Ok(res)
}

fn outer_variable_declaration(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    lex.start();

    // get the type
    let variable_type = simple_variable_type(lex)?;

    // next is the identifier
    let identifier = lex.expect(Token::Identifier)?.slice;

    lex.success();
    Ok(OuterStatement::SimpleVariableDeclaration {
        variable_type,
        identifier,
    })
}

fn outer_complex_variable_declaration(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    lex.start();

    enum ComplexVariableTokenType { Map, List }

    // get the type
    let cx_var_tok_type = match lex.expect_multiple_choices(
        vec![Token::MapType, Token::ListType]
    )? {
        TokenWrapperOwned { token: Token::MapType, .. } => ComplexVariableTokenType::Map,
        TokenWrapperOwned { token: Token::ListType, .. } => ComplexVariableTokenType::List,
        _ => unreachable!()
    };

    // open the <>
    lex.expect(Token::LT)?;

    // the inner type
    let inner_type = simple_variable_type(lex)?;

    lex.expect(Token::GT)?;

    // next is the identifier
    let identifier = lex.expect(Token::Identifier)?.slice;

    lex.success();
    Ok(OuterStatement::ComplexVariableDeclaration {
        variable_type: match cx_var_tok_type {
            ComplexVariableTokenType::Map => ComplexVariableType::Map { inner_type },
            ComplexVariableTokenType::List => ComplexVariableType::List { inner_type }
        },
        identifier
    })
}

fn outer_event_definition(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    // this is where the fun begins
    lex.start();

    let name = lex.expect(Token::Identifier)?.slice;

    // check if there is a dot after an identifier (means that it's a view listener)
    if let Some(_) = lex.expect_failsafe(Token::DOT) {
        let event_name = lex.expect(Token::Identifier)?.slice;

        // parse the body of this event
        // where the real fun begins :sunglasses:
        lex.expect(Token::LBrace)?;

        let statements = inner_statements(lex)?;

        lex.expect(Token::RBrace)?;

        lex.success();
        Ok(OuterStatement::ViewEventListener {
            view_id: name,
            event_name,
            statements
        })

    } else {
        // parse the body of this event
        // where the real fun begins :sunglasses:
        lex.expect(Token::LBrace)?;

        let statements = inner_statements(lex)?;

        lex.expect(Token::RBrace)?;

        Ok(OuterStatement::ActivityEventListener {
            event_name: name,
            statements,
        })
    }
}

fn inner_statements(lex: &mut Lexer) -> LogicParseResult<InnerStatements> {
    todo!()
}

fn inner_statement(lex: &mut Lexer) -> LogicParseResult<InnerStatement> {
    todo!()
}