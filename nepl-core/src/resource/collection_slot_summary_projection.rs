pub(super) use super::summary_projection::{
    compose_translated_summary_suffix_for_params, instantiate_summary_place,
    instantiate_summary_suffix_on_base, summary_place_for_params, summary_suffix_for_params,
    SummaryProjection as CollectionSlotLifecycleSummaryProjection,
};

#[cfg(test)]
pub(super) use super::summary_projection::{
    translate_summary_suffix_for_params, SummaryOffset as CollectionSlotLifecycleSummaryOffset,
};
