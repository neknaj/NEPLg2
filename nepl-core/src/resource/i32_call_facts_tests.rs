extern crate alloc;

use alloc::{string::String, vec, vec::Vec};

use crate::types::TypeCtx;

use super::i32_call_facts::record_direct_call_i32_facts;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceCallTarget, ResourceId};

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

#[test]
fn records_i32_offset_result_for_mangled_add_call_with_constant() {
    let types = TypeCtx::new();
    let source = Place::local(String::from("off"), types.i32());
    let constant = Place::temporary(ResourceId(1), types.i32());
    let output = Place::temporary(ResourceId(2), types.i32());
    let mut raw_aliases = RawCellAddressAliases::default();

    raw_aliases.set_i32_value(&constant, 24);
    record_direct_call_i32_facts(
        &mut raw_aliases,
        &ResourceCallTarget::User {
            name: String::from("add__i32_i32__i32__pure"),
            type_args: Vec::new(),
        },
        &output,
        &[source.clone(), constant],
    );

    assert_eq!(raw_aliases.i32_offset_targets(&source), vec![(output, 24)]);
}

#[test]
fn records_i32_offset_for_symbolic_add_even_when_source_value_is_known() {
    let types = TypeCtx::new();
    let source = Place::local(String::from("off"), types.i32());
    let constant = Place::temporary(ResourceId(1), types.i32());
    let output = Place::temporary(ResourceId(2), types.i32());
    let mut raw_aliases = RawCellAddressAliases::default();

    raw_aliases.set_i32_value(&source, 0);
    raw_aliases.set_i32_value(&constant, 4);
    record_direct_call_i32_facts(
        &mut raw_aliases,
        &ResourceCallTarget::User {
            name: String::from("add__i32_i32__i32__pure"),
            type_args: Vec::new(),
        },
        &output,
        &[source.clone(), constant],
    );

    assert_eq!(raw_aliases.i32_offset_targets(&source), vec![(output, 4)]);
}
