extern crate alloc;

use super::cell_state::place_suffix_after_address_prefix;
use super::cell_state_raw_range_model::{InitializedRawByteRange, InitializedRawRangeUnit};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    I32ValueCondition, Place, PlaceProjection, ResourceI32RelationOp, ResourceOffset,
};

pub(super) fn raw_byte_range_address_covers(
    range: &InitializedRawByteRange,
    address: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let Some(suffix) = place_suffix_after_address_prefix(address, range.address()) else {
        return false;
    };
    match suffix.as_slice() {
        [] => raw_range_count_is_positive(range, raw_aliases),
        [PlaceProjection::StorageOffset(ResourceOffset::Known(offset))] => {
            known_offset_is_in_initialized_range(*offset, range, raw_aliases)
        }
        [PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place })] => {
            if symbolic_offset_is_in_byte_range(place, range, raw_aliases) {
                return true;
            }
            let Some((source, scale)) = raw_aliases.i32_scaled_source(place) else {
                return false;
            };
            scaled_symbolic_offset_is_in_initialized_range(&source, scale, range, raw_aliases)
        }
        [PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic { place, scale })] => {
            scaled_symbolic_offset_is_in_initialized_range(place, *scale, range, raw_aliases)
        }
        _ => false,
    }
}

fn raw_range_count_is_positive(
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    raw_aliases
        .i32_value(range.count())
        .is_some_and(|count| count > 0)
        || raw_aliases.i32_condition_truth(range.count(), I32ValueCondition::Positive) == Some(true)
}

fn known_offset_is_in_initialized_range(
    offset: usize,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let Some(count) = raw_aliases.i32_value(range.count()) else {
        return false;
    };
    let Ok(count) = usize::try_from(count) else {
        return false;
    };
    match range.unit() {
        InitializedRawRangeUnit::Bytes => offset < count,
        InitializedRawRangeUnit::Elements { stride } => {
            stride > 0 && offset % stride == 0 && offset / stride < count
        }
    }
}

fn symbolic_offset_is_in_byte_range(
    place: &Place,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    if range.unit() != InitializedRawRangeUnit::Bytes {
        return false;
    }
    raw_aliases.i32_condition_truth(place, I32ValueCondition::NonNegative) == Some(true)
        && raw_aliases.i32_relation_truth(place, ResourceI32RelationOp::Lt, range.count())
            == Some(true)
}

fn scaled_symbolic_offset_is_in_initialized_range(
    place: &Place,
    scale: usize,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let InitializedRawRangeUnit::Elements { stride } = range.unit() else {
        return false;
    };
    scale == stride
        && raw_aliases.i32_condition_truth(place, I32ValueCondition::NonNegative) == Some(true)
        && raw_aliases.i32_relation_truth(place, ResourceI32RelationOp::Lt, range.count())
            == Some(true)
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::string::String;

    use crate::types::TypeId;

    use super::super::cell_state::CellTable;
    use super::super::cell_state_raw_range::InitializedRawRangeUnit;
    use super::super::initialized_alias::RawCellAddressAliases;
    use super::super::model::{
        I32ValueCondition, Place, PlaceProjection, PlaceRoot, ResourceI32RelationOp, ResourceId,
        ResourceOffset,
    };

    fn local(name: &str) -> Place {
        Place::local(String::from(name), TypeId(1))
    }

    #[test]
    fn element_range_accepts_guarded_scaled_symbolic_offset() {
        let ty = TypeId(1);
        let address = local("p");
        let len = local("len");
        let source = local("i");
        let offset = Place::temporary(ResourceId(1), ty);
        let loaded = Place {
            root: PlaceRoot::Local(String::from("p")),
            projections: alloc::vec![PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
                place: Box::new(offset.clone()),
            })],
            ty,
        };
        let mut cells = CellTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();

        cells.mark_initialized_raw_byte_range(
            &address,
            &len,
            InitializedRawRangeUnit::Elements { stride: 4 },
            ty,
        );
        raw_aliases.add_i32_scale(&source, &offset, 4);
        raw_aliases.add_i32_condition(&source, I32ValueCondition::NonNegative);
        raw_aliases.add_i32_relation(&source, ResourceI32RelationOp::Lt, &len);

        assert!(cells.raw_cell_initialized_by_byte_range(&loaded, ty, &raw_aliases));
    }
}
