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

function before(text, marker) {
    const index = text.indexOf(marker);
    assert.notEqual(index, -1, `missing marker ${marker}`);
    return text.slice(0, index);
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

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_drop_absence_evidence_connector.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";
const source = read(relPath);
const code = stripDocComments(source);
const productionCode = stripDocComments(
    before(source, "//: selfhost_memo_trait_operation_drop_absence_evidence_connector_summary_from_table"),
);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const runner = read(runnerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_drop_absence_evidence_connector",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "drop absence evidence connector must document purpose, contract, current state, complexity, and doctest",
);
assert.ok(
    source.includes("no-drop absence evidence") &&
        source.includes("operation evidence table owner") &&
        source.includes("`Drop: Proven` record"),
    "docs must place this module between no-drop absence producer and operation evidence table",
);
assert.ok(
    source.includes("fake `SelfhostMemoTraitOperationImplCandidate`") &&
        source.includes("fake public impl header") &&
        source.includes("`NoDropRequired` だけ") &&
        source.includes("`SelfhostMemoTraitOperationEvidenceKind::Drop` / `SelfhostMemoTraitAggregateProofStatus::Proven`"),
    "docs must forbid fake candidate/header paths and define the only accepted NoDropRequired to Drop/Proven conversion",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、public surface hash、payload hash、HIR body root、Resource IR graph、proof store record、prechecked artifact") &&
        source.includes("aggregate proof、proof store、generic impl binder、PrivateCache / PrivateState masking、backend artifact、prechecked artifact、solver proof table"),
    "docs must reject source/hash/resource/proof-store authority and keep aggregate proof, private effects, backend, prechecked, and solver table out of scope",
);
assert.ok(
    source.includes("行数や doc comment の長さによる制限は置きません"),
    "docs must explicitly prefer detailed comments over artificial volume gates",
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
        "every drop absence evidence connector declaration, including private helpers and impl blocks, must have a preceding doc comment",
    );
}
assert.doesNotMatch(
    facade,
    /memo_trait_operation_drop_absence_evidence_connector/,
    "drop absence evidence connector must remain facade-private until full proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_drop_absence_evidence_connector/,
    "checker-layer drop absence evidence connector must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_drop_absence_evidence_connector_contract.js"),
    "source policy runner must execute the drop absence evidence connector contract",
);
{
    const memoTraitImports = [...source.matchAll(/^#import "(\.\/memo_trait[^"]+)" as \*/gm)].map(
        (match) => match[1],
    );
    assert.deepEqual(
        memoTraitImports,
        [
            "./memo_trait_operation_drop_absence_producer",
            "./memo_trait_operation_drop_impl_resolver",
            "./memo_trait_operation_evidence_producer",
        ],
        "drop absence evidence connector must keep a minimal checker-layer memo_trait import allow-list",
    );
}
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_scanner|memo_trait_public_impl_header|memo_trait_operation_impl_table|memo_trait_operation_impl_candidate_builder|memo_trait_operation_public_impl_materializer|memo_trait_operation_public_impl_drop_fact_orchestrator|memo_trait_operation_method_body|memo_trait_operation_body_check_resolver|memo_trait_operation_solver|private_cache|private_state)/,
    "drop absence evidence connector must not import Resource IR, backend, proof store/artifact, canonical-key, public-surface, scanner/materializer, impl-table/candidate/header, method-body, solver, PrivateCache, or PrivateState layers",
);
assert.doesNotMatch(
    productionCode,
    /\b(SelfhostMemoTraitOperationImplCandidate|SelfhostMemoTraitPublicImplHeaderInput|SelfhostMemoTraitPublicImplHeaderKind|SelfhostMemoTraitAggregateProof\b|SelfhostMemoTraitProofStore|PrivateCache|PrivateState|prechecked|backend|selfhost_memo_trait_operation_impl_candidate_new|selfhost_memo_trait_public_impl_header_input_new|selfhost_memo_trait_aggregate_proof_to_record)\b/,
    "production connector must not create fake impl candidates, fake public impl headers, aggregate proof, proof-store, private-cache/state, prechecked, or backend values",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationDropAbsenceEvidenceConnectorErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "connector errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "connector APIs must return typed Result errors instead of bool/string errors",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationDropAbsenceEvidenceConnectorErrorKind:",
        "AbsenceProducerRejected %SelfhostMemoTraitOperationDropAbsenceProducerErrorKind",
        "EvidenceTableRejected %SelfhostMemoTraitOperationEvidenceErrorKind",
        "DropEvidenceAlreadyPresent %SelfhostMemoTraitAggregateProofStatus",
        "UnexpectedDropEvidence %SelfhostMemoTraitOperationDropEvidence",
    ],
    "connector errors must preserve producer, evidence-table, existing-Drop, and unexpected lower evidence payloads",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_absence_evidence_connector_preflight_result"),
    [
        "selfhost_memo_trait_operation_evidence_status_for_type_operation_result &table type_id SelfhostMemoTraitOperationEvidenceKind::Drop",
        "Result::Ok status:",
        "selfhost_memo_trait_operation_evidence_table_free table",
        "DropEvidenceAlreadyPresent status",
        "SelfhostMemoTraitOperationEvidenceErrorKind::RecordMissing:",
        "Result::Ok table",
        "SelfhostMemoTraitOperationEvidenceErrorKind::DuplicateRecord:",
        "selfhost_memo_trait_operation_evidence_table_free table",
        "EvidenceTableRejected SelfhostMemoTraitOperationEvidenceErrorKind::DuplicateRecord",
    ],
    "preflight must reject existing Drop evidence and duplicates while freeing the owned table on errors",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_absence_evidence_connector_push_no_drop_result"),
    [
        "selfhost_memo_trait_operation_drop_absence_evidence_connector_preflight_result table type_id",
        "selfhost_memo_trait_operation_drop_absence_evidence_result surface drop_table type_id",
        "SelfhostMemoTraitOperationDropEvidence::NoDropRequired:",
        "selfhost_memo_trait_operation_evidence_record_new type_id SelfhostMemoTraitOperationEvidenceKind::Drop SelfhostMemoTraitAggregateProofStatus::Proven",
        "selfhost_memo_trait_operation_evidence_table_push checked_table record",
        "Result::Err producer_error:",
        "selfhost_memo_trait_operation_evidence_table_free checked_table",
        "AbsenceProducerRejected producer_error",
    ],
    "push_no_drop must call the absence producer, accept only NoDropRequired, create Drop/Proven, and free the table on producer errors",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_absence_evidence_connector_push_no_drop_result"),
    [
        "SelfhostMemoTraitOperationDropEvidence::PureDrop:",
        "UnexpectedDropEvidence evidence",
        "SelfhostMemoTraitOperationDropEvidence::Missing:",
        "UnexpectedDropEvidence evidence",
        "SelfhostMemoTraitOperationDropEvidence::ImpureDrop:",
        "UnexpectedDropEvidence evidence",
        "SelfhostMemoTraitOperationDropEvidence::Unknown:",
        "UnexpectedDropEvidence evidence",
        "SelfhostMemoTraitOperationDropEvidence::NotRequired:",
        "UnexpectedDropEvidence evidence",
    ],
    "push_no_drop must fail closed for every non-NoDropRequired drop evidence variant",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_drop_absence_evidence_connector_push_no_drop_result"),
    /SelfhostMemoTraitOperationDropEvidence::PureDrop:[\s\S]*SelfhostMemoTraitAggregateProofStatus::Proven/,
    "PureDrop must not be converted to Drop/Proven by the no-drop absence connector",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_absence_evidence_connector_error_kind_eq"),
    [
        "AbsenceProducerRejected a_producer:",
        "selfhost_memo_trait_operation_drop_absence_error_kind_eq a_producer b_producer",
        "EvidenceTableRejected a_table:",
        "selfhost_memo_trait_operation_evidence_error_kind_eq a_table b_table",
        "DropEvidenceAlreadyPresent a_status:",
        "selfhost_memo_trait_operation_evidence_status_eq a_status b_status",
        "UnexpectedDropEvidence a_evidence:",
        "selfhost_memo_trait_operation_drop_absence_producer_evidence_result_eq Result::Ok a_evidence b_evidence",
    ],
    "error equality must compare nested producer, evidence table, existing status, and unexpected evidence payloads",
);
assert.ok(
    source.includes("wildcard arm は使いません。variant が増えた場合はこの equality 境界を明示的に更新します"),
    "error equality docs must explicitly forbid wildcard arms",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_absence_evidence_connector_stage0"),
    [
        "selfhost_memo_trait_operation_drop_impl_table_new",
        "selfhost_memo_trait_operation_drop_absence_evidence_connector_stage0_with_table drop_table type_id",
    ],
    "stage0 must exercise the public connector through an owned empty Drop impl table",
);
