// All the blocks used in compilation
// todo: create a some kind of generator for this so its easier to work with
// syntax-ish:
//   func_name opcode "spec" block_category block_type

// example:
//   r#break break "break" control f

// with substack:
//   r#if if "if %b" control e
// (substacks are determined from the block type)

use swrs::api::block::{
    Argument, ArgumentBlockReturnType, ArgValue, Block, BlockCategory, BlockContent, BlockControl,
    Blocks, BlockType
};
use crate::compiler::logic::blocks::types::{Member, TypeData, Type, PrimitiveType};
use std::collections::HashMap;
use lazy_static::lazy_static;

pub mod types;

// decoration to make things consistent
type Number = f64;
type Boolean = bool;

pub fn r#break() -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Control.into(),
        op_code: "break".to_string(),
        content: BlockContent::builder().text("continue").build(),
        block_type: BlockType::Control(BlockControl::EndingBlock)
    }
}

pub fn r#continue() -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Control.into(),
        op_code: "continue".to_string(),
        content: BlockContent::builder().text("continue").build(),
        block_type: BlockType::Control(BlockControl::EndingBlock)
    }
}

pub fn not(arg: ArgValue<Boolean>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "not".to_string(),
        content: BlockContent::builder()
            .text("not")
            .arg(Argument::Boolean {
                name: None,
                value: arg
            })
            .build(),
        block_type: BlockType::Regular
    }
}

pub fn or(first: ArgValue<Boolean>, second: ArgValue<Boolean>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "||".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Boolean { name: None, value: first })
            .text("or")
            .arg(Argument::Boolean { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
    }
}

pub fn and(first: ArgValue<Boolean>, second: ArgValue<Boolean>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "&&".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Boolean { name: None, value: first })
            .text("and")
            .arg(Argument::Boolean { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
    }
}

pub fn lt(first: ArgValue<Number>, second: ArgValue<Number>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "<".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Number { name: None, value: first })
            .text("<")
            .arg(Argument::Number { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
    }
}

pub fn lte(first: ArgValue<Number>, second: ArgValue<Number>) -> Block {
    // fancy sugar to "first < second || first == second"
    or(ArgValue::Block(
        lt(first.clone(), second.clone())
    ), ArgValue::Block(
        eq(first, second)
    ))
}

pub fn gt(first: ArgValue<Number>, second: ArgValue<Number>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: ">".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Number { name: None, value: first })
            .text(">")
            .arg(Argument::Number { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
    }
}

pub fn gte(first: ArgValue<Number>, second: ArgValue<Number>) -> Block {
    // fancy sugar to "first > second || first == second"
    or(ArgValue::Block(
        gt(first.clone(), second.clone())
    ), ArgValue::Block(
        eq(first, second)
    ))
}

pub fn eq(first: ArgValue<Number>, second: ArgValue<Number>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "=".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Number { name: None, value: first })
            .text("==")
            .arg(Argument::Number { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
    }
}

pub fn plus(first: ArgValue<Number>, second: ArgValue<Number>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "+".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Number { name: None, value: first })
            .text("+")
            .arg(Argument::Number { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Number)
    }
}

pub fn minus(first: ArgValue<Number>, second: ArgValue<Number>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "-".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Number { name: None, value: first })
            .text("-")
            .arg(Argument::Number { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Number)
    }
}

pub fn multiply(first: ArgValue<Number>, second: ArgValue<Number>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "*".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Number { name: None, value: first })
            .text("*")
            .arg(Argument::Number { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Number)
    }
}

pub fn divide(first: ArgValue<Number>, second: ArgValue<Number>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "/".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Number { name: None, value: first })
            .text("/")
            .arg(Argument::Number { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Number)
    }
}

pub fn power(first: ArgValue<Number>, second: ArgValue<Number>) -> Block {
    // fancy sugar - ok nvm i dont think this is possible unless with some asd trickery, but
    // as an expression? probably not
    unimplemented!()
}

pub fn minus_unary(value: ArgValue<Number>) -> Block {
    // fancy sugar to "value * -1"
    multiply(value, ArgValue::Value(-1f64))
}

pub fn plus_unary(value: ArgValue<Number>) -> Block {
    // fancy sugar to "value * 1"
    multiply(value, ArgValue::Value(1f64))
}

pub fn repeat(value: ArgValue<Number>, nest: Blocks) -> Block {
    Block {
        sub_stack1: Some(nest),
        sub_stack2: None,
        color: Default::default(),
        op_code: "repeat".to_string(),
        content: BlockContent::builder()
            .text("repeat")
            .arg(Argument::Number { name: None, value })
            .build(),
        block_type: BlockType::Control(BlockControl::OneNest)
    }
}

pub fn forever(nest: Blocks) -> Block {
    Block {
        sub_stack1: Some(nest),
        sub_stack2: None,
        color: Default::default(),
        op_code: "forever".to_string(),
        content: BlockContent::builder().text("forever").build(),
        block_type: BlockType::Control(BlockControl::OneNest)
    }
}

pub fn r#if(condition: ArgValue<Boolean>, body: Blocks) -> Block {
    Block {
        sub_stack1: Some(body),
        sub_stack2: None,
        color: Default::default(),
        op_code: "if".to_string(),
        content: BlockContent::builder()
            .text("if")
            .arg(Argument::Boolean { name: None, value: condition })
            .text("then")
            .build(),
        block_type: BlockType::Control(BlockControl::OneNest)
    }
}

pub fn r#if_else(condition: ArgValue<Boolean>, if_body: Blocks, else_body: Blocks) -> Block {
    Block {
        sub_stack1: Some(if_body),
        sub_stack2: Some(else_body),
        color: Default::default(),
        op_code: "ifElse".to_string(),
        content: BlockContent::builder()
            .text("if")
            .arg(Argument::Boolean { name: None, value: condition })
            .text("then")
            .build(),
        block_type: BlockType::Control(BlockControl::TwoNest)
    }
}

pub fn set_var_int(name: String, value: ArgValue<Number>) -> Block {
    Block::new(
        BlockCategory::Variable,
        "setVarInt".to_string(),
        BlockContent::builder()
            .text("set")
            .arg(Argument::Menu {
                name: "varInt".to_string(),
                value: ArgValue::Value(name)
            })
            .text("to")
            .arg(Argument::Number { name: None, value })
            .build(),
        BlockType::Regular
    )
}

pub fn set_var_boolean(name: String, value: ArgValue<Boolean>) -> Block {
    Block::new(
        BlockCategory::Variable,
        "setVarBoolean".to_string(),
        BlockContent::builder()
            .text("set")
            .arg(Argument::Menu {
                name: "varBool".to_string(),
                value: ArgValue::Value(name)
            })
            .text("to")
            .arg(Argument::Boolean { name: None, value })
            .build(),
        BlockType::Regular
    )
}

pub fn set_var_string(name: String, value: ArgValue<String>) -> Block {
    Block::new(
        BlockCategory::Variable,
        "setVarString".to_string(),
        BlockContent::builder()
            .text("set")
            .arg(Argument::Menu {
                name: "varStr".to_string(),
                value: ArgValue::Value(name)
            })
            .text("to")
            .arg(Argument::String { name: None, value })
            .build(),
        BlockType::Regular
    )
}

pub fn get_var(name: String, arg_type: ArgumentBlockReturnType) -> Block {
    Block::new(
        BlockCategory::Variable,
        "getVar".to_string(),
        BlockContent::builder().text(name).build(),
        BlockType::Argument(arg_type)
    )
}

macro_rules! hashmap {
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = HashMap::new();
            $(m.insert($key.to_string(), $value);)+
            m
        }
     };
}

macro_rules! method {
    (($arg_types:expr) -> $ret_type:expr ; $gen_func:expr) => {
        Member::Method {
            arg_types: $arg_types,
            return_type: $ret_type,
            generate: $gen_func
        }
    };
}

// the fields and methods of the type Number
lazy_static! {
    pub static ref NUMBER_TYPE_DATA: TypeData = TypeData {
        index: HashMap::new(),
        members: hashmap! {
            "toString" => method!((vec![]) -> Type::Primitive(PrimitiveType::String); |val, _| {
                Block::new(
                    BlockCategory::Operator,
                    "toString".to_string(),
                    BlockContent::builder()
                        .text("toString")
                        .arg(Argument::Number { name: None, value: val.to_num() })
                        .text("without")
                        .text("decimal")
                        .build(),
                    BlockType::Argument(ArgumentBlockReturnType::String)
                )
            }),
            "toStringDec" => method!((vec![]) -> Type::Primitive(PrimitiveType::String); |val, _| {
                Block::new(
                    BlockCategory::Operator,
                    "toStringWithDecimal".to_string(),
                    BlockContent::builder()
                        .text("toString")
                        .arg(Argument::Number { name: None, value: val.to_num() })
                        .text("with")
                        .text("decimal")
                        .build(),
                    BlockType::Argument(ArgumentBlockReturnType::String)
                )
            })
        }
    };
}