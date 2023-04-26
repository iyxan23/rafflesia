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
        collection![
            $(
                FunctionDeclaration {
                    this: None,
                    parameters: vec![$($param_typ,)*],
                    name: String::from(stringify!($ident)),
                    return_type: $ret_typ,
                } => FunctionBody {
                    statements: vec![$($stmt,)*],
                },
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
            $($typ => collection![
                $(
                    FunctionDeclaration {
                        this: Some($typ),
                        parameters: vec![$($param_typ,)*],
                        name: String::from(stringify!($ident)),
                        return_type: $ret_typ,
                    } => FunctionBody {
                        statements: vec![$($stmt,)*],
                    },
                )*
            ]),*
        }
    }
}

/*
bindings!{
    functions [
        ident(typ!(s)): Some(typ!(s)) => binding_body!(#),
    ],
    methods {
        typ!(s) => [

        ]
    }
}
*/
macro_rules! bindings {
    { $(functions [
        $($func_ident:ident $(($($func_param_typ:expr),* $(,)*))?: $func_ret_typ:expr => $func_body:expr,)*
    ],)? $(methods {
        $($method_typ:expr => [
            $($method_ident:ident $(($($method_param_typ:expr),* $(,)*))?: $method_ret_typ:expr => $method_body:expr,)*
        ]),* $(,)*
    })? } => {
        collection![
            // functions
            $($(
                binding_func_dec!($func_ident $(($($func_param_typ,)*))*: $func_ret_typ) => $func_body,
            )*)?

            // methods
            $($($(
                binding_method_dec!($method_typ =>
                    $method_ident $(($($method_param_typ,)*))*: $method_ret_typ
                ) => $method_body,
            )*)*)?
        ]
    }
}

macro_rules! binding_func_dec {
    ($func_ident:ident($($func_param_typ:expr),* $(,)*): $func_ret_typ:expr) => {
        BindingDeclaration {
            this: None,
            parameters: Some(vec![$($func_param_typ,)*]),
            name: String::from(stringify!($func_ident)),
            return_type: $func_ret_typ,
        }
    };
    ($func_ident:ident: $func_ret_typ:expr) => {
        BindingDeclaration {
            this: None,
            parameters: None,
            name: String::from(stringify!($func_ident)),
            return_type: $func_ret_typ,
        }
    };
}

macro_rules! binding_method_dec {
    ($this:expr => $method_ident:ident ($($method_param_typ:expr),* $(,)*): $method_ret_typ:expr) => {
        BindingDeclaration {
            this: Some($this),
            parameters: Some(vec![$($method_param_typ,)*]),
            name: String::from(stringify!($method_ident)),
            return_type: $method_ret_typ,
        }
    };
    ($this:expr => $method_ident:ident: $method_ret_typ:expr) => {
        BindingDeclaration {
            this: Some($this),
            parameters: None,
            name: String::from(stringify!($method_ident)),
            return_type: $method_ret_typ,
        }
    };
}

macro_rules! binding_body {
    // block
    //   no arg
    (#$name:ident) => {
        BindingBody::Block {
            opcode: stringify!($name).to_string(),
            arguments: None
        }
    };
    //   with arg
    (#$name:ident($($args:expr),* $(,)*)) => {
        BindingBody::Block {
            opcode: stringify!($name).to_string(),
            arguments: Some(vec![
                $($args,)*
            ])
        }
    };

    // function
    //    no arg
    ($name:ident) => {
        BindingBody::FunctionCall {
            name: stringify!($name).to_string(),
            arguments: None
        }
    };
    //    with arg
    ($name:ident($($args:expr),* $(,)*)) => {
        BindingBody::FunctionCall {
            name: stringify!($name).to_string(),
            arguments: Some(vec![
                $($args,)*
            ]) 
        }
    };

    // method
    //    no args
    (method $this:expr => $name:ident) => {
        BindingBody::MethodCall {
            name: stringify!($name).to_string(),
            this: $this,
            arguments: None
        }
    };
    //    with args
    (method $this:expr => $name:ident ( $($args:expr),* $(,)* )) => {
        BindingBody::MethodCall {
            name: stringify!($name).to_string(),
            this: $this,
            arguments: Some(vec![
                $($args,)*
            ]),
        }
    };
}

macro_rules! expr {
    (this) => { Expression::StaticVariable(StaticVariable::This) };

    // literals
    (b: $lit:expr) => { Expression::StaticVariable(StaticVariable::Literal(Literal::Boolean($lit))) };
    (d: $lit:expr) => { Expression::StaticVariable(StaticVariable::Literal(Literal::Number($lit as u64))) };
    (s: $lit:expr) => { Expression::StaticVariable(StaticVariable::Literal(Literal::String($lit.to_string()))) };

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
    (method $this:expr => $name:ident ( $($args:expr),* $(,)* )) => {
        Expression::MethodCall {
            name: stringify!($name).to_string(),
            arguments: vec![
                $($args,)*
            ],
            this: Box::new($this),
        }
    };
}

macro_rules! arg { ($lit:literal) => { Expression::StaticVariable(StaticVariable::Argument($lit)) }; }

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

pub(super) use {collection, typ, stmt, arg, expr, global_functions, methods, bindings, binding_func_dec, binding_method_dec, binding_body};