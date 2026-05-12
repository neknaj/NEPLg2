use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_test_support::local;
use super::model::{Place, PlaceProjection, ResourceI32RelationOp, ResourceId, ResourceOffset};

use ResourceI32RelationOp::Lt;

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
