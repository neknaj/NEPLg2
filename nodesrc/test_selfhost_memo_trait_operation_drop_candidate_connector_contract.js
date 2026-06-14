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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_drop_candidate_connector.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";
const source = read(relPath);
const code = stripDocComments(source);
const productionCode = stripDocComments(
    before(source, "//: selfhost_memo_trait_operation_drop_candidate_connector_stage0_summary_new"),
);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const runner = read(runnerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_drop_candidate_connector",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "Drop candidate connector must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("Drop 専用の後段 connector") &&
        source.includes("typed public impl materializer record table") &&
        source.includes("no-escape proof gate"),
    "docs must place this module as a typed Drop-only connector after materializer records and no-escape proof",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、method name string、trait name string") &&
        source.includes("trusted operation classifier") &&
        source.includes("non-Drop record は同じ materializer table に混在する正常入力なので skip"),
    "docs must reject source/display authority and define classifier-driven non-Drop skip semantics",
);
assert.ok(
    source.includes("`NoDropRequired` はこの module では合成しません") &&
        source.includes("`DropImplPresent` だけを purity gate へ渡す") &&
        source.includes("Resource IR proof producer、operation evidence record、aggregate proof status、proof store") &&
        source.includes("PrivateCache / PrivateState masking"),
    "docs must state which proof/evidence/cache layers remain out of scope",
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
        "every Drop candidate connector declaration, including private stage0 helpers and impl blocks, must have a preceding doc comment",
    );
}
assert.doesNotMatch(
    facade,
    /memo_trait_operation_drop_candidate_connector/,
    "Drop candidate connector must remain facade-private until public proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_drop_candidate_connector/,
    "checker-layer Drop candidate connector must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_drop_candidate_connector_contract.js"),
    "source policy runner must execute the Drop candidate connector contract",
);
{
    const memoTraitImports = [...source.matchAll(/^#import "(\.\/memo_trait[^"]+)" as \*/gm)].map(
        (match) => match[1],
    );
    assert.deepEqual(
        memoTraitImports,
        [
            "./memo_trait_operation_classifier",
            "./memo_trait_operation_drop_impl_resolver",
            "./memo_trait_operation_drop_no_escape_gate",
            "./memo_trait_operation_evidence_producer",
            "./memo_trait_operation_impl_table",
            "./memo_trait_operation_public_impl_drop_fact_orchestrator",
            "./memo_trait_operation_public_impl_materializer",
            "./memo_trait_operation_purity_gate",
            "./memo_trait_public_impl_header",
        ],
        "Drop candidate connector must keep an explicit checker-layer memo_trait import allow-list",
    );
}
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_scanner|memo_trait_operation_method_body_fact|memo_trait_operation_body_check_resolver|memo_trait_operation_impl_candidate_builder|private_cache|private_state)/,
    "Drop candidate connector must not import Resource IR, backend, proof store/artifact, canonical-key, public-surface, scanner, method-body fact, body-check, candidate-builder, PrivateCache, or PrivateState layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationDropCandidateConnectorErrorKind:",
        "DropFactBuildRejected %SelfhostMemoTraitOperationPublicImplDropFactOrchestratorErrorKind",
        "NoEscapeGateRejected %SelfhostMemoTraitOperationDropNoEscapeGateErrorKind",
        "SourceReadFailed %i32",
        "ClassifierRejected %SelfhostMemoTraitOperationClassifierErrorKind",
        "DropResolveRejected %SelfhostMemoTraitOperationDropImplResolverErrorKind",
        "UnexpectedDropImplAbsent",
        "UnexpectedDropCheckMissing",
        "UnexpectedDropCheckUnknown",
        "UnexpectedDropCheckNotRequired",
        "CandidateRejected %SelfhostMemoTraitOperationPurityGateErrorKind",
        "CandidateDuplicate",
        "CandidateLookupRejected %SelfhostMemoTraitOperationImplTableErrorKind",
        "CandidatePushRejected %SelfhostMemoTraitOperationImplTableErrorKind",
    ],
    "connector errors must preserve typed build, gate, source read, classifier, resolver, candidate, duplicate, lookup, and push failures",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationDropCandidateConnectorErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "connector errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "connector APIs must return typed Result errors instead of bool/string errors",
);
assert.doesNotMatch(
    productionCode,
    /\b(SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_aggregate_proof_to_record)\b/,
    "production connector functions must not construct operation evidence records, aggregate proof, or proof-store values",
);
assert.doesNotMatch(
    productionCode,
    /\bSelfhostMemoTraitOperationDropEvidence::(?:PureDrop|NoDropRequired)\b/,
    "production connector functions must not directly synthesize PureDrop or NoDropRequired",
);
assert.doesNotMatch(
    productionCode,
    /\brecord\.trait_source\.operation\b/,
    "connector must not directly trust the operation kind carried inside the source identity",
);
assert.doesNotMatch(
    productionCode,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\n]*"(?:name|span|source|path|diagnostic|message|text)")/,
    "accepted production authority must not use call names, expression spans, source text, paths, or diagnostic text",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_header_input"),
    [
        "selfhost_memo_trait_public_impl_header_input_new record.visibility record.module_fingerprint record.declaration_ordinal record.impl_kind record.target_type_shape_hash record.trait_application_shape_hash record.type_parameter_count record.type_parameter_bound_count record.generic_binder_evidence",
    ],
    "Drop candidate connector must preserve the materializer record generic binder evidence mode when rebuilding the header input",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_classifier_result"),
    [
        "selfhost_memo_trait_operation_drop_candidate_connector_trait_application_input record",
        "selfhost_memo_trait_operation_classifier_evidence_result trait_application",
        "Result::Ok classifier:",
        "Result::Ok classifier",
        "Result::Err classifier_error:",
        "ClassifierRejected classifier_error",
    ],
    "classifier helper must derive operation authority from classifier evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_operation_is_drop"),
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
    "Drop filter must explicitly classify every current operation variant",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_duplicate_probe_result"),
    [
        "selfhost_memo_trait_operation_impl_candidate_for_type_operation_result output type_id SelfhostMemoTraitOperationEvidenceKind::Drop",
        "Result::Ok _existing:",
        "CandidateDuplicate",
        "SelfhostMemoTraitOperationImplTableErrorKind::CandidateMissing:",
        "Result::Ok unit",
    ],
    "duplicate probe must accept only lookup miss and fail closed on existing Drop candidate",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_present_drop_check_result"),
    [
        "SelfhostMemoTraitOperationDropCheckKind::DropImplPresent:",
        "Result::Ok drop_check",
        "SelfhostMemoTraitOperationDropCheckKind::DropImplAbsent:",
        "UnexpectedDropImplAbsent",
        "SelfhostMemoTraitOperationDropCheckKind::Missing:",
        "UnexpectedDropCheckMissing",
        "SelfhostMemoTraitOperationDropCheckKind::Unknown:",
        "UnexpectedDropCheckUnknown",
        "SelfhostMemoTraitOperationDropCheckKind::NotRequired:",
        "UnexpectedDropCheckNotRequired",
    ],
    "present Drop check gate must only pass DropImplPresent and must fail closed before NoDropRequired can be synthesized",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_append_drop_result"),
    [
        "selfhost_memo_trait_operation_drop_candidate_connector_duplicate_probe_result &output record.type_id",
        "SelfhostMemoTraitOperationMethodBodyCheckKind::NotRequired",
        "selfhost_memo_trait_operation_drop_impl_resolve_result",
        "SelfhostMemoTraitOperationDropImplSurfaceState::Complete",
        "selfhost_memo_trait_operation_drop_candidate_connector_present_drop_check_result drop_check",
        "selfhost_memo_trait_operation_impl_candidate_from_checks_result",
        "selfhost_memo_trait_operation_impl_table_push output candidate",
    ],
    "append_drop must probe duplicate first, require complete Drop facts, reuse purity gate candidate conversion, and push through table API",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_loop"),
    [
        "v::get records index",
        "Option::Some record:",
        "selfhost_memo_trait_operation_drop_candidate_connector_classifier_result record",
        "Result::Ok classifier:",
        "selfhost_memo_trait_operation_drop_candidate_connector_operation_is_drop classifier.operation",
        "then:",
        "selfhost_memo_trait_operation_drop_candidate_connector_append_drop_result output drop_table record classifier",
        "else:",
        "selfhost_memo_trait_operation_drop_candidate_connector_loop output drop_table source add index 1",
        "Result::Err e:",
        "selfhost_memo_trait_operation_impl_table_free output",
        "Option::None:",
        "SourceReadFailed index",
    ],
    "loop must classify records before Drop handling, skip non-Drop records, and clean output on classifier/read failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_append_from_records_result"),
    [
        "selfhost_memo_trait_operation_public_impl_drop_fact_table_from_records_result module source",
        "Result::Ok raw_drop_table:",
        "selfhost_memo_trait_operation_drop_candidate_connector_with_raw_drop_table_result output raw_drop_table source proofs",
        "Result::Err build_error:",
        "selfhost_memo_trait_operation_impl_table_free output",
        "DropFactBuildRejected build_error",
    ],
    "public entry must build typed Drop facts before applying proof gate and close output on build rejection",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_with_raw_drop_table_result"),
    [
        "selfhost_memo_trait_operation_drop_no_escape_gate_table_result &raw_drop_table proofs",
        "Result::Ok gated_drop_table:",
        "selfhost_memo_trait_operation_drop_impl_table_free raw_drop_table",
        "selfhost_memo_trait_operation_drop_candidate_connector_with_gated_table_result output gated_drop_table source",
        "Result::Err gate_error:",
        "selfhost_memo_trait_operation_drop_impl_table_free raw_drop_table",
        "selfhost_memo_trait_operation_impl_table_free output",
        "NoEscapeGateRejected gate_error",
    ],
    "raw Drop table path must apply no-escape gate before candidate append and clean both owners on gate failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_stage0"),
    [
        "selfhost_hir_module_new",
        "selfhost_memo_trait_operation_drop_candidate_connector_stage0_alloc_expr module0 type_id span",
    ],
    "stage0 must exercise the connector through typed HIR setup",
);
assert.ok(
    source.includes("SelfhostMemoTraitOperationDropEvidence::PureDrop") &&
        functionBlock(source, "selfhost_memo_trait_operation_drop_candidate_connector_stage0_duplicate_candidate").includes(
            "SelfhostMemoTraitOperationDropEvidence::PureDrop",
        ),
    "PureDrop may appear only as an existing stage0 candidate fixture, not as production authority",
);
assert.doesNotMatch(
    source,
    /maxLine|line count limit|line-count limit|doc comment length cap|documentation length cap|行数制限|コメント量制限|ドキュメントコメント.{0,12}上限/,
    "Drop candidate connector contract must not introduce line-count or documentation-comment length limits",
);

console.log("selfhost memo trait operation Drop candidate connector contract passed");
