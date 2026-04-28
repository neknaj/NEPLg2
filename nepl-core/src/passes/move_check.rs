extern crate alloc;

mod alias;
mod branch_merge;
mod context_state;
mod provenance;
mod raw_memory;
mod raw_place;
mod raw_state;
mod state;
mod summary;
mod summary_build;
mod visitor;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::hir::{HirFunction, HirModule};
use crate::types::TypeId;

use raw_place::RawPlaceInfo;
use state::{BorrowBinding, BorrowCount, FieldMove, VarState};
use summary::FunctionRawAliasSummary;
use summary_build::build_function_raw_alias_summaries;
use visitor::visit_block_with_escape;

use alias::*;

struct MoveCheckContext<'m> {
    /// String literals referenced by HIR field selector expressions.
    string_literals: Vec<String>,
    /// Function parameter types after monomorphization.
    function_params: BTreeMap<String, Vec<TypeId>>,
    /// Function definitions used to specialize raw alias summaries at call sites.
    function_defs: Rc<BTreeMap<String, &'m HirFunction>>,
    /// Function return provenance summaries after monomorphization.
    function_raw_alias_summaries: BTreeMap<String, FunctionRawAliasSummary>,
    /// State of all variables currently in scope.
    /// Stack of variable states (for shadowing support).
    var_stacks: BTreeMap<String, Vec<VarState>>,
    /// Scope depth for each variable binding, aligned with `var_stacks`.
    var_depth_stacks: BTreeMap<String, Vec<usize>>,
    /// Borrow sources held by each binding, aligned with `var_stacks`.
    borrow_stacks: BTreeMap<String, Vec<Vec<BorrowBinding>>>,
    /// Non-Copy aggregate fields moved out of each binding, aligned with `var_stacks`.
    field_move_stacks: BTreeMap<String, Vec<BTreeSet<FieldMove>>>,
    /// Canonical raw-address aliases for i32 bindings, aligned with `var_stacks`.
    raw_addr_alias_stacks: BTreeMap<String, Vec<Option<String>>>,
    /// Known i32 constants for bindings, aligned with `var_stacks`.
    i32_const_stacks: BTreeMap<String, Vec<Option<i64>>>,
    /// Raw aliases held by enum payloads, aligned with `var_stacks`.
    enum_payload_raw_alias_stacks: BTreeMap<String, Vec<BTreeMap<String, String>>>,
    /// Raw aliases held by aggregate fields, aligned with `var_stacks`.
    aggregate_field_raw_alias_stacks: BTreeMap<String, Vec<BTreeMap<usize, String>>>,
    /// Function aliases held by aggregate fields, aligned with `var_stacks`.
    aggregate_field_function_alias_stacks: BTreeMap<String, Vec<BTreeMap<usize, BTreeSet<String>>>>,
    /// Aggregate-field raw aliases held by enum payloads, aligned with `var_stacks`.
    enum_payload_aggregate_field_raw_alias_stacks:
        BTreeMap<String, Vec<BTreeMap<String, BTreeMap<usize, String>>>>,
    /// Aggregate-field function aliases held by enum payloads, aligned with `var_stacks`.
    enum_payload_aggregate_field_function_alias_stacks:
        BTreeMap<String, Vec<BTreeMap<String, BTreeMap<usize, BTreeSet<String>>>>>,
    /// Function-value aliases held by enum payloads, aligned with `var_stacks`.
    enum_payload_function_alias_stacks: BTreeMap<String, Vec<BTreeMap<String, BTreeSet<String>>>>,
    /// Known function-value aliases for function-typed bindings, aligned with `var_stacks`.
    function_value_alias_stacks: BTreeMap<String, Vec<BTreeSet<String>>>,
    /// Ownership state for trackable raw memory places that carry non-Copy values.
    raw_place_states: BTreeMap<String, RawPlaceInfo>,
    /// Active borrow counts per source variable.
    borrow_counts: BTreeMap<String, BorrowCount>,
    /// Remaining variable uses in active blocks for last-use borrow release.
    use_counts: Vec<BTreeMap<String, usize>>,
    /// Diagnostics (errors) collected.
    diagnostics: Vec<Diagnostic>,
    /// Scopes for variable cleanup
    scopes: Vec<BTreeSet<String>>,
    /// Active raw alias specializations, used to stop recursive call-site expansion.
    raw_alias_specialization_stack: Vec<String>,
}

impl<'m> MoveCheckContext<'m> {
    fn clone_for_alias_summary(&self) -> Self {
        Self {
            string_literals: self.string_literals.clone(),
            function_params: self.function_params.clone(),
            function_defs: self.function_defs.clone(),
            function_raw_alias_summaries: self.function_raw_alias_summaries.clone(),
            var_stacks: self.var_stacks.clone(),
            var_depth_stacks: self.var_depth_stacks.clone(),
            borrow_stacks: self.borrow_stacks.clone(),
            field_move_stacks: self.field_move_stacks.clone(),
            raw_addr_alias_stacks: self.raw_addr_alias_stacks.clone(),
            i32_const_stacks: self.i32_const_stacks.clone(),
            enum_payload_raw_alias_stacks: self.enum_payload_raw_alias_stacks.clone(),
            aggregate_field_raw_alias_stacks: self.aggregate_field_raw_alias_stacks.clone(),
            aggregate_field_function_alias_stacks: self
                .aggregate_field_function_alias_stacks
                .clone(),
            enum_payload_aggregate_field_raw_alias_stacks: self
                .enum_payload_aggregate_field_raw_alias_stacks
                .clone(),
            enum_payload_aggregate_field_function_alias_stacks: self
                .enum_payload_aggregate_field_function_alias_stacks
                .clone(),
            enum_payload_function_alias_stacks: self.enum_payload_function_alias_stacks.clone(),
            function_value_alias_stacks: self.function_value_alias_stacks.clone(),
            raw_place_states: self.raw_place_states.clone(),
            borrow_counts: self.borrow_counts.clone(),
            use_counts: Vec::new(),
            diagnostics: Vec::new(),
            scopes: self.scopes.clone(),
            raw_alias_specialization_stack: self.raw_alias_specialization_stack.clone(),
        }
    }
}

pub fn run(module: &HirModule, types: &crate::types::TypeCtx) -> Vec<Diagnostic> {
    let function_params: BTreeMap<String, Vec<TypeId>> = module
        .functions
        .iter()
        .map(|func| {
            (
                func.name.clone(),
                func.params.iter().map(|param| param.ty).collect(),
            )
        })
        .collect();
    let function_raw_alias_summaries = build_function_raw_alias_summaries(module, types);
    let mut diagnostics = Vec::new();

    for func in &module.functions {
        let mut f_ctx = MoveCheckContext::with_function_params(
            module,
            function_params.clone(),
            function_raw_alias_summaries.clone(),
        );
        for param in &func.params {
            f_ctx.declare_param(param.name.clone());
        }

        match &func.body {
            crate::hir::HirBody::Block(b) => {
                visit_block_with_escape(b, &mut f_ctx, types, Some(0));
            }
            _ => {}
        }

        diagnostics.extend(f_ctx.diagnostics);
    }

    diagnostics
}
