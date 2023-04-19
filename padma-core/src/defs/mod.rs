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

pub mod models;

#[cfg(test)]
mod tests;

use chumsky::{prelude::*, input::{ValueInput, Stream}};
use logos::Logos;

// also export the models
pub use models::*;

pub fn parse_defs(raw: &str) -> Result<Definitions, Vec<Rich<Token, SimpleSpan>>> {
    let lex = Token::lexer(raw)
        .spanned()
        .map(|(tok, span)| (tok, span.into()));

    let stream = Stream::from_iter(lex)
        .spanned((raw.len()..raw.len()).into());

    parser().parse(stream).into_result()
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

    empty()
        .map(|_| Definitions::default())
        .foldl(choice((
                method().map(|(typ, decl, body)| (Some(typ), decl, body)),
                function().map(|(decl, body)| (None, decl, body))
            )).repeated(),
            |mut acc, (typ, func_decl, func_body)| {
                if let Some(typ) = typ {
                    acc.methods
                        .entry(typ)
                        .or_insert_with(|| vec![])
                        .push((func_decl, func_body));
                } else {
                    acc.global_functions
                        .push((func_decl, func_body))
                }

                acc
            })
    
}

fn method<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, (Type, FunctionDeclaration, FunctionBody), extra::Err<Rich<'src, Token<'src>>>> {

    typ()
        .then_ignore(just(Token::Dot))
        .then(function())
        .map(|(typ, (mut func_dec, func_body))| {
            func_dec.this = Some(typ.clone());
            (typ, func_dec, func_body)
        })
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
        .then_ignore(just(Token::Semicolon))
        .repeated()
        .collect::<Vec<Statement>>()
        .map(|statements| FunctionBody { statements })
}

// doesn't take a `;`
fn statement<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, Statement, extra::Err<Rich<'src, Token<'src>>>> {
    let expr = expr().boxed();

    let arguments = 
        expr.clone()
            .separated_by(just(Token::Comma)).allow_trailing()
            .collect::<Vec<Expression>>()
            .boxed();

    let block = just(Token::Hashtag)
        .ignore_then(select! { Token::Identifier(ident) => ident })
        .then(arguments.clone()
            .delimited_by(just(Token::LParen), just(Token::RParen)))
        .map(|(ident, args)| Statement::Block {
            opcode: ident.to_string(),
            arguments: args
        });

    let function_call = 
        expr.clone()
            .then_ignore(just(Token::Dot)).or_not()
            .then(select! { Token::Identifier(ident) => ident })
            .then(arguments
                .delimited_by(just(Token::LParen), just(Token::RParen)))
            .map(|((this, ident), args)| {
                if let Some(this) = this {
                    Statement::MethodCall {
                        name: ident.to_ascii_lowercase(),
                        arguments: args,
                        this: Box::new(this),
                    }
                } else {
                    Statement::FunctionCall {
                        name: ident.to_ascii_lowercase(),
                        arguments: args,
                    }
                }
            });

    let return_stmt = just(Token::Return)
        .ignore_then(expr)
        .map(|expr| Statement::Return { value: expr } );

    choice((block, function_call, return_stmt))
}

fn expr<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>()
    -> impl Parser<'src, I, Expression, extra::Err<Rich<'src, Token<'src>>>> {
    recursive(|expr| {
        let arguments = 
            expr
                .separated_by(just(Token::Comma)).allow_trailing()
                .collect::<Vec<Expression>>()
                .boxed();

        let function_call = 
            select! { Token::Identifier(ident) => ident }
                .then(arguments.clone()
                    .delimited_by(just(Token::LParen), just(Token::RParen)))
                .map(|(ident, args)| Expression::FunctionCall {
                    name: ident.to_string(),
                    arguments: args,
                })
                .boxed();

        let block = just(Token::Hashtag)
            .ignore_then(select! { Token::Identifier(ident) => ident })
            .then(arguments
                .delimited_by(just(Token::LParen), just(Token::RParen)))
            .map(|(ident, args)| Expression::Block {
                opcode: ident.to_string(),
                arguments: args,
            });

        let literal = choice((
            just(Token::LiteralFalse).to(Literal::Boolean(false)),
            just(Token::LiteralTrue).to(Literal::Boolean(true)),

            select! { Token::LiteralNumber(num) => num }
                .map(Literal::Number),

            select! { Token::LiteralString(str) => str }
                .map(|str| Literal::String(str.to_string())),
        )).map(Expression::Literal);

        let other = choice((
            just(Token::ThisArgument).to(Expression::This),
            select! { Token::Argument(num) => Expression::Argument(num) }
        ));

        let expr = choice((
            block, literal, other, function_call.clone()
        )).boxed();

        expr.then(
                just(Token::Dot)
                    .ignore_then(function_call)
                    .or_not())
            .map(|(expr, method_call)| {
                if let Some(method_call) = method_call {
                    // safety: method_call can never be anything else other than
                    //         Expression::FunctionCall, see the parser function_call
                    let Expression::FunctionCall { name, arguments } = method_call
                        else { unreachable!() };

                    Expression::MethodCall { name, arguments, this: Box::new(expr) }
                } else {
                    expr
                }
            })
            .boxed()
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