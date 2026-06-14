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

function before(text, marker) {
    const index = text.indexOf(marker);
    assert.notEqual(index, -1, `missing marker ${marker}`);
    return text.slice(0, index);
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_surface_drop_candidate_connector.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";
const source = read(relPath);
const code = stripDocComments(source);
const productionCode = stripDocComments(
    before(source, "//: selfhost_memo_trait_public_impl_surface_drop_candidate_connector_stage0_summary_new"),
);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const runner = read(runnerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_public_impl_surface_drop_candidate_connector",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "surface Drop candidate connector must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("同じ `operation_records`") &&
        source.includes("caller が別々に作った surface state と materializer record table を混ぜられない"),
    "docs must state that the wrapper preserves scanner-output same-origin and rejects split public state/table authority",
);
assert.ok(
    source.includes("`public_surface_hash` は") &&
        source.includes("transport value") &&
        source.includes("authority には使いません"),
    "docs must state that public surface hash is transport, not proof authority",
);
assert.ok(
    source.includes("Resource IR proof producer") &&
        source.includes("operation evidence record") &&
        source.includes("proof store") &&
        source.includes("PrivateCache / PrivateState masking"),
    "docs must keep Resource proof, evidence record, proof store, and private effects out of scope",
);
assert.ok(
    source.includes("行数や doc comment の長さによる制限は置きません"),
    "docs must explicitly avoid line-count or doc-comment-length limits",
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
        "every surface Drop candidate connector declaration, including private stage0 helpers and impl blocks, must have a preceding doc comment",
    );
}
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_surface_drop_candidate_connector/,
    "surface Drop candidate connector must remain facade-private until full proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_surface_drop_candidate_connector/,
    "checker-layer surface Drop candidate connector must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_public_impl_surface_drop_candidate_connector_contract.js"),
    "source policy runner must execute the surface Drop candidate connector contract",
);
{
    const memoTraitImports = [...source.matchAll(/^#import "(\.\/memo_trait[^"]+)" as \*/gm)].map(
        (match) => match[1],
    );
    assert.deepEqual(
        memoTraitImports,
        [
            "./memo_trait_operation_classifier",
            "./memo_trait_operation_drop_candidate_connector",
            "./memo_trait_operation_drop_no_escape_gate",
            "./memo_trait_operation_impl_table",
            "./memo_trait_operation_public_impl_materializer",
            "./memo_trait_operation_purity_gate",
            "./memo_trait_public_impl_header",
            "./memo_trait_public_impl_scanner",
            "./memo_trait_public_impl_surface_orchestrator",
            "./memo_trait_public_surface_hash",
            "./memo_trait_public_surface_normalizer",
            "./memo_trait_source_evidence_producer",
        ],
        "surface Drop candidate connector must keep an explicit checker-layer memo_trait import allow-list",
    );
}
assert.ok(
    source.includes("facade にはまだ re-export しません") &&
        source.includes("actual Resource IR proof producer") &&
        source.includes("complete public surface 由来の no-drop absence proof"),
    "docs must justify temporary facade privacy and name the remaining Resource/no-drop proof boundaries",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_operation_public_impl_operation_evidence_connector|memo_trait_public_impl_operation_evidence_connector|memo_trait_operation_body_check_resolver|memo_trait_operation_impl_candidate_builder|memo_trait_operation_method_body|memo_trait_operation_drop_impl_resolver|private_cache|private_state)/,
    "surface Drop candidate connector must not import Resource IR, backend, proof store/artifact, canonical-key, operation-evidence connector, body-check, candidate-builder, method-body, Drop resolver, PrivateCache, or PrivateState layers",
);
assert.doesNotMatch(
    code,
    /pub\s+fn\s+selfhost_memo_trait_public_impl_surface_drop_candidate_connector_apply_result/,
    "connector must not expose an apply_result API that accepts separately materialized state and records",
);
assert.doesNotMatch(
    code,
    /pub\s+fn[^\n]*SelfhostMemoTraitPublicImplSurfaceState[^\n]*SelfhostMemoTraitOperationPublicImplMaterializerRecordTable|pub\s+fn[^\n]*SelfhostMemoTraitOperationPublicImplMaterializerRecordTable[^\n]*SelfhostMemoTraitPublicImplSurfaceState/,
    "public API must not accept both a surface state and an independent materializer record table",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_record_operation_result"),
    [
        "selfhost_memo_trait_operation_trait_application_input_new record.trait_source record.trait_type_argument_count record.trait_application_shape_hash",
        "selfhost_memo_trait_operation_classifier_evidence_result trait_application",
        "Result::Ok classifier:",
        "Result::Ok classifier.operation",
        "Result::Err classifier_error:",
        "OperationRecordClassifierRejected classifier_error",
    ],
    "operation filter must derive operation authority through the classifier boundary",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_operation_is_drop"),
    [
        "SelfhostMemoTraitOperationEvidenceKind::Copy:",
        "false",
        "SelfhostMemoTraitOperationEvidenceKind::Drop:",
        "true",
        "SelfhostMemoTraitOperationEvidenceKind::Eq:",
        "false",
        "SelfhostMemoTraitOperationEvidenceKind::Hash:",
        "false",
    ],
    "operation filter must classify every current operation variant explicitly",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_non_drop_filter_loop"),
    [
        "v::get records index",
        "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_record_operation_result record",
        "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_operation_is_drop operation",
        "then:",
        "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_non_drop_filter_loop output source add index 1",
        "else:",
        "selfhost_memo_trait_operation_public_impl_materializer_record_table_push output record",
        "Result::Err e:",
        "selfhost_memo_trait_operation_public_impl_materializer_record_table_free output",
        "Option::None:",
        "OperationRecordFilterSourceReadFailed index",
    ],
    "non-Drop filter must classify every source record, skip Drop records, copy non-Drop records, and clean the temporary table on read/classifier failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_hash_from_scanner_output_result"),
    [
        "field::get_ref scanner_output \"public_declarations\"",
        "selfhost_memo_trait_public_surface_normalizer_partial_input_items_result graph dependencies reexports public_declarations",
        "selfhost_memo_trait_public_surface_hash_from_seed_table_and_partial_items_result seed_table &partial_items",
        "v::free partial_items",
    ],
    "hash helper must read only scanner public declarations and must free partial input items",
);
assert.doesNotMatch(
    stripDocComments(
        functionBlock(source, "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_hash_from_scanner_output_result"),
    ).split("\n").slice(1).join("\n"),
    /operation_records|proofs|SelfhostMemoTraitOperationDropNoEscapeProofTable|drop_candidate/,
    "hash helper must not read operation records or Drop proofs",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_from_scanner_output_result"),
    [
        "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_hash_from_scanner_output_result graph dependencies reexports seed_table scanner_output",
        "Result::Ok public_surface_hash:",
        "field::get_ref scanner_output \"operation_records\"",
        "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_non_drop_records_result operation_records",
        "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_result module &base_records",
        "selfhost_memo_trait_operation_public_impl_materializer_record_table_free base_records",
        "selfhost_memo_trait_operation_drop_candidate_connector_append_from_records_result base_impls module operation_records proofs",
        "Result::Ok next_impls:",
        "Result::Ok SelfhostMemoTraitPublicImplSurfaceState public_surface_hash next_impls",
        "Result::Err connector_error:",
        "DropCandidateConnectorRejected connector_error",
    ],
    "from_scanner_output_result must derive hash, read the same scanner output operation records, materialize non-Drop base records, then append Drop candidates from the original records",
);
{
    const fromScanner = functionBlock(
        source,
        "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_from_scanner_output_result",
    );
    assert.doesNotMatch(
        fromScanner,
        /drop_candidate_connector_append_from_records_result[^\n]*(public_surface_hash|public_hash)/,
        "public surface hash must not be passed into Drop candidate proof acceptance",
    );
}
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_from_ast_records_result"),
    [
        "selfhost_memo_trait_public_impl_scanner_result ast records",
        "Result::Ok scanner_output:",
        "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_from_scanner_output_result module graph dependencies reexports seed_table &scanner_output proofs",
        "selfhost_memo_trait_public_impl_scanner_output_free scanner_output",
        "Result::Err scanner_error:",
        "ScannerRejected scanner_error",
    ],
    "from_ast_records_result must run the scanner, forward the scanner output as the single same-origin source, and close scanner output after downstream success or rejection",
);
assert.doesNotMatch(
    productionCode,
    /\b(SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_aggregate_proof_to_record)\b/,
    "production functions must not construct operation evidence records, aggregate proof, or proof-store values",
);
assert.doesNotMatch(
    productionCode,
    /\bSelfhostMemoTraitOperationDropEvidence::(?:PureDrop|NoDropRequired)\b/,
    "production functions must not directly synthesize PureDrop or NoDropRequired",
);
assert.doesNotMatch(
    productionCode,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\n]*"(?:name|span|source|path|diagnostic|message|text)")/,
    "accepted production authority must not use call names, spans, source text, paths, messages, or diagnostic text",
);
assertOrdered(
    source,
    [
        "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_accepted_len_eq summary.proven 2",
        "SelfhostMemoTraitOperationDropEvidence::PureDrop",
        "SelfhostMemoTraitOperationDropEvidence::Unknown",
        "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_stage0_once SelfhostMemoTraitOperationDropNoEscapeProofStatus::Proven",
        "selfhost_memo_trait_public_impl_surface_drop_candidate_connector_stage0_once SelfhostMemoTraitOperationDropNoEscapeProofStatus::Missing",
    ],
    "stage0 must verify Eq plus Drop candidate count and both Proven and Missing Drop proof paths",
);
assert.doesNotMatch(
    source,
    /maxLine|line count limit|line-count limit|doc comment length cap|documentation length cap|行数制限|コメント量制限|ドキュメントコメント.{0,12}上限/,
    "surface Drop candidate connector contract must not introduce line-count or documentation-comment length limits",
);

console.log("selfhost memo trait public impl surface Drop candidate connector contract passed");
