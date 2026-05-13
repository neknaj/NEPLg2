extern crate alloc;

use crate::runtime_helpers::helper_base_name;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceCallTarget};

pub(super) fn record_direct_call_i32_facts(
    raw_aliases: &mut RawCellAddressAliases,
    target: &ResourceCallTarget,
    output: &Place,
    args: &[Place],
) {
    record_i32_constant_result(raw_aliases, target, output, args);
    record_i32_scale_result(raw_aliases, target, output, args);
    record_i32_difference_result(raw_aliases, target, output, args);
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
    let Some(value) = (match resource_call_target_base_name(target) {
        Some("add") => Some(left.wrapping_add(right)),
        Some("sub") => Some(left.wrapping_sub(right)),
        Some("mul") => Some(left.wrapping_mul(right)),
        _ => None,
    }) else {
        return;
    };
    raw_aliases.set_i32_value(output, value);
}

fn record_i32_scale_result(
    raw_aliases: &mut RawCellAddressAliases,
    target: &ResourceCallTarget,
    output: &Place,
    args: &[Place],
) {
    if resource_call_target_base_name(target) != Some("mul") {
        return;
    }
    let [left, right] = args else {
        return;
    };
    if let Some(scale) = positive_i32_value_as_usize(raw_aliases, left) {
        raw_aliases.add_i32_scale(right, output, scale);
    } else if let Some(scale) = positive_i32_value_as_usize(raw_aliases, right) {
        raw_aliases.add_i32_scale(left, output, scale);
    }
}

fn record_i32_difference_result(
    raw_aliases: &mut RawCellAddressAliases,
    target: &ResourceCallTarget,
    output: &Place,
    args: &[Place],
) {
    if resource_call_target_base_name(target) != Some("sub") {
        return;
    }
    let [minuend, subtrahend] = args else {
        return;
    };
    raw_aliases.add_i32_difference(minuend, subtrahend, output);
}

fn resource_call_target_base_name(target: &ResourceCallTarget) -> Option<&str> {
    match target {
        ResourceCallTarget::Builtin { name } | ResourceCallTarget::User { name, .. } => {
            Some(helper_base_name(name))
        }
        ResourceCallTarget::Trait { method, .. } => Some(helper_base_name(method.as_str())),
    }
}

fn positive_i32_value_as_usize(
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
) -> Option<usize> {
    let value = raw_aliases.i32_value(place)?;
    usize::try_from(value).ok().filter(|value| *value > 0)
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec, vec::Vec};

    use crate::types::TypeCtx;

    use super::super::initialized_alias::RawCellAddressAliases;
    use super::super::model::{Place, ResourceCallTarget, ResourceId};
    use super::record_direct_call_i32_facts;

    #[test]
    fn records_i32_constant_result_for_mangled_add_call() {
        let types = TypeCtx::new();
        let left = Place::temporary(ResourceId(1), types.i32());
        let right = Place::temporary(ResourceId(2), types.i32());
        let output = Place::temporary(ResourceId(3), types.i32());
        let mut raw_aliases = RawCellAddressAliases::default();

        raw_aliases.set_i32_value(&left, 12);
        raw_aliases.set_i32_value(&right, 30);
        record_direct_call_i32_facts(
            &mut raw_aliases,
            &ResourceCallTarget::User {
                name: String::from("add__i32_i32__i32__pure"),
                type_args: Vec::new(),
            },
            &output,
            &[left, right],
        );

        assert_eq!(raw_aliases.i32_value(&output), Some(42));
    }

    #[test]
    fn records_i32_scale_result_for_mangled_mul_call() {
        let types = TypeCtx::new();
        let source = Place::local(String::from("i"), types.i32());
        let source_read = Place::temporary(ResourceId(1), types.i32());
        let constant = Place::temporary(ResourceId(2), types.i32());
        let output = Place::temporary(ResourceId(3), types.i32());
        let mut raw_aliases = RawCellAddressAliases::default();

        raw_aliases.copy_alias_if_tracked(&source, &source_read);
        raw_aliases.set_i32_value(&constant, 4);
        record_direct_call_i32_facts(
            &mut raw_aliases,
            &ResourceCallTarget::User {
                name: String::from("mul__i32_i32__i32__pure"),
                type_args: Vec::new(),
            },
            &output,
            &[source_read, constant],
        );

        assert_eq!(raw_aliases.i32_scaled_source(&output), Some((source, 4)));
    }

    #[test]
    fn records_i32_difference_result_for_mangled_sub_call() {
        let types = TypeCtx::new();
        let minuend = Place::local(String::from("next_cap"), types.i32());
        let minuend_read = Place::temporary(ResourceId(1), types.i32());
        let subtrahend = Place::local(String::from("cap"), types.i32());
        let subtrahend_read = Place::temporary(ResourceId(2), types.i32());
        let output = Place::temporary(ResourceId(3), types.i32());
        let mut raw_aliases = RawCellAddressAliases::default();

        raw_aliases.copy_alias_if_tracked(&minuend, &minuend_read);
        raw_aliases.copy_alias_if_tracked(&subtrahend, &subtrahend_read);
        record_direct_call_i32_facts(
            &mut raw_aliases,
            &ResourceCallTarget::User {
                name: String::from("sub__i32_i32__i32__pure"),
                type_args: Vec::new(),
            },
            &output,
            &[minuend_read, subtrahend_read],
        );

        assert_eq!(
            raw_aliases.i32_difference_sources(&output),
            vec![(minuend, subtrahend)]
        );
    }
}
