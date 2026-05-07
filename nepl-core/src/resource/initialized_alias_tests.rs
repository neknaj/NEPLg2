extern crate alloc;

use alloc::string::String;

use crate::types::TypeId;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceI32RelationOp, ResourceId};

use ResourceI32RelationOp::Lt;

fn local(name: &str) -> Place {
    Place::local(String::from(name), TypeId(1))
}

#[test]
fn i32_relation_facts_follow_alias_copy() {
    let source = local("i");
    let target = local("j");
    let len = local("len");
    let mut aliases = RawCellAddressAliases::default();

    aliases.add_i32_relation(&source, Lt, &len);
    aliases.copy_alias_if_tracked(&source, &target);

    assert_eq!(aliases.i32_relation_truth(&target, Lt, &len), Some(true));
}

#[test]
fn i32_scale_facts_follow_stable_value_copies() {
    let source = local("i");
    let source_read = Place::temporary(ResourceId(1), source.ty);
    let scaled_tmp = Place::temporary(ResourceId(2), source.ty);
    let scaled_local = local("off");
    let scaled_read = Place::temporary(ResourceId(3), source.ty);
    let mut aliases = RawCellAddressAliases::default();

    aliases.copy_alias_if_tracked(&source, &source_read);
    aliases.add_i32_scale(&source_read, &scaled_tmp, 4);
    aliases.copy_alias_if_tracked(&scaled_tmp, &scaled_local);
    aliases.copy_alias_if_tracked(&scaled_local, &scaled_read);

    assert_eq!(aliases.i32_scaled_source(&scaled_read), Some((source, 4)));
}

#[test]
fn i32_relation_facts_match_stable_value_origin_copies() {
    let left = local("i");
    let right = local("len");
    let right_read = Place::temporary(ResourceId(4), right.ty);
    let mut aliases = RawCellAddressAliases::default();

    aliases.copy_alias_if_tracked(&right, &right_read);
    aliases.add_i32_relation(&left, Lt, &right);

    assert_eq!(
        aliases.i32_relation_truth(&left, Lt, &right_read),
        Some(true)
    );
}

#[test]
fn i32_relation_merge_keeps_only_path_common_proofs() {
    let i = local("i");
    let len = local("len");
    let mut left = RawCellAddressAliases::default();
    let right = RawCellAddressAliases::default();

    left.add_i32_relation(&i, Lt, &len);
    let merged = RawCellAddressAliases::merge_paths(&[left.clone(), right]);
    assert_eq!(merged.i32_relation_truth(&i, Lt, &len), None);

    let merged = RawCellAddressAliases::merge_paths(&[left.clone(), left]);
    assert_eq!(merged.i32_relation_truth(&i, Lt, &len), Some(true));
}
