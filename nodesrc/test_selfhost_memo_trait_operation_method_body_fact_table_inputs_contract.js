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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_method_body_fact_table_inputs.nepl";
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
        "# check/module/memo_trait_operation_method_body_fact_table_inputs",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "method body fact table inputs must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("public surface scanner、trait operation classifier、purity gate、operation impl candidate table をこの module に混ぜず") &&
        source.includes("typed input 列の反復投入"),
    "docs must define the input batch module as a narrow typed-input iteration boundary",
);
assert.ok(
    source.includes("input read failure が起きた場合、まだ builder に渡していない output table owner はこの module が") &&
        source.includes("builder rejection が起きた場合、既存 builder が output table owner の cleanup を完結させます"),
    "docs must spell out input-read cleanup and builder-rejection owner recovery",
);
assert.ok(
    source.includes("input table は borrow として読みます") &&
        source.includes("caller は success / failure のあとに `selfhost_memo_trait_operation_method_body_fact_build_input_table_free` で閉じます"),
    "docs must state that input table ownership remains with the caller",
);
assert.ok(
    source.includes("fact constructor、direct table push、duplicate lookup、surface completeness decision、method body evidence 作成、Drop evidence 作成、operation evidence record 作成、Resource IR proof、backend artifact、proof store、public surface scanning を行いません"),
    "docs must exclude direct fact construction, direct push, lookup, completeness, evidence construction, proof, backend, and public surface scanning",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_method_body_fact_table_inputs/,
    "method body fact table inputs must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_method_body_fact_table_inputs/,
    "checker-layer method body fact table inputs must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_method_body_fact_table_inputs_contract.js"),
    "source policy runner must execute the method body fact table inputs contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_method_body_effect_checker\" as *",
        "#import \"./memo_trait_operation_method_body_fact_table_builder\" as *",
        "#import \"./memo_trait_operation_method_body_resolver\" as *",
    ],
    "inputs module must depend on HIR root identity, TypeId, operation kind, typed error helpers, builder, and output table owner",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_drop_impl_resolver|memo_trait_operation_classifier)/,
    "inputs module must not import Resource IR, backend, proof store, artifact, canonical-key, public-surface, impl table, classifier, purity gate, body check resolver, or Drop resolver layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationMethodBodyFactBuildInput:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "root %SelfhostHirExprId",
        "fuel %i32",
        "pub struct SelfhostMemoTraitOperationMethodBodyFactBuildInputTable:",
        "records %Vec SelfhostMemoTraitOperationMethodBodyFactBuildInput",
        "pub struct SelfhostMemoTraitOperationMethodBodyFactTableInputsBuilderRejected:",
        "index %i32",
        "error %SelfhostMemoTraitOperationMethodBodyFactTableBuilderErrorKind",
        "pub enum SelfhostMemoTraitOperationMethodBodyFactTableInputsErrorKind:",
        "InputPushFailed %StdErrorKind",
        "InputReadFailed %i32",
        "BuilderRejected %SelfhostMemoTraitOperationMethodBodyFactTableInputsBuilderRejected",
    ],
    "inputs and nested error payloads must be typed records",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationMethodBodyFactTableInputsErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "input batch errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "input batch APIs must return typed Result errors instead of bool/string errors",
);
assert.doesNotMatch(
    code,
    /\b(SelfhostMemoTraitOperationBodyChecks|SelfhostMemoTraitOperationMethodBodyCheck|SelfhostMemoTraitOperationDropCheck|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitOperationMethodBodyEvidence|SelfhostMemoTraitOperationDropEvidence|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|selfhost_memo_trait_operation_body_check_resolve_result|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_operation_method_body_evidence_new|selfhost_memo_trait_operation_drop_evidence_new|selfhost_memo_trait_aggregate_proof_to_record)/,
    "inputs module must not construct body check pairs, Drop checks, operation evidence records, aggregate proof, or proof-store values",
);
assert.doesNotMatch(
    code,
    /\bselfhost_memo_trait_operation_method_body_resolve_result\b|\bSelfhostMemoTraitOperationMethodBodySurfaceState\b/,
    "inputs module must not run resolver lookup or choose surface completeness",
);
assert.doesNotMatch(
    code,
    /^\s+SelfhostMemoTraitOperationMethodBodyFact\s+/m,
    "inputs module must not bypass the resolver fact constructor with a direct method-body fact struct expression",
);
assert.doesNotMatch(
    code,
    /\bselfhost_memo_trait_operation_method_body_fact_from_(?:summary|hir_root)_result\b|\bselfhost_memo_trait_operation_method_body_fact_new_result\b|\bselfhost_memo_trait_operation_method_body_table_push\b/,
    "inputs module must not bypass the builder through the producer, fact constructor, or direct table push",
);
assert.doesNotMatch(
    code,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\n]*"(?:name|span|source|path|diagnostic|message|text)")/,
    "accepted input authority must not use display name, expression span, source text, path, or diagnostic text",
);
const loopBlock = functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_table_inputs_loop");
assertOrdered(
    loopBlock,
    [
        "ge index selfhost_memo_trait_operation_method_body_fact_build_input_table_len inputs",
        "Result::Ok table",
        "v::get records index",
        "Option::Some input:",
        "selfhost_memo_trait_operation_method_body_fact_table_builder_push_hir_root_result table module input.type_id input.operation input.root input.fuel",
        "Result::Ok next_table:",
        "selfhost_memo_trait_operation_method_body_fact_table_inputs_loop next_table module inputs add index 1",
        "Result::Err builder_error:",
        "Result::Err selfhost_memo_trait_operation_method_body_fact_table_inputs_builder_error index builder_error",
        "Option::None:",
        "selfhost_memo_trait_operation_method_body_table_free table",
        "Result::Err SelfhostMemoTraitOperationMethodBodyFactTableInputsErrorKind::InputReadFailed index",
    ],
    "input loop must read inputs, call the builder, preserve builder errors with index, and free output table on read failure",
);
const builderErrorBranch = before(after(loopBlock, "Result::Err builder_error:"), "Option::None:");
assert.doesNotMatch(
    builderErrorBranch,
    /selfhost_memo_trait_operation_method_body_table_free/,
    "builder rejection branch must not double-free a table owner already consumed by the builder",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_table_build_from_inputs_result"),
    [
        "selfhost_memo_trait_operation_method_body_fact_table_inputs_loop table module inputs 0",
    ],
    "public build API must delegate to the loop from index zero",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_table_inputs_error_kind_eq"),
    [
        "InputPushFailed a_push:",
        "selfhost_memo_trait_operation_method_body_fact_table_inputs_std_error_kind_eq a_push b_push",
        "InputReadFailed a_index:",
        "eq a_index b_index",
        "BuilderRejected a_rejected:",
        "eq a_rejected.index b_rejected.index",
        "selfhost_memo_trait_operation_method_body_fact_table_builder_error_kind_eq a_rejected.error b_rejected.error",
    ],
    "input batch error equality must compare input errors and nested builder payloads",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_table_inputs_stage0_with_owned_inputs"),
    [
        "selfhost_memo_trait_operation_method_body_fact_table_inputs_stage0_build_with_inputs module &inputs",
        "Result::Ok result:",
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_free inputs",
        "Result::Ok result",
        "Result::Err e:",
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_free inputs",
        "Result::Err e",
    ],
    "stage0 must free input table ownership on success and setup failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_table_inputs_stage0_second_copy_rejected"),
    [
        "SelfhostMemoTraitOperationEvidenceKind::Eq pure_id 16",
        "SelfhostMemoTraitOperationEvidenceKind::Copy pure_id 16",
        "selfhost_memo_trait_operation_method_body_fact_table_inputs_stage0_with_owned_inputs module inputs2",
    ],
    "stage0 must prove the second input index is preserved for builder rejection",
);
assert.doesNotMatch(
    code,
    /\b(?:line[_-]?count|doc(?:ument)?[_-]?comment[_-]?(?:length|limit|max)|max[_-]?lines|too[_-]?long|LOC|locLimit|lineLimit)\b/i,
    "input batch policy must not introduce line-count or doc-comment-length caps",
);
assert.doesNotMatch(
    code,
    /\bunwrap_ok\b|\bunwrap_err\b|unreachable/,
    "inputs implementation and smoke helpers must not use unwrap/unreachable shortcuts",
);

console.log("selfhost memo trait operation method body fact table inputs contract passed");
