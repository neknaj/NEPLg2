use alloc::vec::Vec;

use super::initialized_alias_i32_condition_context::I32ConditionQueryContext;
use super::model::Place;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct ScalarAliasMemo {
    place: Place,
    aliases: Vec<Place>,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct I32OffsetSourceMemo {
    place: Place,
    sources: Vec<(Place, i64)>,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct I32OffsetTargetMemo {
    place: Place,
    targets: Vec<(Place, i64)>,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct I32OffsetReachableMemo {
    place: Place,
    reachable: Vec<(Place, i64)>,
}

impl I32ConditionQueryContext {
    pub(super) fn scalar_aliases(&self, place: &Place) -> Option<Vec<Place>> {
        self.scalar_alias_memo
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.aliases.clone())
    }

    pub(super) fn memoize_scalar_aliases(&mut self, place: &Place, aliases: Vec<Place>) {
        self.scalar_alias_memo.push(ScalarAliasMemo {
            place: place.clone(),
            aliases,
        });
    }

    pub(super) fn offset_sources(&self, place: &Place) -> Option<Vec<(Place, i64)>> {
        self.offset_source_memo
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.sources.clone())
    }

    pub(super) fn memoize_offset_sources(&mut self, place: &Place, sources: Vec<(Place, i64)>) {
        self.offset_source_memo.push(I32OffsetSourceMemo {
            place: place.clone(),
            sources,
        });
    }

    pub(super) fn offset_targets(&self, place: &Place) -> Option<Vec<(Place, i64)>> {
        self.offset_target_memo
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.targets.clone())
    }

    pub(super) fn memoize_offset_targets(&mut self, place: &Place, targets: Vec<(Place, i64)>) {
        self.offset_target_memo.push(I32OffsetTargetMemo {
            place: place.clone(),
            targets,
        });
    }

    pub(super) fn offset_reachable(&self, place: &Place) -> Option<Vec<(Place, i64)>> {
        self.offset_reachable_memo
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.reachable.clone())
    }

    pub(super) fn memoize_offset_reachable(&mut self, place: &Place, reachable: Vec<(Place, i64)>) {
        self.offset_reachable_memo.push(I32OffsetReachableMemo {
            place: place.clone(),
            reachable,
        });
    }
}
