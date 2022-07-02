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
    BlockType
};

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

pub fn lt(first: ArgValue<Boolean>, second: ArgValue<Boolean>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "<".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Boolean { name: None, value: first })
            .text("<")
            .arg(Argument::Boolean { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
    }
}

pub fn lte(first: ArgValue<Boolean>, second: ArgValue<Boolean>) -> Block {
    // fancy sugar to "first < second || first == second"
    or(ArgValue::Block(
        lt(first.clone(), second.clone())
    ), ArgValue::Block(
        eq(first, second)
    ))
}

pub fn gt(first: ArgValue<Boolean>, second: ArgValue<Boolean>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: ">".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Boolean { name: None, value: first })
            .text(">")
            .arg(Argument::Boolean { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
    }
}

pub fn gte(first: ArgValue<Boolean>, second: ArgValue<Boolean>) -> Block {
    // fancy sugar to "first > second || first == second"
    or(ArgValue::Block(
        gt(first.clone(), second.clone())
    ), ArgValue::Block(
        eq(first, second)
    ))
}

pub fn eq(first: ArgValue<Boolean>, second: ArgValue<Boolean>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "=".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Boolean { name: None, value: first })
            .text("==")
            .arg(Argument::Boolean { name: None, value: second })
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
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
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
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
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
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
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
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
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