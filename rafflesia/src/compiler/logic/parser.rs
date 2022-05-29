use anyhow::Result;
use crate::compiler::parser::{error, LexerWrapper, TokenWrapperOwned};
use logos::Logos;
use crate::compiler::parser::error::ParseError;
use super::ast::*;

pub fn parse_logic(raw: &str) -> Result<OuterStatements> {
    Ok(todo!())
}

#[derive(Logos, PartialEq, Debug, Clone)]
pub enum Token {

    // arithmetic operations
    #[token("**")] Pow,
    #[token("*")] Mult,
    #[token("/")] Div,
    #[token("+")] Plus,
    #[token("-")] Minus,

    // delimiter
    #[token(",")] Comma,
    #[token("\n")] Newline,

    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token("{")] LBrace,
    #[token("}")] RBrace,

    #[token(".")] DOT,

    // boolean operators
    #[token("!")] Not,
    #[token("==")] DEQ,
    #[token("=")] EQ,
    #[token("<")] LT,
    #[token("<=")] LTE,
    #[token(">")] GT,
    #[token(">=")] GTE,

    #[token("&&")] And,
    #[token("||")] Or,

    // types
    #[token("number")] NumberType,
    #[token("string")] StringType,
    #[token("boolean")] BooleanType,
    #[token("map")] MapType,
    #[token("list")] ListType,

    // compound statements
    #[token("if")] If,
    #[token("else")] Else,
    #[token("repeat")] Repeat,
    #[token("forever")] Forever,

    // simple statements
    #[token("break")] Break,
    #[token("continue")] Continue,

    // literals
    #[token("true")] True,
    #[token("false")] False,

    #[regex(r#""([^"]|\\")*""#)] String,
    #[regex("[0-9]+(?:\\.[0-9]+)")] Number,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")] Identifier,

    #[error]
    #[regex(r"[ \t]+", logos::skip)] // whitespace
    #[regex(r"//[^\n]*\n?", logos::skip)] // comment
    Error
}

pub type LogicParseError = error::ParseError<Token, TokenWrapperOwned<Token>>;
pub type LogicParseResult<T> = Result<T, LogicParseError>;
type Lexer<'a> = LexerWrapper<'a, Token>;

// todo: a "block" rule for `{ ... }`

// A note before you read all this: I have no idea about parser stuff; so if you do know about it,
// please tell me what is your approach to doing this, I feel like there's a waaay better way
// of doing this.

fn outer_statements(lex: &mut Lexer) -> LogicParseResult<OuterStatements> {
    lex.start();
    let mut statements = OuterStatements(vec![]);

    loop {
        // skip any newlines
        while let Some(_) = lex.expect_failsafe(Token::Newline) {}

        // break if we've reached EOF, but propagate lexer errors
        match lex.peek() {
            Err(err) => {
                match err {
                    ParseError::EOF { .. } => break,
                    ParseError::LexerError { .. } => return Err(err),
                    _ => unreachable!()
                }
            }
            _ => ()
        }

        statements.0.push(outer_statement(lex)?);
    }

    lex.success();
    Ok(statements)
}

fn outer_statement(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    lex.start();

    let res = match lex.expect_peek_multiple_choices(
        // expects a type, complex type, or an event identifier
        vec![Token::NumberType, Token::StringType, Token::BooleanType, Token::MapType,
             Token::ListType, Token::Identifier]
    )? {
        TokenWrapperOwned { token: Token::Identifier, .. } => outer_event_definition(lex),
        TokenWrapperOwned {
            token: Token::NumberType | Token::StringType | Token::BooleanType,
            ..
        } => outer_variable_declaration(lex),
        TokenWrapperOwned {
            token: Token::MapType | Token::ListType,
            ..
        } => outer_complex_variable_declaration(lex),
        _ => unreachable!()
    };

    lex.success();
    res
}

fn simple_variable_type(lex: &mut Lexer) -> LogicParseResult<VariableType> {
    lex.start();

    let res = match lex.expect_multiple_choices(
        vec![Token::NumberType, Token::StringType, Token::BooleanType]
    )? {
        TokenWrapperOwned { token: Token::NumberType, .. } => VariableType::Number,
        TokenWrapperOwned { token: Token::StringType, .. } => VariableType::String,
        TokenWrapperOwned { token: Token::BooleanType, .. } => VariableType::Boolean,
        _ => unreachable!()
    };

    lex.success();
    Ok(res)
}

fn outer_variable_declaration(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    lex.start();

    // get the type
    let variable_type = simple_variable_type(lex)?;

    // next is the identifier
    let identifier = lex.expect(Token::Identifier)?.slice;

    lex.success();
    Ok(OuterStatement::SimpleVariableDeclaration {
        variable_type,
        identifier,
    })
}

fn outer_complex_variable_declaration(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    lex.start();

    enum ComplexVariableTokenType { Map, List }

    // get the type
    let cx_var_tok_type = match lex.expect_multiple_choices(
        vec![Token::MapType, Token::ListType]
    )? {
        TokenWrapperOwned { token: Token::MapType, .. } => ComplexVariableTokenType::Map,
        TokenWrapperOwned { token: Token::ListType, .. } => ComplexVariableTokenType::List,
        _ => unreachable!()
    };

    // open the <>
    lex.expect(Token::LT)?;

    // the inner type
    let inner_type = simple_variable_type(lex)?;

    lex.expect(Token::GT)?;

    // next is the identifier
    let identifier = lex.expect(Token::Identifier)?.slice;

    lex.success();
    Ok(OuterStatement::ComplexVariableDeclaration {
        variable_type: match cx_var_tok_type {
            ComplexVariableTokenType::Map => ComplexVariableType::Map { inner_type },
            ComplexVariableTokenType::List => ComplexVariableType::List { inner_type }
        },
        identifier
    })
}

fn outer_event_definition(lex: &mut Lexer) -> LogicParseResult<OuterStatement> {
    // this is where the fun begins
    lex.start();

    let name = lex.expect(Token::Identifier)?.slice;

    // check if there is a dot after an identifier (means that it's a view listener)
    if let Some(_) = lex.expect_failsafe(Token::DOT) {
        let event_name = lex.expect(Token::Identifier)?.slice;

        // parse the body of this event
        // where the real fun begins :sunglasses:
        lex.expect(Token::LBrace)?;

        let statements = inner_statements(lex)?;

        lex.expect(Token::RBrace)?;

        lex.success();
        Ok(OuterStatement::ViewEventListener {
            view_id: name,
            event_name,
            body: statements
        })

    } else {
        // parse the body of this event
        // where the real fun begins :sunglasses:
        lex.expect(Token::LBrace)?;

        let statements = inner_statements(lex)?;

        lex.expect(Token::RBrace)?;

        lex.success();
        Ok(OuterStatement::ActivityEventListener {
            event_name: name,
            body: statements,
        })
    }
}

fn inner_statements(lex: &mut Lexer) -> LogicParseResult<InnerStatements> {
    lex.start();
    let mut statements = InnerStatements(vec![]);

    loop {
        // skip any newlines
        while let Some(_) = lex.expect_failsafe(Token::Newline) {}

        // check if we've reached the end (a closing brace)
        // todo: change this when there's an Error token detection in LexerWrapper
        if let Ok(_) = lex.expect_peek(Token::RBrace) { break; }

        statements.0.push(inner_statement(lex)?);
    }

    lex.success();
    Ok(statements)
}

fn inner_statement(lex: &mut Lexer) -> LogicParseResult<InnerStatement> {
    lex.start();

    let res = match lex.expect_multiple_choices(
        vec![Token::Identifier, Token::If, Token::Repeat, Token::Forever]
    )? {
        TokenWrapperOwned { token: Token::Identifier, .. } =>
            InnerStatement::VariableAssignment(variable_assignment(lex)?),
        TokenWrapperOwned { token: Token::If, .. } =>
            InnerStatement::IfStatement(if_statement(lex)?),
        TokenWrapperOwned { token: Token::Repeat, .. } =>
            InnerStatement::RepeatStatement(repeat_statement(lex)?),
        TokenWrapperOwned { token: Token::Forever, .. } =>
            InnerStatement::ForeverStatement(forever_statement(lex)?),

        TokenWrapperOwned { token: Token::Break, .. } => InnerStatement::Break,
        TokenWrapperOwned { token: Token::Continue, .. } => InnerStatement::Continue,

        // todo: figure out how to fit expressions in here

        _ => unreachable!()
    };

    lex.success();
    Ok(res)
}

fn variable_assignment(lex: &mut Lexer) -> LogicParseResult<VariableAssignment> {
    lex.start();

    // ident = expr
    let identifier = lex.expect(Token::Identifier)?.slice;
    lex.expect(Token::EQ)?;
    let value = expression(lex)?;

    lex.success();
    Ok(VariableAssignment { identifier, value })
}

fn if_statement(lex: &mut Lexer) -> LogicParseResult<IfStatement> {
    lex.start();

    // if expr { inner_statements }
    lex.expect(Token::If)?;
    let condition = expression(lex)?;

    lex.expect(Token::LBrace)?;
    let body = inner_statements(lex)?;
    lex.expect(Token::RBrace)?;

    // check if there is an else
    let else_body = if let Some(_) = lex.expect_failsafe(Token::Else) {
        lex.expect(Token::LBrace)?;
        let else_body = inner_statements(lex)?;
        lex.expect(Token::RBrace)?;

        Some(else_body)
    } else { None };

    Ok(IfStatement {
        condition,
        body,
        else_body,
    })
}

fn repeat_statement(lex: &mut Lexer) -> LogicParseResult<RepeatStatement> {
    lex.start();

    lex.expect(Token::Repeat)?;
    let condition = expression(lex)?;

    lex.expect(Token::LBrace)?;
    let body = inner_statements(lex)?;
    lex.expect(Token::RBrace)?;

    lex.success();
    Ok(RepeatStatement { condition, body })
}

fn forever_statement(lex: &mut Lexer) -> LogicParseResult<ForeverStatement> {
    lex.start();

    lex.expect(Token::Forever)?;

    lex.expect(Token::LBrace)?;
    let body = inner_statements(lex)?;
    lex.expect(Token::RBrace)?;

    lex.success();
    Ok(ForeverStatement { body })
}

fn expression(lex: &mut Lexer) -> LogicParseResult<Expression> {
    lex.start();

    // try to parse boolean expression
    let bool_expr = error::propagate_non_recoverable_wo_eof!(boolean_expression(lex));
    if let Ok(expr) = bool_expr {
        lex.success();
        return Ok(expr);
    } else {
        lex.restore();
    }

    // try again for atom
    let atom = error::propagate_non_recoverable_wo_eof!(atom(lex));
    if let Ok(expr) = atom {
        lex.success();
        return Ok(expr);
    }

    // fixme: i have no idea what to return here, we can't have two different errors to be returned.
    // i feel like this atom branch is unnecessary since it should already been checked inside the
    // many call stacks of bool_expr
    bool_expr
}

// Generates a match block that converts the given token into a binary operator
macro_rules! token_to_binop {
    ($tok_var:ident, { $($tok:ident => $binop:ident),* }) => {
        match $tok_var {
            $(TokenWrapperOwned { token: Token::$tok, .. } => BinaryOperator::$binop,)*
            _ => unreachable!()
        }
    };
}

// todo: DRY on the rules below

fn boolean_expression(lex: &mut Lexer) -> LogicParseResult<Expression> {
    lex.start();
    let first_branch = comparison_expression(lex)?;
    let mut result = first_branch;

    while let Ok(tok) =
        lex.expect_peek_multiple_choices(vec![Token::Or, Token::And]) {

        // skip the next token because we've peeked it
        let _ = lex.next();

        let operator = token_to_binop!(tok, { Or => Or, And => And });
        let second_branch = comparison_expression(lex)?;

        result = Expression::BinOp {
            first: Box::new(result),
            operator,
            second: Box::new(second_branch)
        };
    }

    lex.success();
    Ok(result)
}

fn comparison_expression(lex: &mut Lexer) -> LogicParseResult<Expression> {
    lex.start();

    // "!" comparison-expression
    if lex.expect_failsafe(Token::Not).is_some() {
        let expr = arithmetic_expression(lex)?;

        lex.success();
        return Ok(Expression::UnaryOp {
            value: Box::new(expr),
            operator: UnaryOperator::Not
        })
    }

    let first_branch = arithmetic_expression(lex)?;
    let mut result = first_branch;

    while let Ok(tok) =
        lex.expect_peek_multiple_choices(vec![
            Token::LT, Token::GT, Token::EQ, Token::LTE, Token::GTE
        ]) {

        // skip the next token because we've peeked it
        let _ = lex.next();

        let operator = token_to_binop!(tok, {
            LT => LT,GT => GT, EQ => EQ, LTE => LTE, GTE => GTE
        });

        let second_branch = comparison_expression(lex)?;

        result = Expression::BinOp {
            first: Box::new(result),
            operator,
            second: Box::new(second_branch)
        };
    }

    lex.success();
    Ok(result)
}

fn arithmetic_expression(lex: &mut Lexer) -> LogicParseResult<Expression> {
    lex.start();

    let first_branch = term(lex)?;
    let mut result = first_branch;

    while let Ok(tok) =
        lex.expect_peek_multiple_choices(vec![Token::Plus, Token::Minus]) {

        // skip the next token because we've peeked it
        let _ = lex.next();

        let operator = token_to_binop!(tok, { Plus => Plus, Minus => Minus });
        let second_branch = term(lex)?;

        result = Expression::BinOp {
            first: Box::new(result),
            operator,
            second: Box::new(second_branch)
        };
    }

    lex.success();
    Ok(result)
}

fn term(lex: &mut Lexer) -> LogicParseResult<Expression> {
    lex.start();

    let first_branch = term(lex)?;
    let mut result = first_branch;

    while let Ok(tok) =
        lex.expect_peek_multiple_choices(vec![Token::Mult, Token::Div]) {

        // skip the next token because we've peeked it
        let _ = lex.next();

        let operator = token_to_binop!(tok, { Mult => Multiply, Div => Divide });
        let second_branch = term(lex)?;

        result = Expression::BinOp {
            first: Box::new(result),
            operator,
            second: Box::new(second_branch)
        };
    }

    lex.success();
    Ok(result)
}

fn factor(lex: &mut Lexer) -> LogicParseResult<Expression> {
    lex.start();

    if lex.expect_failsafe(Token::Plus).is_some() {
        let factor = factor(lex)?;

        lex.success();
        return Ok(Expression::UnaryOp {
            value: Box::new(factor),
            operator: UnaryOperator::Plus
        })
    }

    if lex.expect_failsafe(Token::Minus).is_some() {
        let factor = factor(lex)?;

        lex.success();
        return Ok(Expression::UnaryOp {
            value: Box::new(factor),
            operator: UnaryOperator::Minus
        })
    }

    lex.success();
    power(lex)
}

fn power(lex: &mut Lexer) -> LogicParseResult<Expression> {
    lex.start();
    let primary = primary(lex)?;

    Ok(if lex.expect_failsafe(Token::Pow).is_some() {
        let power = power(lex)?;

        lex.success();
        Expression::BinOp {
            first: Box::new(primary),
            operator: BinaryOperator::Power,
            second: Box::new(power)
        }
    } else {
        lex.success();
        primary
    })
}

fn primary(lex: &mut Lexer) -> LogicParseResult<Expression> {
    lex.start();

    let atom = atom(lex)?;
    let mut result = atom;

    while let Ok(tok) =
        lex.expect_peek_multiple_choices(vec![Token::DOT, Token::LBracket, Token::LParen]) {

        // skip the next token because we've peeked it
        let _ = lex.next();

        match tok {
            TokenWrapperOwned { token: Token::DOT, .. } => {
                let ident = lex.expect(Token::Identifier)?;

                result = Expression::PrimaryExpression(PrimaryExpression::VariableAccess {
                    from: Some(Box::new(result)),
                    name: ident.slice
                })
            }
            TokenWrapperOwned { token: Token::LBracket, .. } => {
                let index_expr = expression(lex)?;

                result = Expression::PrimaryExpression(PrimaryExpression::Index {
                    from: Box::new(result),
                    index: Box::new(index_expr)
                })
            }
            TokenWrapperOwned { token: Token::LParen, .. } => {
                let arguments = arguments(lex)?;

                result = Expression::PrimaryExpression(PrimaryExpression::Call {
                    from: Box::new(result),
                    arguments
                })
            }
            _ => unreachable!()
        }
    }

    lex.success();
    Ok(result)
}

fn arguments(lex: &mut Lexer) -> LogicParseResult<Arguments> {
    lex.start();

    if lex.expect_failsafe(Token::RParen).is_some() {
        return Ok(Arguments(vec![]));
    }

    let mut arguments = vec![];

    let first = expression(lex)?;
    arguments.push(first);

    while lex.expect_failsafe(Token::Comma).is_some() {
        if lex.expect_peek(Token::RParen).is_ok() { break; }
        arguments.push(expression(lex)?);
    }

    Ok(Arguments(arguments))
}

fn atom(lex: &mut Lexer) -> LogicParseResult<Expression> {
    lex.start();

    match lex.expect_multiple_choices(
        vec![Token::Identifier, Token::String, Token::Number, Token::False, Token::True]
    )? {
        TokenWrapperOwned { token: Token::Identifier, slice, .. } => {
            lex.success();
            Ok(Expression::PrimaryExpression(PrimaryExpression::VariableAccess {
                from: None,
                name: slice
            }))
        }
        TokenWrapperOwned { token: Token::String, slice, .. } => {
            lex.success();
            Ok(Expression::Literal(Literal::String(slice[1..slice.len()].to_string())))
        }
        TokenWrapperOwned { token: Token::Number, slice, pos } => {
            let num = slice.parse::<f64>()
                .map_err(|e| ParseError::LexerError {
                    err_token: Token::Number,
                    pos: pos.clone(),
                    slice: slice.to_string(),
                })?;

            lex.success();
            Ok(Expression::Literal(Literal::Number(num)))
        }
        TokenWrapperOwned { token: Token::False, .. } => {
            lex.success();
            Ok(Expression::Literal(Literal::Boolean(false)))
        }
        TokenWrapperOwned { token: Token::True, .. } => {
            lex.success();
            Ok(Expression::Literal(Literal::Boolean(true)))
        }
        _ => unreachable!()
    }
}