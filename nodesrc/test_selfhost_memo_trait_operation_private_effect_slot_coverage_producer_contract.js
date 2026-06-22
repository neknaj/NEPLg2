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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_slot_coverage_producer.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";

const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const runner = read(runnerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_private_effect_slot_coverage_producer",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "slot coverage producer must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("片方の slot が無い場合は effect 不在ではなく coverage 欠落として拒否します") &&
        source.includes("EffectAbsentAfterCompleteTraversal") &&
        source.includes("actual traversal が同じ body slot を完全に走査した"),
    "docs must define absence as explicit complete-traversal coverage, not inferred missing slot",
);
assert.ok(
    source.includes("production API は coverage から Resource observation table または proof table を作るところで止めます") &&
        source.includes("mask evidence 接続は stage0 smoke だけ"),
    "docs must keep production API below mask evidence orchestration",
);
for (const [docSnippet, declarationSnippet] of [
    [
        "SelfhostMemoTraitOperationPrivateEffectSlotCoverageStatus",
        "pub enum SelfhostMemoTraitOperationPrivateEffectSlotCoverageStatus:",
    ],
    [
        "SelfhostMemoTraitOperationPrivateEffectSlotCoverageRecord",
        "pub struct SelfhostMemoTraitOperationPrivateEffectSlotCoverageRecord:",
    ],
    [
        "SelfhostMemoTraitOperationPrivateEffectSlotCoverageTable",
        "pub struct SelfhostMemoTraitOperationPrivateEffectSlotCoverageTable:",
    ],
    [
        "SelfhostMemoTraitOperationPrivateEffectSlotCoverageProducerErrorKind",
        "pub enum SelfhostMemoTraitOperationPrivateEffectSlotCoverageProducerErrorKind:",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_slot_coverage_observation_table_result",
        "pub fn selfhost_memo_trait_operation_private_effect_slot_coverage_observation_table_result",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_slot_coverage_proof_table_result",
        "pub fn selfhost_memo_trait_operation_private_effect_slot_coverage_proof_table_result",
    ],
]) {
    assertDocBeforeTopLevel(source, docSnippet, declarationSnippet);
}
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_private_effect_slot_coverage_producer/,
    "slot coverage producer must remain facade-private until full Resource proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_private_effect_slot_coverage_producer/,
    "checker-layer slot coverage producer must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_private_effect_slot_coverage_producer_contract.js"),
    "source policy runner must execute the slot coverage producer contract",
);
assertOrdered(
    source,
    [
        "#import \"./memo_trait_operation_private_effect_mask_evidence\" as *",
        "#import \"./memo_trait_operation_private_effect_no_escape_gate\" as *",
        "#import \"./memo_trait_operation_private_effect_resource_no_escape_producer\" as *",
    ],
    "slot coverage producer may use mask evidence only for stage0 smoke and must delegate proof construction to existing no-escape producer boundaries",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:backend|memo_call|codegen|resource\/|resource_tree|resource_graph|resource_graph_input_scanner|resource_no_escape_materializer|resource_no_escape_traversal_collector|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|memo_trait_operation_drop_no_escape_gate|memo_trait_operation_drop_resource|private_cache|private_state)/,
    "slot coverage producer must not import backend, memo_call, graph scanner/traversal/materializer internals, proof store/artifact, canonical-key, public-surface, evidence producer, impl table, purity gate, Drop proof layers, PrivateCache, or PrivateState layers",
);
assert.doesNotMatch(
    code,
    /\bselfhost_memo_trait_operation_private_effect_no_escape_proof_(?:key_new|record_new|table_new|table_push)\b/,
    "slot coverage producer must not directly construct proof keys, records, or proof tables",
);
assert.doesNotMatch(
    code,
    /(?<!%)\bSelfhostMemoTraitOperationPrivateEffectNoEscapeProof(?:Key|Record|Table)\s+[a-z_]/,
    "slot coverage producer must not bypass Resource no-escape producer with raw proof struct constructors",
);
assert.doesNotMatch(
    code,
    /SelfhostMemoCallBackend|BackendReadiness|GraphInput|ResourceGraph(?:Id|Body|Place|Edge|Input)|resource_graph_|RequestEvidenceProven|PrivateCacheNoEscapeProven|Wasm|LLVM|neplobj|neplproof|artifact|sealed|serializer|reader|prechecked/i,
    "slot coverage producer must not synthesize backend readiness, Resource graph, request evidence, backend bytes, sealed representation, or artifact policy",
);
assert.doesNotMatch(
    source,
    /^pub\s+fn\s+selfhost_memo_trait_operation_private_effect_slot_coverage_mask_evidence_result\b/m,
    "slot coverage producer must not expose a production public mask evidence helper",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoTraitOperationPrivateEffectSlotCoverageStatus"),
    [
        "EffectObservedNoEscape",
        "EffectObservedMayEscape",
        "EffectAbsentAfterCompleteTraversal",
        "ResourceGraphMissing",
        "TraversalUnsupported",
    ],
    "slot coverage status enum must keep observed, explicit absent, missing, and unsupported states distinct",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoTraitOperationPrivateEffectSlotCoverageRecord"),
    [
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
        "status %SelfhostMemoTraitOperationPrivateEffectSlotCoverageStatus",
    ],
    "coverage record must carry typed body identity, effect, escape, and slot status",
);
assert.doesNotMatch(
    topLevelBlock(source, "struct", "SelfhostMemoTraitOperationPrivateEffectSlotCoverageRecord"),
    /payload_hash|signature_hash|body_hash|public_surface|source_text|source_span|source_path|\bspan\b|\bpath\b|\bname\b|diagnostic|message|text/i,
    "coverage record must not use source/display/hash authority instead of typed body identity",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_slot_coverage_validate_effect"),
    [
        "SelfhostEffectKind::PrivateState:",
        "Result::Ok unit",
        "SelfhostEffectKind::PrivateCache:",
        "Result::Ok unit",
        "SelfhostEffectKind::Pure:",
        "CoverageEffectNotPrivate effect",
        "SelfhostEffectKind::InternalAlloc:",
        "CoverageEffectNotPrivate effect",
        "SelfhostEffectKind::Nondet:",
        "CoverageEffectNotPrivate effect",
    ],
    "effect validation must accept only PrivateState and PrivateCache and enumerate all effect variants",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_slot_coverage_validate_record_result"),
    [
        "eq record.body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "selfhost_memo_trait_operation_private_effect_slot_coverage_validate_effect record.effect",
        "SelfhostEffectEscapeState::NotApplicable:",
        "Result::Ok unit",
        "CoverageEscapeNotApplicable record.escape",
    ],
    "record validation must reject placeholder origins and non-NotApplicable escapes",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_slot_coverage_record_key_eq"),
    [
        "selfhost_type_id_eq a.type_id b.type_id",
        "selfhost_memo_trait_operation_evidence_kind_eq a.operation b.operation",
        "eq a.body_module_fingerprint b.body_module_fingerprint",
        "eq selfhost_hir_expr_id_index a.body_root selfhost_hir_expr_id_index b.body_root",
        "selfhost_effect_kind_eq a.effect b.effect",
        "selfhost_memo_trait_operation_private_effect_slot_coverage_escape_state_eq a.escape b.escape",
    ],
    "duplicate detection must compare the full typed body slot key",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_slot_coverage_key_matches"),
    [
        "selfhost_type_id_eq record.type_id type_id",
        "selfhost_memo_trait_operation_evidence_kind_eq record.operation operation",
        "eq record.body_module_fingerprint body_module_fingerprint",
        "eq selfhost_hir_expr_id_index record.body_root selfhost_hir_expr_id_index body_root",
        "selfhost_effect_kind_eq record.effect effect",
        "selfhost_memo_trait_operation_private_effect_slot_coverage_escape_state_eq record.escape SelfhostEffectEscapeState::NotApplicable",
    ],
    "slot lookup must compare exact type, operation, module fingerprint, body root, effect, and NotApplicable escape",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_slot_coverage_status_to_observation_status"),
    [
        "EffectObservedNoEscape:",
        "NoEscapeProven",
        "EffectObservedMayEscape:",
        "MayEscape",
        "EffectAbsentAfterCompleteTraversal:",
        "NoEscapeProven",
        "ResourceGraphMissing:",
        "Missing",
        "TraversalUnsupported:",
        "Unknown",
    ],
    "coverage status mapping must only treat observed no-escape and explicit complete-traversal absence as Proven",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_slot_coverage_observation_table_result"),
    [
        "SelfhostEffectKind::PrivateCache",
        "SelfhostEffectKind::PrivateState",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_table_new",
        "selfhost_memo_trait_operation_private_effect_slot_coverage_output_push_result output0 cache_coverage",
        "selfhost_memo_trait_operation_private_effect_slot_coverage_output_push_result output1 state_coverage",
    ],
    "observation table producer must require both fixed slots and push exactly those typed observations",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_slot_coverage_proof_table_result"),
    [
        "selfhost_memo_trait_operation_private_effect_slot_coverage_observation_table_result source type_id operation body_module_fingerprint body_root",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_producer_table_result &observations",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_table_free observations",
    ],
    "proof table producer must delegate to Resource no-escape producer and close intermediate observation owner",
);
const productionImplementation = [
    "selfhost_memo_trait_operation_private_effect_slot_coverage_observation_table_result",
    "selfhost_memo_trait_operation_private_effect_slot_coverage_proof_table_result",
].map((name) => stripDocComments(functionBlock(source, name))).join("\n");
assert.doesNotMatch(
    productionImplementation,
    /selfhost_memo_trait_operation_private_effect_mask_evidence_result|SelfhostMemoTraitOperationPrivateEffectMaskEvidence|MaskEvidenceRejected/,
    "production slot coverage helpers must not build mask evidence directly",
);
assert.ok(
    source.includes("observed_absent_mask") &&
        source.includes("may_escape_mask_status") &&
        source.includes("missing_mask_status") &&
        source.includes("unknown_mask_status") &&
        source.includes("proof_len") &&
        source.includes("slot_missing_rejected") &&
        source.includes("duplicate_rejected") &&
        source.includes("placeholder_rejected") &&
        source.includes("effect_rejected") &&
        source.includes("escape_rejected"),
    "stage0 must cover mask smoke, proof output, missing slot, duplicate, placeholder, effect, and escape rejection",
);
