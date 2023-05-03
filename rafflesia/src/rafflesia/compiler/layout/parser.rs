use crate::compiler::layout::parser::parser::Token;
use buffered_lexer::BufferedLexer;
use logos::Logos;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct View {
    pub name: String,
    pub attributes: Option<HashMap<String, String>>,
    pub children: Option<Box<Vec<View>>>,
    pub view_id: Option<String>,
}

pub fn parse_layout(raw: &str) -> Result<View, parser::LayoutParseError> {
    let mut lex: BufferedLexer<'_, Token> = BufferedLexer::new(Token::lexer(raw), Token::Error);

    // parse it :sunglasses:
    parser::view(&mut lex)
}

mod parser {
    use super::View;
    use buffered_lexer::{BufferedLexer, SpannedTokenOwned};
    use logos::Logos;
    use std::collections::HashMap;

    #[derive(Logos, PartialEq, Debug, Clone)]
    pub enum Token {
        #[token("(")]
        LParentheses,

        #[token(")")]
        RParentheses,

        #[token("{")]
        LBrace,

        #[token("}")]
        RBrace,

        #[token(":")]
        Colon,

        #[token(",")]
        Comma,

        #[regex("[a-zA-Z_]+")]
        Text,

        #[regex(r#""([^"]|\\")*""#)]
        String,

        #[error]
        #[regex(r"[ \n\t]+", logos::skip)] // whitespace
        #[regex(r"//[^\n]*\n?", logos::skip)] // comment
        Error,
    }

    pub type LayoutParseError = buffered_lexer::error::ParseError<Token, SpannedTokenOwned<Token>>;

    // this signature is ridiculous
    pub fn view(lexer: &mut BufferedLexer<Token>) -> Result<View, LayoutParseError> {
        // todo: better error handling, see ariadne
        lexer.start();

        // view starts with a text as its name
        let name = lexer.expect(Token::Text)?.slice;

        let attributes = lexer
            .expect_failsafe_wo_eof(Token::LParentheses)?
            .map(|_| {
                lexer.previous();
                attributes(lexer)
            })
            .transpose()?;

        let children = lexer
            .expect_failsafe_wo_eof(Token::LBrace)?
            .map(|_| {
                lexer.previous();
                Ok(Box::new(children(lexer)?))
            })
            .transpose()?;

        // fixme: maybe use a new func for this?
        let view_id = if lexer.expect_failsafe_wo_eof(Token::Colon)?.is_some() {
            Some(lexer.expect(Token::Text)?.slice.to_string())
        } else {
            None
        };

        lexer.success();

        Ok(View {
            name,
            attributes,
            children,
            view_id,
        })
    }

    pub fn attributes(
        lexer: &mut BufferedLexer<Token>,
    ) -> Result<HashMap<String, String>, LayoutParseError> {
        lexer.start();
        lexer.expect(Token::LParentheses)?;

        let mut result = HashMap::new();

        if lexer.expect_failsafe_wo_eof(Token::RParentheses)?.is_some() {
            // welp I guess theres nothing here
            return Ok(result);
        }

        // if its not closed then there must be an attribute
        let first = attribute(lexer)?;
        result.insert(first.0, first.1);

        while lexer.expect_failsafe_wo_eof(Token::Comma)?.is_some() {
            // check if next is a closing parentheses, means this is a trailing comma
            if lexer.expect_peek(Token::RParentheses).is_ok() {
                break;
            }

            // parse attribute
            let attr = attribute(lexer)?;
            result.insert(attr.0, attr.1);
        }

        // not a comma, must be a closing parentheses
        lexer.expect(Token::RParentheses)?;
        lexer.success();

        Ok(result)
    }

    pub fn attribute(
        lexer: &mut BufferedLexer<Token>,
    ) -> Result<(String, String), LayoutParseError> {
        lexer.start();

        // attr: value
        let attr = value(lexer)?;
        lexer.expect(Token::Colon)?;
        let value = value(lexer)?;

        lexer.success();
        Ok((attr, value))
    }

    pub fn value(lexer: &mut BufferedLexer<Token>) -> Result<String, LayoutParseError> {
        lexer.start();

        let res = match lexer.expect_multiple_choices(&vec![Token::Text, Token::String])? {
            SpannedTokenOwned {
                token: Token::Text,
                slice,
                ..
            } => slice.to_string(),
            SpannedTokenOwned {
                token: Token::String,
                slice,
                ..
            } =>
            /* remove the `"` around it */
            {
                slice[1..slice.len() - 1].to_string()
            }

            _ => unreachable!(),
        };

        lexer.success();
        Ok(res)
    }

    pub fn children(lexer: &mut BufferedLexer<Token>) -> Result<Vec<View>, LayoutParseError> {
        lexer.start();

        let mut result = Vec::new();

        // expect an opening brace
        lexer.expect(Token::LBrace)?;

        // check if it already ended :l
        if lexer.expect_failsafe_wo_eof(Token::RBrace)?.is_some() {
            // welp i guess theres nothing here
            return Ok(result);
        }

        let first = view(lexer)?;
        result.push(first);

        while lexer.expect_failsafe_wo_eof(Token::Comma)?.is_some() {
            // check if next is a closing brace, means this is a trailing comma
            if lexer.expect_peek(Token::RBrace).is_ok() {
                break;
            }

            // parse child
            result.push(view(lexer)?);
        }

        lexer.expect(Token::RBrace)?;

        lexer.success();
        Ok(result)
    }
}
