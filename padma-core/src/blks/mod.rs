use regex::{Regex};
use swrs::api::block::{BlockCategory, BlockContent, BlockType};

pub mod error;
mod tests;

// todo: list, map, component, and view as block argument types

#[derive(Debug, Clone, PartialEq)]
pub struct BlockDefinitions(pub Vec<BlockDefinition>);

#[derive(Debug, Clone, PartialEq)]
pub struct BlockDefinition {
    block_type: BlockType,
    category: BlockCategory,
    opcode: String,
    spec: BlockContent,
}

const REGEX: &str = r#"^\s*\[(var|list|control|operator|math|file|view|component|moreblock)\]\s*(.+?)\s*(?:\(([bsd])\))?:\s*"(.*)"\s*$"#;

/// Parses a `.blks` file from a `&str`
pub fn parse(data: &str) -> Result<BlockDefinitions, error::BlksParseError> {
    // this function only utilizes a regex matcher
    // we don't need a "real parser" yet
    let mut lines = data.split("\n")

        // preserve the line numbers
        .enumerate()

        // remove any comments
        .filter_map(|(lnum, line)| {
            let trimmed = line.trim();

            let (trimmed, _comment) =
                trimmed.split_once("//")
                    .map(|(line, _com)| (line.trim(), _com))
                    .unwrap_or((trimmed, ""));

            (!trimmed.is_empty()).then_some((lnum, trimmed))
        });

    let mut result = vec![];

    let re = Regex::new(REGEX).unwrap();

    while let Some((lnum, line)) = lines.next() {
        // match with regex
        let Some(capture) = re.captures(line) else {
            return Err(error::BlksParseError::InvalidSyntax {
                line: lnum,
                content: line.to_string()
            })
        };

        // .unwrap()s: these these values will never be None since the regex only matches when these groups matched
        let category = capture.get(1).unwrap().as_str();
        let opcode = capture.get(2).unwrap().as_str();
        let blk_type = capture.get(3).map(|mat| mat.as_str()).unwrap_or(" ");
        let spec = capture.get(4).unwrap().as_str();

        result.push(BlockDefinition {
            block_type: BlockType::from(blk_type, String::new()) // todo: typename in padma blks
                .map_err(|err| error::BlksParseError::InvalidBlockType {
                    line: lnum, blk_type: blk_type.to_string(), source: err
                })?,

            category: match_category(category),
            opcode: opcode.to_string(),
            spec: BlockContent::parse_wo_params(spec)
                .map_err(|err| error::BlksParseError::InvalidBlockSpec { line: lnum, source: err })?
        })
    }

    Ok(BlockDefinitions(result))
}

fn match_category(category: &str) -> BlockCategory {
    match category {
        "var" => BlockCategory::Variable,
        "list" => BlockCategory::List,
        "control" => BlockCategory::Control,
        "operator" => BlockCategory::Operator,
        "math" => BlockCategory::Math,
        "file" => BlockCategory::File,
        "view" => BlockCategory::ViewFunc,
        "component" => BlockCategory::ComponentFunc,
        "moreblock" => BlockCategory::MoreBlock,
        _ => unreachable!()
    }
}