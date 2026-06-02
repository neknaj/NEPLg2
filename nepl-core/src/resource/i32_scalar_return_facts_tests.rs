use alloc::string::String;
use alloc::vec;

use crate::types::TypeCtx;

use super::super::initialized_alias::RawCellAddressAliases;
use super::super::model::{I32ValueCondition, Place, ResourceI32RelationOp, ResourceLocal};
use super::super::owner_summary_i32_condition_leaf::I32LeafProjectionCache;
use super::*;

/// 戻り値の中で同じ i32 値を表す複数の leaf がある場合、その等価性を
/// summary に保存して呼び出し元の戻り値にも復元できることを確認する。
///
/// Vec の len と initialized_len は別 field として戻るが、filter の成功経路では
/// どちらも同じ write index を表す。ここで relation を落とすと、後続の
/// initialized range cleanup が片方の count しか消せなくなる。
#[test]
fn i32_return_facts_preserve_equal_return_leaf_relations() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let pair_ty = types.tuple(vec![i32_ty, i32_ty]);
    let returned_value = Place::local(String::from("pair"), pair_ty);
    let mut leaf_cache = I32LeafProjectionCache::default();
    let leaves = leaf_cache.leaf_places_for_conditions(&types, &returned_value);
    let mut source_aliases = RawCellAddressAliases::default();

    source_aliases.add_i32_relation(
        &leaves[0].place,
        ResourceI32RelationOp::Eq,
        &leaves[1].place,
    );

    let facts = collect_i32_scalar_return_facts_for_value_suffix(
        &[],
        &types,
        &source_aliases,
        &returned_value,
        &[],
    );
    assert!(
        facts.relations.iter().any(|relation| {
            relation.left_return_projection == leaves[0].suffix
                && relation.op == ResourceI32RelationOp::Eq
                && relation.right_return_projection == leaves[1].suffix
        }),
        "戻り値 leaf 同士の等価 relation が summary に保存される必要がある"
    );

    let output = Place::local(String::from("out"), pair_ty);
    let mut applied_aliases = RawCellAddressAliases::default();
    assert!(apply_i32_scalar_return_facts(
        &mut applied_aliases,
        &output,
        &[],
        &facts,
        &types,
    ));
    let mut output_leaf_cache = I32LeafProjectionCache::default();
    let output_leaves = output_leaf_cache.leaf_places_for_conditions(&types, &output);
    assert_eq!(
        applied_aliases.i32_relation_truth(
            &output_leaves[0].place,
            ResourceI32RelationOp::Eq,
            &output_leaves[1].place,
        ),
        Some(true)
    );
}

/// return leaf の等価性は、直接 relation table に記録された Eq だけでなく、
/// offset graph から導出できる Eq でも summary に保存する。
///
/// collection summary では、同じ base counter から同じ offset で作った
/// len / initialized_len のような leaf が別 field として戻る。ここを直接
/// relation だけに狭めると、呼び出し元で range proof に必要な等価性が落ちる。
#[test]
fn i32_return_facts_preserve_offset_derived_equal_leaf_relations() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let pair_ty = types.tuple(vec![i32_ty, i32_ty]);
    let returned_value = Place::local(String::from("pair"), pair_ty);
    let mut leaf_cache = I32LeafProjectionCache::default();
    let leaves = leaf_cache.leaf_places_for_conditions(&types, &returned_value);
    let base = Place::local(String::from("base"), i32_ty);
    let mut source_aliases = RawCellAddressAliases::default();

    source_aliases.add_i32_offset(&base, &leaves[0].place, 1);
    source_aliases.add_i32_offset(&base, &leaves[1].place, 1);

    let facts = collect_i32_scalar_return_facts_for_value_suffix(
        &[],
        &types,
        &source_aliases,
        &returned_value,
        &[],
    );
    assert!(
        facts.relations.iter().any(|relation| {
            relation.left_return_projection == leaves[0].suffix
                && relation.op == ResourceI32RelationOp::Eq
                && relation.right_return_projection == leaves[1].suffix
        }),
        "offset graph から導出できる戻り値 leaf の等価 relation が summary に保存される必要がある"
    );
}

/// 引数 leaf に対して既に成立している i32 condition は、呼び出し元へ戻す
/// parameter condition として保存する。
///
/// condition 探索には高速化のための前提判定があるため、直接条件が存在する
/// 経路で parameter condition を落とさないことを確認する。
#[test]
fn i32_return_facts_preserve_direct_parameter_conditions() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let param = ResourceLocal {
        name: String::from("value"),
        ty: i32_ty,
        mutable: false,
        place: Place::local(String::from("value"), i32_ty),
    };
    let returned_value = Place::local(String::from("out"), i32_ty);
    let mut source_aliases = RawCellAddressAliases::default();

    source_aliases.add_i32_condition(&param.place, I32ValueCondition::NonNegative);

    let facts = collect_i32_scalar_return_facts_for_value_suffix(
        &[param.clone()],
        &types,
        &source_aliases,
        &returned_value,
        &[],
    );

    assert!(
        facts.parameter_conditions.iter().any(|condition| {
            condition.parameter_index == 0
                && condition.parameter_projection.is_empty()
                && condition.condition == I32ValueCondition::NonNegative
        }),
        "直接記録された引数 condition は summary に保存される必要がある"
    );
}

/// literal から offset graph で導ける引数 condition も、直接条件と同じく
/// parameter condition として保存する。
///
/// condition 探索の短絡は、i32 fact table が空でも offset が literal に到達する
/// ケースを安全側に残す必要がある。
#[test]
fn i32_return_facts_preserve_offset_constant_parameter_conditions() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let param = ResourceLocal {
        name: String::from("value"),
        ty: i32_ty,
        mutable: false,
        place: Place::local(String::from("value"), i32_ty),
    };
    let returned_value = Place::local(String::from("out"), i32_ty);
    let zero = Place::i32_constant(0, i32_ty);
    let mut source_aliases = RawCellAddressAliases::default();

    source_aliases.add_i32_offset(&zero, &param.place, 1);

    let facts = collect_i32_scalar_return_facts_for_value_suffix(
        &[param.clone()],
        &types,
        &source_aliases,
        &returned_value,
        &[],
    );

    assert!(
        facts.parameter_conditions.iter().any(|condition| {
            condition.parameter_index == 0
                && condition.parameter_projection.is_empty()
                && condition.condition == I32ValueCondition::Positive
        }),
        "literal 起点の offset から導ける引数 condition は summary に保存される必要がある"
    );
}
