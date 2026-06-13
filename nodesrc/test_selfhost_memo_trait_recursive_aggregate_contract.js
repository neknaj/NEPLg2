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
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_recursive_aggregate.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "ty root re-export file list must include memo_trait_recursive_aggregate.nepl",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "ty split file list must include memo_trait_recursive_aggregate.nepl",
);
function assertRecursiveAggregateOrder(list, label) {
    const layoutIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_layout.nepl");
    const producerIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_producer.nepl");
    const recursiveIndex = list.indexOf(relPath);
    const operationIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_operation_proof.nepl");
    assert.ok(layoutIndex >= 0 && producerIndex >= 0 && recursiveIndex >= 0 && operationIndex >= 0, `${label} must include layout, producer, recursive aggregate, and operation proof files`);
    assert.ok(
        layoutIndex < producerIndex && producerIndex < recursiveIndex && recursiveIndex < operationIndex,
        `${label} must keep layout before producer before recursive aggregate before operation proof`,
    );
}
assertRecursiveAggregateOrder(TY_ROOT_REEXPORT_FILES, "ty root re-export order");
assertRecursiveAggregateOrder(TY_SPLIT_FILES, "ty split order");
assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_recursive_aggregate" as \*$/m,
    "ty facade must re-export the recursive aggregate split module",
);
assert.match(
    facade,
    /pub #import "\.\/ty\/memo_trait_layout" as \*[\s\S]*pub #import "\.\/ty\/memo_trait_producer" as \*[\s\S]*pub #import "\.\/ty\/memo_trait_recursive_aggregate" as \*[\s\S]*pub #import "\.\/ty\/memo_trait_operation_proof" as \*/,
    "ty facade must keep layout, producer, recursive aggregate, and operation proof imports in dependency order",
);
assert.match(
    source,
    /# ty\/memo_trait_recursive_aggregate[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "recursive aggregate documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /operation proof を計算せず[\s\S]*accepted proof record も作りません[\s\S]*producer \/ proof solver の責務/,
    "recursive aggregate docs must state that this module checks traversal shape only and never accepts MemoKey/MemoValue proof",
);
assert.match(
    source,
    /cycle 判定は「現在の ancestry stack」に限定します[\s\S]*sibling branch に再登場することは cycle ではありません/,
    "recursive aggregate docs must define ancestry-stack cycle semantics instead of global first-wins visited authority",
);
assert.match(
    source,
    /`max_depth` は root depth `0` を含む上限[\s\S]*`DepthLimitReached`/,
    "recursive aggregate docs must define the depth boundary explicitly",
);
assert.match(
    source,
    /#import "\.\/memo_trait_layout" as \*[\s\S]*#import "\.\/memo_trait_producer" as \*/,
    "recursive aggregate must depend on layout evidence and the shared aggregate field evidence payload",
);
assert.doesNotMatch(
    codeOnly,
    /#import "\.\/memo_trait_operation_proof"|\bmemo_trait_operation_proof\b/,
    "recursive aggregate must not depend on Copy/Drop/Eq/Hash operation proof computation",
);
assert.doesNotMatch(
    codeOnly,
    /#import "\.\/memo_trait_(?:proof_store|proof_reader|proof_payload_reader|proof_serializer|proof_preseed|proof_stable_map|proof_artifact|proof_index|proof_decoded|artifact_word_codec|canonical_key_payload_codec)"/,
    "recursive aggregate must not depend on proof store, artifact reader, serializer, index, preseed, stable map, or codec modules",
);
assert.doesNotMatch(
    codeOnly,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "recursive aggregate must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitRecursiveAggregateSummary:[\s\S]*root %SelfhostTypeId[\s\S]*aggregate_count %i32[\s\S]*field_count %i32[\s\S]*max_depth_seen %i32/,
    "recursive aggregate summary must expose only traversal shape and must not store accepted proof material",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitRecursiveAggregateErrorKind:[\s\S]*MissingTypeRecord[\s\S]*TargetNotAggregate[\s\S]*LayoutRejected %SelfhostMemoTraitLayoutEvidenceErrorKind[\s\S]*FieldRecordMissing[\s\S]*UnsupportedFieldType[\s\S]*CycleDetected[\s\S]*DepthLimitReached[\s\S]*StackPushFailed %StdErrorKind/,
    "recursive aggregate failures must be typed enum variants and preserve layout and push error payloads",
);
assert.match(
    source,
    /wildcard arm は使いません[\s\S]*pub fn selfhost_memo_trait_recursive_aggregate_error_kind_eq[\s\S]*LayoutRejected a_layout:[\s\S]*LayoutRejected b_layout:[\s\S]*selfhost_memo_trait_layout_error_kind_eq a_layout b_layout[\s\S]*StackPushFailed a_push:[\s\S]*StackPushFailed b_push:[\s\S]*selfhost_memo_trait_recursive_aggregate_std_error_kind_eq a_push b_push/,
    "recursive aggregate error equality must avoid wildcard arms and compare nested layout and push payloads",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_aggregate_stack_contains[\s\S]*selfhost_memo_trait_recursive_aggregate_visit_type_result[\s\S]*CycleDetected/,
    "recursive aggregate traversal must check the ancestry stack and reject in-progress revisits as cycles",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_aggregate_visit_type_result[\s\S]*gt depth max_depth[\s\S]*DepthLimitReached/,
    "recursive aggregate traversal must reject depth limit overflow before unbounded recursion",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_aggregate_visit_known_aggregate_result[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result layout_table types type_id[\s\S]*SelfhostMemoTraitAggregateFieldEvidence::Known range:[\s\S]*selfhost_memo_trait_recursive_aggregate_fields_loop_result/,
    "recursive aggregate traversal must enter field traversal only through the public layout evidence validator",
);
assert.match(
    source,
    /SelfhostMemoTraitAggregateFieldEvidence::MissingLayout:[\s\S]*LayoutRejected SelfhostMemoTraitLayoutEvidenceErrorKind::MissingLayout[\s\S]*SelfhostMemoTraitAggregateFieldEvidence::GenericArgumentUnsubstituted:[\s\S]*LayoutRejected SelfhostMemoTraitLayoutEvidenceErrorKind::GenericArgumentUnsubstituted[\s\S]*SelfhostMemoTraitAggregateFieldEvidence::CycleLimitReached:[\s\S]*LayoutRejected SelfhostMemoTraitLayoutEvidenceErrorKind::CycleLimitReached/,
    "recursive aggregate traversal must preserve layout evidence rejection variants",
);
assert.match(
    source,
    /SelfhostTypeRecord::Primitive _kind:[\s\S]*selfhost_memo_trait_recursive_aggregate_fields_loop_result[\s\S]*SelfhostTypeRecord::Parameter _parameter:[\s\S]*LayoutRejected SelfhostMemoTraitLayoutEvidenceErrorKind::GenericArgumentUnsubstituted[\s\S]*SelfhostTypeRecord::Function _function:[\s\S]*UnsupportedFieldType/,
    "field traversal must treat primitives as leaves and reject parameter/function fields fail-closed",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_aggregate_push_stack_result[\s\S]*v::push stack type_id[\s\S]*v::vec_push_error_vec e[\s\S]*v::free returned[\s\S]*StackPushFailed error/,
    "stack push failure must preserve StdErrorKind and close the Vec owner returned by failed push",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_aggregate_visit_known_aggregate_result[\s\S]*selfhost_memo_trait_recursive_aggregate_push_stack_result[\s\S]*selfhost_memo_trait_recursive_aggregate_fields_loop_result[\s\S]*selfhost_memo_trait_recursive_aggregate_pop_stack/,
    "successful aggregate visits must push before field traversal and pop before returning",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitRecursiveAggregateStage0Summary:[\s\S]*accepted[\s\S]*nested_accepted[\s\S]*cycle_rejected[\s\S]*depth_rejected[\s\S]*target_rejected[\s\S]*missing_type_rejected[\s\S]*parameter_field_rejected[\s\S]*function_field_rejected/,
    "stage0 summary must expose accepted, nested, cycle, depth, target, missing-type, parameter-field, and function-field paths",
);
assert.match(
    source,
    /let accepted[\s\S]*selfhost_memo_trait_recursive_aggregate_result &arena &table root_id 4[\s\S]*let nested_accepted[\s\S]*selfhost_memo_trait_recursive_aggregate_result &arena &table nested_root_id 4[\s\S]*let cycle_rejected[\s\S]*selfhost_memo_trait_recursive_aggregate_result &arena &table child_id 4[\s\S]*let depth_rejected[\s\S]*selfhost_memo_trait_recursive_aggregate_result &arena &table child_id 0[\s\S]*let target_rejected[\s\S]*selfhost_memo_trait_recursive_aggregate_result &arena &table unit_id 4[\s\S]*let missing_type_rejected[\s\S]*selfhost_memo_trait_recursive_aggregate_result &arena &table missing_id 4[\s\S]*let parameter_field_rejected[\s\S]*selfhost_memo_trait_recursive_aggregate_result &arena &table parameter_root_id 4[\s\S]*let function_field_rejected[\s\S]*selfhost_memo_trait_recursive_aggregate_result &arena &table function_root_id 4/,
    "stage0 must run accepted, nested, cycle, depth limit, primitive-target, missing-type, parameter-field, and function-field checks through the public entry",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_aggregate_summary_is_shape summary\.accepted 1 1 0[\s\S]*selfhost_memo_trait_recursive_aggregate_summary_is_shape summary\.nested_accepted 2 2 1[\s\S]*selfhost_memo_trait_recursive_aggregate_error_kind_eq \(unwrap_err summary\.cycle_rejected\) SelfhostMemoTraitRecursiveAggregateErrorKind::CycleDetected[\s\S]*selfhost_memo_trait_recursive_aggregate_error_kind_eq \(unwrap_err summary\.depth_rejected\) SelfhostMemoTraitRecursiveAggregateErrorKind::DepthLimitReached[\s\S]*selfhost_memo_trait_recursive_aggregate_error_kind_eq \(unwrap_err summary\.target_rejected\) SelfhostMemoTraitRecursiveAggregateErrorKind::TargetNotAggregate[\s\S]*selfhost_memo_trait_recursive_aggregate_error_kind_eq \(unwrap_err summary\.missing_type_rejected\) SelfhostMemoTraitRecursiveAggregateErrorKind::MissingTypeRecord[\s\S]*selfhost_memo_trait_recursive_aggregate_error_kind_eq \(unwrap_err summary\.parameter_field_rejected\) SelfhostMemoTraitRecursiveAggregateErrorKind::LayoutRejected SelfhostMemoTraitLayoutEvidenceErrorKind::GenericArgumentUnsubstituted[\s\S]*selfhost_memo_trait_recursive_aggregate_error_kind_eq \(unwrap_err summary\.function_field_rejected\) SelfhostMemoTraitRecursiveAggregateErrorKind::UnsupportedFieldType/,
    "doctest must check representative accepted and fail-closed recursive aggregate paths",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostMemoTraitAggregateProof\b|SelfhostMemoTraitAggregateProofStatus::Proven|selfhost_memo_trait_aggregate_proof_new|selfhost_memo_trait_aggregate_proof_known_new|selfhost_memo_trait_aggregate_proof_to_record|SelfhostMemoTraitEvidenceRecord|selfhost_memo_trait_evidence_table_push/,
    "recursive aggregate must not create aggregate proof objects, accepted aggregate proof, or consumer evidence records",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "recursive aggregate code must not use source text, spans, paths, display names, diagnostics, or lexemes as traversal authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "recursive aggregate policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait recursive aggregate contract passed");
