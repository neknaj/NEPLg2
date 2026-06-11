#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait" as \*$/m,
    "ty facade must re-export the memo trait predicate split module",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "memo trait predicate must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /# ty\/memo_trait[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "memo trait module documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitRejectKind:[\s\S]*MissingTypeRecord[\s\S]*ErrorTypeUnsupported[\s\S]*I64Unsupported[\s\S]*F32KeyUnsupported[\s\S]*F64Unsupported[\s\S]*StrUnsupported[\s\S]*NeverUnsupported[\s\S]*FunctionUnsupported[\s\S]*NamedLayoutUnknown[\s\S]*AppliedLayoutUnknown[\s\S]*ParameterUnresolved/,
    "memo trait predicate must expose typed reject reasons instead of collapsing failure into bool",
);
assert.match(
    source,
    /wildcard arm は使いません[\s\S]*pub fn selfhost_memo_trait_reject_kind_eq[\s\S]*SelfhostMemoTraitRejectKind::MissingTypeRecord:[\s\S]*SelfhostMemoTraitRejectKind::ParameterUnresolved:/,
    "memo trait reject-kind equality must be explicit and update-required when variants are added",
);
assert.match(
    source,
    /fn selfhost_memo_key_primitive_result[\s\S]*SelfhostPrimitiveTypeKind::Unit:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::Bool:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::I32:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::I64:[\s\S]*I64Unsupported[\s\S]*SelfhostPrimitiveTypeKind::U8:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::Char:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::Str:[\s\S]*StrUnsupported[\s\S]*SelfhostPrimitiveTypeKind::F32:[\s\S]*F32KeyUnsupported[\s\S]*SelfhostPrimitiveTypeKind::F64:[\s\S]*F64Unsupported[\s\S]*SelfhostPrimitiveTypeKind::Never:[\s\S]*NeverUnsupported/,
    "MemoKey Phase 1 primitive predicate must accept unit/bool/i32/u8/char and reject f32/f64/i64/str/never",
);
assert.match(
    source,
    /fn selfhost_memo_value_primitive_result[\s\S]*SelfhostPrimitiveTypeKind::Unit:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::Bool:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::I32:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::I64:[\s\S]*I64Unsupported[\s\S]*SelfhostPrimitiveTypeKind::U8:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::Char:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::Str:[\s\S]*StrUnsupported[\s\S]*SelfhostPrimitiveTypeKind::F32:[\s\S]*Result::Ok unit[\s\S]*SelfhostPrimitiveTypeKind::F64:[\s\S]*F64Unsupported[\s\S]*SelfhostPrimitiveTypeKind::Never:[\s\S]*NeverUnsupported/,
    "MemoValue Phase 1 primitive predicate must accept f32 as a value while rejecting unsupported scalar/string/never cases",
);
assert.match(
    source,
    /pub fn selfhost_memo_key_type_result[\s\S]*SelfhostTypeRecord::Named _named:[\s\S]*NamedLayoutUnknown[\s\S]*SelfhostTypeRecord::Parameter _parameter:[\s\S]*ParameterUnresolved[\s\S]*SelfhostTypeRecord::Applied _applied:[\s\S]*AppliedLayoutUnknown[\s\S]*SelfhostTypeRecord::Function _function:[\s\S]*FunctionUnsupported[\s\S]*Option::None:[\s\S]*MissingTypeRecord/,
    "MemoKey type predicate must fail closed for named/applied/parameter/function/missing records",
);
assert.match(
    source,
    /pub fn selfhost_memo_value_type_result[\s\S]*SelfhostTypeRecord::Named _named:[\s\S]*NamedLayoutUnknown[\s\S]*SelfhostTypeRecord::Parameter _parameter:[\s\S]*ParameterUnresolved[\s\S]*SelfhostTypeRecord::Applied _applied:[\s\S]*AppliedLayoutUnknown[\s\S]*SelfhostTypeRecord::Function _function:[\s\S]*FunctionUnsupported[\s\S]*Option::None:[\s\S]*MissingTypeRecord/,
    "MemoValue type predicate must fail closed for named/applied/parameter/function/missing records",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitEvidenceRecord:[\s\S]*type_id %SelfhostTypeId[\s\S]*key_result %Result unit SelfhostMemoTraitRejectKind[\s\S]*value_result %Result unit SelfhostMemoTraitRejectKind[\s\S]*pub struct SelfhostMemoTraitEvidenceTable:[\s\S]*records %Vec SelfhostMemoTraitEvidenceRecord/,
    "memo trait aggregate evidence must use typed Result payloads in a dedicated evidence table",
);
assert.match(
    source,
    /SelfhostMemoTraitEvidenceTable:[\s\S]*永続 artifact の key ではありません[\s\S]*canonical type key と solver policy hash/,
    "memo trait evidence table must be documented as session-local, not a persistent artifact identity",
);
assert.match(
    source,
    /pub fn selfhost_memo_key_type_result_with_evidence[\s\S]*SelfhostTypeRecord::Primitive kind:[\s\S]*selfhost_memo_key_primitive_result kind[\s\S]*SelfhostTypeRecord::Named _named:[\s\S]*selfhost_memo_trait_evidence_table_find evidence type_id[\s\S]*evidence_record\.key_result[\s\S]*NamedLayoutUnknown[\s\S]*SelfhostTypeRecord::Parameter _parameter:[\s\S]*ParameterUnresolved[\s\S]*SelfhostTypeRecord::Applied _applied:[\s\S]*selfhost_memo_trait_evidence_table_find evidence type_id[\s\S]*evidence_record\.key_result[\s\S]*AppliedLayoutUnknown[\s\S]*SelfhostTypeRecord::Function _function:[\s\S]*FunctionUnsupported[\s\S]*Option::None:[\s\S]*MissingTypeRecord/,
    "MemoKey evidence-aware predicate may consume evidence only for named/applied aggregate records",
);
assert.match(
    source,
    /pub fn selfhost_memo_value_type_result_with_evidence[\s\S]*SelfhostTypeRecord::Primitive kind:[\s\S]*selfhost_memo_value_primitive_result kind[\s\S]*SelfhostTypeRecord::Named _named:[\s\S]*selfhost_memo_trait_evidence_table_find evidence type_id[\s\S]*evidence_record\.value_result[\s\S]*NamedLayoutUnknown[\s\S]*SelfhostTypeRecord::Parameter _parameter:[\s\S]*ParameterUnresolved[\s\S]*SelfhostTypeRecord::Applied _applied:[\s\S]*selfhost_memo_trait_evidence_table_find evidence type_id[\s\S]*evidence_record\.value_result[\s\S]*AppliedLayoutUnknown[\s\S]*SelfhostTypeRecord::Function _function:[\s\S]*FunctionUnsupported[\s\S]*Option::None:[\s\S]*MissingTypeRecord/,
    "MemoValue evidence-aware predicate may consume evidence only for named/applied aggregate records",
);
assert.match(
    source,
    /selfhost_memo_trait_reject_kind_eq \(unwrap_err summary\.named_without_evidence\) SelfhostMemoTraitRejectKind::NamedLayoutUnknown[\s\S]*selfhost_memo_trait_result_is_accept summary\.named_key_with_evidence[\s\S]*selfhost_memo_trait_result_is_accept summary\.named_value_with_evidence[\s\S]*selfhost_memo_trait_reject_kind_eq \(unwrap_err summary\.named_key_rejected_by_evidence\) SelfhostMemoTraitRejectKind::StrUnsupported[\s\S]*selfhost_memo_trait_reject_kind_eq \(unwrap_err summary\.applied_without_evidence\) SelfhostMemoTraitRejectKind::AppliedLayoutUnknown[\s\S]*selfhost_memo_trait_result_is_accept summary\.applied_key_with_evidence[\s\S]*selfhost_memo_trait_result_is_accept summary\.applied_value_with_evidence[\s\S]*selfhost_memo_trait_reject_kind_eq \(unwrap_err summary\.applied_key_rejected_by_evidence\) SelfhostMemoTraitRejectKind::StrUnsupported[\s\S]*selfhost_memo_trait_reject_kind_eq \(unwrap_err summary\.f32_key_with_fake_evidence\) SelfhostMemoTraitRejectKind::F32KeyUnsupported[\s\S]*selfhost_memo_trait_reject_kind_eq \(unwrap_err summary\.missing_with_fake_evidence\) SelfhostMemoTraitRejectKind::MissingTypeRecord/,
    "memo trait evidence stage0 must prove named/applied no-evidence reject, evidence-backed accept/reject, primitive override rejection, and missing-record override rejection",
);
assert.match(
    source,
    /pub fn selfhost_memo_key_type_is_allowed[\s\S]*selfhost_memo_trait_result_is_accept selfhost_memo_key_type_result arena type_id[\s\S]*pub fn selfhost_memo_value_type_is_allowed[\s\S]*selfhost_memo_trait_result_is_accept selfhost_memo_value_type_result arena type_id/,
    "bool adapters may exist, but must be derived from the typed Result predicate",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "memo trait predicate policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait predicate contract passed");
