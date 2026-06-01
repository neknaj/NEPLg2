use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::initialized_alias_i32_condition_context::I32ConditionQueryContext;
use super::model::Place;

pub(super) type ScalarAliasMemo = BTreeMap<Place, Vec<Place>>;
pub(super) type I32OffsetSourceMemo = BTreeMap<Place, Vec<(Place, i64)>>;
pub(super) type I32OffsetTargetMemo = BTreeMap<Place, Vec<(Place, i64)>>;
pub(super) type I32OffsetReachableMemo = BTreeMap<Place, Vec<(Place, i64)>>;

impl I32ConditionQueryContext {
    pub(super) fn scalar_aliases(&self, place: &Place) -> Option<Vec<Place>> {
        self.scalar_alias_memo.get(place).cloned()
    }

    pub(super) fn memoize_scalar_aliases(&mut self, place: &Place, aliases: Vec<Place>) {
        self.scalar_alias_memo.insert(place.clone(), aliases);
    }

    pub(super) fn offset_sources(&self, place: &Place) -> Option<Vec<(Place, i64)>> {
        self.offset_source_memo.get(place).cloned()
    }

    pub(super) fn memoize_offset_sources(&mut self, place: &Place, sources: Vec<(Place, i64)>) {
        self.offset_source_memo.insert(place.clone(), sources);
    }

    pub(super) fn offset_targets(&self, place: &Place) -> Option<Vec<(Place, i64)>> {
        self.offset_target_memo.get(place).cloned()
    }

    pub(super) fn memoize_offset_targets(&mut self, place: &Place, targets: Vec<(Place, i64)>) {
        self.offset_target_memo.insert(place.clone(), targets);
    }

    pub(super) fn offset_reachable(&self, place: &Place) -> Option<Vec<(Place, i64)>> {
        self.offset_reachable_memo.get(place).cloned()
    }

    pub(super) fn memoize_offset_reachable(&mut self, place: &Place, reachable: Vec<(Place, i64)>) {
        self.offset_reachable_memo.insert(place.clone(), reachable);
    }
}
