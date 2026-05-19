use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_difference::I32DifferenceFacts;
use super::initialized_alias_host_size::HostSizeFacts;
use super::initialized_alias_offset::I32OffsetFacts;
use super::initialized_alias_relation::I32RelationFacts;
use super::initialized_alias_scalar::I32AliasFacts;
use super::initialized_alias_scale::I32ScaleFacts;
use super::initialized_alias_type_size::I32TypeSizeFacts;
use super::model::Place;

#[derive(Default)]
pub(super) struct ScalarFactCopies {
    i32_facts: I32AliasFacts,
    i32_differences: I32DifferenceFacts,
    i32_relations: I32RelationFacts,
    i32_scales: I32ScaleFacts,
    i32_offsets: I32OffsetFacts,
    i32_type_sizes: I32TypeSizeFacts,
    host_size_facts: HostSizeFacts,
}

impl ScalarFactCopies {
    pub(super) fn extend_from(
        &mut self,
        aliases: &RawCellAddressAliases,
        source: &Place,
        target: &Place,
    ) {
        self.i32_facts
            .extend(aliases.i32_facts.facts_with_replaced_prefix(source, target));
        self.i32_differences.extend(
            aliases
                .i32_differences
                .facts_with_replaced_prefix(source, target),
        );
        self.i32_relations.extend(
            aliases
                .i32_relations
                .facts_with_replaced_prefix(source, target),
        );
        self.i32_scales.extend(
            aliases
                .i32_scales
                .facts_with_replaced_prefix(source, target),
        );
        self.i32_offsets.extend(
            aliases
                .i32_offsets
                .facts_with_replaced_prefix(source, target),
        );
        self.i32_type_sizes.extend(
            aliases
                .i32_type_sizes
                .facts_with_replaced_prefix(source, target),
        );
        self.host_size_facts.extend(
            aliases
                .host_size_facts
                .facts_with_replaced_prefix(source, target),
        );
    }

    pub(super) fn apply_to(self, aliases: &mut RawCellAddressAliases) {
        aliases.i32_facts.extend(self.i32_facts);
        aliases.i32_differences.extend(self.i32_differences);
        aliases.i32_relations.extend(self.i32_relations);
        aliases.i32_scales.extend(self.i32_scales);
        aliases.i32_offsets.extend(self.i32_offsets);
        aliases.i32_type_sizes.extend(self.i32_type_sizes);
        aliases.host_size_facts.extend(self.host_size_facts);
    }
}
