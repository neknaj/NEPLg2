use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, ResourceLocal};
use super::place_utils::{place_suffix_after_prefix, projected_place_with_concrete_type};

pub(super) fn instantiate_summary_target(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    target: &CollectionSlotLifecycleSummaryPlace,
) -> Option<Place> {
    let arg = args.get(target.parameter_index)?;
    Some(projected_place_with_concrete_type(
        engine.types,
        arg,
        &target.suffix,
        target.ty,
    ))
}

pub(super) fn summary_place_for_params(
    params: &[ResourceLocal],
    target: &Place,
) -> Option<CollectionSlotLifecycleSummaryPlace> {
    for (parameter_index, param) in params.iter().enumerate() {
        let Some(suffix) = place_suffix_after_prefix(target, &param.place) else {
            continue;
        };
        return Some(CollectionSlotLifecycleSummaryPlace {
            parameter_index,
            suffix,
            ty: target.ty,
        });
    }
    None
}
