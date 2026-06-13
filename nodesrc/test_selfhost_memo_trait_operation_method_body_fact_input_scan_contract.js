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

function after(text, marker) {
    const index = text.indexOf(marker);
    assert.notEqual(index, -1, `missing marker ${marker}`);
    return text.slice(index + marker.length);
}

function before(text, marker) {
    const index = text.indexOf(marker);
    assert.notEqual(index, -1, `missing marker ${marker}`);
    return text.slice(0, index);
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_method_body_fact_input_scan.nepl";
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
        "# check/module/memo_trait_operation_method_body_fact_input_scan",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "method body fact input scan must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("Eq / Hash だけを `SelfhostMemoTraitOperationMethodBodyFactBuildInputTable` へ投入します") &&
        source.includes("Copy / Drop に method body root が付いている場合") &&
        source.includes("Eq / Hash に method body root が欠けている場合"),
    "docs must describe the operation matrix for method body roots",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、method name string から operation や root を推測しません"),
    "docs must exclude source spelling and display-text authority",
);
assert.ok(
    source.includes("この module は HIR effect checker、fact producer、fact table builder、method body resolver lookup、Drop resolver、purity gate、operation impl candidate table、Resource IR proof、backend artifact、proof store、public surface hash を実行しません"),
    "docs must keep scan separate from checker, resolver, proof, backend, and public surface layers",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_method_body_fact_input_scan/,
    "method body fact input scan must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_method_body_fact_input_scan/,
    "checker-layer method body fact input scan must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_method_body_fact_input_scan_contract.js"),
    "source policy runner must execute the method body fact input scan contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_method_body_fact_table_inputs\" as *",
    ],
    "scan module must depend only on HIR root identity, TypeId, operation kind, and existing build input table boundary",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_drop_impl_resolver|memo_trait_operation_classifier|memo_trait_operation_method_body_effect_checker|memo_trait_operation_method_body_fact_producer|memo_trait_operation_method_body_fact_table_builder|memo_trait_operation_method_body_resolver)/,
    "scan module must not import public-surface, classifier, purity, resolver, builder, effect checker, proof, resource, or backend layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationMethodBodyFactInputScanRecord:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "method_body_root %Option SelfhostHirExprId",
        "fuel %i32",
        "pub enum SelfhostMemoTraitOperationMethodBodyFactInputScanErrorKind:",
        "SourcePushFailed %StdErrorKind",
        "OutputInputAllocFailed %StdErrorKind",
        "SourceReadFailed %i32",
        "OutputInputPushFailed %SelfhostMemoTraitOperationMethodBodyFactTableInputsErrorKind",
        "RequiredMethodBodyMissing %SelfhostMemoTraitOperationMethodBodyFactInputScanIndexedOperation",
        "UnexpectedMethodBodyRoot %SelfhostMemoTraitOperationMethodBodyFactInputScanIndexedOperation",
    ],
    "scan records and errors must be typed payloads",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationMethodBodyFactInputScanErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "scan errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "scan APIs must return typed Result errors instead of bool/string errors",
);
assert.doesNotMatch(
    code,
    /\b(SelfhostMemoTraitOperationBodyChecks|SelfhostMemoTraitOperationMethodBodyCheck|SelfhostMemoTraitOperationDropCheck|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitOperationMethodBodyEvidence|SelfhostMemoTraitOperationDropEvidence|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|selfhost_memo_trait_operation_body_check_resolve_result|selfhost_memo_trait_operation_impl_candidate_from_checks_result|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_aggregate_proof_to_record)/,
    "scan module must not construct body check pairs, operation evidence records, aggregate proof, or proof-store values",
);
assert.doesNotMatch(
    code,
    /\bselfhost_memo_trait_operation_method_body_resolve_result\b|\bSelfhostMemoTraitOperationMethodBodySurfaceState\b|\bselfhost_memo_trait_operation_method_body_fact_from_(?:summary|hir_root)_result\b|\bselfhost_memo_trait_operation_method_body_fact_new_result\b|\bselfhost_memo_trait_operation_method_body_table_push\b/,
    "scan module must not run resolver lookup or bypass through fact constructor/direct table push",
);
assert.doesNotMatch(
    code,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\n]*"(?:name|span|source|path|diagnostic|message|text)")/,
    "accepted scan authority must not use display name, expression span, source text, path, or diagnostic text",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_input_scan_operation_requires_body"),
    [
        "SelfhostMemoTraitOperationEvidenceKind::Copy:",
        "false",
        "SelfhostMemoTraitOperationEvidenceKind::Drop:",
        "false",
        "SelfhostMemoTraitOperationEvidenceKind::Eq:",
        "true",
        "SelfhostMemoTraitOperationEvidenceKind::Hash:",
        "true",
    ],
    "operation matrix must require method bodies only for Eq and Hash",
);
const recordIntoInputs = functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_input_scan_record_into_inputs");
assertOrdered(
    recordIntoInputs,
    [
        "selfhost_memo_trait_operation_method_body_fact_input_scan_operation_requires_body record.operation",
        "Option::Some root:",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_push_output inputs record root",
        "Option::None:",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_required_missing_error inputs index record.operation",
        "Option::Some _unexpected:",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_unexpected_root_error inputs index record.operation",
        "Option::None:",
        "Result::Ok inputs",
    ],
    "record scan must push Eq/Hash roots, reject missing Eq/Hash roots, reject Copy/Drop roots, and skip Copy/Drop none",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_input_scan_required_missing_error"),
    [
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_free inputs",
        "RequiredMethodBodyMissing",
    ],
    "required missing branch must free partial output inputs",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_input_scan_unexpected_root_error"),
    [
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_free inputs",
        "UnexpectedMethodBodyRoot",
    ],
    "unexpected root branch must free partial output inputs",
);
const pushOutput = functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_input_scan_push_output");
assertOrdered(
    pushOutput,
    [
        "selfhost_memo_trait_operation_method_body_fact_build_input_new record.type_id record.operation root record.fuel",
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_push inputs build_input",
        "OutputInputPushFailed push_error",
    ],
    "output scan must use the existing build input constructor and table push boundary",
);
assert.doesNotMatch(
    before(after(pushOutput, "Result::Err push_error:"), "\n\n"),
    /selfhost_memo_trait_operation_method_body_fact_build_input_table_free/,
    "output push rejection branch must not double-free an owner consumed by the output input table push",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_input_scan_loop"),
    [
        "v::get records index",
        "Option::Some record:",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_record_into_inputs inputs record index",
        "Result::Ok next_inputs:",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_loop next_inputs source add index 1",
        "Option::None:",
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_free inputs",
        "SourceReadFailed index",
    ],
    "scan loop must preserve record index, propagate nested errors, and free output inputs on source read failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_inputs_from_scan_records_result"),
    [
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_new",
        "Result::Ok inputs:",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_loop inputs source 0",
        "OutputInputAllocFailed e",
    ],
    "public scan API must allocate output inputs and start scanning at index zero",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_input_scan_error_kind_eq"),
    [
        "OutputInputPushFailed a_output:",
        "selfhost_memo_trait_operation_method_body_fact_table_inputs_error_kind_eq a_output b_output",
        "RequiredMethodBodyMissing a_required:",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_indexed_operation_eq a_required b_required",
        "UnexpectedMethodBodyRoot a_unexpected:",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_indexed_operation_eq a_unexpected b_unexpected",
    ],
    "scan error equality must compare nested output and indexed operation payloads",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_input_scan_stage0_accepted"),
    [
        "SelfhostMemoTraitOperationEvidenceKind::Eq some eq_root",
        "SelfhostMemoTraitOperationEvidenceKind::Hash some hash_root",
        "SelfhostMemoTraitOperationEvidenceKind::Copy none",
        "SelfhostMemoTraitOperationEvidenceKind::Drop none",
    ],
    "stage0 accepted case must prove Eq/Hash are included while Copy/Drop without roots are skipped",
);
assert.doesNotMatch(
    code,
    /\b(?:line[_-]?count|doc(?:ument)?[_-]?comment[_-]?(?:length|limit|max)|max[_-]?lines|too[_-]?long|LOC|locLimit|lineLimit)\b/i,
    "scan policy must not introduce line-count or doc-comment-length caps",
);
assert.doesNotMatch(
    code,
    /\bunwrap_ok\b|\bunwrap_err\b|unreachable/,
    "scan implementation and smoke helpers must not use unwrap/unreachable shortcuts",
);

console.log("selfhost memo trait operation method body fact input scan contract passed");
