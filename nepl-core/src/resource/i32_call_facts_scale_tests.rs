extern crate alloc;

use alloc::{string::String, vec::Vec};

use crate::types::TypeCtx;

use super::i32_call_facts::record_direct_call_i32_facts;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceCallTarget, ResourceId};

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
fn records_i32_scale_result_preferring_literal_scale_over_known_index() {
    let types = TypeCtx::new();
    let source = Place::local(String::from("i"), types.i32());
    let source_read = Place::temporary(ResourceId(1), types.i32());
    let constant = Place::temporary(ResourceId(2), types.i32());
    let output = Place::temporary(ResourceId(3), types.i32());
    let mut raw_aliases = RawCellAddressAliases::default();

    raw_aliases.set_i32_value(&source, 2);
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
