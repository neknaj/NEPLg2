use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ValueAliasSummary {
    pub(super) raw_addr_alias: Option<String>,
    pub(super) aggregate_field_raw_aliases: BTreeMap<usize, String>,
    pub(super) aggregate_field_function_aliases: BTreeMap<usize, BTreeSet<String>>,
    pub(super) enum_payload_raw_aliases: BTreeMap<String, String>,
    pub(super) enum_payload_aggregate_field_raw_aliases: BTreeMap<String, BTreeMap<usize, String>>,
    pub(super) enum_payload_aggregate_field_function_aliases:
        BTreeMap<String, BTreeMap<usize, BTreeSet<String>>>,
    pub(super) enum_payload_function_aliases: BTreeMap<String, BTreeSet<String>>,
    pub(super) function_value_aliases: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RawMemoryEffectSummary {
    Load {
        place: String,
        size: usize,
    },
    Store {
        place: String,
        size: usize,
    },
    Dealloc {
        place: String,
        size: Option<usize>,
    },
    Realloc {
        place: String,
        size: Option<usize>,
    },
    BulkCopy {
        dst: String,
        src: String,
        size: Option<usize>,
    },
    ByteWrite {
        place: String,
        size: Option<usize>,
    },
    IndirectCall {
        callee: String,
        args: Vec<ValueAliasSummary>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct FunctionRawAliasSummary {
    pub(super) raw_addr_alias: Option<String>,
    pub(super) aggregate_field_raw_aliases: BTreeMap<usize, String>,
    pub(super) aggregate_field_function_aliases: BTreeMap<usize, BTreeSet<String>>,
    pub(super) enum_payload_raw_aliases: BTreeMap<String, String>,
    pub(super) enum_payload_aggregate_field_raw_aliases: BTreeMap<String, BTreeMap<usize, String>>,
    pub(super) enum_payload_aggregate_field_function_aliases:
        BTreeMap<String, BTreeMap<usize, BTreeSet<String>>>,
    pub(super) enum_payload_function_aliases: BTreeMap<String, BTreeSet<String>>,
    pub(super) function_value_aliases: BTreeSet<String>,
    pub(super) raw_memory_effects: Vec<RawMemoryEffectSummary>,
}

pub(super) fn extend_unique_raw_memory_effects(
    out: &mut Vec<RawMemoryEffectSummary>,
    effects: impl IntoIterator<Item = RawMemoryEffectSummary>,
) {
    for effect in effects {
        if !out.contains(&effect) {
            out.push(effect);
        }
    }
}

pub(super) fn merge_matching_raw_alias_summaries(
    summaries: impl IntoIterator<Item = FunctionRawAliasSummary>,
) -> FunctionRawAliasSummary {
    let mut iter = summaries.into_iter();
    let Some(mut merged) = iter.next() else {
        return FunctionRawAliasSummary::default();
    };
    for summary in iter {
        if merged.raw_addr_alias != summary.raw_addr_alias {
            merged.raw_addr_alias = None;
        }
        merged.aggregate_field_raw_aliases = retain_matching_aliases(
            &merged.aggregate_field_raw_aliases,
            &summary.aggregate_field_raw_aliases,
        );
        for (offset, aliases) in summary.aggregate_field_function_aliases {
            merged
                .aggregate_field_function_aliases
                .entry(offset)
                .or_default()
                .extend(aliases);
        }
        merged.enum_payload_raw_aliases = retain_matching_aliases(
            &merged.enum_payload_raw_aliases,
            &summary.enum_payload_raw_aliases,
        );
        merged.enum_payload_aggregate_field_raw_aliases = retain_matching_nested_aliases(
            &merged.enum_payload_aggregate_field_raw_aliases,
            &summary.enum_payload_aggregate_field_raw_aliases,
        );
        for (variant, field_aliases) in summary.enum_payload_aggregate_field_function_aliases {
            let merged_field_aliases = merged
                .enum_payload_aggregate_field_function_aliases
                .entry(variant)
                .or_default();
            for (offset, aliases) in field_aliases {
                merged_field_aliases
                    .entry(offset)
                    .or_default()
                    .extend(aliases);
            }
        }
        for (variant, aliases) in summary.enum_payload_function_aliases {
            merged
                .enum_payload_function_aliases
                .entry(variant)
                .or_default()
                .extend(aliases);
        }
        merged
            .function_value_aliases
            .extend(summary.function_value_aliases);
        extend_unique_raw_memory_effects(
            &mut merged.raw_memory_effects,
            summary.raw_memory_effects,
        );
    }
    merged
}

pub(super) fn value_alias_summary_from_raw_summary(
    summary: &FunctionRawAliasSummary,
) -> ValueAliasSummary {
    ValueAliasSummary {
        raw_addr_alias: summary.raw_addr_alias.clone(),
        aggregate_field_raw_aliases: summary.aggregate_field_raw_aliases.clone(),
        aggregate_field_function_aliases: summary.aggregate_field_function_aliases.clone(),
        enum_payload_raw_aliases: summary.enum_payload_raw_aliases.clone(),
        enum_payload_aggregate_field_raw_aliases: summary
            .enum_payload_aggregate_field_raw_aliases
            .clone(),
        enum_payload_aggregate_field_function_aliases: summary
            .enum_payload_aggregate_field_function_aliases
            .clone(),
        enum_payload_function_aliases: summary.enum_payload_function_aliases.clone(),
        function_value_aliases: summary.function_value_aliases.clone(),
    }
}

pub(super) fn add_child_raw_memory_effects(
    summary: &mut FunctionRawAliasSummary,
    children: impl IntoIterator<Item = FunctionRawAliasSummary>,
) {
    for child in children {
        extend_unique_raw_memory_effects(&mut summary.raw_memory_effects, child.raw_memory_effects);
    }
}

fn retain_matching_aliases<K: Ord + Clone>(
    left: &BTreeMap<K, String>,
    right: &BTreeMap<K, String>,
) -> BTreeMap<K, String> {
    left.iter()
        .filter_map(|(key, alias)| {
            if right.get(key) == Some(alias) {
                Some((key.clone(), alias.clone()))
            } else {
                None
            }
        })
        .collect()
}

fn retain_matching_nested_aliases<K: Ord + Clone>(
    left: &BTreeMap<String, BTreeMap<K, String>>,
    right: &BTreeMap<String, BTreeMap<K, String>>,
) -> BTreeMap<String, BTreeMap<K, String>> {
    left.iter()
        .filter_map(|(variant, aliases)| {
            let merged = retain_matching_aliases(aliases, right.get(variant)?);
            if merged.is_empty() {
                None
            } else {
                Some((variant.clone(), merged))
            }
        })
        .collect()
}
