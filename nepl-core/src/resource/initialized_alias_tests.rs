extern crate alloc;

use alloc::string::String;

use crate::types::TypeId;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    I32ValueCondition, Place, PlaceProjection, ResourceI32RelationOp, ResourceId, ResourceOffset,
};

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
fn raw_address_view_origin_does_not_replace_scalar_value_origin() {
    let i = local("i");
    let i_read = Place::temporary(ResourceId(1), i.ty);
    let view_source = i_read.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Unknown),
        i.ty,
    );
    let sub_output = Place::temporary(ResourceId(2), i.ty);
    let im1 = local("im1");
    let im1_read = Place::temporary(ResourceId(3), i.ty);
    let prev_off = Place::temporary(ResourceId(4), i.ty);
    let len = local("len");
    let mut aliases = RawCellAddressAliases::default();

    aliases.copy_alias_if_tracked(&i, &i_read);
    aliases.record_raw_address_view_origin(&view_source, &sub_output);
    aliases.copy_alias_if_tracked(&sub_output, &im1);
    aliases.copy_alias_if_tracked(&im1, &im1_read);
    aliases.add_i32_scale(&im1_read, &prev_off, 4);
    aliases.add_i32_relation(&im1, Lt, &len);

    assert_eq!(
        aliases.canonicalize(&sub_output),
        i.clone().with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Unknown),
            i.ty
        )
    );
    assert_eq!(aliases.canonicalize_scalar(&im1), im1);
    assert_eq!(aliases.i32_scaled_source(&prev_off), Some((im1.clone(), 4)));
    assert_eq!(aliases.i32_relation_truth(&im1_read, Lt, &len), Some(true));
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

#[test]
fn i32_scaled_relation_condition_derives_checked_size_non_negative() {
    let idx = local("idx");
    let argc = local("argc");
    let argc_read = Place::temporary(ResourceId(1), argc.ty);
    let size_tmp = Place::temporary(ResourceId(2), argc.ty);
    let size_local = local("argv_size");
    let size_read = Place::temporary(ResourceId(3), argc.ty);
    let mut aliases = RawCellAddressAliases::default();

    aliases.add_i32_condition(&idx, I32ValueCondition::NonNegative);
    aliases.add_i32_relation(&idx, Lt, &argc);
    aliases.copy_alias_if_tracked(&argc, &argc_read);
    aliases.add_i32_scale(&argc_read, &size_tmp, 4);
    aliases.copy_alias_if_tracked(&size_tmp, &size_local);
    aliases.copy_alias_if_tracked(&size_local, &size_read);

    assert_eq!(
        aliases.i32_condition_truth(&size_read, I32ValueCondition::NonNegative),
        Some(true)
    );
    assert_eq!(
        aliases.i32_condition_truth(&size_read, I32ValueCondition::Negative),
        Some(false)
    );
}
