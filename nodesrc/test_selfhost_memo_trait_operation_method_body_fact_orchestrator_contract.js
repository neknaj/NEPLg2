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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_method_body_fact_orchestrator.nepl";
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
        "# check/module/memo_trait_operation_method_body_fact_orchestrator",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "method body fact orchestrator must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("scan record table をこの module に渡します") &&
        source.includes("actual public impl materializer") &&
        source.includes("後続 stage の責務です"),
    "docs must state that actual public impl materialization remains outside this boundary",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、method name string から operation や HIR root を推測しません"),
    "docs must exclude source spelling and display-text authority",
);
assert.ok(
    source.includes("success / failure のどちらでも build input table owner はこの module が閉じます") &&
        source.includes("`source` scan record table は borrow"),
    "docs must fix source borrow and build-input owner cleanup contracts",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_method_body_fact_orchestrator/,
    "method body fact orchestrator must remain facade-private until full public surface orchestration stabilizes",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_method_body_fact_orchestrator/,
    "checker-layer method body fact orchestrator must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_method_body_fact_orchestrator_contract.js"),
    "source policy runner must execute the method body fact orchestrator contract",
);
assertOrdered(
    source,
    [
        "#import \"./memo_trait_operation_method_body_fact_input_scan\" as *",
        "#import \"./memo_trait_operation_method_body_fact_table_inputs\" as *",
        "#import \"./memo_trait_operation_method_body_resolver\" as *",
    ],
    "orchestrator must connect the typed scan boundary to the batch build boundary and output fact table boundary",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_drop_impl_resolver|memo_trait_operation_classifier|memo_trait_operation_method_body_fact_producer|memo_trait_operation_method_body_fact_table_builder)/,
    "orchestrator must not import public-surface, classifier, purity, impl-table, proof, resource, backend, producer, or low-level builder layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationMethodBodyFactOrchestratorErrorKind:",
        "InputScanRejected %SelfhostMemoTraitOperationMethodBodyFactInputScanErrorKind",
        "OutputTableAllocFailed %StdErrorKind",
        "BatchBuildRejected %SelfhostMemoTraitOperationMethodBodyFactTableInputsErrorKind",
    ],
    "orchestrator errors must be typed nested payloads",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationMethodBodyFactOrchestratorErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "orchestrator errors must not encode failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "orchestrator APIs must return typed Result errors instead of bool/string errors",
);
assert.doesNotMatch(
    code,
    /\b(SelfhostMemoTraitOperationBodyChecks|SelfhostMemoTraitOperationMethodBodyEvidence|SelfhostMemoTraitOperationDropEvidence|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|selfhost_memo_trait_operation_body_check_resolve_result|selfhost_memo_trait_operation_impl_candidate_from_checks_result|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_aggregate_proof_to_record)/,
    "orchestrator must not construct body check pairs, evidence records, aggregate proof, or proof-store values",
);
assert.doesNotMatch(
    code,
    /\bselfhost_memo_trait_operation_method_body_fact_new_result\b|\bselfhost_memo_trait_operation_method_body_fact_from_(?:summary|hir_root)_result\b|\bselfhost_memo_trait_operation_method_body_fact_table_builder_push_hir_root_result\b|\bselfhost_memo_trait_operation_method_body_table_push\b/,
    "orchestrator must not bypass scan and batch boundaries through direct fact constructors, low-level builder, or direct table push",
);
assert.doesNotMatch(
    code,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\n]*"(?:name|span|source|path|diagnostic|message|text)")/,
    "orchestrator accepted authority must not use display name, expression span, source text, path, or diagnostic text",
);

const buildScanned = functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_orchestrator_build_scanned_inputs_result");
assertOrdered(
    buildScanned,
    [
        "selfhost_memo_trait_operation_method_body_table_new",
        "selfhost_memo_trait_operation_method_body_fact_table_build_from_inputs_result table module &inputs",
        "Result::Ok built:",
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_free inputs",
        "Result::Ok built",
        "Result::Err batch_error:",
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_free inputs",
        "BatchBuildRejected batch_error",
        "Result::Err alloc_error:",
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_free inputs",
        "OutputTableAllocFailed alloc_error",
    ],
    "owned build-input bridge must free build inputs on success, batch rejection, and output allocation failure",
);
assert.doesNotMatch(
    before(after(buildScanned, "Result::Err batch_error:"), "Result::Err alloc_error:"),
    /selfhost_memo_trait_operation_method_body_table_free/,
    "batch rejection branch must not double-free an output table consumed by the batch boundary",
);

const publicEntry = functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_table_from_scan_records_result");
assertOrdered(
    publicEntry,
    [
        "selfhost_memo_trait_operation_method_body_fact_inputs_from_scan_records_result source",
        "Result::Ok inputs:",
        "selfhost_memo_trait_operation_method_body_fact_orchestrator_build_scanned_inputs_result module inputs",
        "Result::Err scan_error:",
        "InputScanRejected scan_error",
    ],
    "public entry must run scan first and pass owned build inputs to the owned bridge",
);
assert.doesNotMatch(
    publicEntry,
    /selfhost_memo_trait_operation_method_body_fact_input_scan_record_table_free|selfhost_hir_module_free/,
    "public entry must not close borrowed source scan table or borrowed HIR module",
);

assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_orchestrator_error_kind_eq"),
    [
        "InputScanRejected a_scan:",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_error_kind_eq a_scan b_scan",
        "OutputTableAllocFailed a_alloc:",
        "selfhost_memo_trait_operation_method_body_fact_orchestrator_std_error_kind_eq a_alloc b_alloc",
        "BatchBuildRejected a_batch:",
        "selfhost_memo_trait_operation_method_body_fact_table_inputs_error_kind_eq a_batch b_batch",
    ],
    "error equality must compare nested scan, allocation, and batch payloads without wildcard fallback",
);

assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_orchestrator_stage0"),
    [
        "selfhost_hir_module_new",
        "selfhost_hir_module_add_expr module0 selfhost_hir_expr_unit",
        "selfhost_memo_trait_operation_method_body_fact_orchestrator_stage0_external module1 pure_id type_id span",
    ],
    "stage0 must construct a small typed HIR module instead of relying on source text",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_orchestrator_stage0_finish"),
    [
        "selfhost_memo_trait_operation_method_body_fact_orchestrator_stage0_accepted &module",
        "selfhost_memo_trait_operation_method_body_fact_orchestrator_stage0_scan_missing &module",
        "selfhost_memo_trait_operation_method_body_fact_orchestrator_stage0_batch_missing_root &module",
        "selfhost_hir_module_free module",
    ],
    "stage0 must cover accepted path, scan rejection, batch rejection, and free HIR module owner",
);
assert.doesNotMatch(
    code,
    /line count|line-count|doc comment length|doc-comment length|max lines|max-lines|行数制限|コメント長制限|doc comment 長制限.*(?:enforce|reject|fail)/i,
    "source policy must not introduce line-count or doc-comment-length caps",
);
assert.doesNotMatch(
    code,
    /\bunwrap\b|\bunreachable\b|panic|fallback|first[-_ ]wins/i,
    "orchestrator must not rely on unwrap, unreachable, panic, fallback, or first-wins behavior",
);

console.log("selfhost memo trait operation method body fact orchestrator contract passed");
