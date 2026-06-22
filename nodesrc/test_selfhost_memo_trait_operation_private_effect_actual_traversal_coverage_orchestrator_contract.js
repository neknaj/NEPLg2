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

const relPath =
    "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator.nepl";
const backendRelPath = "stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl";
const bridgeRelPath =
    "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_actual_traversal_coverage_bridge.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";

const source = read(relPath);
const code = stripDocComments(source);
const backend = read(backendRelPath);
const bridge = read(bridgeRelPath);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const runner = read(runnerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "actual traversal coverage orchestrator must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("public `HandoffEvidence` 2 件") &&
        source.includes("public pair 型は追加しません") &&
        source.includes("complete authority、source table、handoff pair、reader context、resolution table"),
    "docs must keep the public two-evidence boundary and private producer payload hiding explicit",
);
for (const [docSnippet, declarationSnippet] of [
    [
        "SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorErrorKind",
        "pub enum SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorErrorKind:",
    ],
    [
        "SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorStage0Summary",
        "pub struct SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorStage0Summary:",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator_readiness_count_from_public_handoff_result",
        "pub fn selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator_readiness_count_from_public_handoff_result",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator_error_kind_eq",
        "pub fn selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator_error_kind_eq",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator_stage0",
        "pub fn selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator_stage0",
    ],
]) {
    assertDocBeforeTopLevel(source, docSnippet, declarationSnippet);
}
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator/,
    "actual traversal coverage orchestrator must remain facade-private until full proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator/,
    "actual traversal coverage orchestrator must not be registered in the ty source list",
);
assert.ok(
    runner.includes(
        "nodesrc/test_selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator_contract.js",
    ),
    "source policy runner must execute the actual traversal coverage orchestrator contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/codegen/memo_call_backend_private_cache_proof_gate\" as *",
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_private_effect_actual_traversal_coverage_bridge\" as *",
    ],
    "orchestrator imports must be limited to backend public handoff API, typed ids/effects, operation kind, and the coverage bridge",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:memo_trait_operation_private_effect_backend_readiness_orchestrator|memo_trait_operation_private_effect_mask_evidence|memo_trait_operation_private_effect_no_escape_gate|memo_trait_operation_private_effect_resource_no_escape|memo_trait_operation_private_effect_slot_coverage_producer|resource|resource_tree|resource_graph|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|memo_trait_operation_drop_no_escape_gate|memo_trait_operation_drop_resource|wasm|llvm|prechecked|artifact)/,
    "orchestrator must not import lower proof producers, mask/backend readiness modules, Resource graph/proof internals, public materializers, backend bytes, or artifacts",
);
assert.doesNotMatch(
    code,
    /SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverage(?:CompleteAuthority|HandoffPair)|SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSource|SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext|SelfhostMemoCallBackendPrivateCacheActualTraversalBodyResolutionTable|FreshWitness|ProofTable|GraphInput|ResourceGraph|ResourceProof|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|Wasm|LLVM|neplobj|neplproof|artifact|sealed|serializer|prechecked/i,
    "orchestrator must not expose backend private pair/authority/source/context/resolution payloads, proof tables, backend bytes, sealed representation, or artifacts",
);
assert.deepEqual(
    enumVariantNames(backend, "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffStatus"),
    [
        "EffectObservedNoEscape",
        "EffectObservedMayEscape",
        "EffectAbsentAfterCompleteTraversal",
        "ResourceGraphMissing",
        "TraversalUnsupported",
    ],
    "backend public coverage handoff status enum must remain the explicit five-state transport",
);
assertOrdered(
    topLevelBlock(backend, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffEvidence"),
    [
        "body_root %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "effect %SelfhostEffectKind",
        "status %SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffStatus",
    ],
    "backend coverage handoff evidence must carry only body identity, effect, and coverage status",
);
for (const privateName of [
    "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageCompleteAuthority",
    "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffPair",
    "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable",
    "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext",
    "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyResolutionTable",
]) {
    assert.doesNotMatch(
        backend,
        new RegExp(`^pub\\s+struct\\s+${privateName}\\b`, "m"),
        `${privateName} must remain backend-private and must not become a public orchestration payload`,
    );
}
assert.match(
    bridge,
    /pub fn selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_backend_readiness_count_from_handoff_pair_result/,
    "coverage bridge must provide the existing public handoff pair readiness helper",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorErrorKind"),
    ["CoverageBridgeRejected"],
    "orchestrator error enum must only wrap coverage bridge rejection",
);
assertOrdered(
    functionBlock(
        source,
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator_readiness_count_from_public_handoff_result",
    ),
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_backend_readiness_count_from_handoff_pair_result type_id operation request_body_root request_body_module_fingerprint gate_result cache_handoff state_handoff",
        "Result::Ok count:",
        "Result::Ok count",
        "Result::Err bridge_error:",
        "CoverageBridgeRejected bridge_error",
    ],
    "production orchestrator helper must delegate public evidence directly to the coverage bridge and wrap bridge rejection",
);
assert.doesNotMatch(
    stripDocComments(
        functionBlock(
            source,
            "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator_readiness_count_from_public_handoff_result",
        ),
    ),
    /cache_handoff\.(?:status|effect|body_root|body_module_fingerprint)|state_handoff\.(?:status|effect|body_root|body_module_fingerprint)|let\s+\S+\s+%SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffEvidence|SelfhostMemoCallBackendPrivateCacheProofGateSummary 1 1/,
    "production orchestrator helper must not inspect handoff payloads, synthesize handoff evidence, or create fixture request evidence",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorStage0Summary"),
    [
        "accepted_absence_count %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorErrorKind",
        "unsupported_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorErrorKind",
        "request_evidence_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorErrorKind",
        "request_identity_mismatch_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorErrorKind",
        "effect_mismatch_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageOrchestratorErrorKind",
    ],
    "stage0 summary must cover accepted absence, unsupported coverage, request-evidence rejection, request identity mismatch, and effect mismatch",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_orchestrator_stage0"),
    [
        "accepted_gate_result",
        "rejected_gate_result",
        "cache_absent",
        "EffectAbsentAfterCompleteTraversal",
        "state_absent",
        "EffectAbsentAfterCompleteTraversal",
        "cache_unsupported",
        "TraversalUnsupported",
        "wrong_cache_effect",
        "SelfhostEffectKind::PrivateState",
        "accepted_absence_count",
        "unsupported_rejected",
        "request_evidence_rejected",
        "request_identity_mismatch_rejected",
        "effect_mismatch_rejected",
    ],
    "stage0 must exercise accepted absence and the fail-closed bridge paths the upper orchestrator forwards",
);

console.log("selfhost actual traversal private-effect coverage orchestrator contract ok");
