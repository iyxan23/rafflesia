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
pub mod error;

use buffered_lexer::propagate_non_recoverable;
use buffered_lexer::{BufferedLexer, SpannedTokenOwned};
use error::DefsParseError;
use logos::Logos;
use models::*;

pub fn parse_defs(raw: &str) -> DefsParseResult<Definitions> {
    let mut lex: BufferedLexer<'_, Token>
        = BufferedLexer::new(Token::lexer(raw), Token::Error);

    functions(&mut lex)
}

#[derive(Logos, PartialEq, Debug, Clone)]
pub enum Token {
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
    #[regex(r#""([^"]|\\")*""#)] LiteralString,
    #[regex("[0-9]+(?:\\.[0-9]+)?")] LiteralNumber,

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

    #[regex(r"@\d+")] Argument,
    #[token("@@")] ThisArgument,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")] Identifier,
    #[regex(r#"`([^`]|\\`)*`"#)] EscapedIdentifier,

    #[error]
    #[regex(r"[ \t\n]+", logos::skip)] // whitespace
    #[regex(r"//[^\n]*\n?", logos::skip)] // comment
    Error
}

pub type DefsParseResult<T> = Result<T, DefsParseError>;
type Lexer<'a> = BufferedLexer<'a, Token>;

fn functions(lex: &mut Lexer) -> DefsParseResult<Definitions> {
    lex.start();

    function(lex)?;

    lex.success();
    todo!()
}

fn function(lex: &mut Lexer) -> DefsParseResult<(FunctionSignature, FunctionDefinition)> {
    let (parent, func_ident) = match lex.expect_multiple_choices(&[
        Token::Identifier,
        Token::TypeBoolean, Token::TypeNumber, Token::TypeString
    ])? {
        ident @ SpannedTokenOwned { token: Token::Identifier, .. } => {
            (None, ident)
        },
        SpannedTokenOwned { token: Token::TypeBoolean, .. } |
        SpannedTokenOwned { token: Token::TypeNumber, .. } |
        SpannedTokenOwned { token: Token::TypeString, .. } => {
            let _ = lex.previous();
            let parent = r#type(lex)?;

            lex.expect(Token::Dot)?;

            let ident = lex.expect(Token::Identifier)?;

            (Some(parent), ident)
        }
        _ => unreachable!()
    };

    let parameters = parameters(lex)?;

    // check if there is a `:` (indicating a return type)
    let ret_type = lex.expect_failsafe(Token::Colon)?.map(|_| r#type(lex)).transpose()?;

    // todo: check for the `=` sign that indicates binding

    // todo: statements

    Ok((FunctionSignature {
        this: parent,
        parameters,
        name: func_ident.slice,
    }, FunctionDefinition {
        blocks: todo!(),
    }))
}

// this function takes `(` and `)` tokens as well
fn parameters(lex: &mut Lexer) -> DefsParseResult<Vec<Type>> {
    lex.start();

    if propagate_non_recoverable!(lex.expect_peek(Token::RParen)).is_ok() {
        return Ok(vec![]);
    }

    let mut parameters = vec![];

    let first = r#type(lex)?;
    parameters.push(first);

    while lex.expect_failsafe(Token::Comma)?.is_some() {
        let r_paren = propagate_non_recoverable!(lex.expect_peek(Token::RParen));
        if r_paren.is_ok() { break; }

        parameters.push(r#type(lex)?);
    }

    lex.success();
    Ok(parameters)
}

fn r#type(lex: &mut Lexer) -> DefsParseResult<Type> {
    Ok(match lex.expect_multiple_choices(&[Token::TypeBoolean, Token::TypeNumber, Token::TypeString])? {
        SpannedTokenOwned { token: Token::TypeBoolean, .. } => Type::Boolean,
        SpannedTokenOwned { token: Token::TypeNumber, .. } => Type::Number,
        SpannedTokenOwned { token: Token::TypeString, .. } => Type::String,
        _ => unreachable!()
    })
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
        blocks.extend(statement(lex)?);
    }

    lex.success();
    Ok(blocks)
}

fn statement(lex: &mut Lexer) -> DefsParseResult<Vec<BlockDispatch>> {
    lex.start();

    // an identifier or an escaped identifier for the "block dispatch"

    lex.expect(Token::Semicolon)?;
    lex.success();
    todo!()
}

fn block_dispatch(lex: &mut Lexer) -> DefsParseResult<Vec<BlockDispatch>> {
    lex.start();
    let mut dispatches = vec![];

    // every block dispatches starts with `#`
    lex.expect(Token::Hashtag)?;

    let identifier = match lex.expect_multiple_choices(&[Token::Identifier, Token::EscapedIdentifier])? {
        SpannedTokenOwned { token: Token::Identifier, slice, .. } => slice,
        SpannedTokenOwned { token: Token::EscapedIdentifier, mut slice, .. } => {
            // remove its ` ` around the identifier
            slice.remove(0); slice.pop();
            slice
        },
        _ => unreachable!()
    };



    lex.success();
    Ok(dispatches)
}

// also takes `(` `)`
fn arguments(lex: &mut Lexer) -> DefsParseResult<Vec<BlockDispatch>> {
    lex.start();
    let mut arguments = vec![];
    lex.expect(Token::LParen)?;

    // check if there's nothing
    if lex.expect_failsafe(Token::RParen)?.is_some() {
        // return early
        return Ok(arguments)
    }

    // there are contents in the parentheses


    lex.success();
    Ok(arguments)
}

// basically arguments of dispatches
fn expression(lex: &mut Lexer) -> DefsParseResult<BlockArgument> {
    lex.start();
    // can be a literal, or a call to another block, or an argument

    lex.success();
    todo!()
}