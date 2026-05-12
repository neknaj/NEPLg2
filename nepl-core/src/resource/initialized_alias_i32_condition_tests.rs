use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_test_support::local;
use super::model::{I32ValueCondition, Place, ResourceI32RelationOp, ResourceId};

use ResourceI32RelationOp::Lt;

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
