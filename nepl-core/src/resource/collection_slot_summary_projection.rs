pub(super) use super::summary_projection::{
    compose_translated_summary_suffix_for_params, instantiate_summary_place_with_aliases,
    instantiate_summary_suffix_on_base, summary_place_for_params_with_scalar_aliases,
    summary_place_for_params_with_scalar_aliases_and_types, summary_suffix_for_params,
    translate_summary_place_for_params_with_aliases,
    SummaryProjection as CollectionSlotLifecycleSummaryProjection,
};

#[cfg(test)]
pub(super) use super::summary_projection::{
    instantiate_summary_place, summary_place_for_params, translate_summary_suffix_for_params,
    SummaryOffset as CollectionSlotLifecycleSummaryOffset,
};
