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

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?${kind}\\s+${name}\\b`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${kind} ${name} not found`);
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

function enumVariantNames(src, enumName) {
    return topLevelBlock(src, "enum", enumName)
        .split("\n")
        .slice(1)
        .map((line) => line.trim())
        .filter(Boolean)
        .map((line) => line.split(/\s+/)[0])
        .filter((name) => !name.startsWith("//"));
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

function assertDocBeforeTopLevel(src, docSnippet, declarationSnippet) {
    const declarationIndex = src.indexOf(declarationSnippet);
    assert.notEqual(declarationIndex, -1, `missing declaration ${declarationSnippet}`);
    const docIndex = src.lastIndexOf(docSnippet, declarationIndex);
    assert.notEqual(docIndex, -1, `missing doc snippet before ${declarationSnippet}`);
    const between = src.slice(docIndex, declarationIndex);
    assert.doesNotMatch(
        between,
        /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/m,
        `${docSnippet} must document the immediately following top-level declaration`,
    );
}

function assertEveryTopLevelDeclarationHasDoc(src) {
    const lines = src.split("\n");
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+\S+/;
    for (let i = 0; i < lines.length; i += 1) {
        if (!topLevel.test(lines[i])) {
            continue;
        }
        let cursor = i - 1;
        while (cursor >= 0 && lines[cursor].trim() === "") {
            cursor -= 1;
        }
        assert.ok(
            cursor >= 0 && lines[cursor].trimStart().startsWith("//:"),
            `top-level declaration must have an immediately preceding doc comment at line ${i + 1}: ${lines[i]}`,
        );
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_backend_readiness_orchestrator.nepl";
const backendRelPath = "stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";

const source = read(relPath);
const code = stripDocComments(source);
const backend = read(backendRelPath);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const runner = read(runnerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_private_effect_backend_readiness_orchestrator",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "backend readiness orchestrator must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("backend module の private upstream evidence 型はここでも使いません") &&
        source.includes("public handoff evidence") &&
        source.includes("non-executable readiness handoff"),
    "docs must keep backend private upstream types hidden and describe the handoff as non-executable readiness",
);
for (const [docSnippet, declarationSnippet] of [
    [
        "SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorErrorKind",
        "pub enum SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorErrorKind:",
    ],
    [
        "SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorStage0Summary",
        "pub struct SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorStage0Summary:",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_backend_readiness_handoff_status_from_mask_status",
        "pub fn selfhost_memo_trait_operation_private_effect_backend_readiness_handoff_status_from_mask_status",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_backend_readiness_handoff_evidence_from_mask_evidence",
        "pub fn selfhost_memo_trait_operation_private_effect_backend_readiness_handoff_evidence_from_mask_evidence",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_backend_readiness_count_from_mask_evidence_result",
        "pub fn selfhost_memo_trait_operation_private_effect_backend_readiness_count_from_mask_evidence_result",
    ],
]) {
    assertDocBeforeTopLevel(source, docSnippet, declarationSnippet);
}
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_private_effect_backend_readiness_orchestrator/,
    "backend readiness orchestrator must remain facade-private until full proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_private_effect_backend_readiness_orchestrator/,
    "backend readiness orchestrator must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_private_effect_backend_readiness_orchestrator_contract.js"),
    "source policy runner must execute the backend readiness orchestrator contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/codegen/memo_call_backend_private_cache_proof_gate\" as *",
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_private_effect_mask_evidence\" as *",
    ],
    "orchestrator imports must be limited to backend public handoff API, typed ids, operation kind, and mask evidence",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|resource_tree|resource_graph|memo_trait_operation_private_effect_no_escape_gate|memo_trait_operation_private_effect_resource_no_escape|memo_trait_operation_private_effect_slot_coverage|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|memo_trait_operation_drop_no_escape_gate|memo_trait_operation_drop_resource|wasm|llvm|prechecked|artifact)/,
    "orchestrator must not import Resource graph/proof internals, lower proof producers, slot coverage producer, proof store/artifact, public-surface materializers, backend bytes, or private cache/state implementation",
);
assert.doesNotMatch(
    code,
    /SelfhostMemoCallBackendPrivateCacheBackendReadinessUpstreamPrivateEffect|UpstreamPrivateEffect/,
    "orchestrator must not name backend-private upstream private-effect status or evidence",
);
assert.doesNotMatch(
    code,
    /GraphInput|ResourceGraph|ResourceProof|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|Wasm|LLVM|neplobj|neplproof|artifact|sealed|serializer|reader|prechecked/i,
    "orchestrator must not synthesize GraphInput, Resource proof records, request proof records, backend bytes, sealed representation, or artifacts",
);
assert.deepEqual(
    enumVariantNames(backend, "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus"),
    ["Proven", "Refuted", "Missing", "Unknown"],
    "backend public handoff status enum must remain narrow",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_backend_readiness_handoff_status_from_mask_status"),
    [
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Proven:",
        "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus::Proven",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Refuted:",
        "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus::Refuted",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Missing:",
        "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus::Missing",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Unknown:",
        "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus::Unknown",
    ],
    "mask status to backend handoff status conversion must preserve all four states",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_backend_readiness_handoff_status_from_mask_status"),
    /_:/,
    "mask status to backend handoff status conversion must not use wildcard fallback",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_backend_readiness_handoff_evidence_from_mask_evidence"),
    [
        "selfhost_memo_trait_operation_private_effect_backend_readiness_handoff_status_from_mask_status evidence.status",
        "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffEvidence evidence.body_root evidence.body_module_fingerprint status",
    ],
    "mask evidence to handoff evidence conversion must preserve body root and fingerprint from checker mask evidence",
);
assert.doesNotMatch(
    stripDocComments(functionBlock(source, "selfhost_memo_trait_operation_private_effect_backend_readiness_handoff_evidence_from_mask_evidence")),
    /evidence\.type_id|evidence\.operation|evidence\.required_slot_count|evidence\.proven_slot_count/,
    "handoff evidence conversion must not let type id, operation, or slot counts override body identity/status",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_backend_readiness_count_from_mask_evidence_result"),
    [
        "Result::Ok mask_evidence:",
        "selfhost_memo_trait_operation_private_effect_backend_readiness_handoff_evidence_from_mask_evidence mask_evidence",
        "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_gate_result_and_private_effect_handoff_evidence request_body_root request_body_module_fingerprint gate_result handoff_evidence",
        "BackendReadinessRejected backend_error",
        "Result::Err mask_error:",
        "MaskEvidenceRejected mask_error",
    ],
    "orchestrator count helper must pass mask success through backend public handoff API and keep mask Err separate",
);
assert.doesNotMatch(
    stripDocComments(functionBlock(source, "selfhost_memo_trait_operation_private_effect_backend_readiness_count_from_mask_evidence_result")),
    /required_slot_count|proven_slot_count|SelfhostMemoTraitOperationPrivateEffectMaskEvidence [a-z_]|SelfhostMemoCallBackendPrivateCacheProofGateSummary 1 1/,
    "production count helper must not inspect slot counts, construct mask evidence, or create fixture request evidence",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorStage0Summary"),
    [
        "accepted_count %Result i32 SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorErrorKind",
        "refuted_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorErrorKind",
        "missing_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorErrorKind",
        "unknown_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorErrorKind",
        "identity_mismatch_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorErrorKind",
        "mask_error_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectBackendReadinessOrchestratorErrorKind",
    ],
    "stage0 summary must cover accepted handoff, fail-closed statuses, identity mismatch, placeholder, and mask producer error",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_backend_readiness_orchestrator_stage0"),
    [
        "accepted_count",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Proven",
        "refuted_rejected",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Refuted",
        "missing_rejected",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Missing",
        "unknown_rejected",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Unknown",
        "identity_mismatch_rejected",
        "mismatched_request_root",
        "placeholder_rejected",
        "request_root 0",
        "BodyModuleFingerprintPlaceholder",
        "mask_error_rejected",
    ],
    "stage0 must exercise accepted, Refuted, Missing, Unknown, mismatch, placeholder, and mask error paths",
);

console.log("selfhost private-effect backend readiness orchestrator contract ok");
