use crate::{defs::{Definitions, FunctionSignature, FunctionDefinition, Dispatch, DispatchKind, Type, BlockArgument}, resolver::models::Block};

use super::parse_defs;

// todo: a macro to generate `Definitions` construction would be nice.
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
            global_functions: vec![
                (FunctionSignature {
                    this: None,
                    parameters: vec![],
                    name: String::from("function"),
                    return_type: None,
                },
                FunctionDefinition {
                    statements: vec![
                        Dispatch {
                            kind: DispatchKind::RawBlock,
                            identifier: String::from("myBlock"),
                            arguments: vec![],
                            this: None
                        }
                    ],
                    return_statement: None,
                })
            ],
            methods: vec![]
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
            global_functions: vec![
                (FunctionSignature {
                    this: None,
                    parameters: vec![Type::String],
                    name: String::from("function"),
                    return_type: None,
                },
                FunctionDefinition {
                    statements: vec![
                        Dispatch {
                            kind: DispatchKind::RawBlock,
                            identifier: String::from("myBlock"),
                            arguments: vec![BlockArgument::Argument { index: 0 }],
                            this: None
                        }
                    ],
                    return_statement: None,
                })
            ],
            methods: vec![]
        })
    );
}

#[test]
fn function_parameters() {
    let code = r#"
function(s, b, d) {
    #myBlock(@0, @1, @2);
}
"#;
    let defs = parse_defs(code);

    assert_eq!(
        defs,
        Ok(Definitions {
            global_functions: vec![
                (FunctionSignature {
                    this: None,
                    parameters: vec![Type::String, Type::Boolean, Type::Number],
                    name: String::from("function"),
                    return_type: None,
                },
                FunctionDefinition {
                    statements: vec![
                        Dispatch {
                            kind: DispatchKind::RawBlock,
                            identifier: String::from("myBlock"),
                            arguments: vec![
                                BlockArgument::Argument { index: 0 },
                                BlockArgument::Argument { index: 1 },
                                BlockArgument::Argument { index: 2 },
                            ],
                            this: None
                        }
                    ],
                    return_statement: None,
                })
            ],
            methods: vec![]
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
            global_functions: vec![
                (FunctionSignature {
                    this: None,
                    parameters: vec![Type::String, Type::Boolean, Type::Number],
                    name: String::from("function"),
                    return_type: None,
                },
                FunctionDefinition {
                    statements: vec![
                        Dispatch {
                            kind: DispatchKind::RawBlock,
                            identifier: String::from("myBlock"),
                            arguments: vec![
                                BlockArgument::Argument { index: 0 },
                                BlockArgument::Argument { index: 1 },
                                BlockArgument::Argument { index: 2 },
                            ],
                            this: None
                        },
                        Dispatch {
                            kind: DispatchKind::RawBlock,
                            identifier: String::from("myOtherBlock"),
                            arguments: vec![
                                BlockArgument::Argument { index: 1 },
                                BlockArgument::Argument { index: 0 },
                                BlockArgument::Argument { index: 2 },
                                BlockArgument::Argument { index: 2 },
                            ],
                            this: None
                        },
                        Dispatch {
                            kind: DispatchKind::RawBlock,
                            identifier: String::from("loremIpsum"),
                            arguments: vec![
                                BlockArgument::Argument { index: 2 },
                                BlockArgument::Argument { index: 1 },
                            ],
                            this: None
                        },
                    ],
                    return_statement: None,
                })
            ],
            methods: vec![]
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
            global_functions: vec![
                (FunctionSignature {
                    this: None,
                    parameters: vec![Type::String],
                    name: String::from("function"),
                    return_type: Some(Type::String),
                },
                FunctionDefinition {
                    statements: vec![],
                    return_statement: Some(Dispatch {
                        kind: DispatchKind::RawBlock,
                        identifier: String::from("opcode"),
                        arguments: vec![BlockArgument::Argument { index: 0 }],
                        this: None,
                    }),
                })
            ],
            methods: vec![]
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
            global_functions: vec![
                (FunctionSignature {
                    this: None,
                    parameters: vec![Type::String],
                    name: String::from("function"),
                    return_type: Some(Type::Number),
                },
                FunctionDefinition {
                    statements: vec![
                        Dispatch {
                            kind: DispatchKind::RawBlock,
                            identifier: String::from("lorem"),
                            arguments: vec![],
                            this: None,
                        },
                        Dispatch {
                            kind: DispatchKind::RawBlock,
                            identifier: String::from("ipsum"),
                            arguments: vec![BlockArgument::Argument { index: 0 }],
                            this: None,
                        },
                    ],
                    return_statement: Some(Dispatch {
                        kind: DispatchKind::RawBlock,
                        identifier: String::from("opcode"),
                        arguments: vec![BlockArgument::Argument { index: 0 }],
                        this: None,
                    }),
                })
            ],
            methods: vec![]
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
            methods: vec![(
                Type::Number, vec![
                    (FunctionSignature {
                        this: Some(Type::Number),
                        parameters: vec![],
                        name: String::from("function"),
                        return_type: None,
                    },
                    FunctionDefinition {
                        statements: vec![
                            Dispatch {
                                kind: DispatchKind::RawBlock,
                                identifier: String::from("myBlock"),
                                arguments: vec![BlockArgument::This],
                                this: None
                            }
                        ],
                        return_statement: None,
                    })
                ])
            ]
        })
    );
}