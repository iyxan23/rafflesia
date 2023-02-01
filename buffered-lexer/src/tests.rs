extern crate logos;

use logos::Logos;

use crate::{BufferedLexer, error::ParseError, SpannedTokenOwned};

#[derive(Logos, Clone, PartialEq, Debug)]
enum Token {
    #[token("hello")] Hello,
    #[token("world")] World,

    #[token("foo")] Foo,
    #[token("bar")] Bar,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")] Identifier,

    #[error]
    #[regex(r"[ \t]+", logos::skip)] // whitespace
    Error
}

#[inline]
fn create(raw: &str) -> BufferedLexer<'_, Token> {
    BufferedLexer::new(Token::lexer(raw), Token::Error)
}

#[test]
fn creation() {
    let raw = "";
    let _ = create(raw);
}

#[test]
fn next_0() {
    let raw = "hello world";
    let mut lex = create(raw);
    assert_eq!(lex.get_index(), 0);

    let next = lex.next().unwrap();

    assert_eq!(next.token, Token::Hello);
    assert_eq!(next.slice, "hello");
    assert_eq!(lex.get_index(), 1);
}

#[test]
fn next_fail() {
    let raw = "hello world";
    let mut lex = create(raw);

    lex.next().unwrap(); lex.next().unwrap();

    let Err(ParseError::EOF { expected: None }) = lex.next() else {
        panic!("Not eof?");
    };
}

#[test]
fn expect() {
    let raw = "hello world";
    let mut lex = create(raw);

    assert_eq!(lex.get_index(), 0);

    let next = lex.expect(Token::Hello).unwrap();
    assert_eq!(next.token, Token::Hello);
    assert_eq!(next.slice, "hello");
    assert_eq!(next.pos, 0..5);
    assert_eq!(lex.get_index(), 1);


    let next = lex.expect(Token::World).unwrap();
    assert_eq!(next.token, Token::World);
    assert_eq!(next.slice, "world");
    assert_eq!(next.pos, 6..11);
    assert_eq!(lex.get_index(), 2);
}

#[test]
fn expect_fail() {
    let raw = "hello world";
    let mut lex = create(raw);

    let Err(ParseError::UnexpectedTokenError {
        expected: Some(expected),
        unexpected_token: SpannedTokenOwned { token: Token::Hello, slice, pos },
        pos: u_pos
    }) = lex.expect(Token::World) else {
        panic!("Not unexpected?");
    };

    assert_eq!(expected, vec![Token::World]);
    assert_eq!(slice, "hello");
    assert_eq!(pos, 0..5); 
    assert_eq!(u_pos, 0..5);
}

#[test]
fn expect_failsafe_0() {
    let raw = "hello world";
    let mut lex = create(raw);

    assert_eq!(
        lex.expect_failsafe(Token::Hello),
        Some(crate::SpannedTokenOwned {
            token: Token::Hello, slice: "hello".to_string(), pos: 0..5
        })
    )
}

#[test]
fn expect_failsafe_1() {
    let raw = "hello world";
    let mut lex = create(raw);

    assert_eq!(lex.expect_failsafe(Token::World), None);
}

#[test]
fn expect_multiple_choices() {
    let raw = "foo hello world bar";
    let mut lex = create(raw);

    for i in 1..5 {
        assert_eq!(i - 1, lex.get_index());

        match lex.expect_multiple_choices(&vec![
                    Token::Hello, Token::World, Token::Foo, Token::Bar
                ]).unwrap() {

            SpannedTokenOwned { 
                token: Token::Hello | Token::World | Token::Foo | Token::Bar,
                ..
            } => {},

            _ => panic!()
        }

        assert_eq!(i, lex.get_index());
    }
}

#[test]
fn expect_multiple_choices_fail() {
    let raw = "foo hello world bar";
    let mut lex = create(raw);

    let expecting = vec![Token::Hello, Token::World, Token::Foo];

    for i in 1..5 {
        assert_eq!(i - 1, lex.get_index());

        match lex.expect_multiple_choices(&expecting) {

            Ok(SpannedTokenOwned { 
                token: Token::Hello | Token::World | Token::Foo,
                ..
            }) => {},

            Ok(SpannedTokenOwned { token: Token::Bar, .. }) => panic!("got Token::Bar while not expecting it"),

            Err(ParseError::UnexpectedTokenError {
                expected: Some(expected),
                unexpected_token: SpannedTokenOwned { token: Token::Bar, slice, pos },
                pos: u_pos
            }) => {
                assert_eq!(expected, expecting);
                assert_eq!(slice, "bar");
                assert_eq!(pos, 16..19);
                assert_eq!(u_pos, 16..19);
            }

            _ => panic!()
        }

        assert_eq!(i, lex.get_index());
    }
}