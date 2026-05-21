use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace;
use super::collection_slot_summary_projection;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, ResourceLocal};

pub(super) fn instantiate_summary_target(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    target: &CollectionSlotLifecycleSummaryPlace,
) -> Option<Place> {
    collection_slot_summary_projection::instantiate_summary_place(engine, args, target)
}

pub(super) fn summary_place_for_params(
    params: &[ResourceLocal],
    target: &Place,
) -> Option<CollectionSlotLifecycleSummaryPlace> {
    collection_slot_summary_projection::summary_place_for_params(params, target)
}
