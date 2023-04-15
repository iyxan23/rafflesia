//! The core parsing functionality of `.defs` files.

// There are a few terms you need to understand:
//  - this parser only acts to parse a defs file, it does not
//    resolve or do anything with them. It only returns the list
//    of functions defined with a list of raw blocks that gets
//    "dispatched".
//
//  - a "function" is the functions defined by defs that will
//    be used by the user (in rafflesia's case, it'll be imported
//    in it's `.logic` files)
//
//  - a "dispatch" is "placing" a raw block from their opcodes.
//    may be defined in a .blks file, but this parser does not
//    care about that file. It only parses a .defs file and that's it

// todo: a method call inside def
// todo: convert to chumsky :>
// todo: implement returning types

pub mod models;

#[cfg(test)]
mod tests;

use chumsky::{prelude::*, input::ValueInput};
use logos::Logos;

// also export the models
pub use models::*;

pub fn parse_defs(raw: &str) -> Result<Definitions, ()> {
    todo!()
}

#[derive(Logos, PartialEq, Debug, Clone)]
pub enum Token<'src> {
    // Block return types
    #[token("b")] TypeBoolean,
    #[token("s")] TypeString,
    #[token("d")] TypeNumber,
    // #[token("l")] TypeList,
    // #[token("p")] TypeComponent,
    // #[token("v")] TypeView,
    #[token("c")] TypeSingleNested,
    #[token("e")] TypeDoubleNested,
    #[token("f")] TypeEnding,

    // literals
    #[token("true")] LiteralTrue,
    #[token("false")] LiteralFalse,
    #[regex(r#""([^"]|\\")*""#, |lex| {
        let slice = lex.slice();
        &slice[1..slice.len() - 1]
    })]
    LiteralString(&'src str),
    #[regex("[0-9]+(?:\\.[0-9]+)?", |lex| {
        let slice = lex.slice();
        slice.parse::<u64>().ok()
    })]
    LiteralNumber(u64),

    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token("{")] LBrace,
    #[token("}")] RBrace,

    #[token("=")] Equal,
    #[token("<")] Return,
    #[token("#")] Hashtag,

    #[token(":")] Colon,
    #[token(";")] Semicolon,
    #[token(",")] Comma,
    #[token(".")] Dot,

    #[regex(r"@\d+", |lex| lex.slice()[1..].parse::<u32>().ok())]
    Argument(u32),
    #[token("@@")] ThisArgument,

    #[regex(r#"`(?:[^`]|\\`)*`"#, |lex| {
        // remove the ` around the string
        let slice = lex.slice();
        &slice[1..slice.len() - 1]
    })]
    #[regex(r"([a-zA-Z_][a-zA-Z0-9_]*)")] Identifier(&'src str),

    #[error]
    #[regex(r"[ \t\n]+", logos::skip)] // whitespace
    #[regex(r"//[^\n]*\n?", logos::skip)] // comment
    Error,
}

fn parser<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, Definitions, extra::Err<Rich<'src, Token<'src>>>> {

    todo()
}

fn method<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, (Type, FunctionDeclaration, FunctionBody), extra::Err<Rich<'src, Token<'src>>>> {

    typ()
        .then_ignore(just(Token::Dot))
        .then(function())
        .map(|(typ, (func_dec, func_body))| (typ, func_dec, func_body))
}

fn function<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, (FunctionDeclaration, FunctionBody), extra::Err<Rich<'src, Token<'src>>>> {

    f_declaration()
        .then(f_body()
            .delimited_by(just(Token::LBrace), just(Token::RBrace)))
}

fn f_declaration<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, FunctionDeclaration, extra::Err<Rich<'src, Token<'src>>>> {

    let ident = select! { Token::Identifier(ident) => ident };

    let args = typ()
        .separated_by(just(Token::Comma)).allow_trailing()
        .collect::<Vec<Type>>()
        .delimited_by(just(Token::LParen), just(Token::RParen));

    let ret_type = just(Token::Colon).then(typ()).or_not();

    ident
        .then(args)
        .then(ret_type)
        .map(|((ident, args), ret_type)| {
            FunctionDeclaration {
                this: None,
                parameters: args,
                name: ident.to_string(),
                return_type: ret_type.map(|a| a.1),
            }
        })
}

// doesn't take `{` nor `}`
fn f_body<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, FunctionBody, extra::Err<Rich<'src, Token<'src>>>> {
    statement()
        .repeated()
        .collect::<Vec<Statement>>()
        .map(|statements| FunctionBody { statements })
}

// doesn't take a `;`
fn statement<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, Statement, extra::Err<Rich<'src, Token<'src>>>> {
    let block = just(Token::Hashtag)
        .ignore_then(select! { Token::Identifier(ident) => ident })
        .then(arguments()
            .delimited_by(just(Token::LParen), just(Token::RParen)))
        .map(|(ident, args)| Statement::Block {
            opcode: ident.to_string(),
            arguments: args
        });

    let function_call = 
        expr()
            .then(just(Token::Dot)).or_not()
            .then(select! { Token::Identifier(ident) => ident })
            .then(arguments()
                .delimited_by(just(Token::LParen), just(Token::RParen)))
            .map(|((this, ident), args)| Statement::FunctionCall {
                name: ident.to_ascii_lowercase(),
                arguments: args,
                this: this.map(|(expr, _tok)| Box::new(expr)),
            });

    let return_stmt = just(Token::Return)
        .ignore_then(expr())
        .map(|expr| Statement::Return { value: expr } );

    choice((block, function_call, return_stmt))
}

// doesn't take `(` nor `)`
fn arguments<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, Vec<Expression>, extra::Err<Rich<'src, Token<'src>>>> {
    expr()
        .separated_by(just(Token::Comma)).allow_trailing()
        .collect::<Vec<Expression>>()
}

fn expr<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, Expression, extra::Err<Rich<'src, Token<'src>>>> {
    recursive(|expr| {
        let block = just(Token::Hashtag)
            .ignore_then(select! { Token::Identifier(ident) => ident })
            .then(arguments()
                .delimited_by(just(Token::LParen), just(Token::RParen)))
            .map(|(ident, args)| Expression::Block {
                opcode: ident.to_string(),
                arguments: args,
            });

        let function_call =
            expr.then(just(Token::Dot)).or_not()
                .then(select! { Token::Identifier(ident) => ident })
                .then(arguments()
                    .delimited_by(just(Token::LParen), just(Token::RParen)))
                .map(|((this, ident), args)| Expression::FunctionCall {
                    name: ident.to_ascii_lowercase(),
                    arguments: args,
                    this: this.map(|(expr, _tok)| Box::new(expr)),
                });

        let literal = choice((
            just(Token::LiteralFalse).to(Literal::Boolean(false)),
            just(Token::LiteralTrue).to(Literal::Boolean(true)),

            select! { Token::LiteralNumber(num) => num }
                .map(|num| Literal::Number(num)),

            select! { Token::LiteralString(str) => str }
                .map(|str| Literal::String(str.to_string())),
        )).map(Expression::Literal);

        choice((block.boxed(), function_call.boxed(), literal))
    })
}

fn typ<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, Type, extra::Err<Rich<'src, Token<'src>>>> {
    choice((
        just(Token::TypeBoolean).to(Type::Boolean),
        just(Token::TypeNumber).to(Type::Number),
        just(Token::TypeString).to(Type::String),
    ))
}