use logos::Logos;
use swrs::api::block::{BlockCategory, BlockContent, BlockType, ArgumentBlockReturnType, BlockControl, BlockContentParseError};
use buffered_lexer::{BufferedLexer, SpannedTokenOwned, error::ParseError};

pub mod error;
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
pub fn parse_blks(raw: &str) -> BlksParseResult<BlockDefinitions> {
    let mut lex: BufferedLexer<'_, Token>
        = BufferedLexer::new(Token::lexer(raw), Token::Error);

    statements(&mut lex)
}

#[derive(Logos, PartialEq, Debug, Clone)]
pub enum Token {
    // Block categories
    #[token("var")] CategoryVariable,
    #[token("list")] CategoryList,
    #[token("control")] CategoryControl,
    #[token("operator")] CategoryOperator,
    #[token("math")] CategoryMath,
    #[token("file")] CategoryFile,
    #[token("view")] CategoryView,
    #[token("component")] CategoryComponent,
    #[token("moreblock")] CategoryMoreblock,

    // Block return types
    #[token("b")] TypeBoolean,
    #[token("s")] TypeString,
    #[token("d")] TypeNumber,
    #[token("l")] TypeList, // any list
    #[token("p")] TypeComponent,
    #[token("v")] TypeView, // any view
    #[token("c")] TypeSingleNested,
    #[token("e")] TypeDoubleNested,
    #[token("f")] TypeEnding,

    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("[")] LBracket,
    #[token("]")] RBracket,

    #[token(":")] Colon,
    #[token(";")] Semicolon,

    #[regex(r#""([^"]|\\")*""#)] String,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")] Identifier,
    #[regex(r#"`([^`]|\\`)*`"#)] EscapedIdentifier,

    #[error]
    #[regex(r"[ \t\n]+", logos::skip)] // whitespace
    #[regex(r"//[^\n]*\n?", logos::skip)] // comment
    Error
}

const CATEGORIES: [Token; 9] = [
    Token::CategoryVariable,
    Token::CategoryList,
    Token::CategoryControl,
    Token::CategoryOperator,
    Token::CategoryMath,
    Token::CategoryFile,
    Token::CategoryView,
    Token::CategoryComponent,
    Token::CategoryMoreblock,
];

const BLOCK_TYPES: [Token; 9] = [
    Token::TypeBoolean,
    Token::TypeString,
    Token::TypeNumber,
    Token::TypeList,
    Token::TypeComponent,
    Token::TypeView,
    Token::TypeSingleNested,
    Token::TypeDoubleNested,
    Token::TypeEnding,
];

#[derive(Debug)]
pub enum BlksParseError {
    ParsingError(ParseError<Token, SpannedTokenOwned<Token>>),
    SpecParseError(BlockContentParseError)
}

impl From<ParseError<Token, SpannedTokenOwned<Token>>> for BlksParseError {
    fn from(value: ParseError<Token, SpannedTokenOwned<Token>>) -> Self {
        Self::ParsingError(value)
    }
}

impl From<BlockContentParseError> for BlksParseError {
    fn from(value: BlockContentParseError) -> Self {
        Self::SpecParseError(value)
    }
}

pub type BlksParseResult<T> = Result<T, BlksParseError>;
type Lexer<'a> = BufferedLexer<'a, Token>;

fn statements(lexer: &mut Lexer) -> BlksParseResult<BlockDefinitions> {
    lexer.start();

    let mut definitions = Vec::new();

    loop {
        // break only when the error is EOF, propagate the error if its other than EOF
        match lexer.peek() {
            Ok(SpannedTokenOwned { token: Token::LParen, .. }) => {},
            Err(ParseError::EOF { .. }) => break,
            Err(err) => Err(err)?,
            _ => {},
        }
        
        definitions.push(statement(lexer)?);
        lexer.expect(Token::Semicolon)?;
    }

    lexer.success();
    Ok(BlockDefinitions(definitions))
}

fn statement(lexer: &mut Lexer) -> BlksParseResult<BlockDefinition> {
    lexer.start();

    fn token_to_category(tok: &Token) -> BlockCategory {
        match tok {
            Token::CategoryVariable => BlockCategory::Variable,
            Token::CategoryList => BlockCategory::List,
            Token::CategoryControl => BlockCategory::Control,
            Token::CategoryOperator => BlockCategory::Operator,
            Token::CategoryMath => BlockCategory::Math,
            Token::CategoryFile => BlockCategory::File,
            Token::CategoryView => BlockCategory::ViewFunc,
            Token::CategoryComponent => BlockCategory::ComponentFunc,
            Token::CategoryMoreblock => BlockCategory::MoreBlock,
            _ => unreachable!(),
        }
    }

    // todo: specify list, component, and view types
    fn token_to_block_type(tok: &Token) -> BlockType {
        match tok {
            Token::TypeBoolean => BlockType::Argument(ArgumentBlockReturnType::Boolean),
            Token::TypeString => BlockType::Argument(ArgumentBlockReturnType::String),
            Token::TypeNumber => BlockType::Argument(ArgumentBlockReturnType::Number),
            Token::TypeList => BlockType::Argument(ArgumentBlockReturnType::List { inner_type: todo!() }),
            Token::TypeComponent => BlockType::Argument(ArgumentBlockReturnType::Component { type_name: todo!() }),
            Token::TypeView => BlockType::Argument(ArgumentBlockReturnType::View { type_name: todo!() }),
            Token::TypeSingleNested => BlockType::Control(BlockControl::OneNest),
            Token::TypeDoubleNested => BlockType::Control(BlockControl::TwoNest),
            Token::TypeEnding => BlockType::Control(BlockControl::EndingBlock),
            _ => unreachable!(),
        }
    }

    lexer.expect(Token::LBracket)?;
    let category_tok = lexer.expect_multiple_choices(&CATEGORIES)?;
    let category = token_to_category(&category_tok.token);
    lexer.expect(Token::RBracket)?;

    let ident = lexer.expect_multiple_choices(&[Token::Identifier, Token::EscapedIdentifier])?;
    let opcode = if matches!(ident, SpannedTokenOwned { token: Token::EscapedIdentifier, .. }) {
        ident.slice[1..ident.slice.len() - 1].to_string()
    } else {
        ident.slice
    };

    let block_type = if let Some(_lparen) = lexer.expect_failsafe(Token::LParen) {
        // block type is specified
        let block_type_tok = lexer.expect_multiple_choices(&BLOCK_TYPES)?;
        lexer.expect(Token::RParen)?;
        token_to_block_type(&block_type_tok.token)
    } else {
        // no block type specified, regular block
        BlockType::Regular
    };

    lexer.expect(Token::Colon)?;

    let spec_tok = lexer.expect(Token::String)?;
    let spec = BlockContent::parse_wo_params(&spec_tok.slice[1..spec_tok.slice.len() - 1])?;
    
    lexer.success();
    Ok(BlockDefinition { block_type, category, opcode, spec })
}
