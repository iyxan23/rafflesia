use std::collections::{BTreeMap, HashMap};

use crate::blks::BlockDefinitions;
use crate::defs::Definitions as DefsDefinitions;

use super::{Resolver, ResolverBlocks, ResolverDefinitions};

type PriorityOrderedMap<T> = BTreeMap<u32, T>;

/// A builder to create a [`Resolver`] struct.
pub struct ResolverBuilder {
    definitions: PriorityOrderedMap<DefsDefinitions>,
    blocks: PriorityOrderedMap<BlockDefinitions>,
}

impl ResolverBuilder {
    /// Creates an empty [`ResolverBuilder`].
    pub fn new_empty() -> Self {
        Self {
            definitions: Default::default(),
            blocks: Default::default(),
        }
    }

    /// Creates a new [`ResolverBuilder`] with its contents.
    pub fn new(
        definitions: Vec<(u32, DefsDefinitions)>,
        blocks: Vec<(u32, BlockDefinitions)>,
    ) -> Self {
        Self {
            definitions: definitions.into_iter().collect(),
            blocks: blocks.into_iter().collect(),
        }
    }

    pub fn add_definitions(&mut self, def: DefsDefinitions, priority: u32) {
        self.definitions.insert(priority, def);
    }

    pub fn add_blocks(&mut self, blocks: BlockDefinitions, priority: u32) {
        self.blocks.insert(priority, blocks);
    }

    /// Builds a [`Resolver`] with the previously given contents.
    pub fn build(self) -> Resolver {
        todo!()
    }

    fn flatten_defs(map: PriorityOrderedMap<DefsDefinitions>) -> ResolverDefinitions {
        // let merged = Self::merge_defs_based_on_priority(map);
        todo!()
    }

    fn flatten_blks(map: PriorityOrderedMap<BlockDefinitions>) -> ResolverBlocks {
        todo!()
    }

    // fn merge_defs_based_on_priority(
    //     map: PriorityOrderedMap<DefsDefinitions>,
    // ) -> DefsDefinitions {
    //     let mut result = DefsDefinitions {
    //         global_functions: HashMap::new(), methods: HashMap::new(), bindings: HashMap::new()
    //     };

    //     for (_priority, defs_definitions) in map {

    //     }

    //     result
    // }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{blks::BlockDefinitions, defs::Definitions as DefsDefinitions};

    use super::ResolverBuilder;

    #[test]
    fn create_an_empty_resolver_builder() {
        let builder = ResolverBuilder::new_empty();
        assert!(builder.blocks.is_empty());
        assert!(builder.definitions.is_empty());
    }

    #[test]
    fn create_resolver_builder_with_contents() {
        let definitions = vec![
            (100, DefsDefinitions::default()),
            (200, DefsDefinitions::default()),
        ];
        let blocks = vec![
            (110, BlockDefinitions::default()),
            (230, BlockDefinitions::default()),
        ];

        let builder = ResolverBuilder::new(definitions.clone(), blocks.clone());

        assert_eq!(
            builder.blocks,
            blocks
                .into_iter()
                .collect::<BTreeMap<u32, BlockDefinitions>>()
        );
        assert_eq!(
            builder.definitions,
            definitions
                .into_iter()
                .collect::<BTreeMap<u32, DefsDefinitions>>()
        );
    }

    #[test]
    fn add_definitions_to_resolver_builder() {
        let definition = DefsDefinitions::default();

        let mut builder = ResolverBuilder::new_empty();
        builder.add_definitions(definition.clone(), 100);

        assert_eq!(builder.definitions.pop_first(), Some((100, definition)));
        assert!(builder.definitions.is_empty());
    }

    #[test]
    fn add_blocks_to_resolver_builder() {
        let blocks = BlockDefinitions::default();

        let mut builder = ResolverBuilder::new_empty();
        builder.add_blocks(blocks.clone(), 100);

        assert_eq!(builder.blocks.pop_first(), Some((100, blocks)));
        assert!(builder.blocks.is_empty());
    }
}
