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

pub mod error;
pub mod models;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use buffered_lexer::{BufferedLexer, SpannedTokenOwned, error::ParseError};
use error::DefsParseError;
use logos::Logos;
use models::*;

pub fn parse_defs(raw: &str) -> DefsParseResult<Definitions> {
    let mut lex: BufferedLexer<'_, Token> = BufferedLexer::new(Token::lexer(raw), Token::Error);

    functions(&mut lex)
}

#[derive(Logos, PartialEq, Debug, Clone)]
pub enum Token {
    // Block return types
    #[token("b")]
    TypeBoolean,
    #[token("s")]
    TypeString,
    #[token("d")]
    TypeNumber,
    // #[token("l")] TypeList,
    // #[token("p")] TypeComponent,
    // #[token("v")] TypeView,
    #[token("c")]
    TypeSingleNested,
    #[token("e")]
    TypeDoubleNested,
    #[token("f")]
    TypeEnding,

    // literals
    #[token("true")]
    LiteralTrue,
    #[token("false")]
    LiteralFalse,
    #[regex(r#""([^"]|\\")*""#)]
    LiteralString,
    #[regex("[0-9]+(?:\\.[0-9]+)?")]
    LiteralNumber,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    #[token("=")]
    Equal,
    #[token("<")]
    Return,
    #[token("#")]
    Hashtag,

    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,

    #[regex(r"@\d+")]
    Argument,
    #[token("@@")]
    ThisArgument,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,
    #[regex(r#"`([^`]|\\`)*`"#)]
    EscapedIdentifier,

    #[error]
    #[regex(r"[ \t\n]+", logos::skip)] // whitespace
    #[regex(r"//[^\n]*\n?", logos::skip)] // comment
    Error,
}

pub type DefsParseResult<T> = Result<T, DefsParseError>;
type Lexer<'a> = BufferedLexer<'a, Token>;

fn functions(lex: &mut Lexer) -> DefsParseResult<Definitions> {
    lex.start();

    let mut defs = Definitions { global_functions: vec![], methods: vec![] };
    let mut methods = HashMap::new();

    loop {
        // break if we've reached EOF, but propagate lexer errors
        if let Err(err) = lex.peek() {
            match err {
                ParseError::EOF { .. } => break,
                ParseError::LexerError { .. } => Err(err)?,
                _ => unreachable!()
            }
        }

        let (func_signature, func_def) = function(lex)?;
        if let Some(sign) = func_signature.this.clone() {
            methods
                .entry(sign)
                .or_insert_with(|| vec![])
                .push((func_signature, func_def));
        } else {
            defs.global_functions.push((func_signature, func_def));
        }
    }

    // after we've collected all the methods, we turn them into a vec
    defs.methods = methods.into_iter().collect();

    lex.success();
    Ok(defs)
}

fn function(lex: &mut Lexer) -> DefsParseResult<(FunctionSignature, FunctionDefinition)> {
    let (this, func_ident) = match lex.expect_multiple_choices(&[
        Token::Identifier,
        Token::TypeBoolean,
        Token::TypeNumber,
        Token::TypeString,
    ])? {
        ident @ SpannedTokenOwned {
            token: Token::Identifier,
            ..
        } => (None, ident),
        SpannedTokenOwned {
            token: Token::TypeBoolean,
            ..
        }
        | SpannedTokenOwned {
            token: Token::TypeNumber,
            ..
        }
        | SpannedTokenOwned {
            token: Token::TypeString,
            ..
        } => {
            let _ = lex.previous();
            let parent = r#type(lex)?;

            lex.expect(Token::Dot)?;

            let ident = lex.expect(Token::Identifier)?;

            (Some(parent), ident)
        }
        _ => unreachable!(),
    };

    println!(
        "got function ident and type: {:?}, {}",
        this, func_ident.slice
    );

    let parameters = parameters(lex)?;

    // check if there is a `:` (indicating a return type)
    let return_type = lex
        .expect_failsafe(Token::Colon)?
        .map(|_| r#type(lex))
        .transpose()?;

    // todo: check for the `=` sign that indicates binding

    let blocks = statements_block(lex)?;

    Ok((
        FunctionSignature {
            this,
            parameters,
            name: func_ident.slice,
            return_type,
        },
        FunctionDefinition { blocks },
    ))
}

// this function takes `(` and `)` tokens as well
fn parameters(lex: &mut Lexer) -> DefsParseResult<Vec<Type>> {
    lex.start();
    lex.expect(Token::LParen)?;

    let mut parameters = vec![];

    loop {
        if let Some(_) = lex.expect_failsafe(Token::RParen)? {
            break;
        }

        parameters.push(r#type(lex)?);
        let Some(_) = lex.expect_failsafe(Token::Comma)? else {
            lex.expect(Token::RParen)?;
            break
        };
    }

    lex.success();
    Ok(parameters)
}

fn r#type(lex: &mut Lexer) -> DefsParseResult<Type> {
    Ok(
        match lex.expect_multiple_choices(&[
            Token::TypeBoolean,
            Token::TypeNumber,
            Token::TypeString,
        ])? {
            SpannedTokenOwned {
                token: Token::TypeBoolean,
                ..
            } => Type::Boolean,
            SpannedTokenOwned {
                token: Token::TypeNumber,
                ..
            } => Type::Number,
            SpannedTokenOwned {
                token: Token::TypeString,
                ..
            } => Type::String,
            _ => unreachable!(),
        },
    )
}

// also parses the `{` `}`
fn statements_block(lex: &mut Lexer) -> DefsParseResult<Vec<BlockDispatch>> {
    lex.start();
    let mut blocks = vec![];

    // open up `{`
    lex.expect(Token::LBrace)?;

    // check if this is empty
    if lex.expect_failsafe(Token::RBrace)?.is_some() {
        // return early
        return Ok(blocks);
    }

    // there are block dispatches in this statements block
    while !lex.expect_failsafe(Token::RBrace)?.is_some() {
        blocks.push(statement(lex)?);
    }

    lex.success();
    Ok(blocks)
}

fn statement(lex: &mut Lexer) -> DefsParseResult<BlockDispatch> {
    lex.start();

    let block = block_dispatch(lex)?;

    lex.expect(Token::Semicolon)?;
    lex.success();
    Ok(block)
}

fn block_dispatch(lex: &mut Lexer) -> DefsParseResult<BlockDispatch> {
    lex.start();

    // every block dispatches starts with `#`
    lex.expect(Token::Hashtag)?;

    let identifier =
        match lex.expect_multiple_choices(&[Token::Identifier, Token::EscapedIdentifier])? {
            SpannedTokenOwned {
                token: Token::Identifier,
                slice,
                ..
            } => slice,
            SpannedTokenOwned {
                token: Token::EscapedIdentifier,
                mut slice,
                ..
            } => {
                // remove its ` ` around the identifier
                slice.remove(0);
                slice.pop();
                slice
            }
            _ => unreachable!(),
        };

    let arguments = arguments(lex)?;

    lex.success();
    Ok(BlockDispatch {
        opcode: identifier,
        arguments,
    })
}

// also takes `(` `)`
fn arguments(lex: &mut Lexer) -> DefsParseResult<Vec<BlockArgument>> {
    lex.start();
    let mut arguments = vec![];
    lex.expect(Token::LParen)?;

    loop {
        if let Some(_) = lex.expect_failsafe(Token::RParen)? {
            break;
        }

        arguments.push(argument(lex)?);
        if lex.expect_failsafe(Token::Comma)?.is_none() {
            lex.expect(Token::RParen)?;
            break;
        }
    }

    lex.success();
    Ok(arguments)
}

// basically arguments of dispatches
fn argument(lex: &mut Lexer) -> DefsParseResult<BlockArgument> {
    lex.start();
    // can be a literal, or a call to another block, or an argument
    Ok(
        match lex.expect_multiple_choices(&[
            // literals
            Token::LiteralFalse,
            Token::LiteralTrue,
            Token::LiteralString,
            Token::LiteralNumber,
            // a block dispatch
            Token::Hashtag,
            // an argument
            Token::Argument,
            Token::ThisArgument,
        ])? {
            // ===> literals
            SpannedTokenOwned {
                token: Token::LiteralTrue,
                ..
            } => {
                lex.success();
                BlockArgument::Literal(Literal::Boolean(true))
            }
            SpannedTokenOwned {
                token: Token::LiteralFalse,
                ..
            } => {
                lex.success();
                BlockArgument::Literal(Literal::Boolean(false))
            }
            SpannedTokenOwned {
                token: Token::LiteralString,
                mut slice,
                ..
            } => {
                lex.success();
                // remove its `"` `"`
                slice.remove(0);
                slice.pop();
                BlockArgument::Literal(Literal::String(slice))
            }
            SpannedTokenOwned {
                token: Token::LiteralNumber,
                slice,
                ..
            } => {
                lex.success();
                // safety: .unwrap() since the LiteralNumber token can only be of numbers as defined in the lexer
                BlockArgument::Literal(Literal::Number(slice.parse().unwrap()))
            }

            // ===> a hashtag indicates an identifier to a block dispatch
            SpannedTokenOwned { token: Token::Hashtag, .. } => {
                // go back since we want to leave the parsing to `block_dispatch`
                lex.previous();
                lex.success();

                BlockArgument::BlockDispatch(block_dispatch(lex)?)
            }

            // ===> arguments
            SpannedTokenOwned {
                token: Token::Argument,
                mut slice,
                ..
            } => {
                lex.success();
                // remove the `@` in the front
                slice.remove(0);
                // safety: .unwrap() since the Argument token is guaranteed to be number after the `@`
                BlockArgument::Argument {
                    index: slice.parse().unwrap(),
                }
            }
            SpannedTokenOwned {
                token: Token::ThisArgument,
                ..
            } => {
                lex.success();
                BlockArgument::This
            }
            _ => unreachable!(),
        },
    )
}
