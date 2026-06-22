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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_mask_evidence.nepl";
const gateRelPath = "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_no_escape_gate.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";

const source = read(relPath);
const code = stripDocComments(source);
const gateSource = read(gateRelPath);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const runner = read(runnerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_private_effect_mask_evidence",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "private-effect mask evidence producer must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("method body effect summary は 1 つの `SelfhostEffectKind` へ fold") &&
        source.includes("`PrivateCache` が `PrivateState` より強い rank") &&
        source.includes("低 rank private effect を隠せます"),
    "docs must explain why both PrivateCache and PrivateState slots are required",
);
assert.ok(
    source.includes("`PrivateCache` と `PrivateState` は固定 2 slot") &&
        source.includes("`Missing` は「effect 不在」ではなく「必須 slot の proof record 欠落」"),
    "docs must define Missing as required proof slot absence, not effect absence",
);
assert.ok(
    source.includes("`Refuted > Missing > Unknown > Proven`") &&
        source.includes("`Unknown` や `Missing` を `Proven` に丸めません"),
    "docs must define fail-closed status priority",
);
assert.ok(
    source.includes("memo_call backend、backend readiness private type") &&
        source.includes("Resource graph/proof internal") &&
        source.includes("`.neplobj`") &&
        source.includes("`.neplproof`"),
    "docs must keep backend, Resource internals, and artifacts out of this boundary",
);
for (const [docSnippet, declarationSnippet] of [
    [
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus",
        "pub enum SelfhostMemoTraitOperationPrivateEffectMaskStatus:",
    ],
    [
        "SelfhostMemoTraitOperationPrivateEffectMaskEvidence",
        "pub struct SelfhostMemoTraitOperationPrivateEffectMaskEvidence:",
    ],
    [
        "SelfhostMemoTraitOperationPrivateEffectMaskEvidenceErrorKind",
        "pub enum SelfhostMemoTraitOperationPrivateEffectMaskEvidenceErrorKind:",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_mask_evidence_result",
        "pub fn selfhost_memo_trait_operation_private_effect_mask_evidence_result",
    ],
]) {
    assertDocBeforeTopLevel(source, docSnippet, declarationSnippet);
}
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_private_effect_mask_evidence/,
    "private-effect mask evidence producer must remain facade-private until upper orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_private_effect_mask_evidence/,
    "checker-layer private-effect mask evidence producer must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_private_effect_mask_evidence_contract.js"),
    "source policy runner must execute the private-effect mask evidence contract",
);
assertOrdered(
    source,
    [
        "#import \"core/math\" as *",
        "#import \"core/result\" as *",
        "#import \"core/traits/copy\" as *",
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_private_effect_no_escape_gate\" as *",
    ],
    "mask evidence producer must depend only on math/result/copy, typed HIR id, effect/type ids, operation kind, and the private-effect no-escape gate",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:backend|memo_call|codegen|resource\/|resource_tree|resource_graph|memo_trait_operation_private_effect_resource_no_escape|resource_graph_input_scanner|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|memo_trait_operation_drop_no_escape_gate|memo_trait_operation_drop_resource|private_cache|private_state)/,
    "mask evidence producer must not import backend, memo_call, codegen, Resource graph/proof internals, proof store/artifact, canonical-key, public-surface, evidence producer, impl table, scanner/materializer, purity gate, Drop proof layers, PrivateCache, or PrivateState layers",
);
assert.doesNotMatch(
    code,
    /SelfhostMemoCallBackend|BackendReadiness|PrivateEffectMaskProven|PrivateEffectMaskRefuted|PrivateEffectMaskMissing|PrivateEffectMaskUnknown/,
    "mask evidence producer must not name backend readiness private types or backend-private mask variants",
);
assert.doesNotMatch(
    code,
    /GraphInput|ResourceGraph|RequestEvidenceProven|PrivateCacheNoEscapeProven|Wasm|LLVM|neplobj|neplproof|artifact|sealed|serializer|reader|prechecked/i,
    "mask evidence producer must not synthesize Resource graph, request evidence, backend bytes, sealed representation, or artifact policy",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoTraitOperationPrivateEffectMaskStatus"),
    ["Proven", "Refuted", "Missing", "Unknown"],
    "mask status enum must stay exact and must not add fallback variants",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoTraitOperationPrivateEffectMaskEvidence"),
    [
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "status %SelfhostMemoTraitOperationPrivateEffectMaskStatus",
        "required_slot_count %i32",
        "proven_slot_count %i32",
    ],
    "mask evidence must retain typed identity, status, required slot count, and proven slot count",
);
assert.doesNotMatch(
    topLevelBlock(source, "struct", "SelfhostMemoTraitOperationPrivateEffectMaskEvidence"),
    /payload_hash|signature_hash|body_hash|public_surface|source_text|source_span|source_path|\bspan\b|\bpath\b|\bname\b|diagnostic|message|text/i,
    "mask evidence must not use source/display/hash authority instead of typed body identity",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationPrivateEffectMaskEvidenceErrorKind"),
    [
        "ProofTableAllocFailed %StdErrorKind",
        "ProofTableBuildRejected %SelfhostMemoTraitOperationPrivateEffectNoEscapeGateErrorKind",
        "BodyModuleFingerprintPlaceholder",
        "ProofLookupRejected %SelfhostMemoTraitOperationPrivateEffectNoEscapeGateErrorKind",
    ],
    "mask evidence error enum must keep setup, placeholder, and lookup errors typed",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_mask_status_from_proof_status"),
    [
        "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofStatus::Proven:",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Proven",
        "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofStatus::Refuted:",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Refuted",
        "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofStatus::Missing:",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Missing",
        "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofStatus::Unknown:",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Unknown",
    ],
    "proof status mapping must enumerate every status and must not mask Missing/Unknown",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_mask_status_fold"),
    [
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Refuted:",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Refuted",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Missing:",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Refuted",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Missing",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Unknown:",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Missing",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Unknown",
        "SelfhostMemoTraitOperationPrivateEffectMaskStatus::Proven:",
    ],
    "status fold must encode Refuted > Missing > Unknown > Proven without wildcard fallback",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_mask_status_fold"),
    /^\s+_\w*:/m,
    "status fold must not use wildcard branches",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_mask_lookup_slot_result"),
    [
        "selfhost_memo_trait_operation_private_effect_no_escape_proof_lookup_not_applicable_result proofs type_id operation body_module_fingerprint body_root effect",
        "Result::Err gate_error:",
        "SelfhostMemoTraitOperationPrivateEffectMaskEvidenceErrorKind::ProofLookupRejected gate_error",
    ],
    "slot lookup must use the no-escape gate narrow public lookup and must preserve lookup errors",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_mask_lookup_slot_result"),
    /field::get_ref\s+proofs\s+"records"|v::get|proof_lookup_result\b/,
    "slot lookup must not directly read proof table records or call the broad private lookup",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_mask_evidence_result"),
    [
        "eq body_module_fingerprint 0",
        "Result::Err SelfhostMemoTraitOperationPrivateEffectMaskEvidenceErrorKind::BodyModuleFingerprintPlaceholder",
        "SelfhostEffectKind::PrivateCache",
        "SelfhostEffectKind::PrivateState",
        "selfhost_memo_trait_operation_private_effect_mask_evidence_from_slots type_id operation body_module_fingerprint body_root cache_status state_status",
    ],
    "evidence result must reject placeholder origins and must lookup both PrivateCache and PrivateState slots before producing evidence",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_mask_evidence_result"),
    /field::get_ref\s+proofs\s+"records"|v::get|Resource|GraphInput|SelfhostMemoCallBackend|BackendReadiness/,
    "evidence result must not inspect proof table internals or mix Resource/backend layers",
);
assert.ok(
    gateSource.includes("pub fn selfhost_memo_trait_operation_private_effect_no_escape_proof_lookup_not_applicable_result"),
    "no-escape gate must expose a narrow public NotApplicable lookup for downstream checker modules",
);
assertOrdered(
    functionBlock(gateSource, "selfhost_memo_trait_operation_private_effect_no_escape_proof_lookup_not_applicable_result"),
    [
        "selfhost_memo_trait_operation_private_effect_no_escape_proof_key_new type_id operation body_module_fingerprint body_root effect SelfhostEffectEscapeState::NotApplicable",
        "selfhost_memo_trait_operation_private_effect_no_escape_proof_lookup_result proofs key",
    ],
    "no-escape gate narrow lookup must force NotApplicable and share the private exact-key lookup implementation",
);
assert.ok(
    source.includes("both_proven_evidence") &&
        source.includes("cache_refuted_status") &&
        source.includes("state_refuted_status") &&
        source.includes("cache_missing_status") &&
        source.includes("state_missing_status") &&
        source.includes("cache_unknown_status") &&
        source.includes("state_unknown_status") &&
        source.includes("placeholder_rejected") &&
        source.includes("duplicate_rejected"),
    "stage0 must cover both Proven, each Refuted, each Missing, each Unknown, placeholder rejection, and duplicate proof rejection",
);
