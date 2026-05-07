use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state::{place_suffix_after_address_prefix, raw_addresses_overlap, CellTable};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    I32ValueCondition, Place, PlaceProjection, ResourceI32RelationOp, ResourceOffset,
};
use super::place_utils::replace_place_prefix;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InitializedRawByteRange {
    address: Place,
    count: Place,
    unit: InitializedRawRangeUnit,
    ty: TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InitializedRawRangeUnit {
    Bytes,
    Elements { stride: usize },
}

impl CellTable {
    pub(super) fn initialized_raw_byte_ranges(&self) -> &[InitializedRawByteRange] {
        &self.initialized_raw_byte_ranges
    }

    pub(super) fn raw_cell_initialized_by_byte_range(
        &self,
        address: &Place,
        ty: TypeId,
        raw_aliases: &RawCellAddressAliases,
    ) -> bool {
        self.initialized_raw_byte_ranges.iter().any(|range| {
            range.ty == ty && raw_byte_range_address_covers(range, address, raw_aliases)
        })
    }

    pub(super) fn mark_initialized_raw_byte_range(
        &mut self,
        address: &Place,
        count: &Place,
        unit: InitializedRawRangeUnit,
        ty: TypeId,
    ) {
        self.clear_initialized_raw_byte_ranges_under(address);
        let range = InitializedRawByteRange {
            address: address.clone(),
            count: count.clone(),
            unit,
            ty,
        };
        if !self
            .initialized_raw_byte_ranges
            .iter()
            .any(|existing| existing == &range)
        {
            self.initialized_raw_byte_ranges.push(range);
        }
    }

    pub(super) fn clear_initialized_raw_byte_ranges_under(&mut self, address: &Place) {
        self.initialized_raw_byte_ranges
            .retain(|range| !raw_addresses_overlap(&range.address, address));
    }

    pub(super) fn copy_initialized_raw_byte_range_counts(
        &mut self,
        source: &Place,
        target: &Place,
    ) {
        let mut copied = Vec::new();
        for range in &self.initialized_raw_byte_ranges {
            let Some(count) = replace_place_prefix(&range.count, source, target) else {
                continue;
            };
            copied.push(InitializedRawByteRange {
                address: range.address.clone(),
                count,
                unit: range.unit,
                ty: range.ty,
            });
        }
        for range in copied {
            if !self.initialized_raw_byte_ranges.contains(&range) {
                self.initialized_raw_byte_ranges.push(range);
            }
        }
    }
}

impl InitializedRawByteRange {
    pub(super) fn address(&self) -> &Place {
        &self.address
    }

    pub(super) fn count(&self) -> &Place {
        &self.count
    }

    pub(super) fn unit(&self) -> InitializedRawRangeUnit {
        self.unit
    }

    pub(super) fn ty(&self) -> TypeId {
        self.ty
    }
}

pub(super) fn rekey_initialized_raw_byte_ranges(
    ranges: &mut Vec<InitializedRawByteRange>,
    source: &Place,
    target: &Place,
) {
    for range in ranges {
        if let Some(address) = replace_place_prefix(&range.address, source, target) {
            range.address = address;
        }
        if let Some(count) = replace_place_prefix(&range.count, source, target) {
            range.count = count;
        }
    }
}

pub(super) fn merge_initialized_raw_byte_ranges(
    paths: &[CellTable],
) -> Vec<InitializedRawByteRange> {
    let Some((first, rest)) = paths.split_first() else {
        return Vec::new();
    };
    first
        .initialized_raw_byte_ranges
        .iter()
        .filter(|range| {
            rest.iter()
                .all(|path| path.initialized_raw_byte_ranges.contains(range))
        })
        .cloned()
        .collect()
}

fn raw_byte_range_address_covers(
    range: &InitializedRawByteRange,
    address: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let Some(suffix) = place_suffix_after_address_prefix(address, &range.address) else {
        return false;
    };
    match suffix.as_slice() {
        [] => raw_aliases
            .i32_value(&range.count)
            .is_some_and(|count| count > 0),
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

fn known_offset_is_in_initialized_range(
    offset: usize,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let Some(count) = raw_aliases.i32_value(&range.count) else {
        return false;
    };
    let Ok(count) = usize::try_from(count) else {
        return false;
    };
    match range.unit {
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
    if range.unit != InitializedRawRangeUnit::Bytes {
        return false;
    }
    raw_aliases.i32_condition_truth(place, I32ValueCondition::NonNegative) == Some(true)
        && raw_aliases.i32_relation_truth(place, ResourceI32RelationOp::Lt, &range.count)
            == Some(true)
}

fn scaled_symbolic_offset_is_in_initialized_range(
    place: &Place,
    scale: usize,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let InitializedRawRangeUnit::Elements { stride } = range.unit else {
        return false;
    };
    scale == stride
        && raw_aliases.i32_condition_truth(place, I32ValueCondition::NonNegative) == Some(true)
        && raw_aliases.i32_relation_truth(place, ResourceI32RelationOp::Lt, &range.count)
            == Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::boxed::Box;
    use alloc::string::String;

    use super::super::model::{PlaceRoot, ResourceId};

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
