#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const selfhostRoot = path.join(repoRoot, "stdlib", "neplg2");
const DOC_GAP_TRACKING_ISSUE = "issues/items/ISS-20260605T150033175Z-SELFHOST-COMPILER-DOC-COMMENTS-NEED--FF439E41.md";

const BASELINE = {
    moduleNoDoc: 77,
    moduleNoDoctest: 60,
    declarationNoDoc: 304,
    declarationNoDoctest: 1434,
    publicNoDoc: 51,
    publicNoDoctest: 1239,
    privateNoDoc: 253,
    privateNoDoctest: 195,
};
const HARD_DOC_BASELINE_KEYS = [
    "moduleNoDoc",
    "declarationNoDoc",
    "publicNoDoc",
    "privateNoDoc",
];
const REPORT_ONLY_DOCTEST_BASELINE_KEYS = [
    "moduleNoDoctest",
    "declarationNoDoctest",
    "publicNoDoctest",
    "privateNoDoctest",
];

const PUBLIC_DOC_REQUIRED_PREFIXES = [
    "stdlib/neplg2/cli/args/emit.nepl",
    "stdlib/neplg2/core/check/expr/argument.nepl",
    "stdlib/neplg2/core/check/expr/ascription.nepl",
    "stdlib/neplg2/core/check/expr/call_reduce.nepl",
    "stdlib/neplg2/core/check/module/",
    "stdlib/neplg2/core/hir/hir/expr.nepl",
    "stdlib/neplg2/core/proof/solver/resource.nepl",
    "stdlib/neplg2/core/syntax/lexer/",
];
const REQUIRED_SCANNER_SENTINELS = [
    "stdlib/neplg2/cli/args/emit.nepl",
    "stdlib/neplg2/core/check/module/summary.nepl",
    "stdlib/neplg2/core/check/module/declaration_adapter.nepl",
    "stdlib/neplg2/core/hir/hir/expr.nepl",
    "stdlib/neplg2/core/proof/solver/resource.nepl",
    "stdlib/neplg2/core/syntax/lexer/byte.nepl",
];
const DOC_SECTION_REQUIREMENTS = [
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_new", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_empty", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_all", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_add", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "SelfhostExprArgumentMatchErrorKind", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "SelfhostExprArgumentMatchError", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "SelfhostExprArgumentOwnedMatch", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_match", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_checked_argument", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_expected_type_is_function", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_function_value_error_from_candidate_collect", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_function_value_candidate_is_monomorphic", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_function_value_candidate", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_function_value_candidates", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_function_value_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_range_from_prefix", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_find_prefix_item_by_token_loop", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_find_prefix_item_by_token", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_validate_ascription_expected", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_span_from_ascription_error", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_ascribed_with_projection", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_ascribed_at_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_at_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "SelfhostExprAscriptionError", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "SelfhostExprAscriptionProjection", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "SelfhostExprAscriptionHeadProjection", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_expectation", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_tail", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_type_id", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_expectation", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_expression_first_token", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_first_token_is_percent", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_push_type_item", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_type_items_loop", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_type_prefix_list_from_range", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_type_span_from_range", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_tail_span_from_tokens", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_expression_tail_range", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_expression_first_token", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_reduced", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_head_reduced", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_expectation", ["purpose", "contract", "returns", "complexity", "doctest"], {
        doctestUses: [
            "selfhost_expr_ascription_project_expectation",
            "selfhost_expr_ascription_projection_tail",
            "selfhost_expr_ascription_projection_free",
        ],
    }),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_head_expectation", ["purpose", "contract", "returns", "complexity", "doctest"], {
        doctestUses: [
            "selfhost_expr_ascription_project_head_expectation",
            "selfhost_expr_ascription_head_projection_expression_first_token",
            "selfhost_expr_ascription_head_projection_free",
        ],
    }),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_expectation_with_constructors", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/block_body.nepl", "selfhost_block_body_result_segment_span", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/block_body.nepl", "selfhost_block_body_result_from_expression_segment", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/block_body.nepl", "selfhost_block_body_result_from_single_segment", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/block_body.nepl", "selfhost_block_body_result_from_segment_list", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/body_line.nepl", "selfhost_check_expr_syntax_range_span", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/body_line.nepl", "selfhost_check_expr_head_starts_with_percent", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/body_line.nepl", "selfhost_check_expr_reduce_body_segment_with_projected_ascription", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_make_prefix", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_make_prefix_with_first_arg", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_make_prefix_with_ascribed_first_arg", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_make_candidate_vec", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_reduce", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_error_is", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_direct_ok", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_partial_rejected", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_expected_rejected", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_argument_type_rejected", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_ascribed_argument_unsupported", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_generic_rejected", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_ambiguous_rejected", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_add_two_i32_function", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_scope", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_value_types", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_signatures", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_empty_value_context", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_with_binding_only", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_with_typed_value", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_with_function", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_with_shadowed_function_value", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_make_named_candidate_vec", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_make_candidate_vec", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_collect_candidates_from_fixture_scope", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_add_two_i32_function", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_add_one_i32_function", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_add_function_value_consumer", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_function_value_argument_segment", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_implicit_function_value_argument_segment", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_make_function_value_argument_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_make_implicit_function_value_argument_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_shadowed_function_argument_uses_value_evidence", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_success_is_two_arg_direct_call", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_success_is_one_arg_direct_call", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_success_has_function_value_argument", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_segment_span", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_reduce_body_segment_with_value_context", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_reduce_block_intro_with_value_context", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_reduce_body_segment_with_empty_values", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_reduce_block_intro_with_empty_values", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_function_value_argument_ok_with_scope", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_bare_function_value_argument_rejected", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_with_candidate", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascription_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascription_conflict_with_types", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_argument_conflict_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascription_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascription_conflict_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_argument_conflict_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_named_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_named_argument_missing_evidence_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_named_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_nested_call_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_trailing_block_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_shadowed_function_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_function_value_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_bare_function_value_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_function_value_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_bare_function_value_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_named_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_named_argument_missing_evidence_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_named_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_nested_call_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_trailing_block_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_shadowed_function_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_ascription_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_ascribed_argument_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_named_argument_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_nested_call_argument_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_function_value_argument_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_trailing_block_argument_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_error_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_existing_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_argument_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "SelfhostCallReduceArgumentCheckState", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_check_state_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_check_state_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_check_state_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_argument_state_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_push_checked_argument", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_error_from_candidate_collect", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_error_from_block_body_result", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_generic_state_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_expected_result", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_match_direct_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_consume_loop_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_nested_single_named_candidate_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_nested_named_candidates_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_match_at_with_source_or_nested", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_single_named_candidate", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_single_named_candidate_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_named_prefix", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_named_prefix_with_source_and_trailing_block", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_named_prefix_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_prefix_with_source_and_trailing_block", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_prefix_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_directive_item_kind", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_directive_item_fact", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_directive_fact", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_item_directive_state", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_item_span", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_declaration_item_fact", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "ownerBoundary"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_item_declaration_header", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_diag", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_labeled_diag", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_raw_block_empty_diag", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_unexpected_proof_diag", ["purpose", "contract", "returns", "complexity", "errorVariant"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_directive_duplicate_message", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_directive_duplicate_label", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_directive_duplicate_diag", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_declaration_header_missing_diag", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_declaration_header_invalid_diag", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_index_unavailable_diag", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_refutation_diag", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_raw_backend_item_kind", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_raw_backend_item_fact", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_check_raw_backend_fact", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_check_item_raw_state", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_check_finish_raw_state", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "SelfhostModuleCheckSummary", ["purpose"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_item_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_doc_comment_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_directive_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_entry_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_target_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_import_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_declaration_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_function_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_type_declaration_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_impl_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_raw_block_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_raw_text_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary_update.nepl", "selfhost_module_check_summary_new", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary_update.nepl", "selfhost_module_check_summary_record", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "SelfhostModuleCheckStep", ["purpose", "contract", "complexity", "ownerBoundary"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "selfhost_module_check_step_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "selfhost_module_check_item", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "selfhost_check_module_ast_loop", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "selfhost_check_module_ast", ["purpose", "returns", "complexity"]),
    resourceSolverRequirement("selfhost_proof_resource_cell_proven", ["purpose", "contract", "returns", "complexity", "resourceBoundary"], [
        requiredPattern("resource cell transition evidence", /\bSelfhostProofEvidence::ResourceCellTransition\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_resource_cell_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "resourceBoundary"], [
        requiredPattern("resource cell transition refutation", /\bSelfhostProofRefutation::ResourceCellTransitionInvalid\b/),
        requiredPattern("resource cell transition error", /\bSelfhostResourceCellTransitionError\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_resource_cell_uninitialized", ["purpose", "contract", "returns", "complexity", "errorVariant", "resourceBoundary"], [
        requiredPattern("uninitialized state", /\bSelfhostResourceCellState::Uninitialized\b/),
        requiredPattern("initialize event", /\bSelfhostResourceCellEventKind::Initialize\b/),
        requiredPattern("move uninitialized error", /\bSelfhostResourceCellTransitionError::MoveUninitialized\b/),
        requiredPattern("drop uninitialized error", /\bSelfhostResourceCellTransitionError::DropUninitialized\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_resource_cell_initialized", ["purpose", "contract", "returns", "complexity", "errorVariant", "resourceBoundary"], [
        requiredPattern("initialized state", /\bSelfhostResourceCellState::Initialized\b/),
        requiredPattern("already initialized error", /\bSelfhostResourceCellTransitionError::InitializeAlreadyInitialized\b/),
        requiredPattern("move target state", /\bSelfhostResourceCellState::Moved\b/),
        requiredPattern("drop target state", /\bSelfhostResourceCellState::Dropped\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_resource_cell_moved", ["purpose", "contract", "returns", "complexity", "errorVariant", "resourceBoundary"], [
        requiredPattern("moved state", /\bSelfhostResourceCellState::Moved\b/),
        requiredPattern("move after move error", /\bSelfhostResourceCellTransitionError::MoveAfterMove\b/),
        requiredPattern("drop after move error", /\bSelfhostResourceCellTransitionError::DropAfterMove\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_resource_cell_dropped", ["purpose", "contract", "returns", "complexity", "errorVariant", "resourceBoundary"], [
        requiredPattern("dropped state", /\bSelfhostResourceCellState::Dropped\b/),
        requiredPattern("initialize after drop error", /\bSelfhostResourceCellTransitionError::InitializeAfterDrop\b/),
        requiredPattern("move after drop error", /\bSelfhostResourceCellTransitionError::MoveAfterDrop\b/),
        requiredPattern("double drop error", /\bSelfhostResourceCellTransitionError::DoubleDrop\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_resource_cell_transition", ["purpose", "contract", "returns", "complexity", "errorVariant", "resourceBoundary"], [
        requiredPattern("all resource cell states", /\bSelfhostResourceCellState::Uninitialized\b[\s\S]*\bSelfhostResourceCellState::Initialized\b[\s\S]*\bSelfhostResourceCellState::Moved\b[\s\S]*\bSelfhostResourceCellState::Dropped\b/),
        requiredPattern("resource cell evidence", /\bSelfhostProofEvidence::ResourceCellTransition\b/),
        requiredPattern("resource cell refutation", /\bSelfhostProofRefutation::ResourceCellTransitionInvalid\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_owner_transition_proven", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("owner transition evidence", /\bSelfhostProofEvidence::OwnerTransition\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_owner_transition_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("owner transition refutation", /\bSelfhostProofRefutation::OwnerTransitionInvalid\b/),
        requiredPattern("owner transition error", /\bSelfhostOwnerTransitionError\b/),
        requiredPattern("owner storage authority", /\bfact\.storage\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_owner_transition_storage_mismatch", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("storage mismatch error", /\bSelfhostOwnerTransitionError::StorageIdMismatch\b/),
        requiredPattern("no owner mismatch branch", /\bSelfhostOwnerState::NoOwner\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_owner_transition_with_storage", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("owned state", /\bSelfhostOwnerState::Owned\b/),
        requiredPattern("moved state", /\bSelfhostOwnerState::Moved\b/),
        requiredPattern("released state", /\bSelfhostOwnerState::Released\b/),
        requiredPattern("acquire event", /\bSelfhostOwnerEventKind::Acquire\b/),
        requiredPattern("move event", /\bSelfhostOwnerEventKind::MoveOut\b/),
        requiredPattern("release event", /\bSelfhostOwnerEventKind::Release\b/),
        requiredPattern("borrow view event", /\bSelfhostOwnerEventKind::BorrowView\b/),
        requiredPattern("acquire while owned error", /\bSelfhostOwnerTransitionError::AcquireWhileOwned\b/),
        requiredPattern("view after release error", /\bSelfhostOwnerTransitionError::ViewAfterRelease\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_owner_transition_no_owner", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("no owner state", /\bSelfhostOwnerState::NoOwner\b/),
        requiredPattern("acquire event", /\bSelfhostOwnerEventKind::Acquire\b/),
        requiredPattern("move without owner error", /\bSelfhostOwnerTransitionError::MoveWithoutOwner\b/),
        requiredPattern("release without owner error", /\bSelfhostOwnerTransitionError::ReleaseWithoutOwner\b/),
        requiredPattern("view without owner error", /\bSelfhostOwnerTransitionError::ViewWithoutOwner\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_owner_transition", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("invalid storage id error", /\bSelfhostOwnerTransitionError::InvalidStorageId\b/),
        requiredPattern("owner transition evidence", /\bSelfhostProofEvidence::OwnerTransition\b/),
        requiredPattern("owner transition refutation", /\bSelfhostProofRefutation::OwnerTransitionInvalid\b/),
        requiredPattern("no-owner state", /\bSelfhostOwnerState::NoOwner\b/),
        requiredPattern("owned state", /\bSelfhostOwnerState::Owned\b/),
        requiredPattern("moved state", /\bSelfhostOwnerState::Moved\b/),
        requiredPattern("released state", /\bSelfhostOwnerState::Released\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_borrow_access_proven", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("borrow access evidence", /\bSelfhostProofEvidence::ResourceBorrowAccess\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_borrow_access_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("borrow access refutation", /\bSelfhostProofRefutation::BorrowAccessInvalid\b/),
        requiredPattern("borrow access error", /\bSelfhostBorrowAccessError\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_borrow_access_invalid_shared_count", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("invalid shared count error", /\bSelfhostBorrowAccessError::InvalidSharedBorrowCount\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_borrow_access_unborrowed", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("unborrowed state", /\bSelfhostBorrowState::Unborrowed\b/),
        requiredPattern("start shared request", /\bSelfhostBorrowRequestKind::StartShared\b/),
        requiredPattern("start mutable request", /\bSelfhostBorrowRequestKind::StartMutable\b/),
        requiredPattern("end shared without shared error", /\bSelfhostBorrowAccessError::EndSharedWithoutSharedBorrow\b/),
        requiredPattern("end mutable without mutable error", /\bSelfhostBorrowAccessError::EndMutableWithoutMutableBorrow\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_borrow_access_shared_valid", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("shared state", /\bSelfhostBorrowState::Shared\b/),
        requiredPattern("mutable while shared error", /\bSelfhostBorrowAccessError::MutableBorrowWhileShared\b/),
        requiredPattern("shared count one transition", /count が 1/),
        requiredPattern("end mutable without mutable error", /\bSelfhostBorrowAccessError::EndMutableWithoutMutableBorrow\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_borrow_access_shared", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("shared state", /\bSelfhostBorrowState::Shared\b/),
        requiredPattern("shared count validity helper", /\bselfhost_borrow_shared_count_is_valid\b/),
        requiredPattern("invalid shared count error", /\bSelfhostBorrowAccessError::InvalidSharedBorrowCount\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_borrow_access_mutable", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("mutable state", /\bSelfhostBorrowState::Mutable\b/),
        requiredPattern("shared while mutable error", /\bSelfhostBorrowAccessError::SharedBorrowWhileMutable\b/),
        requiredPattern("mutable while mutable error", /\bSelfhostBorrowAccessError::MutableBorrowWhileMutable\b/),
        requiredPattern("end shared without shared error", /\bSelfhostBorrowAccessError::EndSharedWithoutSharedBorrow\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_borrow_access", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("all borrow states", /\bSelfhostBorrowState::Unborrowed\b[\s\S]*\bShared count\b[\s\S]*\bMutable\b/),
        requiredPattern("borrow access evidence", /\bSelfhostProofEvidence::ResourceBorrowAccess\b/),
        requiredPattern("borrow access refutation", /\bSelfhostProofRefutation::BorrowAccessInvalid\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_lifetime_outlives_proven", ["purpose", "contract", "returns", "complexity", "resourceBoundary"], [
        requiredPattern("lifetime outlives evidence", /\bSelfhostProofEvidence::LifetimeOutlives\b/),
        requiredPattern("same lifetime success relation", /\bSelfhostLifetimeRelation::SameLifetime\b/),
        requiredPattern("subject outlives success relation", /\bSubjectOutlivesRequired\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_lifetime_outlives_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "resourceBoundary"], [
        requiredPattern("lifetime outlives refutation", /\bSelfhostProofRefutation::LifetimeOutlivesInvalid\b/),
        requiredPattern("lifetime outlives error", /\bSelfhostLifetimeOutlivesError\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_lifetime_outlives_relation", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "resourceBoundary"], [
        requiredPattern("invalid subject relation", /\bSelfhostLifetimeRelation::InvalidSubject\b/),
        requiredPattern("invalid required relation", /\bSelfhostLifetimeRelation::InvalidRequired\b/),
        requiredPattern("same lifetime relation", /\bSelfhostLifetimeRelation::SameLifetime\b/),
        requiredPattern("subject outlives relation", /\bSelfhostLifetimeRelation::SubjectOutlivesRequired\b/),
        requiredPattern("subject shorter error", /\bSelfhostLifetimeOutlivesError::SubjectDoesNotOutliveRequired\b/),
        requiredPattern("unrelated error", /\bSelfhostLifetimeOutlivesError::UnrelatedLifetimes\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_lifetime_outlives", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "resourceBoundary"], [
        requiredPattern("required lifetime mismatch error", /\bSelfhostLifetimeOutlivesError::RequiredLifetimeMismatch\b/),
        requiredPattern("lifetime id equality authority", /\bselfhost_lifetime_id_eq\b/),
        requiredPattern("lifetime evidence", /\bSelfhostProofEvidence::LifetimeOutlives\b/),
        requiredPattern("lifetime refutation", /\bSelfhostProofRefutation::LifetimeOutlivesInvalid\b/),
    ]),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_effect_allowed_result", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("typed effect evidence", /\bSelfhostProofEvidence::EffectAllowed\b/),
            requiredPattern("effect context authority", /\bSelfhostEffectContext\b/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_effect_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("typed effect refutation", /\bSelfhostProofRefutation::EffectBoundaryInvalid\b/),
            requiredPattern("typed effect boundary error", /\bSelfhostEffectBoundaryError\b/),
            requiredPattern("unsafe memory boundary error", /\bSelfhostEffectBoundaryError::UnsafeMemoryOutsideBoundary\b/),
            requiredPattern("impure effect in pure context error", /\bSelfhostEffectBoundaryError::ImpureEffectInPureContext\b/),
            requiredPattern("internal allocation escape error", /\bSelfhostEffectBoundaryError::InternalAllocEscapeNotProven\b/),
            requiredPattern("observed effect kind payload", /\bSelfhostEffectKind\b/),
            requiredPattern("observed escape-state payload", /\bSelfhostEffectEscapeState\b/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_internal_alloc_allowed", ["purpose", "contract", "returns", "complexity", "errorVariant", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("internal allocation effect kind", /\bSelfhostEffectKind::InternalAlloc\b/),
            requiredPattern("no-escape success state", /\bSelfhostEffectEscapeState::NoEscapeProven\b/),
            requiredPattern("not-applicable escape failure state", /\bSelfhostEffectEscapeState::NotApplicable\b/),
            requiredPattern("may-escape failure state", /\bSelfhostEffectEscapeState::MayEscape\b/),
            requiredPattern("internal allocation escape error", /\bSelfhostEffectBoundaryError::InternalAllocEscapeNotProven\b/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_solve_effect_pure_context", ["purpose", "contract", "returns", "complexity", "errorVariant", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("pure context authority", /\bSelfhostEffectContext::PureContext\b/),
            requiredPattern("internal allocation branch", /\bSelfhostEffectKind::InternalAlloc\b/),
            requiredPattern("unsafe memory branch", /\bSelfhostEffectKind::UnsafeMemory\b/),
            requiredPattern("external io branch", /\bSelfhostEffectKind::ExternalIo\b/),
            requiredPattern("nondeterminism branch", /\bSelfhostEffectKind::Nondet\b/),
            requiredPattern("unsafe memory boundary error", /\bSelfhostEffectBoundaryError::UnsafeMemoryOutsideBoundary\b/),
            requiredPattern("impure effect in pure context error", /\bSelfhostEffectBoundaryError::ImpureEffectInPureContext\b/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_solve_effect_impure_context", ["purpose", "contract", "returns", "complexity", "errorVariant", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("impure context authority", /\bSelfhostEffectContext::ImpureContext\b/),
            requiredPattern("internal allocation branch", /\bSelfhostEffectKind::InternalAlloc\b/),
            requiredPattern("unsafe memory branch", /\bSelfhostEffectKind::UnsafeMemory\b/),
            requiredPattern("external io branch", /\bSelfhostEffectKind::ExternalIo\b/),
            requiredPattern("nondeterminism branch", /\bSelfhostEffectKind::Nondet\b/),
            requiredPattern("unsafe memory boundary error", /\bSelfhostEffectBoundaryError::UnsafeMemoryOutsideBoundary\b/),
            requiredPattern("internal allocation escape error", /\bSelfhostEffectBoundaryError::InternalAllocEscapeNotProven\b/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_solve_effect_unsafe_boundary", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("unsafe boundary authority", /\bSelfhostEffectContext::UnsafeBoundary\b/),
            requiredPattern("pure effect branch", /\bSelfhostEffectKind::Pure\b/),
            requiredPattern("internal allocation branch", /\bSelfhostEffectKind::InternalAlloc\b/),
            requiredPattern("unsafe memory branch", /\bSelfhostEffectKind::UnsafeMemory\b/),
            requiredPattern("external io branch", /\bSelfhostEffectKind::ExternalIo\b/),
            requiredPattern("nondeterminism branch", /\bSelfhostEffectKind::Nondet\b/),
            requiredPattern("unsafe-boundary evidence payload", /\bSelfhostProofEvidence::EffectAllowed UnsafeBoundary\b/),
            requiredPattern("effect kind is not preserved in evidence", /effect kind .*evidence payload .*保存されません/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_solve_effect_allowed", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("pure context dispatch", /\bSelfhostEffectContext::PureContext\b/),
            requiredPattern("impure context dispatch", /\bSelfhostEffectContext::ImpureContext\b/),
            requiredPattern("unsafe boundary dispatch", /\bSelfhostEffectContext::UnsafeBoundary\b/),
            requiredPattern("unsafe memory boundary error", /\bSelfhostEffectBoundaryError::UnsafeMemoryOutsideBoundary\b/),
            requiredPattern("impure effect in pure context error", /\bSelfhostEffectBoundaryError::ImpureEffectInPureContext\b/),
            requiredPattern("internal allocation escape error", /\bSelfhostEffectBoundaryError::InternalAllocEscapeNotProven\b/),
            requiredPattern("typed effect evidence", /\bSelfhostProofEvidence::EffectAllowed\b/),
            requiredPattern("typed effect refutation", /\bSelfhostProofRefutation::EffectBoundaryInvalid\b/),
            requiredPattern("context match static check", /match .*網羅性検査/),
        ],
    }),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirExprKind", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirFunctionValueIdentityBuildError", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirCallExpr", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirValueIdentity", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirExprPayload", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirExpr", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/byte.nepl", "lex_byte_or_end", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/byte.nepl", "lex_is_digit", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/diagnostic.nepl", "LexDiagnostic", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/directive.nepl", "SelfhostLexerDirectiveKind", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/syntax/lexer/directive.nepl", "lex_directive_word_at", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/indent.nepl", "lex_line_indent_width", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/literal.nepl", "lex_is_hex_digit", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/raw_mode.nepl", "SelfhostLexerRawMode", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/syntax/lexer/token_build.nepl", "lex_token_slice", ["purpose", "contract", "complexity"]),
];

const SECTION_PATTERNS = {
    purpose: /\[目的\/もくてき\]/,
    contract: /\[契約\/けいやく\]/,
    returns: /\[戻\/もど\]り\[値\/ち\]/,
    complexity: /\[計算量\/けいさんりょう\]/,
    doctest: /\bneplg2:test\b/,
    errorVariant: /\b(SelfhostCheckerDiagnosticCode::[A-Za-z0-9_]+|SelfhostDiagnosticCode::Checker|SelfhostProofRefutation::[A-Za-z0-9_]+|SelfhostEffectBoundaryError::[A-Za-z0-9_]+|SelfhostResourceCellTransitionError::[A-Za-z0-9_]+|SelfhostOwnerTransitionError::[A-Za-z0-9_]+|SelfhostBorrowAccessError::[A-Za-z0-9_]+|SelfhostLifetimeOutlivesError::[A-Za-z0-9_]+)\b/,
    authorityBoundary: /\b(authority|typed evidence|parser-provided evidence|parser\/proof|proof layer|source spelling|source text|kind stream|message .*authority|diagnostic kind の authority|表示.*authority)\b/,
    effectBoundary: /\b(SelfhostEffectKind::[A-Za-z0-9_]+|SelfhostEffectContext::[A-Za-z0-9_]+|SelfhostEffectBoundaryError::[A-Za-z0-9_]+|SelfhostProofEvidence::EffectAllowed|SelfhostEffectEscapeState::[A-Za-z0-9_]+)\b/,
    resourceBoundary: /\b(SelfhostResourceCellState::[A-Za-z0-9_]+|SelfhostResourceCellEventKind::[A-Za-z0-9_]+|SelfhostResourceCellTransitionError::[A-Za-z0-9_]+|SelfhostOwnerState::[A-Za-z0-9_]+|SelfhostOwnerEventKind::[A-Za-z0-9_]+|SelfhostOwnerTransitionError::[A-Za-z0-9_]+|SelfhostBorrowState::[A-Za-z0-9_]+|SelfhostBorrowRequestKind::[A-Za-z0-9_]+|SelfhostBorrowAccessError::[A-Za-z0-9_]+|SelfhostLifetimeRelation::[A-Za-z0-9_]+|SelfhostLifetimeOutlivesError::[A-Za-z0-9_]+|SelfhostProofEvidence::(ResourceCellTransition|OwnerTransition|ResourceBorrowAccess|LifetimeOutlives)|SelfhostProofRefutation::(ResourceCellTransitionInvalid|OwnerTransitionInvalid|BorrowAccessInvalid|LifetimeOutlivesInvalid))\b/,
    ownerBoundary: /\b(owner|cleanup obligation|cleanup|borrow|未処理 owner|owner 変換|解放)\b/,
};

function requirement(relPath, name, sections, options = {}) {
    return {
        relPath,
        name,
        sections,
        doctestUses: options.doctestUses || [],
        requiredPatterns: options.requiredPatterns || [],
    };
}

function resourceSolverRequirement(name, sections, requiredPatterns = []) {
    return requirement("stdlib/neplg2/core/proof/solver/resource.nepl", name, sections, {
        requiredPatterns,
    });
}

function requiredPattern(label, pattern) {
    return { label, pattern };
}

function sectionRequirementKey(relPath, name) {
    return `${relPath}#${name}`;
}

function docHasSection(docLines, section) {
    const pattern = SECTION_PATTERNS[section];
    assert.ok(pattern, `unknown documentation section requirement: ${section}`);
    return docLines.some((line) => pattern.test(line));
}

const docSectionRequirementByKey = new Map(
    DOC_SECTION_REQUIREMENTS.map((item) => [sectionRequirementKey(item.relPath, item.name), item]),
);

function walkNeplFiles(dir) {
    const files = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const child = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...walkNeplFiles(child));
        } else if (entry.isFile() && entry.name.endsWith(".nepl")) {
            files.push(child);
        }
    }
    return files;
}

function toRepoPath(filePath) {
    return path.relative(repoRoot, filePath).split(path.sep).join("/");
}

function hasDoctest(docLines) {
    return docLines.some((line) => /\bneplg2:test\b/.test(line));
}

function declarationAt(line) {
    return line.match(/^\s*(pub\s+)?(fn|struct|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)\b/);
}

function moduleDocLines(lines) {
    for (let index = 0; index < lines.length; index += 1) {
        const trimmed = lines[index].trim();
        if (trimmed === "" || trimmed === "#indent 4") {
            continue;
        }
        if (declarationAt(lines[index]) || trimmed.startsWith("#import")) {
            return [];
        }
        if (!lines[index].trimStart().startsWith("//:")) {
            return [];
        }
        const doc = [];
        for (let cursor = index; cursor < lines.length; cursor += 1) {
            if (!lines[cursor].trimStart().startsWith("//:")) {
                break;
            }
            doc.push(lines[cursor]);
        }
        if (doc.length > 0 && doc[0].trimStart().startsWith("//: #")) {
            return doc;
        }
        return [];
    }
    return [];
}

function precedingDocLines(lines, index) {
    let cursor = index - 1;
    while (cursor >= 0 && lines[cursor].trim() === "") {
        cursor -= 1;
    }
    const doc = [];
    while (cursor >= 0 && lines[cursor].trimStart().startsWith("//:")) {
        doc.push(lines[cursor]);
        cursor -= 1;
    }
    return doc.reverse();
}

function indentOf(line) {
    const match = line.match(/^(\s*)/);
    return match ? match[1].length : 0;
}

function implHeaderAt(line) {
    return line.match(/^\s*impl(?:\b|<)/);
}

const stats = {
    files: 0,
    moduleNoDoc: 0,
    moduleNoDoctest: 0,
    declarations: 0,
    declarationNoDoc: 0,
    declarationNoDoctest: 0,
    publicNoDoc: 0,
    publicNoDoctest: 0,
    privateNoDoc: 0,
    privateNoDoctest: 0,
};

const samples = [];
const publicDocRequiredPrefixGaps = [];
const moduleDocRequiredPrefixGaps = [];
const docSectionGaps = [];
const seenDocSectionRequirementKeys = new Set();
const seenRepoPaths = new Set();

function sample(message) {
    if (samples.length < 60) {
        samples.push(message);
    }
}

for (const filePath of walkNeplFiles(selfhostRoot).sort()) {
    stats.files += 1;
    const repoPath = toRepoPath(filePath);
    seenRepoPaths.add(repoPath);
    const text = fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
    const lines = text.split("\n");
    const moduleDoc = moduleDocLines(lines);
    if (moduleDoc.length === 0) {
        stats.moduleNoDoc += 1;
        sample(`${repoPath}: module doc is missing`);
        if (PUBLIC_DOC_REQUIRED_PREFIXES.some((prefix) => repoPath.startsWith(prefix))) {
            moduleDocRequiredPrefixGaps.push(`${repoPath}: module doc heading is missing`);
        }
    } else if (!hasDoctest(moduleDoc)) {
        stats.moduleNoDoctest += 1;
    }

    let implBlockIndent = null;
    for (let index = 0; index < lines.length; index += 1) {
        const line = lines[index];
        const trimmed = line.trim();
        const indentation = indentOf(line);
        const startsImpl = implHeaderAt(line);
        if (
            implBlockIndent !== null
            && trimmed !== ""
            && !trimmed.startsWith("//:")
            && indentation <= implBlockIndent
            && !startsImpl
        ) {
            implBlockIndent = null;
        }
        if (startsImpl) {
            implBlockIndent = indentation;
            continue;
        }
        if (implBlockIndent !== null) {
            continue;
        }
        const declaration = declarationAt(line);
        if (!declaration) {
            continue;
        }

        stats.declarations += 1;
        const isPublic = Boolean(declaration[1]);
        const doc = precedingDocLines(lines, index);
        if (doc.length === 0) {
            stats.declarationNoDoc += 1;
            if (isPublic) {
                stats.publicNoDoc += 1;
            } else {
                stats.privateNoDoc += 1;
            }
            const gap = `${repoPath}:${index + 1}: ${declaration[2]} ${declaration[3]} doc is missing`;
            sample(gap);
            if (isPublic && PUBLIC_DOC_REQUIRED_PREFIXES.some((prefix) => repoPath.startsWith(prefix))) {
                publicDocRequiredPrefixGaps.push(gap);
            }
        } else {
            const requirementKey = sectionRequirementKey(repoPath, declaration[3]);
            const sectionRequirement = docSectionRequirementByKey.get(requirementKey);
            if (sectionRequirement) {
                seenDocSectionRequirementKeys.add(requirementKey);
                for (const section of sectionRequirement.sections) {
                    if (!docHasSection(doc, section)) {
                        docSectionGaps.push(`${repoPath}:${index + 1}: ${declaration[2]} ${declaration[3]} doc is missing [${section}] section`);
                    }
                }
                for (const usageName of sectionRequirement.doctestUses) {
                    if (!doc.some((docLine) => docLine.includes(usageName))) {
                        docSectionGaps.push(`${repoPath}:${index + 1}: ${declaration[2]} ${declaration[3]} doc doctest must explain representative use of ${usageName}`);
                    }
                }
                for (const requiredDocPattern of sectionRequirement.requiredPatterns) {
                    if (!doc.some((docLine) => requiredDocPattern.pattern.test(docLine))) {
                        docSectionGaps.push(`${repoPath}:${index + 1}: ${declaration[2]} ${declaration[3]} doc must mention ${requiredDocPattern.label}`);
                    }
                }
            }
        }
        if (doc.length > 0 && !hasDoctest(doc)) {
            stats.declarationNoDoctest += 1;
            if (isPublic) {
                stats.publicNoDoctest += 1;
            } else {
                stats.privateNoDoctest += 1;
            }
        }
    }
}

for (const repoPath of REQUIRED_SCANNER_SENTINELS) {
    assert.ok(
        seenRepoPaths.has(repoPath),
        `${repoPath} must be included in the selfhost documentation scan`,
    );
}
assert(
    fs.existsSync(path.join(repoRoot, DOC_GAP_TRACKING_ISSUE)),
    `selfhost documentation baseline gaps must be tracked by ${DOC_GAP_TRACKING_ISSUE}`,
);
const docGapTrackingIssueText = fs.readFileSync(path.join(repoRoot, DOC_GAP_TRACKING_ISSUE), "utf8").replace(/\r\n/g, "\n");
assert.match(
    docGapTrackingIssueText,
    /^status:\s*open$/m,
    "selfhost documentation baseline issue must remain open while baseline gaps remain",
);
assert.match(
    docGapTrackingIssueText,
    /^resolved:\s*false$/m,
    "selfhost documentation baseline issue must remain unresolved while baseline gaps remain",
);
assert.ok(
    docGapTrackingIssueText.includes("not an accepted quality level"),
    "selfhost documentation baseline issue must state that the baseline is not an accepted quality level",
);
assert.ok(
    docGapTrackingIssueText.includes("fail-closed debt boundary"),
    "selfhost documentation baseline issue must state that the baseline is a fail-closed debt boundary",
);
for (const [key, value] of Object.entries(BASELINE)) {
    assert.ok(
        docGapTrackingIssueText.includes(`${key}=${value}`),
        `selfhost documentation baseline issue must record ${key}=${value}`,
    );
}
for (const key of HARD_DOC_BASELINE_KEYS) {
    assert(
        stats[key] <= BASELINE[key],
        `selfhost documentation gaps increased for ${key}: ${stats[key]} > ${BASELINE[key]}`,
    );
}
for (const key of REPORT_ONLY_DOCTEST_BASELINE_KEYS) {
    assert.ok(
        Object.hasOwn(BASELINE, key),
        `selfhost doctest debt counter must remain visible in the baseline issue: ${key}`,
    );
}
assert.deepEqual(
    moduleDocRequiredPrefixGaps,
    [],
    `selfhost fixed documentation slices must have explicit module doc headings:\n${moduleDocRequiredPrefixGaps.join("\n")}`,
);
assert.deepEqual(
    publicDocRequiredPrefixGaps,
    [],
    `selfhost fixed public documentation slices must not have public declaration doc gaps:\n${publicDocRequiredPrefixGaps.join("\n")}`,
);
const missingSectionRequirementTargets = [...docSectionRequirementByKey.keys()]
    .filter((key) => !seenDocSectionRequirementKeys.has(key));
assert.deepEqual(
    missingSectionRequirementTargets,
    [],
    `selfhost documentation section requirement targets must be found:\n${missingSectionRequirementTargets.join("\n")}`,
);
assert.deepEqual(
    docSectionGaps,
    [],
    `selfhost fixed documentation slices must preserve the required Zenn-policy doc sections:\n${docSectionGaps.join("\n")}`,
);

console.log("selfhost documentation contract baseline ok");
console.log(JSON.stringify(stats, null, 2));
if (samples.length > 0) {
    console.log("sample gaps:");
    for (const line of samples) {
        console.log(`- ${line}`);
    }
}
