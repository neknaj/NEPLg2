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

function functionBlock(src, name) {
    return topLevelBlock(src, "fn", name);
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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_actual_traversal_coverage_bridge.nepl";
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
        "# check/module/memo_trait_operation_private_effect_actual_traversal_coverage_bridge",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "actual traversal coverage bridge must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("caller context の `type_id` / `operation`") &&
        source.includes("coverage table 構築前") &&
        source.includes("EffectAbsentAfterCompleteTraversal") &&
        source.includes("backend module の private upstream evidence 型は使いません"),
    "docs must keep caller context, pre-table validation, explicit absence, and backend-private upstream hiding explicit",
);
for (const [docSnippet, declarationSnippet] of [
    [
        "SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind",
        "pub enum SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind:",
    ],
    [
        "SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeStage0Summary",
        "pub struct SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeStage0Summary:",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_slot_status_from_handoff_status",
        "pub fn selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_slot_status_from_handoff_status",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_record_from_handoff_evidence",
        "pub fn selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_record_from_handoff_evidence",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_mask_evidence_from_handoff_pair_result",
        "pub fn selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_mask_evidence_from_handoff_pair_result",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_backend_readiness_count_from_handoff_pair_result",
        "pub fn selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_backend_readiness_count_from_handoff_pair_result",
    ],
]) {
    assertDocBeforeTopLevel(source, docSnippet, declarationSnippet);
}
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_private_effect_actual_traversal_coverage_bridge/,
    "actual traversal coverage bridge must remain facade-private until full proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_private_effect_actual_traversal_coverage_bridge/,
    "actual traversal coverage bridge must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_contract.js"),
    "source policy runner must execute the actual traversal coverage bridge contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/codegen/memo_call_backend_private_cache_proof_gate\" as *",
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_private_effect_backend_readiness_orchestrator\" as *",
        "#import \"./memo_trait_operation_private_effect_mask_evidence\" as *",
        "#import \"./memo_trait_operation_private_effect_no_escape_gate\" as *",
        "#import \"./memo_trait_operation_private_effect_slot_coverage_producer\" as *",
    ],
    "bridge imports must be limited to backend public handoff API, typed ids/effects, readiness orchestrator, mask evidence, proof free, and slot coverage producer",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|resource_tree|resource_graph|memo_trait_operation_private_effect_resource_no_escape|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|memo_trait_operation_drop_no_escape_gate|memo_trait_operation_drop_resource|wasm|llvm|prechecked|artifact)/,
    "bridge must not import Resource graph/proof internals, lower traversal/materializer layers, proof store/artifact, public-surface materializers, backend bytes, or Drop proof layers",
);
assert.doesNotMatch(
    code,
    /SelfhostMemoCallBackendPrivateCacheBackendReadinessUpstreamPrivateEffect|UpstreamPrivateEffect/,
    "bridge must not name backend-private upstream private-effect status or evidence",
);
assert.doesNotMatch(
    code,
    /GraphInput|ResourceGraph(?:Id|Input|Body|Place|Edge|Record|Table)|ResourceProof|RequestEvidenceProven|PrivateCacheNoEscapeProven|Wasm|LLVM|neplobj|neplproof|artifact|sealed|serializer|prechecked/i,
    "bridge must not synthesize GraphInput, Resource proof records, request proof records, backend bytes, sealed representation, or artifacts",
);
assert.doesNotMatch(
    code,
    /\bselfhost_memo_trait_operation_private_effect_no_escape_proof_(?:key_new|record_new|table_new|table_push)\b/,
    "bridge must not directly construct proof keys, records, or proof tables",
);
assert.doesNotMatch(
    code,
    /(?<!%)\bSelfhostMemoTraitOperationPrivateEffectNoEscapeProof(?:Key|Record|Table)\s+[a-z_]/,
    "bridge must not bypass slot coverage producer with raw no-escape proof struct constructors",
);
assert.match(
    code,
    /\bselfhost_memo_trait_operation_private_effect_no_escape_proof_table_free\b/,
    "bridge may use the narrow proof table free helper after borrowed mask evidence lookup",
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
assert.doesNotMatch(
    stripDocComments(topLevelBlock(backend, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffEvidence")),
    /ActualWalkerTraversalSource|Proof|MaskEvidence|SlotCoverage|GraphInput|Wasm|LLVM|neplobj|neplproof|artifact|sealed/i,
    "backend coverage handoff evidence must not expose traversal source tables, checker proof, graph, backend, or artifact payloads",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind"),
    [
        "CoverageTableAllocFailed",
        "CoverageHandoffIdentityMismatch",
        "CoverageHandoffRequestIdentityMismatch",
        "CoverageHandoffEffectMismatch",
        "SlotCoverageRejected",
        "MaskEvidenceRejected",
        "BackendReadinessRejected",
    ],
    "bridge error enum must keep allocation, identity/effect, slot, mask, and backend readiness errors distinct",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_slot_status_from_handoff_status"),
    [
        "EffectObservedNoEscape:",
        "SelfhostMemoTraitOperationPrivateEffectSlotCoverageStatus::EffectObservedNoEscape",
        "EffectObservedMayEscape:",
        "SelfhostMemoTraitOperationPrivateEffectSlotCoverageStatus::EffectObservedMayEscape",
        "EffectAbsentAfterCompleteTraversal:",
        "SelfhostMemoTraitOperationPrivateEffectSlotCoverageStatus::EffectAbsentAfterCompleteTraversal",
        "ResourceGraphMissing:",
        "SelfhostMemoTraitOperationPrivateEffectSlotCoverageStatus::ResourceGraphMissing",
        "TraversalUnsupported:",
        "SelfhostMemoTraitOperationPrivateEffectSlotCoverageStatus::TraversalUnsupported",
    ],
    "handoff status conversion must preserve all five coverage states",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_slot_status_from_handoff_status"),
    /_:/,
    "handoff status conversion must not use wildcard fallback",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_record_from_handoff_evidence"),
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_slot_status_from_handoff_status handoff.status",
        "selfhost_memo_trait_operation_private_effect_slot_coverage_record_new type_id operation handoff.body_module_fingerprint handoff.body_root handoff.effect SelfhostEffectEscapeState::NotApplicable status",
    ],
    "handoff evidence conversion must add caller type_id/operation and preserve body identity, effect, and status",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_validate_pair_result"),
    [
        "eq request_body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "eq cache_handoff.body_module_fingerprint 0",
        "eq state_handoff.body_module_fingerprint 0",
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_handoff_body_identity_eq cache_handoff state_handoff",
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_handoff_request_identity_eq cache_handoff request_body_root request_body_module_fingerprint",
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_validate_effect_result cache_handoff SelfhostEffectKind::PrivateCache",
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_validate_effect_result state_handoff SelfhostEffectKind::PrivateState",
    ],
    "bridge must validate placeholder, pair identity, request identity, and exact PrivateCache/PrivateState effects before table construction",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_table_from_handoff_pair_result"),
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_validate_pair_result",
        "selfhost_memo_trait_operation_private_effect_slot_coverage_table_new",
        "selfhost_memo_trait_operation_private_effect_slot_coverage_table_push table0 cache_record",
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_push_state_result table1",
    ],
    "bridge must validate before building a fixed two-slot coverage table and push cache before state",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_mask_evidence_from_table_result"),
    [
        "selfhost_memo_trait_operation_private_effect_slot_coverage_proof_table_result &table type_id operation body_module_fingerprint body_root",
        "selfhost_memo_trait_operation_private_effect_mask_evidence_result &proofs type_id operation body_module_fingerprint body_root",
        "MaskEvidenceRejected mask_error",
        "selfhost_memo_trait_operation_private_effect_no_escape_proof_table_free proofs",
        "selfhost_memo_trait_operation_private_effect_slot_coverage_table_free table",
    ],
    "bridge must produce proof table through slot coverage producer, borrow it for mask evidence, and free all owners",
);
assert.doesNotMatch(
    stripDocComments(functionBlock(source, "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_mask_evidence_from_handoff_pair_result")),
    /stage0|SelfhostMemoTraitOperationPrivateEffectMaskEvidence [a-z_].*SelfhostMemoTraitOperationPrivateEffectMaskEvidence/,
    "production mask helper must not call stage0 helpers or synthesize mask evidence directly",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_backend_readiness_count_from_handoff_pair_result"),
    [
        "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_mask_evidence_from_handoff_pair_result",
        "Result::Ok mask_evidence:",
        "selfhost_memo_trait_operation_private_effect_backend_readiness_count_from_mask_evidence_result request_body_root request_body_module_fingerprint gate_result mask_result",
        "BackendReadinessRejected backend_error",
    ],
    "bridge readiness helper must pass mask success through existing backend readiness orchestrator",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeStage0Summary"),
    [
        "absent_readiness_count %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind",
        "observed_absent_readiness_count %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind",
        "may_escape_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind",
        "missing_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind",
        "unknown_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind",
        "identity_mismatch_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind",
        "request_identity_mismatch_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind",
        "effect_mismatch_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoTraitOperationPrivateEffectActualTraversalCoverageBridgeErrorKind",
    ],
    "stage0 summary must cover accepted coverage, fail-closed statuses, identity errors, effect mismatch, and placeholder",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_actual_traversal_coverage_bridge_stage0"),
    [
        "cache_absent",
        "EffectAbsentAfterCompleteTraversal",
        "cache_observed",
        "EffectObservedNoEscape",
        "cache_may_escape",
        "EffectObservedMayEscape",
        "cache_missing",
        "ResourceGraphMissing",
        "cache_unknown",
        "TraversalUnsupported",
        "identity_mismatch_rejected",
        "request_identity_mismatch_rejected",
        "effect_mismatch_rejected",
        "placeholder_rejected",
    ],
    "stage0 must exercise explicit absence, observed no-escape, may escape, missing, unknown, pair mismatch, request mismatch, effect mismatch, and placeholder",
);

console.log("selfhost private-effect actual traversal coverage bridge contract ok");
