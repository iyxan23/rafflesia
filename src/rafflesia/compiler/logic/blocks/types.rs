use lazy_static::lazy_static;
use thiserror::Error;
use swrs::api::block::{Argument, ArgumentBlockReturnType, ArgValue, Block, BlockCategory, BlockContent, BlockType};
use std::collections::HashMap;

// A type
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Type {
    Void,
    Primitive(PrimitiveType),
    Complex(ComplexType),
    View(ViewType),
    Component(ComponentType),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PrimitiveType {
    Number, String, Boolean,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ComplexType {
    List { inner_type: PrimitiveType },
    Map // todo: map
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ViewType {
    LinearLayout, ScrollView, Button, TextView, EditText, ImageView, WebView, ProgressBar, ListView,
    Spinner, CheckBox, Switch, SeekBar, CalendarView, Fab, AdView, MapView
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ComponentType {
    // todo: components
}

impl Type {
    pub fn from_arg_block(block_type: BlockType) -> Option<Self> {
        Some(Type::Primitive(match block_type {
            BlockType::Argument(ArgumentBlockReturnType::Number) => PrimitiveType::Number,
            BlockType::Argument(ArgumentBlockReturnType::String) => PrimitiveType::String,
            BlockType::Argument(ArgumentBlockReturnType::Boolean) => PrimitiveType::Boolean,
            // todo: Component, and View
            //       requires indexing of enum variants, we only have the typename string here
            _ => return None,
        }))
    }
}

/// Value of [`Type`]
#[derive(Debug, Clone)]
pub enum TypeValue {
    Number(ArgValue<super::Number>),
    String(ArgValue<String>),
    Boolean(ArgValue<super::Boolean>),

    List { inner_type: PrimitiveType, var_name: ArgValue<String> },
    Map { var_name: ArgValue<String> }, // todo: map
    View { view_type: ViewType, id: ArgValue<String> },
    Component { component_type: ComponentType, id: ArgValue<String> }
}

impl TypeValue {
    pub fn as_type(&self) -> Type {
        match self {
            TypeValue::Number(_) => Type::Primitive(PrimitiveType::Number),
            TypeValue::String(_) => Type::Primitive(PrimitiveType::String),
            TypeValue::Boolean(_) => Type::Primitive(PrimitiveType::Boolean),
            TypeValue::List { inner_type, .. } => Type::Complex(ComplexType::List { inner_type: *inner_type }),
            TypeValue::Map { .. } => Type::Complex(ComplexType::Map),
            TypeValue::View { view_type, .. } => Type::View(*view_type),
            TypeValue::Component { component_type, .. } => Type::Component(*component_type),
        }
    }
}

macro_rules! type_value_func {
    ($name:ident $fn_name:ident -> $ret:ty { $typ_val:ident $decnstr:tt => $ret_v:expr }) => {
        fn $fn_name(self) -> $ret {
            match self {
                TypeValue::$typ_val $decnstr => $ret_v,
                other => panic!("expected {}, got {:?}", stringify!($name), other)
            }
        }
    };
}

impl TypeValue {
    // these functions should not be called outside of the lazy_static block below.
    // the arguments provided there are already checked so there is no need to return Result.
    //
    // because i kinda dont want to do error handling on the generate functions, i want them to be
    // as clean as possible.

    type_value_func!(number  to_num  -> ArgValue<super::Number>  { Number  (arg) => arg });
    type_value_func!(string  to_str  -> ArgValue<String>         { String  (arg) => arg });
    type_value_func!(boolean to_bool -> ArgValue<super::Boolean> { Boolean (arg) => arg });

    type_value_func!(list to_list -> (PrimitiveType, ArgValue<String>) {
        List { inner_type, var_name } => (inner_type, var_name)
    });

    type_value_func!(map to_map -> ArgValue<String> {
        Map { var_name } => var_name
    });

    type_value_func!(view to_view -> (ViewType, ArgValue<String>) {
        View { view_type, id } => (view_type, id)
    });

    type_value_func!(component to_component -> (ComponentType, ArgValue<String>) {
        Component { component_type, id } => (component_type, id)
    });
}

#[derive(Debug, Clone)]
pub struct Function {
    // if function is attached to an object; e.g. var.method()
    // the var value will then be the first argument on the generate function given
    pub method: bool,

    pub name: String,
    pub return_type: Type,
    pub argument_types: Vec<Type>,
    generate: fn(Vec<TypeValue>) -> Vec<Block>,
}

impl Function {
    /// generates blocks from this function with the specified arguments
    pub fn generate(&self, args: Vec<TypeValue>) -> Result<Vec<Block>, GenerateError> {
        let args_types = args.iter()
            .map(|arg| arg.as_type())
            .collect::<Vec<Type>>();

        // kind of a safety wrapper that checks if the args are valid
        if args_types.len() != self.argument_types.len() {
            return Err(GenerateError::InvalidArgumentCount {
                expected: self.argument_types.clone(),
                got: args_types
            })
        }

        // check for every types
        for index in 0..args_types.len() {
            // unwrap: vector lengths are already checked to be the same above
            if args_types.get(index).unwrap() != self.argument_types.get(index).unwrap() {
                return Err(GenerateError::InvalidArgumentType {
                    expected: self.argument_types.clone(),
                    got: args_types.clone(),
                    index
                })
            }
        }

        // then call it
        Ok((self.generate)(args))
    }
}

#[derive(Debug, Error, Clone)]
pub enum GenerateError {
    #[error("invalid arguments given: expected {} total arguments, got {}", .expected.len(), .got.len())]
    InvalidArgumentCount {
        expected: Vec<Type>,
        got: Vec<Type>
    },

    #[error(
        "invalid {}th argument type given: expected {:?}, got {:?}",
        .index, .expected.get(*.index).unwrap(), .got.get(*.index).unwrap()
    )]
    InvalidArgumentType {
        expected: Vec<Type>,
        got: Vec<Type>,
        index: usize
    },
}

pub fn resolve_function(name: &str) -> Option<&Function> {
    FUNCTIONS.get(name)
}

pub fn resolve_method(t: Type, name: &str) -> Option<&Function> {
    TYPES.get(&t)?.get(name)
}

// to make creating hashmaps easier
macro_rules! hashmap {
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = HashMap::new();
            $(m.insert($key, $value);)+
            m
        }
     };
}

macro_rules! hashmap_str {
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = HashMap::new();
            $(m.insert($key.to_string(), $value);)+
            m
        }
     };
}

lazy_static! {
    static ref FUNCTIONS: HashMap<String, Function> = {
        hashmap_str! {
            "toast" => new_func("toast", Type::Void, vec![], |mut args| {
                let text = args.remove(0).to_str();

                vec![Block::new(
                    BlockCategory::ComponentFunc,
                    "doToast".to_string(),
                    BlockContent::builder()
                        .text("Toast")
                        .arg(Argument::String { name: None, value: text })
                        .build(),
                    BlockType::Regular
                )]
            })
        }
    };

    static ref TYPES: HashMap<Type, HashMap<String, Function>> = {
        hashmap! {
            Type::Primitive(PrimitiveType::Number) => hashmap_str! {
                "toString" => new_method("toString", Type::Primitive(PrimitiveType::String), vec![], |mut args| {
                    let num = args.remove(0).to_num();

                    vec![Block::new(
                        BlockCategory::Operator,
                        "toString".to_string(),
                        BlockContent::builder()
                            .text("toString")
                            .arg(Argument::Number { name: None, value: num })
                            .text("without")
                            .text("decimal")
                            .build(),
                        BlockType::Argument(ArgumentBlockReturnType::String)
                    )]
                })
            }
        }
    };
}

// shortcut of creating a new func
fn new_func(
    name: &str,
    return_type: Type,
    argument_types: Vec<Type>,
    generate_fn: fn(Vec<TypeValue>) -> Vec<Block>
) -> Function {
    Function {
        method: false,
        name: name.to_string(),
        return_type,
        argument_types,
        generate: generate_fn
    }
}

// shortcut of creating a new method
fn new_method(
    name: &str,
    return_type: Type,
    argument_types: Vec<Type>,
    generate_fn: fn(Vec<TypeValue>) -> Vec<Block>
) -> Function {
    Function {
        method: true,
        name: name.to_string(),
        return_type,
        argument_types,
        generate: generate_fn
    }
}