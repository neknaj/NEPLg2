#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    TY_ROOT_REEXPORT_FILES,
    TY_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_operation_solver.nepl";
const layoutPath = "stdlib/neplg2/core/ty/ty/memo_trait_layout.nepl";
const operationEvidencePath = "stdlib/neplg2/core/ty/ty/memo_trait_operation_evidence.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const layoutSource = readRepoFile(repoRoot, layoutPath);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "ty root re-export file list must include memo_trait_operation_solver.nepl",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "ty split file list must include memo_trait_operation_solver.nepl",
);

function assertOperationSolverOrder(list, label) {
    const layoutIndex = list.indexOf(layoutPath);
    const producerIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_producer.nepl");
    const recursiveAggregateIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_recursive_aggregate.nepl");
    const operationProofIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_operation_proof.nepl");
    const operationEvidenceIndex = list.indexOf(operationEvidencePath);
    const operationSolverIndex = list.indexOf(relPath);
    const recursiveProducerIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_recursive_producer.nepl");
    assert.ok(
        layoutIndex >= 0
            && producerIndex >= 0
            && recursiveAggregateIndex >= 0
            && operationProofIndex >= 0
            && operationEvidenceIndex >= 0
            && operationSolverIndex >= 0
            && recursiveProducerIndex >= 0,
        `${label} must include layout, producer, recursive aggregate, operation proof, operation evidence, operation solver, and recursive producer files`,
    );
    assert.ok(
        layoutIndex < producerIndex
            && producerIndex < recursiveAggregateIndex
            && recursiveAggregateIndex < operationProofIndex
            && operationProofIndex < operationEvidenceIndex
            && operationEvidenceIndex < operationSolverIndex
            && operationSolverIndex < recursiveProducerIndex,
        `${label} must keep operation solver after operation proof/evidence transport and before recursive producer connector`,
    );
}

assertOperationSolverOrder(TY_ROOT_REEXPORT_FILES, "ty root re-export order");
assertOperationSolverOrder(TY_SPLIT_FILES, "ty split order");
assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_operation_solver" as \*$/m,
    "ty facade must re-export the memo trait operation solver split module",
);
assert.match(
    facade,
    /pub #import "\.\/ty\/memo_trait_operation_proof" as \*[\s\S]*pub #import "\.\/ty\/memo_trait_operation_evidence" as \*[\s\S]*pub #import "\.\/ty\/memo_trait_operation_solver" as \*[\s\S]*pub #import "\.\/ty\/memo_trait_recursive_producer" as \*/,
    "ty facade must keep operation proof before operation evidence before operation solver before recursive producer",
);
assert.match(
    source,
    /# ty\/memo_trait_operation_solver[\s\S]*\[目的\/もくてき\]:[\s\S]*nested aggregate field[\s\S]*root operation evidence[\s\S]*\[契約\/けいやく\]:[\s\S]*selfhost_memo_trait_recursive_aggregate_result[\s\S]*SelfhostMemoTraitOperationEvidenceTable[\s\S]*\[現状\/げんじょう\]:[\s\S]*evidence 付き入口[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "operation solver documentation must record purpose, evidence input contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /source text、span、path、display name、diagnostic text、lexeme は proof authority にしません/,
    "operation solver docs must explicitly reject source text, spans, paths, display names, diagnostics, and lexemes as proof authority",
);
assert.match(
    source,
    /proof store、`\.neplproof` artifact、canonical key codec、HIR、Resource IR、backend へ依存しません/,
    "operation solver docs must keep artifact, HIR, Resource IR, and backend layers out of this checkpoint",
);
assert.match(
    layoutSource,
    /pub fn selfhost_memo_trait_layout_field_type_at_result %fn &SelfhostMemoTraitLayoutEvidenceTable fn SelfhostMemoTraitAggregateFieldRange fn i32 Result SelfhostTypeId SelfhostMemoTraitLayoutEvidenceErrorKind/,
    "layout module must expose a typed field accessor for validated layout ranges",
);
assert.match(
    layoutSource,
    /selfhost_memo_trait_layout_field_type_at_result[\s\S]*caller は layout validator が返した `Known\(range\)` を渡します[\s\S]*range と index を再検査[\s\S]*source position ではありません[\s\S]*field name、span、source text、diagnostic string は proof authority として返しません/,
    "layout field accessor docs must describe the validated range contract and reject source-derived authority",
);
assert.match(
    layoutSource,
    /selfhost_memo_trait_layout_field_type_at_result[\s\S]*not selfhost_memo_trait_layout_range_in_table table range[\s\S]*InvalidFieldRange[\s\S]*lt idx 0[\s\S]*InvalidFieldRange[\s\S]*ge idx range\.field_count[\s\S]*InvalidFieldRange[\s\S]*v::get fields add range\.first_field idx[\s\S]*Result::Ok record\.field_type/,
    "layout field accessor must re-check range bounds, reject invalid indexes, and return only the field TypeId",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_layout" as \*$/m,
    "operation solver must depend on layout evidence instead of reading raw source layout",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_operation_evidence" as \*$/m,
    "operation solver must depend on the typed operation evidence table for root trait operation evidence",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_operation_proof" as \*$/m,
    "operation solver must write operation proof records through the operation proof transport boundary",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_producer" as \*$/m,
    "operation solver must reuse the existing aggregate proof status enum from the producer layer",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_recursive_aggregate" as \*$/m,
    "operation solver must use the recursive aggregate gate before constructing operation proof tables",
);
assert.doesNotMatch(
    codeOnly,
    /#import "\.\/memo_trait_(?:proof_store|proof_reader|proof_payload_reader|proof_serializer|proof_preseed|proof_stable_map|proof_artifact|proof_index|proof_decoded|artifact_word_codec|canonical_key|canonical_key_payload|canonical_key_payload_codec)"/,
    "operation solver must not depend on proof store, artifacts, canonical key codecs, readers, serializers, indexes, or preseed modules",
);
assert.doesNotMatch(
    codeOnly,
    /#import "\.\/memo_trait_recursive_producer"/,
    "operation solver must not import the accepted evidence producer connector",
);
assert.doesNotMatch(
    codeOnly,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "operation solver must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostMemoTraitEvidenceRecord|selfhost_memo_trait_aggregate_proof_(?:new|to_record)|selfhost_memo_trait_recursive_producer_record_result/,
    "operation solver must not construct accepted evidence records or call producer acceptance helpers",
);
assert.doesNotMatch(
    source,
    /pub fn selfhost_memo_trait_operation_solver_record_result\b/,
    "record_result must remain a private lower helper so public callers cannot bypass the recursive aggregate gate",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitOperationSolverErrorKind:[\s\S]*RecursiveRejected %SelfhostMemoTraitRecursiveAggregateErrorKind[\s\S]*LayoutRejected %SelfhostMemoTraitLayoutEvidenceErrorKind[\s\S]*FieldReadRejected %SelfhostMemoTraitLayoutEvidenceErrorKind[\s\S]*TableAllocFailed %StdErrorKind[\s\S]*TablePushRejected %SelfhostMemoTraitOperationProofErrorKind[\s\S]*OperationEvidenceRejected %SelfhostMemoTraitOperationEvidenceErrorKind/,
    "operation solver errors must preserve recursive, layout, field read, table allocation, table push, and operation evidence failures as typed payloads",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_primitive_record[\s\S]*SelfhostPrimitiveTypeKind::Unit:[\s\S]*record_all_proven[\s\S]*SelfhostPrimitiveTypeKind::Bool:[\s\S]*record_all_proven[\s\S]*SelfhostPrimitiveTypeKind::I32:[\s\S]*record_all_proven[\s\S]*SelfhostPrimitiveTypeKind::U8:[\s\S]*record_all_proven[\s\S]*SelfhostPrimitiveTypeKind::Char:[\s\S]*record_all_proven[\s\S]*SelfhostPrimitiveTypeKind::F32:[\s\S]*record_f32_value_only/,
    "operation solver must prove stable scalar primitives and keep f32 on the conservative value-only path",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_record_f32_value_only[\s\S]*SelfhostMemoTraitAggregateProofStatus::Proven SelfhostMemoTraitAggregateProofStatus::Proven SelfhostMemoTraitAggregateProofStatus::Unknown SelfhostMemoTraitAggregateProofStatus::Unknown/,
    "f32 operation status must keep Copy and Drop Proven while Eq and Hash remain Unknown",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_field_record_result[\s\S]*SelfhostTypeRecord::Named _named:[\s\S]*selfhost_memo_trait_operation_solver_record_result types layout_table field_type[\s\S]*SelfhostTypeRecord::Applied _applied:[\s\S]*selfhost_memo_trait_operation_solver_record_result types layout_table field_type[\s\S]*SelfhostTypeRecord::Parameter _parameter:[\s\S]*record_unknown[\s\S]*SelfhostTypeRecord::Function _function:[\s\S]*record_unknown/,
    "named and applied aggregate fields must recurse through the same layout evidence while parameter and function fields remain fail-closed as Unknown",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_record_result[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result layout_table types type_id[\s\S]*SelfhostMemoTraitAggregateFieldEvidence::Known range:[\s\S]*selfhost_memo_trait_operation_solver_fold_fields_result types layout_table range 0 initial[\s\S]*MissingLayout:[\s\S]*LayoutRejected SelfhostMemoTraitLayoutEvidenceErrorKind::MissingLayout[\s\S]*GenericArgumentUnsubstituted:[\s\S]*LayoutRejected SelfhostMemoTraitLayoutEvidenceErrorKind::GenericArgumentUnsubstituted[\s\S]*CycleLimitReached:[\s\S]*LayoutRejected SelfhostMemoTraitLayoutEvidenceErrorKind::CycleLimitReached/,
    "record solver must use the public layout validator, fold only Known ranges, and preserve typed layout rejection payloads",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_fold_fields_result[\s\S]*selfhost_memo_trait_layout_field_type_at_result layout_table range idx[\s\S]*selfhost_memo_trait_operation_solver_field_record_result types layout_table field_type[\s\S]*Result::Ok field_record:[\s\S]*selfhost_memo_trait_operation_solver_record_merge aggregate field_record[\s\S]*Result::Err field_error:[\s\S]*Result::Err field_error[\s\S]*FieldReadRejected field_error/,
    "field fold must read field types through the layout accessor, propagate nested solver errors, merge successful records, and preserve field read failures",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_table_for_type_result[\s\S]*impure fn SelfhostTypeId impure fn i32 Result SelfhostMemoTraitOperationProofTable[\s\S]*selfhost_memo_trait_recursive_aggregate_result types layout_table type_id max_depth[\s\S]*Result::Ok _summary:[\s\S]*selfhost_memo_trait_operation_proof_table_new[\s\S]*selfhost_memo_trait_operation_solver_record_result types layout_table type_id[\s\S]*selfhost_memo_trait_operation_proof_table_push table record[\s\S]*TablePushRejected push_error[\s\S]*selfhost_memo_trait_operation_proof_table_free table[\s\S]*Result::Err solve_error[\s\S]*Result::Err recursive_error:[\s\S]*RecursiveRejected recursive_error/,
    "table construction must run the recursive aggregate gate before allocating operation proof transport, push the computed record, close partial table on solve errors, and preserve typed recursive or push failures",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_record_with_operation_evidence_result[\s\S]*selfhost_memo_trait_operation_solver_record_result types layout_table type_id[\s\S]*selfhost_memo_trait_operation_evidence_record_for_type_or_missing_result evidence_table type_id[\s\S]*selfhost_memo_trait_operation_solver_record_merge structural_record evidence_record[\s\S]*OperationEvidenceRejected evidence_error/,
    "operation solver must merge root operation evidence through the typed evidence table and preserve duplicate evidence as a solver error",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_operation_solver_table_for_type_with_operation_evidence_result[\s\S]*&SelfhostMemoTraitOperationEvidenceTable[\s\S]*selfhost_memo_trait_recursive_aggregate_result types layout_table type_id max_depth[\s\S]*selfhost_memo_trait_operation_solver_record_with_operation_evidence_result types layout_table evidence_table type_id[\s\S]*selfhost_memo_trait_operation_proof_table_push table record[\s\S]*selfhost_memo_trait_operation_proof_table_free table[\s\S]*Result::Err solve_error/,
    "operation solver must expose an evidence-aware table constructor that runs the recursive gate, folds root evidence, pushes one proof record, and closes partial tables on solver errors",
);
assert.match(
    source,
    /wildcard arm は使いません[\s\S]*selfhost_memo_trait_operation_solver_error_kind_eq[\s\S]*RecursiveRejected a_recursive:[\s\S]*selfhost_memo_trait_recursive_aggregate_error_kind_eq a_recursive b_recursive[\s\S]*LayoutRejected a_layout:[\s\S]*selfhost_memo_trait_layout_error_kind_eq a_layout b_layout[\s\S]*FieldReadRejected a_field:[\s\S]*selfhost_memo_trait_layout_error_kind_eq a_field b_field[\s\S]*TableAllocFailed a_alloc:[\s\S]*selfhost_memo_trait_operation_solver_std_error_kind_eq a_alloc b_alloc[\s\S]*TablePushRejected a_push:[\s\S]*selfhost_memo_trait_operation_proof_error_kind_eq a_push b_push[\s\S]*OperationEvidenceRejected a_evidence:[\s\S]*selfhost_memo_trait_operation_evidence_error_kind_eq a_evidence b_evidence/,
    "operation solver error equality must avoid wildcard arms and compare nested typed payloads, including operation evidence errors",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitOperationSolverStage0Summary:[\s\S]*empty_record[\s\S]*i32_field_record[\s\S]*f32_field_record[\s\S]*missing_layout_rejected[\s\S]*i32_table_lookup[\s\S]*nested_i32_record[\s\S]*nested_f32_record[\s\S]*nested_missing_layout_rejected[\s\S]*recursive_cycle_rejected/,
    "stage0 summary must expose empty aggregate, primitive field aggregate, nested aggregate, missing layout, table lookup, nested layout rejection, and recursive cycle paths",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_stage0[\s\S]*selfhost_memo_trait_operation_solver_record_is_all_proven_result summary\.empty_record[\s\S]*selfhost_memo_trait_operation_solver_record_is_all_proven_result summary\.i32_field_record[\s\S]*selfhost_memo_trait_operation_solver_record_has_hash_unknown_result summary\.f32_field_record[\s\S]*LayoutRejected SelfhostMemoTraitLayoutEvidenceErrorKind::MissingLayout[\s\S]*selfhost_memo_trait_operation_proof_result_is_accept summary\.i32_table_lookup[\s\S]*selfhost_memo_trait_operation_solver_record_is_all_proven_result summary\.nested_i32_record[\s\S]*selfhost_memo_trait_operation_solver_record_has_hash_unknown_result summary\.nested_f32_record[\s\S]*RecursiveRejected SelfhostMemoTraitRecursiveAggregateErrorKind::LayoutRejected SelfhostMemoTraitLayoutEvidenceErrorKind::MissingLayout[\s\S]*unwrap_err summary\.nested_missing_layout_rejected[\s\S]*RecursiveRejected SelfhostMemoTraitRecursiveAggregateErrorKind::CycleDetected[\s\S]*unwrap_err summary\.recursive_cycle_rejected/,
    "doctest must check proven empty and i32 records, conservative f32 hash status, typed missing layout rejection, operation table lookup, nested aggregate folding, nested layout rejection, and recursive cycle rejection",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_stage0_nested_record[\s\S]*selfhost_memo_trait_layout_field_record_new child_nominal 0 field_id[\s\S]*selfhost_memo_trait_layout_record_product_named child_nominal 0 1[\s\S]*selfhost_memo_trait_layout_field_record_new root_nominal 0 child_id[\s\S]*selfhost_memo_trait_layout_record_product_named root_nominal 1 1[\s\S]*selfhost_memo_trait_operation_solver_table_for_type_result &arena &layout_table root_id 8[\s\S]*selfhost_memo_trait_operation_proof_record_for_type_result &operation_table root_id/,
    "stage0 nested smoke must route child aggregate status through the public table solver and root table lookup",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_stage0_nested_missing_layout[\s\S]*root field が指す child aggregate の layout を登録しません[\s\S]*selfhost_memo_trait_layout_field_record_new root_nominal 0 child_id[\s\S]*selfhost_memo_trait_layout_record_product_named root_nominal 0 1[\s\S]*selfhost_memo_trait_operation_solver_table_for_type_result &arena &layout_table root_id 8[\s\S]*Result::Err solver_error:[\s\S]*Result::Ok Result::Err solver_error/,
    "stage0 nested missing layout smoke must verify that the public recursive gate returns a typed solver error before record folding",
);
assert.doesNotMatch(
    codeOnly,
    /global_visited|visited_table|visited_first|first_wins|firstWins/,
    "operation solver must not add a global visited first-wins cache for nested aggregate folding",
);
assert.doesNotMatch(
    codeOnly,
    /v::get fields add range\.first_field|field::get_ref table "fields"/,
    "operation solver must not read layout field vectors directly; it must use the validated layout accessor",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_solver_stage0_recursive_cycle[\s\S]*selfhost_memo_trait_layout_field_record_new nominal_id 0 root_id[\s\S]*selfhost_memo_trait_layout_record_product_named nominal_id 0 1[\s\S]*selfhost_memo_trait_operation_solver_table_for_type_result &arena &layout_table root_id 4[\s\S]*Result::Err error:[\s\S]*Result::Ok Result::Err error/,
    "stage0 recursive cycle smoke must route a self-referential field through the public table solver boundary",
);
assert.doesNotMatch(
    source,
    /\(unwrap_err/,
    "operation solver doctest must use current NEPLg2.1 syntax and must not reintroduce parenthesized unwrap expressions",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "operation solver code must not use source text, spans, paths, display names, diagnostics, or lexemes as proof authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "operation solver policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation solver contract passed");
