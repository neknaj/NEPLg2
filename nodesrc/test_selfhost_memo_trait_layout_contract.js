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
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_layout.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);

assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "ty root re-export file list must include memo_trait_layout.nepl",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "ty split file list must include memo_trait_layout.nepl",
);
assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_layout" as \*$/m,
    "ty facade must re-export the memo trait layout evidence split module",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "memo trait layout evidence must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /# ty\/memo_trait_layout[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "memo trait layout module documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitLayoutFieldRecord:[\s\S]*owner %SelfhostNamedTypeId[\s\S]*field_index %i32[\s\S]*field_type %SelfhostTypeId[\s\S]*pub struct SelfhostMemoTraitLayoutRecord:[\s\S]*nominal_id %SelfhostNamedTypeId[\s\S]*target_type %Option SelfhostTypeId[\s\S]*arity %i32[\s\S]*fields %SelfhostMemoTraitAggregateFieldRange/,
    "layout evidence must be constructor-identity keyed, concrete-applied-type keyed, and must store typed field TypeIds rather than raw names",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitLayoutEvidenceTable:[\s\S]*layouts %Vec SelfhostMemoTraitLayoutRecord[\s\S]*fields %Vec SelfhostMemoTraitLayoutFieldRecord/,
    "layout evidence must use a dedicated table for constructor layouts and field records",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitLayoutEvidenceErrorKind:[\s\S]*MissingTypeRecord[\s\S]*TargetNotAggregate[\s\S]*MissingLayout[\s\S]*UnsupportedLayoutKind[\s\S]*InvalidConstructorArity[\s\S]*InvalidFieldRange[\s\S]*FieldOwnerMismatch[\s\S]*FieldIndexMismatch[\s\S]*MissingFieldType[\s\S]*GenericArgumentUnsubstituted[\s\S]*NamedConstructorHasTypeParameters[\s\S]*AppliedArgumentArityMismatch[\s\S]*CycleLimitReached/,
    "layout evidence failures must be typed enum variants that cover missing layout, invalid range, owner mismatch, generic leakage, and arity mismatch",
);
assert.match(
    source,
    /wildcard arm は使いません[\s\S]*fn selfhost_memo_trait_layout_error_kind_code[\s\S]*SelfhostMemoTraitLayoutEvidenceErrorKind::MissingTypeRecord:[\s\S]*SelfhostMemoTraitLayoutEvidenceErrorKind::CycleLimitReached:[\s\S]*pub fn selfhost_memo_trait_layout_error_kind_eq/,
    "layout error-kind equality must be backed by an explicit exhaustive variant projection",
);
assert.match(
    source,
    /selfhost_memo_trait_layout_record_new %fn SelfhostNamedTypeId fn Option SelfhostTypeId[\s\S]*selfhost_memo_trait_layout_record_product_named[\s\S]*selfhost_memo_trait_layout_record_product_applied[\s\S]*some type_id/,
    "layout record constructors must distinguish named constructor-level layouts from concrete applied TypeId layouts",
);
assert.match(
    source,
    /fn selfhost_memo_trait_layout_record_target_eq[\s\S]*Option::Some left:[\s\S]*Option::Some right:[\s\S]*selfhost_type_id_eq left right[\s\S]*Option::None:[\s\S]*true[\s\S]*fn selfhost_memo_trait_layout_table_find_loop[\s\S]*selfhost_memo_trait_layout_record_target_eq record\.target_type target/,
    "layout lookup must compare concrete applied target TypeIds and must not reuse substituted layouts by constructor identity alone",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_layout_record_field_evidence[\s\S]*Result SelfhostMemoTraitAggregateFieldEvidence SelfhostMemoTraitLayoutEvidenceErrorKind[\s\S]*InvalidConstructorArity[\s\S]*selfhost_memo_trait_layout_range_in_table[\s\S]*InvalidFieldRange[\s\S]*selfhost_memo_trait_layout_validate_fields_loop[\s\S]*Result::Ok SelfhostMemoTraitAggregateFieldEvidence::Known layout\.fields/,
    "layout record materialization must be Result-returning and must validate arity, range, and field records before Known evidence is created",
);
assert.match(
    source,
    /fn selfhost_memo_trait_layout_validate_fields_loop[\s\S]*not selfhost_named_type_id_eq record\.owner nominal_id[\s\S]*FieldOwnerMismatch[\s\S]*not eq record\.field_index idx[\s\S]*FieldIndexMismatch[\s\S]*selfhost_memo_trait_layout_validate_field_type types record\.field_type/,
    "layout field validation must reject owner mismatch and field-index mismatch before accepting field type evidence",
);
assert.match(
    source,
    /fn selfhost_memo_trait_layout_validate_field_type[\s\S]*SelfhostTypeRecord::Parameter _parameter:[\s\S]*GenericArgumentUnsubstituted[\s\S]*Option::None:[\s\S]*MissingFieldType/,
    "field type validation must reject unsubstituted generic parameters and missing field TypeIds",
);
assert.match(
    source,
    /fn selfhost_memo_trait_layout_named_result[\s\S]*selfhost_memo_trait_layout_table_find_named table nominal_id[\s\S]*not eq layout\.arity 0[\s\S]*NamedConstructorHasTypeParameters[\s\S]*selfhost_memo_trait_layout_record_field_evidence table types layout/,
    "Named layout evidence must require a named-only arity-0 constructor layout before field evidence is returned",
);
assert.match(
    source,
    /fn selfhost_memo_trait_layout_applied_result[\s\S]*selfhost_memo_trait_layout_table_find_applied table nominal_id type_id[\s\S]*selfhost_applied_type_arg_range_count selfhost_applied_type_record_args applied[\s\S]*not eq layout\.arity arg_count[\s\S]*AppliedArgumentArityMismatch[\s\S]*selfhost_memo_trait_layout_validate_applied_args_loop types type_id arg_count 0[\s\S]*selfhost_memo_trait_layout_record_field_evidence table types layout/,
    "Applied layout evidence must require a concrete applied TypeId layout and verify constructor arity and argument substitution before field evidence is returned",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_layout_evidence_for_type_result[\s\S]*SelfhostTypeRecord::Primitive _kind:[\s\S]*TargetNotAggregate[\s\S]*SelfhostTypeRecord::Named named:[\s\S]*selfhost_memo_trait_layout_named_result[\s\S]*SelfhostTypeRecord::Parameter _parameter:[\s\S]*GenericArgumentUnsubstituted[\s\S]*SelfhostTypeRecord::Applied applied:[\s\S]*selfhost_memo_trait_layout_applied_result[\s\S]*SelfhostTypeRecord::Function _function:[\s\S]*TargetNotAggregate[\s\S]*Option::None:[\s\S]*MissingTypeRecord/,
    "layout evidence entry point must handle every SelfhostTypeRecord variant fail-closed",
);
assert.doesNotMatch(
    source,
    /%str|fn str|SelfhostSourceSpan|SelfhostSourceText|SelfhostDiagnostic/,
    "layout evidence accepted path must not use raw strings, source spans, source text, or diagnostics as authority",
);
assert.match(
    source,
    /selfhost_memo_trait_layout_stage0_named_layout[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result[\s\S]*selfhost_memo_trait_layout_stage0_applied_layout[\s\S]*selfhost_memo_trait_layout_record_product_applied nominal_id applied_id[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result/,
    "layout stage0 helpers must run the public named/applied validator entries instead of returning handwritten sample results",
);
assert.match(
    source,
    /selfhost_memo_trait_layout_stage0_missing_layout[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result[\s\S]*selfhost_memo_trait_layout_stage0_invalid_range[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result[\s\S]*selfhost_memo_trait_layout_stage0_owner_mismatch[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result[\s\S]*selfhost_memo_trait_layout_stage0_generic_field[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result[\s\S]*selfhost_memo_trait_layout_stage0_named_type_parameters[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result[\s\S]*selfhost_memo_trait_layout_stage0_applied_arity_mismatch[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result/,
    "layout stage0 fail-closed helpers must also run the public validator instead of returning handwritten error payloads",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_layout_stage0[\s\S]*selfhost_memo_trait_layout_stage0_missing_layout[\s\S]*selfhost_memo_trait_layout_stage0_invalid_range[\s\S]*selfhost_memo_trait_layout_stage0_owner_mismatch[\s\S]*selfhost_memo_trait_layout_stage0_generic_field[\s\S]*selfhost_memo_trait_layout_stage0_named_type_parameters[\s\S]*selfhost_memo_trait_layout_stage0_applied_arity_mismatch/,
    "layout stage0 summary must gather fail-closed results from the validator-backed helper functions",
);
assert.match(
    source,
    /selfhost_memo_trait_layout_evidence_result_is_accept summary\.named_layout[\s\S]*selfhost_memo_trait_layout_evidence_result_is_accept summary\.applied_layout[\s\S]*MissingLayout[\s\S]*InvalidFieldRange[\s\S]*FieldOwnerMismatch[\s\S]*GenericArgumentUnsubstituted[\s\S]*NamedConstructorHasTypeParameters[\s\S]*AppliedArgumentArityMismatch/,
    "layout stage0 doctest must check named/applied accepted paths and expose representative fail-closed layout errors",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "memo trait layout policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait layout contract passed");
