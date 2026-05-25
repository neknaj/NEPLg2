use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_test_support::local;
use super::model::{I32ValueCondition, Place, ResourceI32RelationOp, ResourceId};

use ResourceI32RelationOp::{Eq, Lt};

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

#[test]
fn i32_relation_condition_cycle_terminates_without_losing_real_proofs() {
    let a = local("a");
    let b = local("b");
    let c = local("c");
    let d = local("d");
    let mut aliases = RawCellAddressAliases::default();

    aliases.add_i32_relation(&a, Eq, &b);
    aliases.add_i32_relation(&b, Eq, &c);
    aliases.add_i32_relation(&c, Eq, &a);
    aliases.add_i32_relation(&c, Eq, &d);
    aliases.add_i32_relation(&d, Eq, &b);
    aliases.add_i32_condition(&d, I32ValueCondition::Positive);

    assert_eq!(
        aliases.i32_condition_truth(&a, I32ValueCondition::Positive),
        Some(true)
    );
    assert_eq!(
        aliases.i32_condition_truth(&a, I32ValueCondition::Negative),
        Some(false)
    );
}

#[test]
fn i32_condition_combines_non_negative_and_non_zero_as_positive() {
    let len = local("len");
    let mut aliases = RawCellAddressAliases::default();

    aliases.add_i32_condition(&len, I32ValueCondition::NonNegative);
    aliases.add_i32_condition(&len, I32ValueCondition::NeZero);

    assert_eq!(
        aliases.i32_condition_truth(&len, I32ValueCondition::Positive),
        Some(true)
    );
    assert_eq!(
        aliases.i32_condition_truth(&len, I32ValueCondition::Negative),
        Some(false)
    );
}

#[test]
fn i32_offset_condition_derives_decremented_positive_is_non_negative() {
    let len = local("len");
    let next_len = local("next_len");
    let mut aliases = RawCellAddressAliases::default();

    aliases.add_i32_condition(&len, I32ValueCondition::NonNegative);
    aliases.add_i32_condition(&len, I32ValueCondition::NeZero);
    aliases.add_i32_offset(&len, &next_len, -1);

    assert_eq!(
        aliases.i32_condition_truth(&next_len, I32ValueCondition::NonNegative),
        Some(true)
    );
    assert_eq!(
        aliases.i32_condition_truth(&next_len, I32ValueCondition::Negative),
        Some(false)
    );
    assert_eq!(aliases.i32_relation_truth(&next_len, Lt, &len), Some(true));
}

/// offset chain で表された exact な i32 値を、条件判定にも利用できることを確認する。
///
/// 関数 summary の return path precondition は `EqZero` や `NeZero` として保存される。
/// そのため `len1 = 0 + 1`, `len2 = len1 + 1` のように offset fact だけで
/// 長さが伝搬した場合でも、条件判定が定数値まで戻って不可能な path を除外する必要がある。
#[test]
fn i32_condition_uses_exact_value_derived_from_offset_chain() {
    let zero = local("zero");
    let len1 = local("len1");
    let len2 = local("len2");
    let mut aliases = RawCellAddressAliases::default();

    aliases.set_i32_value(&zero, 0);
    aliases.add_i32_offset(&zero, &len1, 1);
    aliases.add_i32_offset(&len1, &len2, 1);

    assert_eq!(
        aliases.i32_condition_truth(&len2, I32ValueCondition::EqZero),
        Some(false)
    );
    assert_eq!(
        aliases.i32_condition_truth(&len2, I32ValueCondition::Positive),
        Some(true)
    );
}
