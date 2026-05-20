use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceRoot, ResourceCallTarget};
use super::scalar_primitive::I32ArithmeticPrimitive;

pub(super) fn record_i32_scale_result(
    raw_aliases: &mut RawCellAddressAliases,
    target: &ResourceCallTarget,
    output: &Place,
    args: &[Place],
) {
    if I32ArithmeticPrimitive::from_resource_call_target(target)
        != Some(I32ArithmeticPrimitive::Mul)
    {
        return;
    }
    let [left, right] = args else {
        return;
    };
    match (
        positive_i32_scale_operand(raw_aliases, left),
        positive_i32_scale_operand(raw_aliases, right),
    ) {
        (Some(left_scale), Some(right_scale)) => {
            if left_scale.rank <= right_scale.rank {
                raw_aliases.add_i32_scale(right, output, left_scale.value);
            } else {
                raw_aliases.add_i32_scale(left, output, right_scale.value);
            }
        }
        (Some(scale), None) => {
            raw_aliases.add_i32_scale(right, output, scale.value);
        }
        (None, Some(scale)) => {
            raw_aliases.add_i32_scale(left, output, scale.value);
        }
        (None, None) => {
            if let Some(ty) = raw_aliases.i32_type_size(left) {
                raw_aliases.add_i32_type_size_scale(right, output, ty);
            } else if let Some(ty) = raw_aliases.i32_type_size(right) {
                raw_aliases.add_i32_type_size_scale(left, output, ty);
            }
        }
    }
}

fn positive_i32_value_as_usize(
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
) -> Option<usize> {
    let value = raw_aliases.i32_value(place)?;
    usize::try_from(value).ok().filter(|value| *value > 0)
}

#[derive(Clone, Copy)]
struct ScaleOperand {
    value: usize,
    rank: u8,
}

fn positive_i32_scale_operand(
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
) -> Option<ScaleOperand> {
    Some(ScaleOperand {
        value: positive_i32_value_as_usize(raw_aliases, place)?,
        rank: scale_operand_rank(raw_aliases, place),
    })
}

fn scale_operand_rank(raw_aliases: &RawCellAddressAliases, place: &Place) -> u8 {
    let canonical = raw_aliases.canonicalize_scalar(place);
    match canonical.root {
        PlaceRoot::I32Constant(_) => 0,
        PlaceRoot::Temporary(_) if canonical == *place => 0,
        PlaceRoot::Local(_) | PlaceRoot::Return | PlaceRoot::Storage(_) => 1,
        PlaceRoot::Temporary(_) => 2,
        PlaceRoot::Unknown => 3,
    }
}
