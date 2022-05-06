use anyhow::Result;
use std::collections::HashMap;
use logos::Logos;
use logos_iter::{PeekableLexer, LogosIter};

#[derive(Debug, PartialEq)]
pub struct View {
    pub name: String,
    pub attributes: Option<HashMap<String, String>>,
    pub children: Option<Box<Vec<View>>>,
    pub view_id: Option<String>,
}

pub fn parse_layout(raw: &str) -> Result<View> {
    let mut lex: PeekableLexer<'_, _, Token> = Token::lexer(raw).peekable_lexer();

    // parse it :sunglasses:
    parser::view(&mut lex)
}

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

mod parser {
    use std::collections::HashMap;
    use anyhow::{Result, bail, Context};
    use logos_iter::{LogosIter, PeekableLexer};
    use super::Token;
    use super::View;

    // this signature is ridiculous
    pub fn view<'a, L>(lexer: &mut PeekableLexer<'a, L, Token>) -> Result<View>
    where L: LogosIter<'a, Token> {
        // todo: better error handling, see ariadne

        // view starts with a text as its name
        let name = if let Token::Text = lexer.next().context("expected a text at the begining of view")? {
            lexer.slice().to_string()
        } else {
            bail!("token is not text {:?}", lexer.span());
        };

        let attributes = if let Some(Token::LParentheses) = lexer.peek() {
            Some(attributes(lexer).context("Failed to parse attributes")?)
        } else { None };

        let children = if let Some(Token::LBrace) = lexer.peek() {
            // need box because `View` is Sized
            Some(Box::new(children(lexer).context("Failed to parse children")?))
        } else { None };

        let view_id = if let Some(Token::Colon) = lexer.peek() {
            let _ = lexer.next();

            if let Token::Text = lexer.next().context("expected a text as id after colon")? {} else {
                bail!("expected a text as id after colon, got `{}` instead", lexer.slice());
            }

            Some(lexer.slice().to_string())
        } else { None };

        Ok(View {
            name,
            attributes,
            children,
            view_id,
        })
    }

    pub fn attributes<'a, L>(lexer: &mut PeekableLexer<'a, L, Token>) -> Result<HashMap<String, String>>
        where L: LogosIter<'a, Token> {

        if let Some(Token::LParentheses) = lexer.next() {} else {
            bail!("expected a left parentheses `(`, got `{}` instead", lexer.slice());
        }

        let mut result = HashMap::new();

        if let Some(Token::RParentheses) = lexer.peek() {
            let _ = lexer.next();

            // welp i guess theres nothing here
            return Ok(result);
        }

        // if its not closed then there must be an attribute
        let first = attribute(lexer).context("Failed to parse attribute")?;
        result.insert(first.0, first.1);

        while let Some(Token::Comma) = lexer.peek() {
            let _ = lexer.next();

            // check if next is a closing parentheses, means this is a trailing comma
            if let Some(Token::RParentheses) = lexer.peek() { break; }

            // parse attribute
            let attr = attribute(lexer).context("Failed to parse attribute")?;
            result.insert(attr.0, attr.1);
        }

        // not a comma, must be a closing parentheses
        if let Some(Token::RParentheses) = lexer.next() {} else {
            bail!("expected a closing parentheses, got `{}` instead", lexer.slice());
        }

        Ok(result)
    }

    pub fn attribute<'a, L>(lexer: &mut PeekableLexer<'a, L, Token>) -> Result<(String, String)>
        where L: LogosIter<'a, Token> {

        let attr = value(lexer).context("Failed to parse attribute name")?;

        // expect a colon
        if let Some(Token::Colon) = lexer.next() {} else {
            bail!("expected a colon, got `{}` instead", lexer.slice());
        }

        let value = value(lexer).context("Failed to parse attribute value")?;

        Ok((attr, value))
    }

    pub fn value<'a, L>(lexer: &mut PeekableLexer<'a, L, Token>) -> Result<String>
        where L: LogosIter<'a, Token> {

        Ok(match lexer.next() {
            Some(Token::Text) => lexer.slice().to_string(),
            Some(Token::String) => {
                let string: &str = lexer.slice();

                // remove the `"` around it
                string[1..string.len() - 1].to_string()
            },
            None => bail!("expected a text or a string"),
            _ => bail!("expected a text or a string, got `{}` instead", lexer.slice())
        })
    }

    pub fn children<'a, L>(lexer: &mut PeekableLexer<'a, L, Token>) -> Result<Vec<View>>
        where L: LogosIter<'a, Token> {

        let mut result = Vec::new();

        // expect an opening brace
        if let Some(Token::LBrace) = lexer.next() {} else {
            bail!("expected an opening brace, got `{}` instead", lexer.slice());
        }

        // check if it already ended :l
        if let Some(Token::RBrace) = lexer.peek() {
            let _ = lexer.next();
            // welp i guess theres nothing here
            return Ok(result);
        }

        let first = view(lexer).context("Failed parsing the first child")?;
        result.push(first);

        while let Some(Token::Comma) = lexer.peek() {
            let _ = lexer.next();

            // check if next is a closing brace, means this is a trailing comma
            if let Some(Token::RBrace) = lexer.peek() { break; }

            // parse child
            result.push(view(lexer).context("Failed to parse child")?);
        }

        if let Some(Token::RBrace) = lexer.next() {} else {
            bail!("expected a closing brace, got `{}` instead", lexer.slice());
        }

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