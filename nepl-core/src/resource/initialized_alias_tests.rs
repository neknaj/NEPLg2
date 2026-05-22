use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_test_support::local;
use super::model::{Place, PlaceProjection, ResourceI32RelationOp, ResourceId, ResourceOffset};
use super::raw_cell_value_flow_alias::raw_cell_place_alias_candidates;

use alloc::boxed::Box;

use ResourceI32RelationOp::{Eq, Ge, Gt, Le, Lt, Ne};

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
fn i32_offset_facts_keep_parameter_alias_when_local_alias_is_cleared() {
    let source_field = local("v")
        .with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            local("v").ty,
        )
        .with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            local("v").ty,
        );
    let loaded = Place::temporary(ResourceId(1), source_field.ty);
    let local_len = local("v_len");
    let next_len = local("next_len");
    let mut aliases = RawCellAddressAliases::default();

    aliases.copy_alias_if_tracked(&source_field, &loaded);
    aliases.copy_alias_if_tracked(&loaded, &local_len);
    aliases.add_i32_offset(&local_len, &next_len, -1);
    aliases.clear(&local_len);

    assert!(
        aliases
            .i32_offset_sources(&next_len)
            .iter()
            .any(|(source, offset)| source == &source_field && *offset == -1),
        "offset facts must retain the parameter-derived source alias after a local alias is cleared"
    );
}

#[test]
fn i32_offset_facts_prove_relative_ordering() {
    let len = local("len");
    let next_len = local("next_len");
    let prev_len = local("prev_len");
    let mut aliases = RawCellAddressAliases::default();

    aliases.add_i32_offset(&len, &next_len, 1);
    aliases.add_i32_offset(&len, &prev_len, -1);

    assert_eq!(aliases.i32_relation_truth(&len, Lt, &next_len), Some(true));
    assert_eq!(aliases.i32_relation_truth(&len, Le, &next_len), Some(true));
    assert_eq!(aliases.i32_relation_truth(&next_len, Gt, &len), Some(true));
    assert_eq!(aliases.i32_relation_truth(&next_len, Ge, &len), Some(true));
    assert_eq!(aliases.i32_relation_truth(&len, Eq, &next_len), Some(false));
    assert_eq!(aliases.i32_relation_truth(&len, Ne, &next_len), Some(true));
    assert_eq!(aliases.i32_relation_truth(&len, Gt, &next_len), Some(false));
    assert_eq!(aliases.i32_relation_truth(&prev_len, Lt, &len), Some(true));
}

#[test]
fn i32_offset_facts_prove_transitive_relative_ordering() {
    let len0 = local("len0");
    let len1 = local("len1");
    let len2 = local("len2");
    let mut aliases = RawCellAddressAliases::default();

    aliases.add_i32_offset(&len0, &len1, 1);
    aliases.add_i32_offset(&len1, &len2, 1);

    assert_eq!(aliases.i32_relation_truth(&len0, Lt, &len2), Some(true));
    assert_eq!(aliases.i32_relation_truth(&len2, Gt, &len0), Some(true));
    assert_eq!(aliases.i32_relation_truth(&len0, Eq, &len2), Some(false));
    assert_eq!(aliases.i32_relation_truth(&len0, Ne, &len2), Some(true));
}

#[test]
fn i32_relation_derives_constant_through_offset_chain() {
    let len0 = local("len0");
    let len1 = local("len1");
    let len2 = local("len2");
    let len3 = local("len3");
    let zero = Place::i32_constant(0, len0.ty);
    let mut aliases = RawCellAddressAliases::default();

    aliases.set_i32_value(&len0, 0);
    aliases.add_i32_offset(&len0, &len1, 1);
    aliases.add_i32_offset(&len1, &len2, 1);
    aliases.add_i32_offset(&len2, &len3, -1);

    assert_eq!(aliases.i32_relation_truth(&zero, Lt, &len3), Some(true));
}

#[test]
fn i32_offset_fact_keeps_noncanonical_source_after_alias_copy() {
    let len0 = local("len0");
    let len1 = local("len1");
    let returned_len = local("returned_len");
    let mut aliases = RawCellAddressAliases::default();

    aliases.set_i32_value(&len0, 0);
    aliases.add_i32_offset(&len0, &len1, 1);
    aliases.copy_scalar_facts_if_tracked(&len1, &returned_len);
    aliases.add_i32_offset(&len1, &returned_len, 0);

    assert_eq!(
        aliases.i32_relation_truth(&Place::i32_constant(0, len0.ty), Lt, &returned_len),
        Some(true)
    );
    assert!(
        aliases
            .i32_offset_sources(&returned_len)
            .iter()
            .any(|(source, offset)| source == &len1 && *offset == 0),
        "zero-offset return facts must preserve the original source even after source and target become aliases"
    );
}

#[test]
fn i32_offset_assignment_clears_stale_target_alias_value() {
    let len = local("len");
    let returned_len = local("returned_len");
    let zero = Place::i32_constant(0, len.ty);
    let one = Place::i32_constant(1, len.ty);
    let mut aliases = RawCellAddressAliases::default();

    aliases.set_i32_value(&len, 2);
    aliases.copy_scalar_facts_if_tracked(&len, &returned_len);
    aliases.add_i32_offset(&len, &returned_len, -1);

    assert!(
        aliases
            .i32_offset_sources(&returned_len)
            .iter()
            .any(|(source, offset)| source == &len && *offset == -1),
        "sources={:?} targets={:?}",
        aliases.i32_offset_sources(&returned_len),
        aliases.i32_offset_targets(&len)
    );
    assert_eq!(
        aliases.i32_relation_truth(&zero, Lt, &returned_len),
        Some(true)
    );
    assert_eq!(
        aliases.i32_relation_truth(&one, Lt, &returned_len),
        Some(false),
        "an offset assignment must replace stale target aliases instead of preserving the old value"
    );
}

#[test]
fn i32_offset_facts_follow_stable_origin_queries() {
    let len = local("len");
    let len_read = Place::temporary(ResourceId(21), len.ty);
    let next_len = local("next_len");
    let mut aliases = RawCellAddressAliases::default();

    aliases.copy_alias_if_tracked(&len, &len_read);
    aliases.add_i32_offset(&len_read, &next_len, 1);

    assert_eq!(aliases.i32_relation_truth(&len, Lt, &next_len), Some(true));
    assert!(
        aliases
            .i32_offset_targets(&len)
            .iter()
            .any(|(target, offset)| target == &next_len && *offset == 1),
        "stable scalar origin queries must include facts recorded on read temporaries"
    );
}

#[test]
fn scalar_aliases_follow_transitive_stable_origin_queries() {
    let vec_buffer = local("v").with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        local("v").ty,
    );
    let vec_buffer_len = vec_buffer.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        vec_buffer.ty,
    );
    let v_buffer = local("v_buffer");
    let v_buffer_len = v_buffer.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        v_buffer.ty,
    );
    let v_len = local("v_len");
    let mut aliases = RawCellAddressAliases::default();

    aliases.copy_scalar_facts_if_tracked(&vec_buffer_len, &v_len);
    aliases.copy_scalar_facts_if_tracked(&vec_buffer, &v_buffer);

    assert!(
        aliases.scalar_aliases_for(&v_buffer_len).contains(&v_len),
        "field-level scalar aliases must follow stable origins through a copied aggregate value"
    );
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
fn owner_cell_canonicalization_prefers_storage_offset_identity() {
    let base = local("data").with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        local("data").ty,
    );
    let offset_slot = base.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        base.ty,
    );
    let mem_ptr_local = local("slot1").with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        local("slot1").ty,
    );
    let mut aliases = RawCellAddressAliases::default();

    aliases.copy_explicit_raw_address_alias(&offset_slot, &mem_ptr_local);

    assert_eq!(
        aliases.canonicalize_owner_cell_address(&mem_ptr_local),
        offset_slot
    );
}

#[test]
fn raw_cell_candidates_project_prefix_aliases_through_scaled_offsets() {
    let storage = local("storage");
    let region = local("region");
    let len = local("len");
    let target = storage
        .clone()
        .with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            storage.ty,
        )
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledOffset {
                place: Box::new(len),
                offset: -1,
                scale: 4,
            }),
            storage.ty,
        )
        .with_projection(PlaceProjection::Deref, storage.ty);
    let expected = region
        .clone()
        .with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            region.ty,
        )
        .with_projection(target.projections[1].clone(), region.ty)
        .with_projection(PlaceProjection::Deref, region.ty);
    let mut aliases = RawCellAddressAliases::default();

    aliases.copy_explicit_raw_address_alias(&storage, &region);

    assert!(
        raw_cell_place_alias_candidates(&target, &aliases).contains(&expected),
        "raw cell alias candidates must preserve deep suffixes when a storage prefix is aliased"
    );
}

#[test]
fn raw_cell_candidates_project_raw_view_origins_through_scaled_offsets() {
    let storage_view = Place::temporary(ResourceId(57), local("storage_view").ty)
        .with_projection(PlaceProjection::Deref, local("storage_view").ty);
    let region = local("region");
    let len = local("len");
    let view_len = local("buffer").with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        local("buffer").ty,
    );
    let target = storage_view
        .clone()
        .with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            storage_view.ty,
        )
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledOffset {
                place: Box::new(view_len.clone()),
                offset: -1,
                scale: 4,
            }),
            storage_view.ty,
        )
        .with_projection(PlaceProjection::Deref, storage_view.ty);
    let expected = region
        .clone()
        .with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            region.ty,
        )
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledOffset {
                place: Box::new(len.clone()),
                offset: -1,
                scale: 4,
            }),
            region.ty,
        )
        .with_projection(PlaceProjection::Deref, region.ty);
    let mut aliases = RawCellAddressAliases::default();

    aliases.record_raw_address_view_origin(&region, &storage_view);
    aliases.copy_scalar_facts_if_tracked(&len, &view_len);

    assert!(
        raw_cell_place_alias_candidates(&target, &aliases).contains(&expected),
        "raw view origins must project deep storage suffixes and canonicalize symbolic slot indices"
    );
}

#[test]
fn raw_address_summary_mark_preserves_raw_view_origins() {
    let storage_view = Place::temporary(ResourceId(57), local("storage_view").ty)
        .with_projection(PlaceProjection::Deref, local("storage_view").ty);
    let region = local("region");
    let len = local("len");
    let view_len = local("buffer").with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        local("buffer").ty,
    );
    let target = storage_view
        .clone()
        .with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            storage_view.ty,
        )
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledOffset {
                place: Box::new(view_len.clone()),
                offset: -1,
                scale: 4,
            }),
            storage_view.ty,
        )
        .with_projection(PlaceProjection::Deref, storage_view.ty);
    let expected = region
        .clone()
        .with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            region.ty,
        )
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledOffset {
                place: Box::new(len.clone()),
                offset: -1,
                scale: 4,
            }),
            region.ty,
        )
        .with_projection(PlaceProjection::Deref, region.ty);
    let mut aliases = RawCellAddressAliases::default();

    aliases.record_raw_address_view_origin(&region, &storage_view);
    aliases.copy_scalar_facts_if_tracked(&len, &view_len);
    aliases.ensure_marked(&storage_view);

    assert!(
        aliases
            .raw_address_aliases_for_value(&storage_view)
            .contains(&region),
        "summary-derived raw-address marking must not erase the view origin needed by slot proofs"
    );
    assert!(
        raw_cell_place_alias_candidates(&target, &aliases).contains(&expected),
        "summary-derived raw-address marking must preserve deep slot alias proof candidates"
    );
}

#[test]
fn raw_cell_candidates_fold_scaled_offset_source_offsets() {
    let storage = local("storage");
    let len = local("len");
    let next_len = local("next_len");
    let target = storage
        .clone()
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledOffset {
                place: Box::new(next_len.clone()),
                offset: -1,
                scale: 4,
            }),
            storage.ty,
        )
        .with_projection(PlaceProjection::Deref, storage.ty);
    let expected = storage
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
                place: Box::new(len.clone()),
                scale: 4,
            }),
            len.ty,
        )
        .with_projection(PlaceProjection::Deref, len.ty);
    let mut aliases = RawCellAddressAliases::default();

    aliases.add_i32_offset(&len, &next_len, 1);

    assert!(
        raw_cell_place_alias_candidates(&target, &aliases).contains(&expected),
        "a slot addressed as (len + 1) - 1 must canonicalize to the same symbolic len slot"
    );
}
