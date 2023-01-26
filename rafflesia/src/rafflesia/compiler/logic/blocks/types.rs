use lazy_static::lazy_static;
use thiserror::Error;
use swrs::api::block::{Argument, ArgumentBlockReturnType, ArgValue, Block, BlockCategory, BlockContent, BlockType, ListItem};
use std::collections::HashMap;
use std::str::FromStr;
use swrs::api::view::{View, ViewType as SWRSViewType};
use swrs::LinkedHashMap;
use lazy_static::__Deref;
use swrs::parser::logic::variable::{Variable as SWRSVariable, VariableType as SWRSVariableType};
use swrs::parser::logic::list_variable::ListVariable as SWRSListVariable;

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
    List { inner_type: PrimitiveType }, // todo: restrict to only Number and String
    Map // todo: map
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ViewType {
    LinearLayout, ScrollView, Button, TextView, EditText, ImageView, WebView, ProgressBar, ListView,
    Spinner, CheckBox, Switch, SeekBar, CalendarView, Fab, AdView, MapView
}

impl FromStr for ViewType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "LinearLayout" => ViewType::LinearLayout, "ScrollView" => ViewType::ScrollView,
            "Button" => ViewType::Button, "TextView" => ViewType::TextView,
            "EditText" => ViewType::EditText, "ImageView" => ViewType::ImageView,
            "WebView" => ViewType::WebView, "ProgressBar" => ViewType::ProgressBar,
            "ListView" => ViewType::ListView, "Spinner" => ViewType::Spinner,
            "CheckBox" => ViewType::CheckBox, "Switch" => ViewType::Switch,
            "SeekBar" => ViewType::SeekBar, "CalendarView" => ViewType::CalendarView,
            "Fab" => ViewType::Fab, // "Fab" might not be valid as a typename
            "AdView" => ViewType::AdView, "MapView" => ViewType::MapView,
            _ => return Err(())
        })
    }
}

impl ToString for ViewType {
    fn to_string(&self) -> String {
        match self {
            ViewType::LinearLayout => "LinearLayout", ViewType::ScrollView => "ScrollView",
            ViewType::Button => "Button", ViewType::TextView => "TextView",
            ViewType::EditText => "EditText", ViewType::ImageView => "ImageView",
            ViewType::WebView => "WebView", ViewType::ProgressBar => "ProgressBar",
            ViewType::ListView => "ListView", ViewType::Spinner => "Spinner",
            ViewType::CheckBox => "CheckBox", ViewType::Switch => "Switch",
            ViewType::SeekBar => "SeekBar", ViewType::CalendarView => "CalendarView",
            ViewType::Fab => "Fab", // "Fab" might not be valid as a typename
            ViewType::AdView => "AdView", ViewType::MapView => "MapView",
        }.to_string()
    }
}

impl ViewType {
    // yeah
    pub fn from_swrs_view(typ: &SWRSViewType) -> ViewType {
        match typ {
            SWRSViewType::LinearLayout { .. } => ViewType::LinearLayout,
            SWRSViewType::ScrollView { .. } => ViewType::ScrollView,
            SWRSViewType::Button { .. } => ViewType::Button,
            SWRSViewType::TextView { .. } => ViewType::TextView,
            SWRSViewType::EditText { .. } => ViewType::EditText,
            SWRSViewType::ImageView { .. } => ViewType::ImageView,
            SWRSViewType::WebView => ViewType::WebView,
            SWRSViewType::ProgressBar { .. } => ViewType::ProgressBar,
            SWRSViewType::ListView { .. } => ViewType::ListView,
            SWRSViewType::Spinner { .. } => ViewType::Spinner,
            SWRSViewType::CheckBox { .. } => ViewType::CheckBox,
            SWRSViewType::Switch { .. } => ViewType::Switch,
            SWRSViewType::SeekBar { .. } => ViewType::SeekBar,
            SWRSViewType::CalendarView { .. } => ViewType::CalendarView,
            SWRSViewType::Fab { .. } => ViewType::Fab,
            SWRSViewType::AdView { .. } => ViewType::AdView,
            SWRSViewType::MapView => ViewType::MapView
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ComponentType {
    // todo: components
}

impl Type {
    pub fn from_arg_block(arg_block_type: &ArgumentBlockReturnType) -> Option<Self> {
        Some(match arg_block_type {
            ArgumentBlockReturnType::Number => Type::Primitive(PrimitiveType::Number),
            ArgumentBlockReturnType::String => Type::Primitive(PrimitiveType::String),
            ArgumentBlockReturnType::Boolean => Type::Primitive(PrimitiveType::Boolean),
            ArgumentBlockReturnType::List { inner_type } =>
                Type::Complex(ComplexType::List {
                    inner_type: match inner_type {
                        ListItem::String => PrimitiveType::String,
                        ListItem::Number => PrimitiveType::Number
                    }
                }),

            ArgumentBlockReturnType::View { type_name } =>
                Type::View(ViewType::from_str(type_name).ok()?),

            ArgumentBlockReturnType::Component { type_name } =>
                // Type::Component(ComponentType::from_str(&type_name).ok?)
                todo!("components"),
        })
    }

    pub fn to_arg_block_type(&self) -> ArgumentBlockReturnType {
        match self {
            Type::Void => unreachable!(),
            Type::Primitive(PrimitiveType::Number) =>
                ArgumentBlockReturnType::Number,
            Type::Primitive(PrimitiveType::Boolean) =>
                ArgumentBlockReturnType::Boolean,
            Type::Primitive(PrimitiveType::String) =>
                ArgumentBlockReturnType::String,
            Type::Complex(ComplexType::List { inner_type }) =>
                ArgumentBlockReturnType::List {
                    inner_type: match inner_type {
                        PrimitiveType::Number => ListItem::Number,
                        PrimitiveType::String => ListItem::String,
                        PrimitiveType::Boolean => unreachable!(),
                    }
                },
            Type::Complex(ComplexType::Map) => todo!("map"),
            Type::View(view) =>
                // fixme: uhhh i think this might not work on some views
                ArgumentBlockReturnType::View { type_name: view.to_string() },
            Type::Component(_) => todo!("components")
        }
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
        pub(in super) fn $fn_name(self) -> $ret {
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

/// A simple struct that stores any variable types for the logic code to use. This provides APIs
/// to resolve variables/functions (and their fields/methods) and construct blocks out of them
#[derive(Debug, Clone)]
pub struct Definitions<'a> {
    // type can only be Type::Primitive and Type::Complex
    variables: LinkedHashMap<String, Type>,
    layout_ref: &'a View,
}

impl<'a> Definitions<'a> {
    pub fn new(layout_ref: &'a View) -> Self {
        Self {
            variables: Default::default(),
            layout_ref
        }
    }

    // returns None when the variable name is already used
    pub fn add_variable(&mut self, name: String, typ: Type) -> Option<String> {
        if self.variables.contains_key(&name) { return None; }
        if self.layout_ref.find_id(&name).is_some() { return None; }

        self.variables.insert(name.clone(), typ);

        Some(name)
    }

    pub fn get_var(&self, name: &str) -> Option<Type> {
        if let Some(var) = self.variables.get(name) { return Some(*var); }
        if let Some(View { view: Ok(view), .. }) = self.layout_ref.find_id(name) {
            return Some(Type::View(ViewType::from_swrs_view(view)))
        }

        None
    }

    pub fn get_type_data(typ: Type) -> Option<&'static TypeData> {
        match typ {
            Type::Void => None,
            Type::Primitive(PrimitiveType::String) => None, // todo
            Type::Primitive(PrimitiveType::Number) => Some(super::NUMBER_TYPE_DATA.deref()),
            Type::Primitive(PrimitiveType::Boolean) => None, // todo
            Type::Complex(ComplexType::Map) => None, // todo
            Type::Complex(ComplexType::List { inner_type: PrimitiveType::String }) => None, // todo
            Type::Complex(ComplexType::List { inner_type: PrimitiveType::Number }) => None, // todo
            Type::View(_) => None, // todo
            Type::Component(_) => None, // todo
            _ => panic!("list cant have bool inner type")
        }
    }

    pub fn get_global_func(name: &str) -> Option<&GlobalFunction> {
        GLOBAL_FUNCTIONS.get(name)
    }

    pub fn deconstruct(self)
        -> (LinkedHashMap<String, SWRSVariable>, LinkedHashMap<String, SWRSListVariable>) {
        let mut variables = LinkedHashMap::new();
        let mut list_variables = LinkedHashMap::new();

        for (name, typ) in self.variables {
            match typ {
                Type::Primitive(prim) => {
                    variables.insert(
                        name.clone(),
                        SWRSVariable {
                            name,
                            r#type: match prim {
                                PrimitiveType::Number => SWRSVariableType::Integer,
                                PrimitiveType::String => SWRSVariableType::String,
                                PrimitiveType::Boolean => SWRSVariableType::Boolean,
                            }
                        }
                    );
                }

                Type::Complex(ComplexType::List { inner_type }) => {
                    list_variables.insert(
                        name.clone(),
                        SWRSListVariable {
                            name,
                            r#type: match inner_type {
                                PrimitiveType::Number => SWRSVariableType::Integer,
                                PrimitiveType::String => SWRSVariableType::String,
                                PrimitiveType::Boolean =>
                                    panic!("ComplexType::List cannot have a boolean inner type")
                            }
                        }
                    );
                }

                Type::Complex(ComplexType::Map) => todo!(),
                _ => ()
            }
        }

        (variables, list_variables)
    }
}

#[derive(Debug, Clone)]
pub enum Member {
    Field {
        generate: fn(TypeValue) -> Block,
        return_type: Type,
    },
    Method {
        arg_types: Vec<Type>,
        //           value      args
        generate: fn(TypeValue, Vec<TypeValue>) -> Block,
        return_type: Type,
    },
}

impl Member {
    pub fn new_method(
        arg_types: Vec<Type>, ret_type: Type, gen: fn(TypeValue, Vec<TypeValue>) -> Block
    ) -> Member {
        Member::Method {
            arg_types,
            generate: gen,
            return_type: ret_type
        }
    }

    pub fn new_field(ret_type: Type, gen: fn(TypeValue) -> Block) -> Member {
        Member::Field {
            generate: gen,
            return_type: ret_type
        }
    }

    pub fn method_gen(&self, val: TypeValue, args: Vec<TypeValue>) -> Result<Block, GenerateError> {
        // make sure its a method
        if let Member::Method {
            arg_types: method_arg_types,
            generate,
            ..
        } = self {
            let args_types = args.iter()
                .map(|arg| arg.as_type())
                .collect::<Vec<Type>>();

            if method_arg_types.len() != args_types.len() {
                return Err(GenerateError::InvalidArgumentCount {
                    expected: method_arg_types.clone(),
                    got: args_types
                })
            }

            // check if the parameter types have the same types as the argument types
            for index in 0..method_arg_types.len() {
                if method_arg_types.get(index).unwrap() != args_types.get(index).unwrap() {
                    return Err(GenerateError::InvalidArgumentType {
                        expected: method_arg_types.into_iter().cloned().collect(),
                        got: args_types,
                        index,
                    })
                }
            }

            Ok((generate)(val, args))
        } else { panic!("not a method") }
    }

    pub fn field_gen(&self, value: TypeValue) -> Result<Block, GenerateError> {
        // make sure its a method
        if let Member::Field {
            generate, ..
        } = self {
            Ok((generate)(value))
        } else { panic!("not a field") }
    }
}

// stores stuff about a type
#[derive(Debug, Clone)]
pub struct TypeData {
    // hashmap of <type used to index> -> <generate function>
    //
    // a type can have multiple ways of indexing depending on the indexing value type
    //
    // the fn parameter [TypeValue; 2]: [0] is the var getting indexed, [1] is the value that's
    // used to index
    pub index: HashMap<Type, fn([TypeValue; 2]) -> Block>,

    // all the members of this type data
    pub members: HashMap<String, Member>,
}

// a global function
#[derive(Debug, Clone)]
pub struct GlobalFunction {
    pub name: String,
    pub return_type: Type,
    pub argument_types: Vec<Type>,
    generate: fn(Vec<TypeValue>) -> Block
}

impl GlobalFunction {
    pub fn generate(&self, args: Vec<TypeValue>) -> Result<Block, GenerateError> {
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

// to make creating hashmaps easier
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
    static ref GLOBAL_FUNCTIONS: HashMap<String, GlobalFunction> = {
        hashmap_str! {
            "toast" => new_func(
                "toast", Type::Void,
                vec![Type::Primitive(PrimitiveType::String)], |mut args| {
                let text = args.remove(0).to_str();

                Block::new(
                    BlockCategory::ComponentFunc,
                    "doToast".to_string(),
                    BlockContent::builder()
                        .text("Toast")
                        .arg(Argument::String { name: None, value: text })
                        .build(),
                    BlockType::Regular
                )
            })
        }
    };
}

// shortcut of creating a new func
fn new_func(
    name: &str,
    return_type: Type,
    argument_types: Vec<Type>,
    generate_fn: fn(Vec<TypeValue>) -> Block
) -> GlobalFunction {
    GlobalFunction {
        name: name.to_string(),
        return_type,
        argument_types,
        generate: generate_fn
    }
}