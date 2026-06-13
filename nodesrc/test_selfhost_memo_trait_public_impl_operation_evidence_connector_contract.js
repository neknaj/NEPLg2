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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_operation_evidence_connector.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_public_impl_operation_evidence_connector",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "operation evidence connector must document purpose, contract, current limits, complexity, and a doctest",
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
        "every connector declaration, including private stage0 helpers and impl blocks, must have a preceding doc comment",
    );
}
assert.ok(
    source.includes("public impl surface state または operation impl table") &&
        source.includes("root aggregate 用の operation evidence table"),
    "docs must place the connector between public impl surface/impl table and root operation evidence",
);
assert.ok(
    source.includes("`CandidateMissing` だけは") &&
        source.includes("後段の `selfhost_memo_trait_operation_evidence_record_for_type_or_missing_result` が Missing status へ畳みます"),
    "docs must state that only CandidateMissing becomes absence and that Missing status is folded by the evidence table boundary",
);
assert.ok(
    source.includes("`CandidateDuplicate`") &&
        source.includes("Missing に潰さず typed error として返します"),
    "docs must preserve duplicate/classifier/producer failures as typed errors",
);
assert.ok(
    source.includes("Drop candidate が無い場合に `NoDropRequired` や `PureDrop` を合成しません"),
    "docs must forbid synthesizing Drop proof from lookup absence",
);
assert.ok(
    source.includes("行数や doc comment の長さによる制限は置きません"),
    "docs must explicitly avoid line-count or doc-comment-length limits",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_operation_evidence_connector/,
    "operation evidence connector must remain facade-private until full proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_operation_evidence_connector/,
    "checker-layer connector must not be registered in the ty source list",
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
            "./memo_trait_public_impl_surface_orchestrator",
        ],
        "connector must keep an explicit checker-layer memo_trait import allow-list",
    );
}
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_operation_method_body|memo_trait_operation_drop_impl_resolver|memo_trait_operation_body_check_resolver|memo_trait_operation_impl_candidate_builder|memo_trait_operation_public_impl_materializer|memo_trait_public_impl_scanner)/,
    "connector must not import Resource IR, backend, proof store, canonical-key, method-body, Drop resolver, body-check, candidate-builder, materializer, or scanner layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplOperationEvidenceConnectorErrorKind:",
        "PublicSurfaceHashMissing",
        "EvidenceTableAllocFailed %StdErrorKind",
        "OperationImplRejected %SelfhostMemoTraitOperationImplTableErrorKind",
        "EvidenceTablePushRejected %SelfhostMemoTraitOperationEvidenceErrorKind",
        "OperationSolverRejected %SelfhostMemoTraitOperationSolverErrorKind",
    ],
    "connector errors must preserve state/hash, allocation, impl-table, push, and solver failures as typed variants",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_operation_evidence_connector_push_operation_result"),
    [
        "selfhost_memo_trait_operation_impl_record_for_type_operation_result impls type_id operation",
        "Result::Ok record:",
        "selfhost_memo_trait_operation_evidence_table_push table record",
        "Result::Err impl_error:",
        "SelfhostMemoTraitOperationImplTableErrorKind::CandidateMissing:",
        "Result::Ok table",
        "SelfhostMemoTraitOperationImplTableErrorKind::CandidateDuplicate:",
        "selfhost_memo_trait_operation_evidence_table_free table",
        "selfhost_memo_trait_public_impl_operation_evidence_connector_operation_error_result",
    ],
    "push_operation must append producer-validated records, skip only CandidateMissing, and free the evidence table before typed impl-table errors",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_public_impl_operation_evidence_connector_push_operation_result"),
    /SelfhostMemoTraitAggregateProofStatus::Proven|SelfhostMemoTraitOperationDropEvidence::NoDropRequired|SelfhostMemoTraitOperationDropEvidence::PureDrop/,
    "push_operation must not synthesize proven status, NoDropRequired, or PureDrop",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_operation_evidence_table_for_type_from_impl_table_result"),
    [
        "selfhost_memo_trait_operation_evidence_table_new",
        "SelfhostMemoTraitOperationEvidenceKind::Copy",
        "SelfhostMemoTraitOperationEvidenceKind::Drop",
        "SelfhostMemoTraitOperationEvidenceKind::Eq",
        "SelfhostMemoTraitOperationEvidenceKind::Hash",
    ],
    "impl-table connector must process Copy, Drop, Eq, Hash in deterministic order",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_operation_evidence_table_for_type_from_surface_state_result"),
    [
        "eq selfhost_memo_trait_public_impl_surface_state_public_surface_hash state 0",
        "PublicSurfaceHashMissing",
        "field::get_ref state \"operation_impls\"",
        "selfhost_memo_trait_public_impl_operation_evidence_table_for_type_from_impl_table_result impls type_id",
    ],
    "surface-state connector must reject zero hash before borrowing operation_impls",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_operation_proof_table_for_type_from_impl_table_result"),
    [
        "selfhost_memo_trait_public_impl_operation_evidence_table_for_type_from_impl_table_result impls type_id",
        "selfhost_memo_trait_operation_solver_table_for_type_with_operation_evidence_result types layout_table &evidence_table type_id max_depth",
        "selfhost_memo_trait_operation_evidence_table_free evidence_table",
        "OperationSolverRejected solver_error",
    ],
    "proof-table wrapper must delegate aggregate solving to the existing solver and free the temporary evidence table",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_operation_evidence_connector_error_kind_eq"),
    [
        "PublicSurfaceHashMissing:",
        "EvidenceTableAllocFailed a_alloc:",
        "selfhost_memo_trait_public_impl_operation_evidence_connector_std_error_kind_eq a_alloc b_alloc",
        "OperationImplRejected a_impl:",
        "selfhost_memo_trait_operation_impl_table_error_kind_eq a_impl b_impl",
        "EvidenceTablePushRejected a_push:",
        "selfhost_memo_trait_operation_evidence_error_kind_eq a_push b_push",
        "OperationSolverRejected a_solver:",
        "selfhost_memo_trait_operation_solver_error_kind_eq a_solver b_solver",
    ],
    "error equality must be exhaustive and compare nested error payloads",
);
assert.ok(
    source.includes("wildcard arm は使いません。variant が増えた場合はこの equality 境界を明示的に更新します"),
    "error equality docs must explicitly forbid wildcard arms",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_operation_evidence_connector_stage0_summary_eq"),
    [
        "summary.accepted 1 SelfhostMemoTraitAggregateProofStatus::Proven SelfhostMemoTraitAggregateProofStatus::Missing",
        "summary.missing 0 SelfhostMemoTraitAggregateProofStatus::Missing SelfhostMemoTraitAggregateProofStatus::Missing",
        "OperationImplRejected SelfhostMemoTraitOperationImplTableErrorKind::CandidateDuplicate",
        "summary.zero_hash_rejected SelfhostMemoTraitPublicImplOperationEvidenceConnectorErrorKind::PublicSurfaceHashMissing",
    ],
    "stage0 summary must verify accepted Copy, Drop Missing, fully missing type, duplicate rejection, and zero-hash rejection",
);
