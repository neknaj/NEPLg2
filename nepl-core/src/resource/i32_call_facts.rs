use super::i32_call_facts_scale::record_i32_scale_result;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceCallTarget};
use super::scalar_primitive::I32ArithmeticPrimitive;

pub(super) fn record_direct_call_i32_facts(
    raw_aliases: &mut RawCellAddressAliases,
    target: &ResourceCallTarget,
    output: &Place,
    args: &[Place],
) {
    record_i32_scale_result(raw_aliases, target, output, args);
    record_i32_offset_result(raw_aliases, target, output, args);
    record_i32_difference_result(raw_aliases, target, output, args);
    record_i32_constant_result(raw_aliases, target, output, args);
}
fn record_i32_constant_result(
    raw_aliases: &mut RawCellAddressAliases,
    target: &ResourceCallTarget,
    output: &Place,
    args: &[Place],
) {
    let [left, right] = args else {
        return;
    };
    let (Some(left), Some(right)) = (raw_aliases.i32_value(left), raw_aliases.i32_value(right))
    else {
        return;
    };
    let Some(op) = I32ArithmeticPrimitive::from_resource_call_target(target) else {
        return;
    };
    let value = op.wrapping_i32(left, right);
    raw_aliases.set_i32_value(output, value);
}

fn record_i32_difference_result(
    raw_aliases: &mut RawCellAddressAliases,
    target: &ResourceCallTarget,
    output: &Place,
    args: &[Place],
) {
    if I32ArithmeticPrimitive::from_resource_call_target(target)
        != Some(I32ArithmeticPrimitive::Sub)
    {
        return;
    }
    let [minuend, subtrahend] = args else {
        return;
    };
    raw_aliases.add_i32_difference(minuend, subtrahend, output);
}

fn record_i32_offset_result(
    raw_aliases: &mut RawCellAddressAliases,
    target: &ResourceCallTarget,
    output: &Place,
    args: &[Place],
) {
    let [left, right] = args else {
        return;
    };
    match I32ArithmeticPrimitive::from_resource_call_target(target) {
        Some(I32ArithmeticPrimitive::Add) => {
            if let Some(offset) = raw_aliases.i32_value(left).map(i64::from) {
                raw_aliases.add_i32_offset(right, output, offset);
            }
            if let Some(offset) = raw_aliases.i32_value(right).map(i64::from) {
                raw_aliases.add_i32_offset(left, output, offset);
            }
        }
        Some(I32ArithmeticPrimitive::Sub) => {
            if let Some(offset) = raw_aliases.i32_value(right).map(i64::from) {
                raw_aliases.add_i32_offset(left, output, -offset);
            }
        }
        Some(I32ArithmeticPrimitive::Mul) | None => {}
    }
}
