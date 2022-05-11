use std::collections::HashMap;
use crate::compiler::parser::LexerWrapper;

#[derive(Debug, PartialEq)]
pub struct View {
    pub name: String,
    pub attributes: Option<HashMap<String, String>>,
    pub children: Option<Box<Vec<View>>>,
    pub view_id: Option<String>,
}

pub fn parse_layout(raw: &str) -> Result<View, parser::LayoutParseError> {
    let mut lex: LexerWrapper<'_, parser::Token> = LexerWrapper::new(parser::Token::lexer(raw));

    // parse it :sunglasses:
    parser::view(&mut lex)
}

mod parser {
    use std::collections::HashMap;
    use crate::compiler::parser::{error, LexerWrapper, TokenWrapper, TokenWrapperOwned};
    use super::View;
    use logos::Logos;

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

    pub type LayoutParseError = error::ParseError<Token, TokenWrapperOwned<Token>>;

    // this signature is ridiculous
    pub fn view(lexer: &mut LexerWrapper<Token>) -> Result<View, LayoutParseError> {
        // todo: better error handling, see ariadne
        lexer.start();

        // view starts with a text as its name
        let name = lexer.expect(Token::Text)?.slice;
        let attributes = if lexer.expect_peek(Token::LParentheses).is_ok() {
            Some(attributes(lexer)?)
        } else { None };

        let children = if lexer.expect_peek(Token::LBrace).is_ok() {
            // need box because `View` is Sized
            Some(Box::new(children(lexer)?))
        } else { None };

        let view_id = if lexer.expect_failsafe(Token::Colon).is_some() {
            Some(lexer.expect(Token::Text)?.slice.to_string())
        } else { None };

        lexer.success();

        Ok(View {
            name,
            attributes,
            children,
            view_id,
        })
    }

    pub fn attributes(lexer: &mut LexerWrapper<Token>)
        -> Result<HashMap<String, String>, LayoutParseError> {

        lexer.start();
        lexer.expect(Token::LParentheses)?;

        let mut result = HashMap::new();

        if lexer.expect_failsafe(Token::RParentheses).is_some() {
            // welp I guess theres nothing here
            return Ok(result);
        }

        // if its not closed then there must be an attribute
        let first = attribute(lexer)?;
        result.insert(first.0, first.1);

        while lexer.expect_failsafe(Token::Comma).is_some() {
            // check if next is a closing parentheses, means this is a trailing comma
            if lexer.expect_peek(Token::RParentheses).is_ok() { break; }

            // parse attribute
            let attr = attribute(lexer)?;
            result.insert(attr.0, attr.1);
        }

        // not a comma, must be a closing parentheses
        lexer.expect(Token::RParentheses)?;
        lexer.success();

        Ok(result)
    }

    pub fn attribute(lexer: &mut LexerWrapper<Token>)
        -> Result<(String, String), LayoutParseError> {
        lexer.start();

        // attr: value
        let attr = value(lexer)?;
        lexer.expect(Token::Colon)?;
        let value = value(lexer)?;

        lexer.success();
        Ok((attr, value))
    }

    pub fn value(lexer: &mut LexerWrapper<Token>) -> Result<String, LayoutParseError> {
        lexer.start();

        let res = match lexer.next() {
            Some(TokenWrapper { token: Token::Text, slice, .. }) => Ok(slice.to_string()),
            Some(TokenWrapper { token: Token::String, slice, .. }) =>
                /* remove the `"` around it */
                Ok(slice[1..slice.len() - 1].to_string()),

            // other tokens
            Some(tok) => {
                let cloned_tok = tok.clone();
                return Err(error::ParseError::UnexpectedTokenError {
                    expected: Some(vec![Token::Text, Token::String]),
                    pos: cloned_tok.pos.clone(),
                    unexpected_token: cloned_tok.into()
                })
            },
            None => {
                return Err(error::ParseError::EOF {
                    expected: Some(vec![Token::Text, Token::String])
                })
            },
        };

        lexer.success();
        res
    }

    pub fn children(lexer: &mut LexerWrapper<Token>) -> Result<Vec<View>, LayoutParseError> {
        lexer.start();

        let mut result = Vec::new();

        // expect an opening brace
        lexer.expect(Token::LBrace)?;

        // check if it already ended :l
        if lexer.expect_failsafe(Token::RBrace).is_some() {
            // welp i guess theres nothing here
            return Ok(result);
        }

        let first = view(lexer)?;
        result.push(first);

        while lexer.expect_failsafe(Token::Comma).is_some() {
            // check if next is a closing brace, means this is a trailing comma
            if lexer.expect_peek(Token::RBrace).is_ok() { break; }

            // parse child
            result.push(view(lexer)?);
        }

        lexer.expect(Token::RBrace)?;

        lexer.success();
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::parse_layout;

    #[test]
    fn simple() {
        let input =
r#"LinearLayout (hello: "world") {
    TextView (text: hi): myText,
    TextView (
        "another": "text",
        trailing: comma,
    ),
}"#;
        let result = parse_layout(input).unwrap();

        // todo: assert
        println!("{:?}", result);
    }
}