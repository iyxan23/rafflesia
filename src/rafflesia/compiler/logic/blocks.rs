// All the blocks used in compilation

use swrs::api::block::{Argument, ArgumentBlockReturnType, ArgValue, Block, BlockCategory, BlockContent, BlockControl, BlockType};

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

pub fn not(arg: ArgValue<bool>) -> Block {
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

pub fn or(first: ArgValue<bool>, second: ArgValue<bool>) -> Block {
    Block {
        sub_stack1: None,
        sub_stack2: None,
        color: BlockCategory::Math.into(),
        op_code: "||".to_string(),
        content: BlockContent::builder()
            .arg(Argument::Boolean { name: None, value: first })
            .text("||")
            .arg(Argument::Boolean { name: None, value: second })
            .build(),
        block_type: BlockType::Argument(ArgumentBlockReturnType::Boolean)
    }
}