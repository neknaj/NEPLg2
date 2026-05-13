use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_return_apply_source::owner_projection_source_place_for_arg;
use super::summary::OwnerValueCondition;

pub(super) fn owner_value_condition_truth(
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    condition: &OwnerValueCondition,
) -> Option<bool> {
    match condition {
        OwnerValueCondition::Always => Some(true),
        OwnerValueCondition::Param { source, condition } => {
            let arg = args.get(source.parameter_index)?;
            let place = owner_projection_source_place_for_arg(arg, source);
            let place = raw_aliases.canonicalize(&place);
            raw_aliases.i32_condition_truth(&place, *condition)
        }
        OwnerValueCondition::Any(conditions) => {
            let mut has_unknown = false;
            for condition in conditions {
                match owner_value_condition_truth(raw_aliases, args, condition) {
                    Some(true) => return Some(true),
                    Some(false) => {}
                    None => has_unknown = true,
                }
            }
            if has_unknown {
                None
            } else {
                Some(false)
            }
        }
        OwnerValueCondition::All(conditions) => {
            let mut has_unknown = false;
            for condition in conditions {
                match owner_value_condition_truth(raw_aliases, args, condition) {
                    Some(true) => {}
                    Some(false) => return Some(false),
                    None => has_unknown = true,
                }
            }
            if has_unknown {
                None
            } else {
                Some(true)
            }
        }
    }
}
