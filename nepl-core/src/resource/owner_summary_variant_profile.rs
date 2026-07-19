use super::timing::ResourceFunctionStageTimer;
use super::{model::Place, model::ResourceOp, owner_variant::PendingVariantOwnerEffectProfile};

fn authority_trace_requested(function_name: &str) -> bool {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        return *ENABLED.get_or_init(|| {
            std::env::var("NEPL_RESOURCE_AUTHORITY_TRACE").as_deref() == Ok("1")
        }) && super::timing::resource_timing_function_matches(function_name);
    }
    #[cfg(any(target_os = "none", target_arch = "wasm32"))]
    {
        let _ = function_name;
        false
    }
}

#[derive(Clone, Copy)]
pub(super) enum OwnerVariantProfilePhase {
    StateClone,
    BranchFork,
    MatchEntry,
    PathReplay,
    Terminal,
}

pub(super) struct OwnerVariantProfileTimer {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    start: Option<std::time::Instant>,
}

#[cfg_attr(any(target_os = "none", target_arch = "wasm32"), allow(dead_code))]
#[derive(Default)]
pub(super) struct OwnerVariantReturnProfile {
    enabled: bool,
    authority_trace_enabled: bool,
    nested_calls: usize,
    path_calls: usize,
    branch_forks: usize,
    match_arms: usize,
    recursive_paths: usize,
    constructed_paths: usize,
    sequential_replay_ops: usize,
    path_replay_ops: usize,
    max_depth: usize,
    state_clone_ns: u128,
    branch_fork_ns: u128,
    match_entry_ns: u128,
    sequential_replay_ns: u128,
    sequential_branch_ops: usize,
    sequential_match_ops: usize,
    sequential_other_ops: usize,
    sequential_return_control_ops: usize,
    sequential_branch_ns: u128,
    sequential_match_ns: u128,
    sequential_other_ns: u128,
    sequential_return_control_ns: u128,
    sequential_max_op_ns: u128,
    sequential_max_op_depth: usize,
    effect_consumptions: usize,
    effect_returns: usize,
    effect_parameter_returns: usize,
    effect_fresh_returns: usize,
    effect_unknown_returns: usize,
    effect_maybe_returns: usize,
    effect_temporary_sources: usize,
    effect_unreachable_variants: usize,
    effect_payload_conditions: usize,
    effect_value_conditions: usize,
    effect_scrutinee_owner_entries: usize,
    path_replay_ns: u128,
    terminal_ns: u128,
}

impl OwnerVariantReturnProfile {
    pub(super) fn new(function_name: &str) -> Self {
        let enabled = ResourceFunctionStageTimer::measurements_enabled(function_name);
        Self {
            enabled,
            authority_trace_enabled: authority_trace_requested(function_name),
            ..Self::default()
        }
    }

    pub(super) fn start(&self) -> OwnerVariantProfileTimer {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        {
            OwnerVariantProfileTimer {
                start: self.enabled.then(std::time::Instant::now),
            }
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        OwnerVariantProfileTimer {}
    }

    pub(super) fn into_enabled(self) -> Option<Self> {
        self.enabled.then_some(self)
    }

    pub(super) fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(super) fn trace_control_authority(
        &self,
        function_name: &str,
        op_index: usize,
        kind: &'static str,
        adopted: bool,
        reason: &'static str,
        reachable_paths: usize,
        depth: usize,
    ) {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        if self.authority_trace_enabled {
            std::eprintln!(
                "[resource-owner-variant-authority] function={} op_index={} kind={} decision={} reason={} reachable_paths={} depth={}",
                function_name,
                op_index,
                kind,
                if adopted { "adopted" } else { "fallback" },
                reason,
                reachable_paths,
                depth,
            );
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            let _ = kind;
            let _ = function_name;
            let _ = op_index;
            let _ = adopted;
            let _ = reason;
            let _ = reachable_paths;
            let _ = depth;
        }
    }

    pub(super) fn trace_sequential_replay(
        &self,
        function_name: &str,
        op_index: usize,
        op: &ResourceOp,
        action: &'static str,
        return_value: &Place,
        depth: usize,
    ) {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        if self.authority_trace_enabled {
            let (kind, return_control) = match op {
                ResourceOp::Branch { output, .. } => ("branch", output == return_value),
                ResourceOp::Loop { .. } => ("loop", false),
                ResourceOp::Match { output, .. } => ("match", output == return_value),
                _ => return,
            };
            std::eprintln!(
                "[resource-owner-variant-replay] function={} op_index={} kind={} action={} return_control={} depth={}",
                function_name,
                op_index,
                kind,
                action,
                return_control,
                depth,
            );
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            let _ = op;
            let _ = function_name;
            let _ = op_index;
            let _ = action;
            let _ = return_value;
            let _ = depth;
        }
    }

    pub(super) fn finish(
        &mut self,
        timer: OwnerVariantProfileTimer,
        phase: OwnerVariantProfilePhase,
    ) {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        if let Some(start) = timer.start {
            let elapsed_ns = start.elapsed().as_nanos();
            match phase {
                OwnerVariantProfilePhase::StateClone => self.state_clone_ns += elapsed_ns,
                OwnerVariantProfilePhase::BranchFork => self.branch_fork_ns += elapsed_ns,
                OwnerVariantProfilePhase::MatchEntry => self.match_entry_ns += elapsed_ns,
                OwnerVariantProfilePhase::PathReplay => self.path_replay_ns += elapsed_ns,
                OwnerVariantProfilePhase::Terminal => self.terminal_ns += elapsed_ns,
            }
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            let _ = timer;
            let _ = phase;
        }
    }

    pub(super) fn observe_nested(&mut self, depth: usize) {
        if self.enabled {
            self.nested_calls += 1;
            self.max_depth = self.max_depth.max(depth);
        }
    }

    pub(super) fn observe_path(&mut self, depth: usize) {
        if self.enabled {
            self.path_calls += 1;
            self.max_depth = self.max_depth.max(depth);
        }
    }

    pub(super) fn observe_branch_fork(&mut self) {
        self.branch_forks += usize::from(self.enabled);
    }

    pub(super) fn observe_match_arm(&mut self) {
        self.match_arms += usize::from(self.enabled);
    }

    pub(super) fn observe_recursive_path(&mut self) {
        self.recursive_paths += usize::from(self.enabled);
    }

    pub(super) fn observe_constructed_path(&mut self) {
        self.constructed_paths += usize::from(self.enabled);
    }

    pub(super) fn observe_sequential_replay(&mut self, op_count: usize) {
        if self.enabled {
            self.sequential_replay_ops += op_count;
        }
    }

    pub(super) fn finish_sequential_replay(
        &mut self,
        timer: OwnerVariantProfileTimer,
        op: &ResourceOp,
        return_value: &Place,
        depth: usize,
    ) {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        if let Some(start) = timer.start {
            let elapsed_ns = start.elapsed().as_nanos();
            self.sequential_replay_ns += elapsed_ns;
            match op {
                ResourceOp::Branch { output, .. } => {
                    self.sequential_branch_ops += 1;
                    self.sequential_branch_ns += elapsed_ns;
                    if output == return_value {
                        self.sequential_return_control_ops += 1;
                        self.sequential_return_control_ns += elapsed_ns;
                    }
                }
                ResourceOp::Match { output, .. } => {
                    self.sequential_match_ops += 1;
                    self.sequential_match_ns += elapsed_ns;
                    if output == return_value {
                        self.sequential_return_control_ops += 1;
                        self.sequential_return_control_ns += elapsed_ns;
                    }
                }
                _ => {
                    self.sequential_other_ops += 1;
                    self.sequential_other_ns += elapsed_ns;
                }
            }
            if elapsed_ns > self.sequential_max_op_ns {
                self.sequential_max_op_ns = elapsed_ns;
                self.sequential_max_op_depth = depth;
            }
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            let _ = timer;
            let _ = op;
            let _ = return_value;
            let _ = depth;
        }
    }

    pub(super) fn observe_path_replay(&mut self, op_count: usize) {
        if self.enabled {
            self.path_replay_ops += op_count;
        }
    }

    pub(super) fn observe_pending_effects(&mut self, effects: PendingVariantOwnerEffectProfile) {
        if !self.enabled {
            return;
        }
        self.effect_consumptions += effects.consumptions;
        self.effect_returns += effects.returns;
        self.effect_parameter_returns += effects.parameter_returns;
        self.effect_fresh_returns += effects.fresh_returns;
        self.effect_unknown_returns += effects.unknown_returns;
        self.effect_maybe_returns += effects.maybe_returns;
        self.effect_temporary_sources += effects.temporary_sources;
        self.effect_unreachable_variants += effects.unreachable_variants;
        self.effect_payload_conditions += effects.payload_conditions;
        self.effect_value_conditions += effects.value_conditions;
        self.effect_scrutinee_owner_entries += effects.scrutinee_owner_entries;
    }

    pub(super) fn log(self, function_name: &str) {
        if !self.enabled {
            return;
        }
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        std::eprintln!(
            "[resource-owner-variant-profile] function={} nested_calls={} path_calls={} branch_forks={} match_arms={} recursive_paths={} constructed_paths={} max_depth={} sequential_replay_ops={} sequential_branch_ops={} sequential_match_ops={} sequential_other_ops={} sequential_return_control_ops={} effect_consumptions={} effect_returns={} effect_parameter_returns={} effect_fresh_returns={} effect_unknown_returns={} effect_maybe_returns={} effect_temporary_sources={} effect_unreachable_variants={} effect_payload_conditions={} effect_value_conditions={} effect_scrutinee_owner_entries={} path_replay_ops={} state_clone_us={} branch_fork_us={} match_entry_us={} sequential_replay_us={} sequential_branch_us={} sequential_match_us={} sequential_other_us={} sequential_return_control_us={} sequential_max_op_us={} sequential_max_op_depth={} path_replay_us={} terminal_us={}",
            function_name,
            self.nested_calls,
            self.path_calls,
            self.branch_forks,
            self.match_arms,
            self.recursive_paths,
            self.constructed_paths,
            self.max_depth,
            self.sequential_replay_ops,
            self.sequential_branch_ops,
            self.sequential_match_ops,
            self.sequential_other_ops,
            self.sequential_return_control_ops,
            self.effect_consumptions,
            self.effect_returns,
            self.effect_parameter_returns,
            self.effect_fresh_returns,
            self.effect_unknown_returns,
            self.effect_maybe_returns,
            self.effect_temporary_sources,
            self.effect_unreachable_variants,
            self.effect_payload_conditions,
            self.effect_value_conditions,
            self.effect_scrutinee_owner_entries,
            self.path_replay_ops,
            self.state_clone_ns / 1_000,
            self.branch_fork_ns / 1_000,
            self.match_entry_ns / 1_000,
            self.sequential_replay_ns / 1_000,
            self.sequential_branch_ns / 1_000,
            self.sequential_match_ns / 1_000,
            self.sequential_other_ns / 1_000,
            self.sequential_return_control_ns / 1_000,
            self.sequential_max_op_ns / 1_000,
            self.sequential_max_op_depth,
            self.path_replay_ns / 1_000,
            self.terminal_ns / 1_000,
        );
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        let _ = function_name;
    }
}
