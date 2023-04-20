use crate::defs::{
    Definitions, FunctionDeclaration, Type, FunctionBody, Statement, Expression,
    Literal, StaticVariable, BindingDeclaration, BindingBody
};

use super::parse_defs;
use super::test_macros::{collection, typ, stmt, arg, expr, global_functions, methods, bindings, binding_func, binding_method, binding_body};

// todo: write tests for methods, raw block as argument, function as argument,
//       literals, calling functions from literals, returning literals
//       literals as argument,

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
            methods: collection![],
            bindings: vec![]
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
            methods: collection![],
            bindings: vec![]
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
            methods: collection![],
            bindings: vec![]
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
            methods: collection![],
            bindings: vec![]
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
            methods: collection![],
            bindings: vec![]
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
            methods: collection![],
            bindings: vec![]
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
            },
            bindings: vec![]
        })
    );
}

// ===== Binding-related tests

#[test]
fn binding_function_block() {
    let code = r#"toString = #toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString: None => binding_body!(#toString),
                ],
                methods {}
            }
        })
    );
}

#[test]
fn binding_function_block_return_value() {
    let code = r#"toString: s = #toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString: Some(typ!(s)) => binding_body!(#toString),
                ],
                methods {}
            }
        })
    );
}

#[test]
fn binding_function_block_param() {
    let code = r#"toString(s) = #toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString(typ!(s)): None => binding_body!(#toString),
                ],
                methods {}
            }
        })
    );
}

#[test]
fn binding_function_block_params() {
    let code = r#"toString(s, b, d) = #toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString(typ!(s), typ!(b), typ!(d)): None => binding_body!(#toString),
                ],
                methods {}
            }
        })
    );
}

#[test]
fn binding_function_block_args() {
    let code = r#"toString(s, b, d) = #toString(@1, @0, @2);"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString(typ!(s), typ!(b), typ!(d)): None
                        => binding_body!(#toString(arg!(1), arg!(0), arg!(2))),
                ],
                methods {}
            }
        })
    );
}

#[test]
fn binding_function_function() {
    let code = r#"toString = toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString: None => binding_body!(toString),
                ],
                methods {}
            }
        })
    );
}

#[test]
fn binding_function_function_return_value() {
    let code = r#"toString: s = toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString: Some(typ!(s)) => binding_body!(toString),
                ],
                methods {}
            }
        })
    );
}

#[test]
fn binding_function_function_param() {
    let code = r#"toString(s) = toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString(typ!(s)): None => binding_body!(toString),
                ],
                methods {}
            }
        })
    );
}

#[test]
fn binding_function_function_params() {
    let code = r#"toString(s, b, d) = toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString(typ!(s), typ!(b), typ!(d)): None => binding_body!(toString),
                ],
                methods {}
            }
        })
    );
}

#[test]
fn binding_function_function_args() {
    let code = r#"toString(s, b, d) = toString(@1, @0, @2);"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString(typ!(s), typ!(b), typ!(d)): None
                        => binding_body!(toString(arg!(1), arg!(0), arg!(2))),
                ],
                methods {}
            }
        })
    );
}

#[test]
fn binding_function_complex_expression() {
    let code = r#"toString(s, b, d) = toString(#doThings(#lorem(@0), ipsum(@2)), lorem(@0, @2));"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    toString(typ!(s), typ!(b), typ!(d)): None
                        => binding_body!(
                            toString(
                                expr!(#doThings(expr!(#lorem(arg!(0))), expr!(ipsum(arg!(2))))),
                                expr!(lorem(arg!(0), arg!(2))),
                            )
                        ),
                ],
                methods {}
            }
        })
    );
}

// ================================

#[test]
fn binding_method_block() {
    let code = r#"d.toString = #toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString: None => binding_body!(#toString),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_block_return_value() {
    let code = r#"d.toString: s = #toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString: Some(typ!(s)) => binding_body!(#toString),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_block_param() {
    let code = r#"d.toString(s) = #toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString(typ!(s)): None => binding_body!(#toString),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_block_params() {
    let code = r#"d.toString(s, b, d) = #toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString(typ!(s), typ!(b), typ!(d)): None => binding_body!(#toString),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_block_args() {
    let code = r#"d.toString(s, b, d) = #toString(@1, @0, @2);"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString(typ!(s), typ!(b), typ!(d)): None
                            => binding_body!(#toString(arg!(1), arg!(0), arg!(2))),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_function() {
    let code = r#"d.toString = toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString: None => binding_body!(toString),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_function_return_value() {
    let code = r#"d.toString: s = toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString: Some(typ!(s)) => binding_body!(toString),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_function_param() {
    let code = r#"d.toString(s) = toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString(typ!(s)): None => binding_body!(toString),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_function_params() {
    let code = r#"d.toString(s, b, d) = toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString(typ!(s), typ!(b), typ!(d)): None => binding_body!(toString),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_function_args() {
    let code = r#"d.toString(s, b, d) = toString(@1, @0, @2);"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString(typ!(s), typ!(b), typ!(d)): None
                            => binding_body!(toString(arg!(1), arg!(0), arg!(2))),
                    ]
                }
            }
        })
    );
}


#[test]
fn binding_method_method() {
    let code = r#"d.toString(s, b, d) = @@.toString;"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString(typ!(s), typ!(b), typ!(d)): None
                            => binding_body!(method expr!(this) => toString),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_method_args() {
    let code = r#"d.toString(s, b, d) = @@.toString(@0, @2, @1);"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString(typ!(s), typ!(b), typ!(d)): None
                            => binding_body!(method expr!(this) => toString(arg!(0), arg!(2), arg!(1))),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_method_complex() {
    let code = r#"d.toString(s) = #other(@1).toString(@@, @1);"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString(typ!(s)): None
                            => binding_body!(
                                method expr!(#other(arg!(1))) => toString(expr!(this), arg!(1))
                            ),
                    ]
                }
            }
        })
    );
}

#[test]
fn binding_method_complex_expression() {
    let code = r#"d.toString(s, b, d) = toString(#doThings(#lorem(@0), ipsum(@2)), lorem(@0, @2));"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [],
                methods {
                    typ!(d) => [
                        toString(typ!(s), typ!(b), typ!(d)): None
                            => binding_body!(
                                toString(
                                    expr!(#doThings(expr!(#lorem(arg!(0))), expr!(ipsum(arg!(2))))),
                                    expr!(lorem(arg!(0), arg!(2))),
                                )
                            ),
                    ],
                }
            }
        })
    );
}

#[test]
fn binding_multiple_complex_expression() {
    let code = r#"    
otherThings(b): s = compose(this(function(@0), @0));
hmm(b, s, d, b): s = dolor(sit(@0, amet(@1, @2), @3), @0);

b.doThings(s, d): s = #block(function(@0, @1), @@);
s.whatThe() = ok(lorem(@@, @@, @@));
s.otherThings(s, d): b = interesting;

d.toString(s, b, d) = toString(#doThings(#lorem(@0), ipsum(@2)), lorem(@@, @@));
"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: global_functions![],
            methods: methods! {},
            bindings: bindings! {
                functions [
                    otherThings(typ!(b)): Some(typ!(s)) =>
                        binding_body!(
                            compose(expr!(this(expr!(function(arg!(0))), arg!(0))))
                        ),

                    hmm(typ!(b), typ!(s), typ!(d), typ!(b)): Some(typ!(s)) =>
                        binding_body!(
                            dolor(expr!(sit(arg!(0), expr!(amet(arg!(1), arg!(2))), arg!(3))), arg!(0))
                        ),
                ],
                methods {
                    typ!(b) => [
                        doThings(typ!(s), typ!(d)): Some(typ!(s))
                            => binding_body!(#block(expr!(function(arg!(0), arg!(1))), expr!(this))),
                    ],
                    typ!(s) => [
                        whatThe(): None
                            => binding_body!(ok(expr!(lorem(expr!(this), expr!(this), expr!(this))))),
                        otherThings(typ!(s), typ!(d)): Some(typ!(b))
                            => binding_body!(interesting),
                    ],
                    typ!(d) => [
                        toString(typ!(s), typ!(b), typ!(d)): None
                            => binding_body!(
                                toString(
                                    expr!(#doThings(expr!(#lorem(arg!(0))), expr!(ipsum(arg!(2))))),
                                    expr!(lorem(expr!(this), expr!(this))),
                                )
                            ),
                    ],
                }
            }
        })
    );
}