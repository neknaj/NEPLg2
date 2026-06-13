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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_classifier.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const operationProducerRelPath = "stdlib/neplg2/core/check/module/memo_trait_operation_evidence_producer.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";

const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const operationProducer = read(operationProducerRelPath);
const proofStore = read(proofStoreRelPath);
const operationProducerCode = stripDocComments(operationProducer);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_classifier",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "operation classifier must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("MemoKey` / `MemoValue` の source identity とは別の型") &&
        source.includes("operation trait source を混同しない"),
    "docs must separate Copy/Drop/Eq/Hash operation trait sources from MemoKey/MemoValue source identity",
);
assert.ok(
    source.includes("trait application shape hash を再導出") &&
        source.includes("operation kind だけ、trait name string だけ、method name string だけでは accepted classifier evidence を作りません"),
    "docs must require shape re-derivation and reject operation/name-only authority",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、HIR、Resource IR、backend artifact、proof store record は accepted classifier authority に入りません"),
    "docs must exclude source/display/diagnostic/module path/HIR/Resource/backend/proof-store authority",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitOperationSourceIdentity:[\s\S]*operation %SelfhostMemoTraitOperationEvidenceKind[\s\S]*module_hash %i32[\s\S]*symbol_hash %i32[\s\S]*signature_hash %i32/,
    "operation source identity must carry operation kind and stable source fingerprints",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitOperationClassifierEvidence:[\s\S]*operation %SelfhostMemoTraitOperationEvidenceKind[\s\S]*classified_trait_application_shape_hash %Option i32/,
    "classifier module must own the operation classifier evidence consumed by the operation evidence producer",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitOperationTrustedSourceRegistry:[\s\S]*copy_source %SelfhostMemoTraitOperationSourceIdentity[\s\S]*drop_source %SelfhostMemoTraitOperationSourceIdentity[\s\S]*eq_source %SelfhostMemoTraitOperationSourceIdentity[\s\S]*hash_source %SelfhostMemoTraitOperationSourceIdentity/,
    "trusted operation source registry must contain Copy, Drop, Eq, and Hash source identities",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitOperationTraitApplicationInput:[\s\S]*trait_source %SelfhostMemoTraitOperationSourceIdentity[\s\S]*type_argument_count %i32[\s\S]*trait_application_shape_hash %Option i32/,
    "classifier input must carry trusted source candidate, type argument count, and normalized shape hash",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitOperationClassifierErrorKind:[\s\S]*SourceModuleFingerprintPlaceholder %SelfhostMemoTraitOperationEvidenceKind[\s\S]*TraitSourceNotTrusted %SelfhostMemoTraitOperationEvidenceKind[\s\S]*TraitTypeArgumentCountNegative[\s\S]*TraitTypeArgumentUnsupported[\s\S]*TraitApplicationShapeHashMissing[\s\S]*TraitApplicationShapeHashPlaceholder[\s\S]*TraitApplicationShapeHashMismatch[\s\S]*DerivedTraitApplicationShapeHashPlaceholder/,
    "classifier errors must distinguish source fingerprints, registry membership, type arguments, missing/placeholder shape, mismatch, and derived zero hash",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_classifier/,
    "operation classifier must remain facade-private until operation evidence orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_classifier/,
    "checker-layer classifier must not be registered in the ty source list",
);
assert.doesNotMatch(
    proofStore,
    /memo_trait_operation_classifier|SelfhostMemoTraitOperationTrustedSourceRegistry/,
    "proof store must not depend on checker-layer operation classifier",
);
assert.doesNotMatch(
    source,
    /#import "\.\/memo_trait_operation_evidence_producer" as \*/,
    "classifier must not import the downstream operation evidence producer",
);
assert.match(
    operationProducerCode,
    /#import "\.\/memo_trait_operation_classifier" as \*/,
    "operation evidence producer must consume classifier-owned evidence through a one-way import",
);
assert.doesNotMatch(
    operationProducerCode,
    /pub struct SelfhostMemoTraitOperationClassifierEvidence|pub fn selfhost_memo_trait_operation_classifier_evidence_new/,
    "operation evidence producer must not own classifier evidence types or constructors",
);
assert.doesNotMatch(
    source,
    /pub fn selfhost_memo_trait_operation_classifier_evidence_new/,
    "classifier evidence constructor must remain private so callers cannot bypass registry and shape validation",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_operation_classifier_evidence_result %fn SelfhostMemoTraitOperationTraitApplicationInput Result SelfhostMemoTraitOperationClassifierEvidence SelfhostMemoTraitOperationClassifierErrorKind \\input:/,
    "public classifier result API must take only the classifier input and obtain the trusted registry internally",
);
assert.doesNotMatch(
    source,
    /pub fn selfhost_memo_trait_operation_classifier_evidence_result %fn SelfhostMemoTraitOperationTrustedSourceRegistry/,
    "public classifier result API must not accept a caller-supplied trusted registry",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_trusted_source_registry_current_result"),
    [
        "selfhost_memo_trait_operation_current_copy_source",
        "selfhost_memo_trait_operation_source_fingerprint_result copy_source",
        "selfhost_memo_trait_operation_current_drop_source",
        "selfhost_memo_trait_operation_source_fingerprint_result drop_source",
        "selfhost_memo_trait_operation_current_eq_source",
        "selfhost_memo_trait_operation_source_fingerprint_result eq_source",
        "selfhost_memo_trait_operation_current_hash_source",
        "selfhost_memo_trait_operation_source_fingerprint_result hash_source",
        "selfhost_memo_trait_operation_trusted_source_registry_new",
    ],
    "current registry must validate all four prepared operation sources before constructing the registry",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_source_trusted_result"),
    [
        "selfhost_memo_trait_operation_source_fingerprint_result source",
        "selfhost_memo_trait_operation_trusted_source_for_operation registry source.operation",
        "selfhost_memo_trait_operation_source_identity_eq source expected",
        "Result::Ok unit",
        "TraitSourceNotTrusted source.operation",
    ],
    "source trust check must compare against the registry identity for the source operation",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_trait_application_shape_hash_result"),
    [
        "lt type_argument_count 0",
        "TraitTypeArgumentCountNegative",
        "gt type_argument_count 0",
        "TraitTypeArgumentUnsupported",
        "selfhost_memo_trait_operation_source_fingerprint_result source",
        "selfhost_memo_trait_operation_trait_application_shape_schema_version",
        "selfhost_memo_trait_operation_kind_shape_code source.operation",
        "source.module_hash source.symbol_hash source.signature_hash",
        "eq shape_hash 0",
        "DerivedTraitApplicationShapeHashPlaceholder",
    ],
    "shape hash derivation must fail closed on type args, validate source fingerprints, include schema/operation/source fields, and reject zero hash",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_classifier_evidence_with_registry_result"),
    [
        "selfhost_memo_trait_operation_source_trusted_result registry input.trait_source",
        "selfhost_memo_trait_operation_trait_application_shape_hash_result input.trait_source input.type_argument_count",
        "selfhost_memo_trait_operation_classifier_input_shape_result input",
        "eq expected_shape_hash input_shape_hash",
        "selfhost_memo_trait_operation_classifier_evidence_new input.trait_source.operation (some input_shape_hash)",
        "TraitApplicationShapeHashMismatch",
    ],
    "private registry-backed classifier helper must validate source registry membership, rederive the shape hash, compare it with input shape, and return shape-bound classifier evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_classifier_evidence_result"),
    [
        "selfhost_memo_trait_operation_trusted_source_registry_current_result",
        "Result::Ok registry:",
        "selfhost_memo_trait_operation_classifier_evidence_with_registry_result registry input",
        "Result::Err registry_error:",
        "Result::Err registry_error",
    ],
    "public classifier result must obtain the current trusted registry internally and delegate to the private registry-backed helper",
);
assert.doesNotMatch(
    code,
    /"Copy"|"Drop"|"Eq"|"Hash"|trait_name|method_name|display_name|diagnostic|source_text|span|lexeme|module_path/,
    "classifier implementation must not use operation names, method names, display text, diagnostics, source text, spans, lexemes, or module paths as authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "operation classifier policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation classifier contract passed");
