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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_surface_operation_proof_orchestrator.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";
const source = read(relPath);
const code = stripDocComments(source);
const productionCode = stripDocComments(source.split("//: # Stage0 fixture")[0]);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const runner = read(runnerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_public_impl_surface_operation_proof_orchestrator",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "surface operation proof orchestrator must document purpose, contract, current limits, complexity, and a doctest",
);
{
    const lines = source.split("\n");
    const missingDocs = [];
    for (let i = 0; i < lines.length; i += 1) {
        if (/^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/.test(lines[i])) {
            let j = i - 1;
            while (j >= 0 && lines[j].trim() === "") {
                j -= 1;
            }
            if (j < 0 || !lines[j].trimStart().startsWith("//:")) {
                missingDocs.push(`${i + 1}: ${lines[i]}`);
            }
        }
    }
    assert.deepEqual(
        missingDocs,
        [],
        "every orchestrator declaration, including private stage0 helpers and impl blocks, must have a preceding doc comment",
    );
}
assert.ok(
    source.includes("Drop candidate [増補/ぞうほ][済/ず]み public impl surface state") &&
        source.includes("既存の operation evidence / operation proof pipeline"),
    "module docs must place the orchestrator between Drop-augmented public impl surface state and existing operation evidence/proof pipeline",
);
assert.ok(
    source.includes("`public_surface_hash` は transport consistency guard") &&
        source.includes("operation proof の authority ではありません"),
    "docs must state that public_surface_hash is not proof authority",
);
assert.ok(
    source.includes("`NoDropRequired`、`PureDrop`、method body `Pure`、aggregate `Proven` を合成しません"),
    "docs must forbid synthesis of no-drop, pure-drop, method purity, and aggregate proven evidence",
);
assert.ok(
    source.includes("行数や doc comment の長さによる制限は置きません"),
    "docs must explicitly avoid line-count or doc-comment-length limits",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_surface_operation_proof_orchestrator/,
    "surface operation proof orchestrator must remain facade-private until full proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_surface_operation_proof_orchestrator/,
    "checker-layer orchestrator must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_public_impl_surface_operation_proof_orchestrator_contract.js"),
    "source policy runner must include the surface operation proof orchestrator contract",
);
{
    const memoTraitImports = [...source.matchAll(/^#import "(\.\/memo_trait[^"]+)" as \*/gm)].map((match) => match[1]);
    assert.deepEqual(
        memoTraitImports,
        [
            "./memo_trait_operation_classifier",
            "./memo_trait_operation_evidence_producer",
            "./memo_trait_operation_impl_table",
            "./memo_trait_public_impl_header",
            "./memo_trait_public_impl_operation_evidence_connector",
            "./memo_trait_public_impl_surface_orchestrator",
        ],
        "orchestrator must keep an explicit checker-layer memo_trait import allow-list",
    );
}
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_operation_method_body|memo_trait_operation_drop_impl_resolver|memo_trait_operation_body_check_resolver|memo_trait_operation_impl_candidate_builder|memo_trait_operation_public_impl_materializer|memo_trait_public_impl_materializer|memo_trait_private_cache|memo_trait_private_state|prechecked)/,
    "orchestrator must not import Resource IR, backend, proof store, canonical-key, method-body, Drop resolver, body-check, candidate-builder, materializer internals, private effect, or prechecked artifact layers",
);
assert.doesNotMatch(
    productionCode,
    /SelfhostMemoTraitOperationDropEvidence::(?:NoDropRequired|PureDrop)|SelfhostMemoTraitOperationMethodBodyEvidence::Pure|SelfhostMemoTraitAggregateProofStatus::Proven|selfhost_memo_trait_operation_impl_candidate_new|selfhost_memo_trait_operation_impl_table_push/,
    "production orchestrator code must not synthesize drop evidence, method purity, aggregate proven status, or fake operation candidates",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplSurfaceOperationProofOrchestratorErrorKind:",
        "OperationEvidenceRejected %SelfhostMemoTraitPublicImplOperationEvidenceConnectorErrorKind",
    ],
    "orchestrator errors must preserve operation evidence connector failures as typed variants",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_operation_evidence_table_for_type_from_drop_augmented_surface_state_result"),
    [
        "selfhost_memo_trait_public_impl_operation_evidence_table_for_type_from_surface_state_result state type_id",
        "Result::Ok evidence_table:",
        "Result::Ok evidence_table",
        "Result::Err operation_error:",
        "OperationEvidenceRejected operation_error",
    ],
    "state evidence wrapper must only delegate to the existing operation evidence connector and wrap typed errors",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_operation_evidence_table_from_drop_augmented_surface_state_owner_result"),
    [
        "selfhost_memo_trait_public_impl_surface_operation_evidence_table_for_type_from_drop_augmented_surface_state_result &state type_id",
        "selfhost_memo_trait_public_impl_surface_state_free state",
    ],
    "owner evidence wrapper must delegate to the existing connector and free the state owner",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_operation_proof_table_from_drop_augmented_surface_state_owner_result"),
    [
        "selfhost_memo_trait_public_impl_surface_operation_proof_table_for_type_from_drop_augmented_surface_state_result types layout_table &state type_id max_depth",
        "selfhost_memo_trait_public_impl_surface_state_free state",
    ],
    "owner proof wrapper must delegate to the existing proof connector and free the state owner",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_operation_proof_orchestrator_error_kind_eq"),
    [
        "OperationEvidenceRejected a_operation:",
        "selfhost_memo_trait_public_impl_operation_evidence_connector_error_kind_eq a_operation b_operation",
    ],
    "error equality must delegate operation evidence connector errors to the existing equality helper",
);
assert.ok(
    source.includes("wildcard arm は使いません。variant が増えた場合はこの equality 境界を明示的に更新します"),
    "error equality docs must explicitly forbid wildcard arms",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_operation_proof_orchestrator_stage0_summary_eq"),
    [
        "summary.accepted 2 SelfhostMemoTraitAggregateProofStatus::Proven SelfhostMemoTraitAggregateProofStatus::Proven",
        "OperationEvidenceRejected SelfhostMemoTraitPublicImplOperationEvidenceConnectorErrorKind::PublicSurfaceHashMissing",
        "summary.zero_hash_rejected zero_hash_error",
    ],
    "stage0 summary must verify Drop-augmented Copy/Drop evidence and zero-hash rejection",
);
assert.ok(
    source.includes("//: # Stage0 fixture"),
    "fixture-only candidate synthesis must be separated from production code by an explicit documented marker",
);
