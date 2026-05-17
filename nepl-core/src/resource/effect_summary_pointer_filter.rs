use super::effect_summary::RawPointerReturnSummary;
use super::effect_summary_projection::summary_projection_is_valid;
use super::model::{Place, ResourceFunction, ResourceId};
use crate::types::TypeCtx;

pub(super) fn filter_raw_pointer_return_summary(
    summary: &mut RawPointerReturnSummary,
    function: &ResourceFunction,
    types: Option<&TypeCtx>,
) {
    let Some(types) = types else {
        return;
    };
    let return_place = Place::temporary(ResourceId(usize::MAX), function.result);
    summary.parameter_returns.retain(|item| {
        let Some(parameter) = function.params.get(item.parameter_index) else {
            return false;
        };
        summary_projection_is_valid(
            types,
            &parameter.place,
            &item.source_projections,
            item.source_ty,
        ) && summary_projection_is_valid(
            types,
            &return_place,
            &item.return_projections,
            item.return_ty,
        )
    });
}
