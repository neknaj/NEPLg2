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

function after(text, marker) {
    const index = text.indexOf(marker);
    assert.notEqual(index, -1, `missing marker ${marker}`);
    return text.slice(index + marker.length);
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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_drop_fact_orchestrator.nepl";
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
        "# check/module/memo_trait_operation_public_impl_drop_fact_orchestrator",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "public impl Drop fact orchestrator must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("trusted operation classifier で Drop と再確認された record だけ") &&
        source.includes("Drop impl fact table 専用の入力として読み") &&
        source.includes("public impl header が invalid な record、count-only generic record、generic instantiation / bound solving が未接続な detailed generic record"),
    "docs must define this module as a header-validated classifier-confirmed Drop-only projection from materializer records",
);
assert.ok(
    source.includes("record.trait_source.operation` は直接信用しません") &&
        source.includes("non-Drop record は error ではなく skip") &&
        source.includes("Eq / Hash record が method body root を持つことは正常"),
    "docs must reject caller-supplied operation authority and explain non-Drop skip semantics",
);
assert.ok(
    source.includes("Drop evidence、operation evidence record、aggregate proof status、Resource IR no-escape proof") &&
        source.includes("NoDropRequired") &&
        source.includes("PureDrop"),
    "docs must state that this boundary does not synthesize Drop evidence or Resource proof",
);
assertDocBeforeTopLevel(
    source,
    "Clone for SelfhostMemoTraitOperationPublicImplDropFactOrchestratorErrorKind",
    "impl Clone for SelfhostMemoTraitOperationPublicImplDropFactOrchestratorErrorKind:",
);
assertDocBeforeTopLevel(
    source,
    "Copy for SelfhostMemoTraitOperationPublicImplDropFactOrchestratorErrorKind",
    "impl Copy for SelfhostMemoTraitOperationPublicImplDropFactOrchestratorErrorKind:",
);
assertDocBeforeTopLevel(
    source,
    "Clone for SelfhostMemoTraitOperationPublicImplDropFactOrchestratorAcceptedSummary",
    "impl Clone for SelfhostMemoTraitOperationPublicImplDropFactOrchestratorAcceptedSummary:",
);
assertDocBeforeTopLevel(
    source,
    "Copy for SelfhostMemoTraitOperationPublicImplDropFactOrchestratorStage0Summary",
    "impl Copy for SelfhostMemoTraitOperationPublicImplDropFactOrchestratorStage0Summary:",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_public_impl_drop_fact_orchestrator/,
    "public impl Drop fact orchestrator must remain facade-private until full Resource proof orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_public_impl_drop_fact_orchestrator/,
    "checker-layer public impl Drop fact orchestrator must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_contract.js"),
    "source policy runner must execute the public impl Drop fact orchestrator contract",
);
assertOrdered(
    source,
    [
        "#import \"./memo_trait_operation_classifier\" as *",
        "#import \"./memo_trait_operation_drop_impl_fact_table_builder\" as *",
        "#import \"./memo_trait_operation_drop_impl_resolver\" as *",
        "#import \"./memo_trait_operation_public_impl_materializer\" as *",
        "#import \"./memo_trait_public_impl_header\" as *",
    ],
    "orchestrator imports must go through classifier, Drop fact builder, resolver, public impl materializer records, and header fixture types",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_impl_candidate_builder|memo_trait_public_impl_scanner)/,
    "orchestrator must not import Resource IR, backend, proof store, canonical-key, public-surface, evidence producer, impl table, purity gate, body-check resolver, candidate builder, or scanner layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationPublicImplDropFactOrchestratorErrorKind:",
        "OutputTableAllocFailed %StdErrorKind",
        "SourceReadFailed %i32",
        "HeaderRejected %SelfhostMemoTraitPublicImplHeaderErrorKind",
        "ClassifierRejected %SelfhostMemoTraitOperationClassifierErrorKind",
        "GenericImplInstantiationUnsupported",
        "RequiredDropBodyRootMissing %i32",
        "BuilderRejected %SelfhostMemoTraitOperationDropImplFactTableBuilderErrorKind",
    ],
    "orchestrator errors must keep typed allocation, source read, header, classifier, generic unsupported, required root, and builder failures",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationPublicImplDropFactOrchestratorErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "orchestrator errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "orchestrator APIs must return typed Result errors instead of bool/string errors",
);
assert.doesNotMatch(
    code,
    /\b(SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitOperationEvidenceProducerInput|SelfhostMemoTraitOperationMethodBodyEvidence|SelfhostMemoTraitOperationDropEvidence|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_operation_evidence_producer_input_new|selfhost_memo_trait_operation_method_body_evidence_new|selfhost_memo_trait_operation_drop_evidence_new|selfhost_memo_trait_aggregate_proof_to_record)/,
    "orchestrator must not construct operation evidence, Drop evidence, aggregate proof, or proof-store values",
);
assert.doesNotMatch(
    code,
    /\bSelfhostMemoTraitOperationDropCheckKind::DropImplAbsent\b|\bSelfhostMemoTraitOperationDropEvidence::(?:NoDropRequired|PureDrop)\b/,
    "orchestrator must not synthesize DropImplAbsent, NoDropRequired, or PureDrop",
);
assert.doesNotMatch(
    code,
    /\brecord\.trait_source\.operation\b/,
    "orchestrator must not directly trust the operation kind carried inside the source identity",
);
assert.doesNotMatch(
    code,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\n]*"(?:name|span|source|path|diagnostic|message|text)")/,
    "accepted input authority must not use call names, expression spans, source text, path, or diagnostic text",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_header_result"),
    [
        "selfhost_memo_trait_public_impl_header_input_new record.visibility record.module_fingerprint record.declaration_ordinal record.impl_kind record.target_type_shape_hash record.trait_application_shape_hash record.type_parameter_count record.type_parameter_bound_count record.generic_binder_evidence",
        "selfhost_memo_trait_public_impl_header_evidence_result header_input",
        "SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence::Monomorphic:",
        "Result::Ok unit",
        "SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence::Detailed _evidence:",
        "GenericImplInstantiationUnsupported",
        "HeaderRejected header_error",
    ],
    "header helper must validate record header fields, preserve generic binder evidence mode, and reject detailed generic records before Drop fact construction",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_classifier_result"),
    [
        "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_trait_application_input record",
        "selfhost_memo_trait_operation_classifier_evidence_result trait_application",
        "Result::Ok classifier:",
        "Result::Ok classifier",
        "Result::Err classifier_error:",
        "ClassifierRejected classifier_error",
    ],
    "classifier helper must derive operation authority from classifier evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_operation_is_drop"),
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
const pushDropBlock = functionBlock(
    source,
    "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_push_drop_record_result",
);
assertOrdered(
    pushDropBlock,
    [
        "match record.method_body_root:",
        "Option::Some root:",
        "selfhost_memo_trait_operation_drop_impl_fact_table_builder_push_hir_root_result table module record.type_id record.module_fingerprint root record.fuel",
        "Result::Ok next_table:",
        "Result::Ok next_table",
        "Result::Err builder_error:",
        "BuilderRejected builder_error",
        "Option::None:",
        "selfhost_memo_trait_operation_drop_impl_table_free table",
        "RequiredDropBodyRootMissing index",
    ],
    "Drop record push must require a body root, delegate to the builder, and free the table on missing root",
);
const builderRejectionBranch = before(after(pushDropBlock, "Result::Err builder_error:"), "Option::None:");
assert.doesNotMatch(
    builderRejectionBranch,
    /selfhost_memo_trait_operation_drop_impl_table_free/,
    "builder rejection branch must not double-free a table owner consumed by the builder",
);
const loopBlock = functionBlock(source, "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_loop");
assertOrdered(
    loopBlock,
    [
        "v::get records index",
        "Option::Some record:",
        "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_header_result record",
        "Result::Ok _header_ok:",
        "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_classifier_result record",
        "Result::Ok classifier:",
        "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_operation_is_drop classifier.operation",
        "then:",
        "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_push_drop_record_result table module record index",
        "else:",
        "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_loop table module source add index 1",
        "Result::Err e:",
        "selfhost_memo_trait_operation_drop_impl_table_free table",
        "Result::Err e:",
        "selfhost_memo_trait_operation_drop_impl_table_free table",
        "Option::None:",
        "selfhost_memo_trait_operation_drop_impl_table_free table",
        "SourceReadFailed index",
    ],
    "loop must validate the header before classifier use, classify before reading Drop root, skip non-Drop records, and clean up table on header/classifier/read failure",
);
assert.ok(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_stage0_accepted_source").includes(
        "registry.eq_source",
    ),
    "stage0 accepted source must include a non-Drop record with a method root to prove skip semantics",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_stage0_accepted_from_table"),
    [
        "selfhost_memo_trait_operation_drop_impl_table_len &table",
        "selfhost_memo_trait_operation_drop_impl_resolve_result",
        "SelfhostMemoTraitOperationDropImplSurfaceState::Complete",
        "selfhost_memo_trait_operation_drop_impl_table_free table",
    ],
    "accepted stage0 must observe the built table through resolver and then free the owner",
);
assert.doesNotMatch(
    source,
    /maxLine|line count limit|line-count limit|doc comment length cap|documentation length cap|行数制限|コメント量制限|ドキュメントコメント.{0,12}上限/,
    "orchestrator contract must not introduce line-count or documentation-comment length limits",
);

console.log("selfhost memo trait operation public impl Drop fact orchestrator contract ok");
