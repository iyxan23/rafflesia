use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};
use logos::Logos;
use swrs::api::block::{
    ArgumentBlockReturnType, BlockCategory, BlockContent, BlockControl, BlockType,
};

mod tests;

// todo: list, map, component, and view as block argument types

#[derive(Debug, Clone, PartialEq)]
pub struct BlockDefinitions(pub Vec<BlockDefinition>);

#[derive(Debug, Clone, PartialEq)]
pub struct BlockDefinition {
    pub block_type: BlockType,
    pub category: BlockCategory,
    pub opcode: String,
    pub spec: BlockContent,
}

/// Parses a `.blks` code into [`BlockDefinitions`]
pub fn parse_blks<'src>(
    raw: &'src str,
) -> Result<BlockDefinitions, Vec<Rich<'src, Token<'src>, SimpleSpan>>> {
    let lex = Token::lexer(raw)
        .spanned()
        .map(|(tok, span)| (tok, span.into()));

    let stream = Stream::from_iter(lex).spanned((raw.len()..raw.len()).into());

    parser().parse(stream).into_result()
}

#[derive(Logos, PartialEq, Debug, Clone)]
pub enum Token<'src> {
    #[token("var")]
    CategoryVariable,
    #[token("list")]
    CategoryList,
    #[token("control")]
    CategoryControl,
    #[token("operator")]
    CategoryOperator,
    #[token("math")]
    CategoryMath,
    #[token("file")]
    CategoryFile,
    #[token("view")]
    CategoryView,
    #[token("component")]
    CategoryComponent,
    #[token("moreblock")]
    CategoryMoreblock,

    // Block return types
    #[token("b")]
    TypeBoolean,
    #[token("s")]
    TypeString,
    #[token("d")]
    TypeNumber,
    #[token("l")]
    TypeList, // any list
    #[token("p")]
    TypeComponent,
    #[token("v")]
    TypeView, // any view
    #[token("c")]
    TypeSingleNested,
    #[token("e")]
    TypeDoubleNested,
    #[token("f")]
    TypeEnding,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,

    #[regex(r#""(?:[^"]|\\")*""#, |lex| {
        // remove the `""` around the string
        let slice = lex.slice();
        &slice[1..slice.len() - 1]
    })]
    String(&'src str),

    #[regex(r#"`(?:[^`]|\\`)*`"#, |lex| {
        // remove the ` around the string
        let slice = lex.slice();
        &slice[1..slice.len() - 1]
    })]
    #[regex(r"([a-zA-Z_][a-zA-Z0-9_]*)")]
    Identifier(&'src str),

    #[error]
    #[regex(r"[ \t\n]+", logos::skip)] // whitespace
    #[regex(r"//[^\n]*\n?", logos::skip)] // comment
    Error,
}

fn parser<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>(
) -> impl Parser<'src, I, BlockDefinitions, extra::Err<Rich<'src, Token<'src>>>> {
    let definition = category()
        .then(opcode())
        .then(typ().or_not())
        .then_ignore(just(Token::Colon))
        .then(select! { Token::String(str) => str })
        .map(|(((category, opcode), typ), content)| (category, opcode, typ, content))
        .try_map(|(category, opcode, typ, content), span| {
            Ok((
                category,
                opcode,
                typ,
                BlockContent::parse_wo_params(content)
                    .map_err(|err| Rich::custom(span, err.to_string()))?,
            ))
        });

    // i have no idea how else to use fold lmao
    empty()
        .map(|_| Vec::<BlockDefinition>::new())
        .foldl(
            definition.then_ignore(just(Token::Semicolon)).repeated(),
            |mut acc, (category, opcode, typ, content)| {
                acc.push(BlockDefinition {
                    block_type: typ.unwrap_or(BlockType::Regular),
                    category,
                    opcode: opcode.to_string(),
                    spec: content,
                });
                acc
            },
        )
        .map(BlockDefinitions)
}

fn typ<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>(
) -> impl Parser<'src, I, BlockType, extra::Err<Rich<'src, Token<'src>>>> {
    select! {
        Token::TypeBoolean => BlockType::Argument(ArgumentBlockReturnType::Boolean),
        Token::TypeNumber => BlockType::Argument(ArgumentBlockReturnType::Number),
        Token::TypeString => BlockType::Argument(ArgumentBlockReturnType::String),
        Token::TypeComponent => BlockType::Argument(ArgumentBlockReturnType::Component { type_name: todo!() }),
        Token::TypeView => BlockType::Argument(ArgumentBlockReturnType::View { type_name: todo!() }),
        Token::TypeSingleNested => BlockType::Control(BlockControl::OneNest),
        Token::TypeDoubleNested => BlockType::Control(BlockControl::TwoNest),
        Token::TypeEnding => BlockType::Control(BlockControl::EndingBlock),
    }.delimited_by(just(Token::LParen), just(Token::RParen))
}

fn opcode<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>(
) -> impl Parser<'src, I, &'src str, extra::Err<Rich<'src, Token<'src>>>> {
    select! {
        Token::Identifier(ident) => ident,
    }
}

fn category<'src, I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>>(
) -> impl Parser<'src, I, BlockCategory, extra::Err<Rich<'src, Token<'src>>>> {
    choice((
        just(Token::CategoryComponent).to(BlockCategory::ComponentFunc),
        just(Token::CategoryControl).to(BlockCategory::Control),
        just(Token::CategoryFile).to(BlockCategory::File),
        just(Token::CategoryList).to(BlockCategory::List),
        just(Token::CategoryMath).to(BlockCategory::Math),
        just(Token::CategoryMoreblock).to(BlockCategory::MoreBlock),
        just(Token::CategoryOperator).to(BlockCategory::Operator),
        just(Token::CategoryVariable).to(BlockCategory::Variable),
        just(Token::CategoryView).to(BlockCategory::ViewFunc),
    ))
    .delimited_by(just(Token::LBracket), just(Token::RBracket))
}

// fn category<'src>() -> impl Parser<'src, &'src Token, BlockCategory, Rich<'src, Token>> {
//     choice((
//         text::keyword("component").to(BlockCategory::ComponentFunc),
//         text::keyword("control"  ).to(BlockCategory::Control),
//         text::keyword("file"     ).to(BlockCategory::File),
//         text::keyword("list"     ).to(BlockCategory::List),
//         text::keyword("math"     ).to(BlockCategory::Math),
//         text::keyword("moreblock").to(BlockCategory::MoreBlock),
//         text::keyword("operator" ).to(BlockCategory::Operator),
//         text::keyword("variable" ).to(BlockCategory::Variable),
//         text::keyword("view"     ).to(BlockCategory::ViewFunc),
//     )).padded()
//         .delimited_by(just('['), just(']'))
// }
