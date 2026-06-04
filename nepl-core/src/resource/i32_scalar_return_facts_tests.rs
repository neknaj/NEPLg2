use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::types::{EnumVariantInfo, TypeCtx, TypeKind};

use super::super::initialized_alias::RawCellAddressAliases;
use super::super::initialized_alias_i32_condition_context::I32ConditionQueryContext;
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

/// parameter condition の最終結果が空に確定した後でも、戻り値 fact は収集を続ける。
///
/// i32 scalar return summary では parameter condition だけが全 return path の共通部分で
/// merge される。空の path を見た後の後続 path では parameter condition 探索を省けるが、
/// return constant / alias / offset は path ごとの戻り値 fact として必要であり、同時に
/// 落としてはならない。
#[test]
fn i32_return_facts_can_skip_parameter_conditions_without_dropping_return_facts() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let param = ResourceLocal {
        name: String::from("value"),
        ty: i32_ty,
        mutable: false,
        place: Place::local(String::from("value"), i32_ty),
    };
    let returned_value = Place::local(String::from("out"), i32_ty);
    let mut leaf_cache = I32LeafProjectionCache::default();
    let mut source_aliases = RawCellAddressAliases::default();

    source_aliases.add_i32_condition(&param.place, I32ValueCondition::NonNegative);
    source_aliases.set_i32_value(&returned_value, 7);

    let facts =
        collect_i32_scalar_return_facts_for_value_suffix_cached_without_parameter_conditions(
            &[param],
            &types,
            &source_aliases,
            &returned_value,
            &[],
            &mut leaf_cache,
            |_| true,
        );

    assert!(
        facts
            .constants
            .iter()
            .any(|constant| { constant.return_projection.is_empty() && constant.value == 7 }),
        "parameter condition を省略しても戻り値 constant fact は保持する必要がある"
    );
    assert!(
        facts.parameter_conditions.is_empty(),
        "空確定後の path では parameter condition を収集しない"
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

/// condition 探索の事前判定は、raw alias graph 全体ではなく対象 value に届く
/// scalar fact だけを見て true にする。
///
/// RPN の StringBuilder / ByteBuilder 系では、同じ関数内に無関係な i32 fact が
/// 多数存在する。ここを global な「何か証明できる」判定に戻すと、cold leaf ごとに
/// 全 condition を照会し直し、静的検査の固定費が増える。
#[test]
fn i32_condition_gate_ignores_unrelated_scalar_facts() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let cold = Place::local(String::from("cold"), i32_ty);
    let direct = Place::local(String::from("direct"), i32_ty);
    let offset = Place::local(String::from("offset"), i32_ty);
    let relation_left = Place::local(String::from("relation_left"), i32_ty);
    let relation_right = Place::local(String::from("relation_right"), i32_ty);
    let scale_source = Place::local(String::from("scale_source"), i32_ty);
    let scale_target = Place::local(String::from("scale_target"), i32_ty);
    let unrelated = Place::local(String::from("unrelated"), i32_ty);
    let mut aliases = RawCellAddressAliases::default();

    aliases.add_i32_condition(&unrelated, I32ValueCondition::NonNegative);
    assert!(
        !aliases.can_prove_i32_value_condition_for_value_with_context(
            &cold,
            &mut I32ConditionQueryContext::default()
        ),
        "無関係な i32 condition だけでは cold leaf の探索を開始しない"
    );

    aliases.add_i32_condition(&direct, I32ValueCondition::NonNegative);
    assert!(
        aliases.can_prove_i32_value_condition_for_value_with_context(
            &direct,
            &mut I32ConditionQueryContext::default()
        ),
        "対象 value 自身の direct condition は探索対象として扱う"
    );

    aliases.add_i32_offset(&Place::i32_constant(0, i32_ty), &offset, 1);
    assert!(
        aliases.can_prove_i32_value_condition_for_value_with_context(
            &offset,
            &mut I32ConditionQueryContext::default()
        ),
        "対象 value に届く offset fact は探索対象として扱う"
    );

    aliases.add_i32_relation(&relation_left, ResourceI32RelationOp::Eq, &relation_right);
    assert!(
        aliases.can_prove_i32_value_condition_for_value_with_context(
            &relation_left,
            &mut I32ConditionQueryContext::default()
        ),
        "対象 value に届く relation fact は探索対象として扱う"
    );

    aliases.add_i32_scale(&scale_source, &scale_target, 4);
    assert!(
        aliases.can_prove_i32_value_condition_for_value_with_context(
            &scale_target,
            &mut I32ConditionQueryContext::default()
        ),
        "対象 value に届く scale fact は探索対象として扱う"
    );
}

/// concrete variant などの上位解析で到達不能だと分かっている return leaf は、
/// fact 収集の前に除外できる。
///
/// Result の Ok / Err のような sibling variant payload を後から捨てるのではなく、
/// 不可能な projection への condition 探索を開始しないことを確認する。filter が
/// 不明な場合は caller が常に true を返すため、従来の保守的な全探索に戻る。
#[test]
fn i32_return_facts_projection_filter_skips_impossible_leaf() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let pair_ty = types.tuple(vec![i32_ty, i32_ty]);
    let returned_value = Place::local(String::from("pair"), pair_ty);
    let mut leaf_cache = I32LeafProjectionCache::default();
    let leaves = leaf_cache.leaf_places_for_conditions(&types, &returned_value);
    let mut source_aliases = RawCellAddressAliases::default();

    source_aliases.set_i32_value(&leaves[0].place, 10);
    source_aliases.set_i32_value(&leaves[1].place, 20);

    let facts = collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter(
        &[],
        &types,
        &source_aliases,
        &returned_value,
        &[],
        &mut leaf_cache,
        |projection| projection == leaves[0].suffix.as_slice(),
    );

    assert!(
        facts.constants.iter().any(|constant| {
            constant.return_projection == leaves[0].suffix && constant.value == 10
        }),
        "到達可能な leaf の constant fact は保持する必要がある"
    );
    assert!(
        !facts.constants.iter().any(|constant| {
            constant.return_projection == leaves[1].suffix && constant.value == 20
        }),
        "到達不能な leaf の constant fact は収集しない"
    );
}

/// filter が不明な場合は、到達不能かもしれない projection も保守的に残す。
///
/// concrete variant がまだ分からない時点で sibling payload を削ると、後続の path merge で
/// 別 path から戻る payload fact を失う。caller が常に true を返す fail-open 経路では、
/// 従来と同じ全 leaf 収集になることを固定する。
#[test]
fn i32_return_facts_projection_filter_fail_open_keeps_all_leaves() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let result_ty = result_i32_i32_type(&mut types, i32_ty);
    let returned_value = Place::local(String::from("result"), result_ty);
    let mut leaf_cache = I32LeafProjectionCache::default();
    let leaves = leaf_cache.leaf_places_for_conditions(&types, &returned_value);
    let mut source_aliases = RawCellAddressAliases::default();

    source_aliases.set_i32_value(&leaves[0].place, 10);
    source_aliases.set_i32_value(&leaves[1].place, 20);

    let facts = collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter(
        &[],
        &types,
        &source_aliases,
        &returned_value,
        &[],
        &mut leaf_cache,
        |_| true,
    );

    assert!(
        facts.constants.iter().any(|constant| {
            constant.return_projection == leaves[0].suffix && constant.value == 10
        }),
        "fail-open では先頭 variant payload の constant fact を保持する必要がある"
    );
    assert!(
        facts.constants.iter().any(|constant| {
            constant.return_projection == leaves[1].suffix && constant.value == 20
        }),
        "fail-open では sibling variant payload の constant fact も保持する必要がある"
    );
}

/// return leaf の filter は、戻り値 leaf の探索だけを狭め、引数 condition は落とさない。
///
/// 引数 condition は呼び出し元の事前条件として扱う fact であり、戻り値の concrete variant
/// payload が片側だけ可能な場合でも、関数本体で要求される parameter condition は
/// summary に残す必要がある。
#[test]
fn i32_return_facts_projection_filter_preserves_parameter_conditions() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let result_ty = result_i32_i32_type(&mut types, i32_ty);
    let returned_value = Place::local(String::from("result"), result_ty);
    let param = ResourceLocal {
        name: String::from("value"),
        ty: i32_ty,
        mutable: false,
        place: Place::local(String::from("value"), i32_ty),
    };
    let mut leaf_cache = I32LeafProjectionCache::default();
    let leaves = leaf_cache.leaf_places_for_conditions(&types, &returned_value);
    let mut source_aliases = RawCellAddressAliases::default();

    source_aliases.add_i32_condition(&param.place, I32ValueCondition::NonNegative);
    source_aliases.set_i32_value(&leaves[0].place, 10);
    source_aliases.set_i32_value(&leaves[1].place, 20);

    let facts = collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter(
        &[param.clone()],
        &types,
        &source_aliases,
        &returned_value,
        &[],
        &mut leaf_cache,
        |projection| projection == leaves[0].suffix.as_slice(),
    );

    assert!(
        !facts.constants.iter().any(|constant| {
            constant.return_projection == leaves[1].suffix && constant.value == 20
        }),
        "到達不能な sibling payload の戻り値 fact は収集しない"
    );
    assert!(
        facts.parameter_conditions.iter().any(|condition| {
            condition.parameter_index == 0
                && condition.parameter_projection.is_empty()
                && condition.condition == I32ValueCondition::NonNegative
        }),
        "戻り値 projection filter は引数 condition を削ってはならない"
    );
}

/// 到達可能な leaf の offset 由来 relation は保持し、到達不能 sibling leaf だけを落とす。
///
/// collection summary では offset graph から戻り値 leaf の等価性を復元するため、
/// filter は「到達不能 leaf を候補に入れない」だけに留め、到達可能 leaf 同士の
/// relation proof を壊してはならない。
#[test]
fn i32_return_facts_projection_filter_keeps_possible_offset_relation() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let ok_pair_ty = types.tuple(vec![i32_ty, i32_ty]);
    let result_ty = types.register_named(
        String::from("Result"),
        TypeKind::Enum {
            name: String::from("Result"),
            type_params: Vec::new(),
            variants: vec![
                EnumVariantInfo {
                    name: String::from("Ok"),
                    payload: Some(ok_pair_ty),
                },
                EnumVariantInfo {
                    name: String::from("Err"),
                    payload: Some(i32_ty),
                },
            ],
        },
    );
    let returned_value = Place::local(String::from("result"), result_ty);
    let mut leaf_cache = I32LeafProjectionCache::default();
    let leaves = leaf_cache.leaf_places_for_conditions(&types, &returned_value);
    let base = Place::local(String::from("base"), i32_ty);
    let mut source_aliases = RawCellAddressAliases::default();
    let ok_first = leaves
        .iter()
        .find(|leaf| {
            matches!(
                leaf.suffix.first(),
                Some(PlaceProjection::EnumPayload { variant }) if variant == "Ok"
            ) && matches!(
                leaf.suffix.get(1),
                Some(PlaceProjection::TupleField { index: 0, .. })
            )
        })
        .expect("Ok first tuple leaf must exist");
    let ok_second = leaves
        .iter()
        .find(|leaf| {
            matches!(
                leaf.suffix.first(),
                Some(PlaceProjection::EnumPayload { variant }) if variant == "Ok"
            ) && matches!(
                leaf.suffix.get(1),
                Some(PlaceProjection::TupleField { index: 1, .. })
            )
        })
        .expect("Ok second tuple leaf must exist");
    let err_leaf = leaves
        .iter()
        .find(|leaf| {
            matches!(
                leaf.suffix.first(),
                Some(PlaceProjection::EnumPayload { variant }) if variant == "Err"
            )
        })
        .expect("Err payload leaf must exist");

    source_aliases.add_i32_offset(&base, &ok_first.place, 1);
    source_aliases.add_i32_offset(&base, &ok_second.place, 1);
    source_aliases.add_i32_offset(&base, &err_leaf.place, 1);

    let facts = collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter(
        &[],
        &types,
        &source_aliases,
        &returned_value,
        &[],
        &mut leaf_cache,
        |projection| {
            matches!(
                projection.first(),
                Some(PlaceProjection::EnumPayload { variant }) if variant == "Ok"
            )
        },
    );

    assert!(
        facts.relations.iter().any(|relation| {
            relation.left_return_projection == ok_first.suffix
                && relation.op == ResourceI32RelationOp::Eq
                && relation.right_return_projection == ok_second.suffix
        }),
        "到達可能な Ok payload leaf 同士の offset 由来 Eq relation は保持する必要がある"
    );
    assert!(
        !facts.relations.iter().any(|relation| {
            relation.left_return_projection == err_leaf.suffix
                || relation.right_return_projection == err_leaf.suffix
        }),
        "到達不能な Err payload leaf を含む relation は収集しない"
    );
}

fn result_i32_i32_type(types: &mut TypeCtx, i32_ty: crate::types::TypeId) -> crate::types::TypeId {
    types.register_named(
        String::from("Result"),
        TypeKind::Enum {
            name: String::from("Result"),
            type_params: Vec::new(),
            variants: vec![
                EnumVariantInfo {
                    name: String::from("Ok"),
                    payload: Some(i32_ty),
                },
                EnumVariantInfo {
                    name: String::from("Err"),
                    payload: Some(i32_ty),
                },
            ],
        },
    )
}
