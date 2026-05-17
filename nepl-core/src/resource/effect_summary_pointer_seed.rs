use crate::types::TypeCtx;

use super::model::Place;
use super::place_utils::place_suffix_after_prefix;
use super::raw_pointer_type::type_can_seed_non_owning_raw_pointer_alias;

pub(super) fn summary_seed_can_carry_raw_pointer(
    types: Option<&TypeCtx>,
    parameter: &Place,
    seed: &Place,
) -> bool {
    let Some(types) = types else {
        return true;
    };
    type_can_seed_non_owning_raw_pointer_alias(types, seed.ty)
        || (place_suffix_after_prefix(seed, parameter).is_some()
            && type_can_seed_non_owning_raw_pointer_alias(types, parameter.ty))
}
