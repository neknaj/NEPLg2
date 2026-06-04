extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
pub(super) use super::initialized_path_state_merge::merge_path_alternatives_into;
use super::initialized_path_state_merge::{
    dedup_resource_check_states, merge_resource_check_states,
};
use super::initialized_summary_engine::summary_check_engine;
use super::model::{Place, ResourceOp};
use super::raw_realloc::PendingRawReallocs;
use super::{
    drop_point_path::ResourceDropPointPath,
    initialized_variant::PendingVariantRawCellInitializations,
};
use crate::types::{TypeCtx, TypeKind};

const MAX_PATH_SENSITIVE_ALTERNATIVES: usize = 4;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct ResourceCheckState {
    pub(super) cells: CellTable,
    pub(super) collection_slots: CollectionSlotStateTable,
    pub(super) raw_aliases: RawCellAddressAliases,
    pub(super) function_aliases: FunctionAliasTable,
    pub(super) pending_reallocs: PendingRawReallocs,
    pub(super) variant_initializations: PendingVariantRawCellInitializations,
}

impl ResourceCheckState {
    pub(super) fn new(
        cells: CellTable,
        collection_slots: CollectionSlotStateTable,
        raw_aliases: RawCellAddressAliases,
        function_aliases: FunctionAliasTable,
        pending_reallocs: PendingRawReallocs,
        variant_initializations: PendingVariantRawCellInitializations,
    ) -> Self {
        Self {
            cells,
            collection_slots,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
        }
    }
}

#[derive(Clone, Default)]
pub(super) enum ResourcePathAlternatives {
    #[default]
    None,
    Feasible(Vec<ResourceCheckState>),
}

pub(super) fn path_states_need_replay(states: &[ResourceCheckState]) -> bool {
    let Some(first) = states.first() else {
        return false;
    };
    // Path replay は、各 feasible path に閉じた proof state を後続 operation に渡す
    // ための精密化である。CellTable と RawCellAddressAliases も単なる診断補助では
    // なく、branch/match が作った戻り値の initialized fact、raw-cell の到達性、
    // i32 の値・比較関係を保持する。merged state は安全側へ畳めるが、そのまま
    // 後続の cleanup、return、collection slot proof を実行すると、path ごとには
    // 初期化済み・範囲証明済みだった値を MaybeMoved/Uninit 扱いしてしまう。
    // そのため、表のどれかに差分が残る場合は、上限付き alternatives として
    // 後続 operation を再実行する。
    states.iter().skip(1).any(|state| {
        state.cells != first.cells
            || state.collection_slots != first.collection_slots
            || state.raw_aliases != first.raw_aliases
            || state.function_aliases != first.function_aliases
            || state.pending_reallocs != first.pending_reallocs
            || state.variant_initializations != first.variant_initializations
    })
}

#[cfg(test)]
pub(super) fn control_path_states_need_replay(
    types: &TypeCtx,
    states: &[ResourceCheckState],
    output: &Place,
) -> bool {
    control_path_states_need_replay_with_merged(types, states, output, None)
}

/// control output の path replay が必要かを、必要なら既存の merged state で判定する。
///
/// branch / match の caller は後続の直線 state へ path alternatives を必ず merge するため、
/// replay 判定が copy/unit output の maybe-moved 確認を行うと、同じ alternatives を二度
/// merge してしまう。merged_state が渡された場合はその結果を再利用し、渡されない
/// helper/test 経路では従来通り必要になった時点で merge する。
pub(super) fn control_path_states_need_replay_with_merged(
    types: &TypeCtx,
    states: &[ResourceCheckState],
    output: &Place,
    merged_state: Option<&ResourceCheckState>,
) -> bool {
    if !path_states_need_replay(states) {
        return false;
    }
    if states_have_resource_replay_relevant_difference(states) {
        return true;
    }
    if !place_type_is_unit(types, output) && !types.is_copy(output.ty) {
        return true;
    }
    let Some(merged_state) = merged_state else {
        return merge_resource_check_states(states)
            .cells
            .has_maybe_moved_non_copy_entries(types);
    };
    merged_state.cells.has_maybe_moved_non_copy_entries(types)
}

fn states_have_resource_replay_relevant_difference(states: &[ResourceCheckState]) -> bool {
    let Some(first) = states.first() else {
        return false;
    };
    states.iter().skip(1).any(|state| {
        state.collection_slots != first.collection_slots
            || state.function_aliases != first.function_aliases
            || state.pending_reallocs != first.pending_reallocs
            || !state
                .variant_initializations
                .resource_payload_entries_equal(&first.variant_initializations)
    })
}

fn place_type_is_unit(types: &TypeCtx, place: &Place) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(place.ty));
    resolved == types.unit() || matches!(types.get_ref(resolved), TypeKind::Unit)
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
pub(super) fn log_path_state_replay_reason(
    function: &str,
    label: &str,
    states: &[ResourceCheckState],
) {
    let Some(filter) = std::env::var("NEPL_RESOURCE_PATH_REPLAY_DEBUG_FUNCTION")
        .ok()
        .filter(|filter| function.contains(filter))
    else {
        return;
    };
    let _ = filter;
    let Some(first) = states.first() else {
        return;
    };
    for (index, state) in states.iter().enumerate().skip(1) {
        std::eprintln!(
            "[resource-path-replay] function={} label={} path={} cells={} collection_slots={} raw_aliases={} function_aliases={} pending_reallocs={} variant_initializations={}",
            function,
            label,
            index,
            state.cells != first.cells,
            state.collection_slots != first.collection_slots,
            state.raw_aliases != first.raw_aliases,
            state.function_aliases != first.function_aliases,
            state.pending_reallocs != first.pending_reallocs,
            state.variant_initializations != first.variant_initializations,
        );
    }
}

#[cfg(any(target_os = "none", target_arch = "wasm32"))]
pub(super) fn log_path_state_replay_reason(
    _function: &str,
    _label: &str,
    _states: &[ResourceCheckState],
) {
}

impl ResourcePathAlternatives {
    pub(super) fn from_states(mut states: Vec<ResourceCheckState>) -> Self {
        if states.len() > MAX_PATH_SENSITIVE_ALTERNATIVES {
            dedup_resource_check_states(&mut states);
        }
        // 空の候補集合は、既存の path-sensitive replay が全候補を棄却した
        // 結果として使われる。単に path-sensitive refinement が存在しない
        // 場合は、呼び出し元で `None` のままにして直線状態を保持する。
        if states.len() > MAX_PATH_SENSITIVE_ALTERNATIVES {
            // 分岐ごとの replay は診断位置を細かく保つための精密化であり、
            // 安全性そのものは merge lattice で保守的に表現できる。上限を
            // 超えた場合は全候補を一つの merged state に畳み、探索空間が
            // 後続 operation ごとに指数的に増え続けることを防ぐ。
            states = vec![merge_resource_check_states(&states)];
        }
        Self::Feasible(states)
    }

    pub(super) fn from_states_preserving_result_variants(
        states: Vec<ResourceCheckState>,
        result: &super::model::Place,
    ) -> Self {
        if states.len() <= MAX_PATH_SENSITIVE_ALTERNATIVES {
            return Self::from_states(states);
        }
        let mut groups: Vec<(Option<alloc::string::String>, Vec<ResourceCheckState>)> = Vec::new();
        for state in states {
            let variant = state
                .variant_initializations
                .concrete_variant(result)
                .map(alloc::string::String::from);
            if let Some((_, grouped)) = groups
                .iter_mut()
                .find(|(existing_variant, _)| existing_variant == &variant)
            {
                grouped.push(state);
            } else {
                groups.push((variant, vec![state]));
            }
        }
        if groups
            .iter()
            .all(|(variant, _)| variant.as_deref().is_none())
        {
            let states = groups
                .into_iter()
                .flat_map(|(_, grouped)| grouped)
                .collect::<Vec<_>>();
            return Self::from_states(states);
        }

        let mut states = Vec::new();
        for (variant, mut group) in groups {
            if group.len() > MAX_PATH_SENSITIVE_ALTERNATIVES {
                dedup_resource_check_states(&mut group);
            }
            let mut collapsed = if group.len() > MAX_PATH_SENSITIVE_ALTERNATIVES {
                merge_resource_check_states(&group)
            } else if group.len() == 1 {
                group.remove(0)
            } else {
                merge_resource_check_states(&group)
            };
            if let Some(variant) = variant {
                collapsed
                    .variant_initializations
                    .record_concrete_variant(result, &variant);
            }
            states.push(collapsed);
        }
        Self::Feasible(states)
    }

    pub(super) fn into_feasible_states(self) -> Option<Vec<ResourceCheckState>> {
        match self {
            ResourcePathAlternatives::None => None,
            ResourcePathAlternatives::Feasible(states) => Some(states),
        }
    }

    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    pub(super) fn len(&self) -> usize {
        match self {
            ResourcePathAlternatives::None => 0,
            ResourcePathAlternatives::Feasible(states) => states.len(),
        }
    }
}

impl ResourceCheckEngine<'_> {
    pub(super) fn advance_path_alternatives_after_op(
        &mut self,
        alternatives: Vec<ResourceCheckState>,
        op: &ResourceOp,
        path: ResourceDropPointPath,
    ) -> Vec<ResourceCheckState> {
        let mut out = Vec::new();
        for mut state in alternatives {
            let mut path_engine = summary_check_engine(self);
            path_engine.check_op(
                &mut state.cells,
                &mut state.collection_slots,
                &mut state.raw_aliases,
                &mut state.function_aliases,
                &mut state.pending_reallocs,
                &mut state.variant_initializations,
                op,
                path.clone(),
            );
            match core::mem::take(&mut path_engine.path_alternatives).into_feasible_states() {
                Some(path_alternatives) => out.extend(path_alternatives),
                None => out.push(state),
            }
            self.absorb_path_engine_output(path_engine);
        }
        out
    }

    fn absorb_path_engine_output(&mut self, path_engine: ResourceCheckEngine<'_>) {
        for diagnostic in path_engine.diagnostics {
            if !self.diagnostics.contains(&diagnostic) {
                self.diagnostics.push(diagnostic);
            }
        }
        for drop_point in path_engine.auto_drop_points {
            if !self.auto_drop_points.contains(&drop_point) {
                self.auto_drop_points.push(drop_point);
            }
        }
        self.deferred.branch_merges += path_engine.deferred.branch_merges;
        self.deferred.loop_merges += path_engine.deferred.loop_merges;
        self.deferred.match_merges += path_engine.deferred.match_merges;
    }
}

#[cfg(test)]
#[path = "initialized_path_state_tests.rs"]
mod tests;
