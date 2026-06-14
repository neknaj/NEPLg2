#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function stripDocComments(src) {
    return src
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

function functionBlock(src, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_binder.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_public_impl_generic_binder",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "generic binder module must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("count だけで generic impl を受理する退行を防ぎます") &&
        source.includes("generic impl の semantic success ではありません"),
    "docs must reject count-only generic impl acceptance and separate detailed shape evidence from semantic trait solving",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store record は accepted shape material に入りません"),
    "docs must exclude source, display, public-surface-hash, HIR, Resource IR, backend, and proof-store authority",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_generic_binder/,
    "generic binder must remain facade-private until public impl header orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_generic_binder/,
    "checker-layer generic binder must not be registered in the ty source list",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_operation_impl_candidate_builder|memo_trait_operation_classifier|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop|memo_trait_public_impl_header|private_cache|private_state)/,
    "generic binder must not import HIR, Resource IR, backend, proof store, operation classifier/candidate/proof layers, public impl header, or private effect layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericParameter:",
        "parameter_ordinal %i32",
        "binding %SelfhostTypeParameterBinding",
        "stable_symbol_hash %i32",
        "first_bound_index %i32",
        "bound_count %i32",
    ],
    "parameter evidence must carry ordinal, binder-indexed type-parameter identity, stable symbol hash, and a contiguous bound range",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericBound:",
        "parameter_ordinal %i32",
        "bound_ordinal %i32",
        "trait_application_shape_hash %Option i32",
        "trait_type_argument_count %i32",
    ],
    "bound evidence must carry parameter ordinal, bound ordinal, trait application shape hash, and type argument count",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplGenericBinderErrorKind:",
        "ParameterTableAllocFailed %StdErrorKind",
        "ParameterPushFailed %StdErrorKind",
        "BoundTableAllocFailed %StdErrorKind",
        "BoundPushFailed %StdErrorKind",
        "TypeParameterCountNegative",
        "TypeParameterBoundCountNegative",
        "ParameterCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "BoundCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "ParameterReadFailed %i32",
        "BoundReadFailed %i32",
        "ParameterOrdinalMismatch %SelfhostMemoTraitPublicImplGenericBinderOrdinalMismatch",
        "ParameterOrdinalPlaceholder %i32",
        "ParameterBindingInvalid %i32",
        "ParameterBindingDepthUnsupported %i32",
        "ParameterBindingIndexMismatch %SelfhostMemoTraitPublicImplGenericBinderOrdinalMismatch",
        "ParameterSymbolHashPlaceholder %i32",
        "BoundRangeStartNegative %i32",
        "BoundRangeStartMismatch %SelfhostMemoTraitPublicImplGenericBinderRangeMismatch",
        "BoundRangeCountNegative %i32",
        "BoundRangeOutOfBounds %SelfhostMemoTraitPublicImplGenericBinderRangeOutOfBounds",
        "BoundParameterOrdinalOutOfRange %i32",
        "BoundParameterOrdinalMismatch %SelfhostMemoTraitPublicImplGenericBinderBoundParameterMismatch",
        "BoundOrdinalMismatch %SelfhostMemoTraitPublicImplGenericBinderOrdinalMismatch",
        "BoundTraitApplicationShapeHashMissing %i32",
        "BoundTraitApplicationShapeHashPlaceholder %i32",
        "BoundTraitTypeArgumentCountNegative %i32",
        "DerivedBinderShapeHashPlaceholder",
    ],
    "errors must preserve setup, count, ordinal, range, bound shape, and derived hash failures as typed variants",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_binder_evidence_result"),
    [
        "lt expected_type_parameter_count 0",
        "TypeParameterCountNegative",
        "lt expected_type_parameter_bound_count 0",
        "TypeParameterBoundCountNegative",
        "selfhost_memo_trait_public_impl_generic_parameter_table_len parameters",
        "selfhost_memo_trait_public_impl_generic_bound_table_len bounds",
        "ParameterCountMismatch",
        "BoundCountMismatch",
        "selfhost_memo_trait_public_impl_generic_binder_validate_parameter_loop parameters bounds expected_type_parameter_count bound_len",
        "DerivedBinderShapeHashPlaceholder",
        "SelfhostMemoTraitPublicImplGenericBinderEvidence schema expected_type_parameter_count expected_type_parameter_bound_count shape_hash",
    ],
    "evidence API must validate expected counts, table lengths, detailed records, and nonzero derived shape hash before success",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_binder_validate_parameter_result"),
    [
        "lt parameter.parameter_ordinal 1",
        "ParameterOrdinalPlaceholder",
        "not eq parameter.parameter_ordinal expected_ordinal",
        "ParameterOrdinalMismatch",
        "not selfhost_type_parameter_binding_is_valid parameter.binding",
        "ParameterBindingInvalid",
        "not eq parameter.binding.binder_depth 0",
        "ParameterBindingDepthUnsupported",
        "not eq parameter.binding.parameter_index index",
        "ParameterBindingIndexMismatch",
        "eq parameter.stable_symbol_hash 0",
        "ParameterSymbolHashPlaceholder",
        "lt parameter.first_bound_index 0",
        "BoundRangeStartNegative",
        "not eq parameter.first_bound_index expected_bound_start",
        "BoundRangeStartMismatch",
        "lt parameter.bound_count 0",
        "BoundRangeCountNegative",
        "gt range_end bound_table_len",
        "BoundRangeOutOfBounds",
        "selfhost_memo_trait_public_impl_generic_binder_validate_bound_range_loop",
    ],
    "parameter validation must enforce ordered ordinal, valid phase-1 type-parameter binding, nonzero symbol hash, contiguous range, nonnegative count, and range bounds",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_binder_validate_bound_result"),
    [
        "lt bound.parameter_ordinal 1",
        "BoundParameterOrdinalOutOfRange",
        "gt bound.parameter_ordinal type_parameter_count",
        "BoundParameterOrdinalOutOfRange",
        "not eq bound.parameter_ordinal expected_parameter",
        "BoundParameterOrdinalMismatch",
        "not eq bound.bound_ordinal expected_bound_ordinal",
        "BoundOrdinalMismatch",
        "lt bound.trait_type_argument_count 0",
        "BoundTraitTypeArgumentCountNegative",
        "Option::Some shape_hash:",
        "eq shape_hash 0",
        "BoundTraitApplicationShapeHashPlaceholder",
        "Option::None:",
        "BoundTraitApplicationShapeHashMissing",
    ],
    "bound validation must enforce parameter ordinal, bound ordinal, nonnegative type-argument count, and explicit nonzero trait shape hash",
);
assert.doesNotMatch(
    code,
    /SelfhostMemoTraitOperationImplCandidate|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitAggregateProofStatus|NoDropRequired|PureDrop|PrivateCache|PrivateState|memo_call|SourceBacked|public_surface_hash|hash32\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|mix\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|\.path\b|\.alias\b|\.span\b|\.lexeme\b|display_name|diagnostic_text|source_path/,
    "generic binder must not fabricate impl candidates, operation evidence, aggregate proof status, private effects, memo_call acceptance, public surface hash authority, or source-derived hash material",
);
assert.doesNotMatch(
    source,
    /maxLine|line count limit|line-count limit|doc comment length cap|documentation length cap|行数制限|コメント量制限|ドキュメントコメント.{0,12}上限/,
    "generic binder contract must not introduce line-count or documentation-comment length limits",
);

console.log("selfhost memo trait public impl generic binder contract ok");
