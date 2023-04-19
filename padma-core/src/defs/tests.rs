use crate::{defs::{Definitions, FunctionDeclaration, Type, FunctionBody, Statement, Expression, Literal}};

use super::parse_defs;

// todo: write tests for methods, raw block as argument, function as argument,
//       literals, calling functions from literals, returning literals
//       literals as argument,

macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        core::convert::From::from([$($v,)*])
    }};
}

/*
// a bit complicated, but this is how the macro should work:

global_functions![
    function(typ!(s)): Some(typ!(s)) => {
        stmt!(#block(arg!(0), expr!(b: true), expr!(d: 10), expr!(s: "hello world"))));
        stmt!(func());
    },
]
*/
macro_rules! global_functions {
    [$($ident:ident($($param_typ:expr),* $(,)*): $ret_typ:expr => {
        $($stmt:expr;)*
    },)*] => {
        vec![
            $(
                (FunctionDeclaration {
                    this: None,
                    parameters: vec![$($param_typ,)*],
                    name: String::from(stringify!($ident)),
                    return_type: $ret_typ,
                },
                FunctionBody {
                    statements: vec![$($stmt,)*],
                }),
            )*
        ]
    }
}

/*
// a bit complicated, but this is how the macro should work:

methods! {
    typ!(s) => [
        function(typ!(s)): Some(typ!(s)) => {
            stmt!(#block(arg!(0), expr!(b: true), expr!(d: 10), expr!(s: "hello world"))));
            stmt!(func());
        },
    ],
    typ!(d) => [ .. ]
}
*/
macro_rules! methods {
    {
        $($typ:expr => 
            [$($ident:ident($($param_typ:expr),* $(,)*): $ret_typ:expr => {
                $($stmt:expr;)*
            },)*]),* $(,)*
    } => {
        collection! {
            $($typ => vec![
                $(
                    (FunctionDeclaration {
                        this: Some($typ),
                        parameters: vec![$($param_typ,)*],
                        name: String::from(stringify!($ident)),
                        return_type: $ret_typ,
                    },
                    FunctionBody {
                        statements: vec![$($stmt,)*],
                    }),
                )*
            ]),*
        }
    }
}

macro_rules! expr {
    (this) => { Expression::This };

    // literals
    (b: $lit:expr) => { Expression::Literal(Literal::Boolean($lit)) };
    (d: $lit:expr) => { Expression::Literal(Literal::Number($lit as u64)) };
    (s: $lit:expr) => { Expression::Literal(Literal::String($lit.to_string())) };

    // block
    (#$name:ident($($args:expr),* $(,)*)) => {
        Expression::Block {
            opcode: stringify!($name).to_string(),
            arguments: vec![
                $($args,)*
            ]
        }
    };
    // function
    ($name:ident($($args:expr),* $(,)*)) => {
        Expression::FunctionCall {
            name: stringify!($name).to_string(),
            arguments: vec![
                $($args,)*
            ] 
        }
    };
    // method
    (method $this:expr => ( $($args:expr),* $(,)* )) => {
        Expression::MethodCall {
            name: stringify!($name).to_string(),
            arguments: vec![
                $($args,)*
            ],
            this: Box::new($this),
        }
    };
}
macro_rules! arg { ($lit:literal) => { Expression::Argument($lit) }; }
macro_rules! typ {
    (b) => { Type::Boolean };
    (d) => { Type::Number };
    (s) => { Type::String };
}

macro_rules! stmt {
    // block
    (#$name:ident($($args:expr),* $(,)*)) => {
        Statement::Block {
            opcode: stringify!($name).to_string(),
            arguments: vec![
                $($args,)*
            ]
        }
    };
    // function
    ($name:ident($($args:expr),* $(,)*)) => {
        Statement::FunctionCall {
            name: stringify!($name).to_string(),
            arguments: vec![
                $($args,)*
            ] 
        }
    };
    // method
    (method $this:expr => ( $($args:expr),* $(,)* )) => {
        Statement::MethodCall {
            name: stringify!($name).to_string(),
            arguments: vec![
                $($args,)*
            ],
            this: Box::new($this),
        }
    };
    // return
    (< $expr:expr) => {
        Statement::Return {
            value: $expr,
        }
    }
}

#[test]
fn function() {
    let code = r#"
function() {
    #myBlock();
}
"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![
                function(): None => {
                    stmt!(#myBlock());
                },
            ],
            methods: collection![]
        })
    );
}

#[test]
fn function_parameter() {
    let code = r#"
function(s) {
    #myBlock(@0);
}
"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![
                function(typ!(s)): None => {
                    stmt!(#myBlock(arg!(0)));
                },
            ],
            methods: collection![]
        })
    );
}

#[test]
fn function_parameters() {
    let code = r#"
function(s, b, d) {
    #myBlock(@0, @1, @2, "literal", 25, false);
}
"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![
                function(typ!(s), typ!(b), typ!(d)): None => {
                    stmt!(#myBlock(
                        arg!(0), arg!(1), arg!(2),
                        expr!(s: "literal"), expr!(d: 25), expr!(b: false)
                    ));
                },
            ],
            methods: collection![]
        })
    );
}

#[test]
fn function_multiple_statements() {
    let code = r#"
function(s, b, d) {
    #myBlock(@0, @1, @2);
    #myOtherBlock(@1, @0, @2, @2);
    #loremIpsum(@2, @1);
}
"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![
                function(typ!(s), typ!(b), typ!(d)): None => {
                    stmt!(#myBlock(arg!(0), arg!(1), arg!(2)));
                    stmt!(#myOtherBlock(arg!(1), arg!(0), arg!(2), arg!(2)));
                    stmt!(#loremIpsum(arg!(2), arg!(1)));
                },
            ],
            methods: collection![]
        })
    );
}

#[test]
fn function_returning_value() {
    let code = r#"
function(s): s {
    < #opcode(@0);
}
"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![
                function(typ!(s)): Some(typ!(s)) => {
                    stmt!(< expr!(#opcode(arg!(0))));
                },
            ],
            methods: collection![]
        })
    );
}

#[test]
fn function_mutliple_statements_returning_value() {
    let code = r#"
function(s): d {
    #lorem();
    #ipsum(@0);
    < #opcode(@0);
}
"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![
                function(typ!(s)): Some(typ!(d)) => {
                    stmt!(#lorem());
                    stmt!(#ipsum(arg!(0)));
                    stmt!(< expr!(#opcode(arg!(0))));
                },
            ],
            methods: collection![]
        })
    );
}

#[test]
fn method() {
    let code = r#"
d.function() {
    #myBlock(@@);
}
"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: vec![],
            methods: methods! {
                typ!(d) => [
                    function(): None => {
                        stmt!(#myBlock(expr!(this)));
                    },
                ],
            }
        })
    );
}