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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_method_body_fact_table_builder.nepl";
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
        "# check/module/memo_trait_operation_method_body_fact_table_builder",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "method body fact table builder must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("fact producer と resolver table push の責務を混ぜず") &&
        source.includes("producer error、table push error、owner cleanup の位置を typed enum と文書化された契約で固定します"),
    "docs must define the builder as a narrow producer-result-to-table-owner boundary",
);
assert.ok(
    source.includes("fact producer が失敗した場合、table owner はまだ table push に渡されていないため、この module が `selfhost_memo_trait_operation_method_body_table_free` で閉じます") &&
        source.includes("table push が失敗した場合、既存 resolver の `selfhost_memo_trait_operation_method_body_table_push` が `Vec` owner を回収して閉じます"),
    "docs must spell out producer-failure cleanup and table-push owner recovery",
);
assert.ok(
    source.includes("`Result::Err` を受け取った caller は、渡した table owner を再利用してはいけません") &&
        source.includes("`Result::Err` の後に caller が元の `table` を再利用したり free したりしてはいけません"),
    "docs must make the destructive append Err ownership contract explicit",
);
assert.ok(
    source.includes("table lookup、duplicate rejection、surface completeness decision、operation evidence record 作成、method body evidence 作成、Drop evidence 作成、Resource IR proof、backend artifact、proof store、public surface scanning を行いません"),
    "docs must exclude lookup, duplicate rejection, completeness decisions, evidence construction, proof, backend, and public surface scanning",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_method_body_fact_table_builder/,
    "method body fact table builder must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_method_body_fact_table_builder/,
    "checker-layer method body fact table builder must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_method_body_fact_table_builder_contract.js"),
    "source policy runner must execute the method body fact table builder contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_method_body_effect_checker\" as *",
        "#import \"./memo_trait_operation_method_body_fact_producer\" as *",
        "#import \"./memo_trait_operation_method_body_resolver\" as *",
    ],
    "builder must depend on HIR root identity, TypeId, operation kind, effect checker errors, fact producer, and resolver table owner",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_drop_impl_resolver)/,
    "builder must not import Resource IR, backend, proof store, artifact, public-surface, impl table, purity gate, body check resolver, or Drop resolver layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationMethodBodyFactTableBuilderErrorKind:",
        "FactProducerRejected %SelfhostMemoTraitOperationMethodBodyFactProducerErrorKind",
        "TableRejected %SelfhostMemoTraitOperationMethodBodyResolverErrorKind",
        "pub struct SelfhostMemoTraitOperationMethodBodyFactTableBuilderStage0Summary:",
        "pure_push_len %Result i32 SelfhostMemoTraitOperationMethodBodyFactTableBuilderErrorKind",
        "missing_root_rejected %Result i32 SelfhostMemoTraitOperationMethodBodyFactTableBuilderErrorKind",
    ],
    "builder outputs and nested errors must be typed payloads",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationMethodBodyFactTableBuilderErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "builder errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "builder APIs must return typed Result errors instead of bool/string errors",
);
assert.doesNotMatch(
    code,
    /\b(SelfhostMemoTraitOperationBodyChecks|SelfhostMemoTraitOperationMethodBodyCheck|SelfhostMemoTraitOperationDropCheck|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitOperationMethodBodyEvidence|SelfhostMemoTraitOperationDropEvidence|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|selfhost_memo_trait_operation_body_check_resolve_result|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_operation_method_body_evidence_new|selfhost_memo_trait_operation_drop_evidence_new|selfhost_memo_trait_aggregate_proof_to_record)/,
    "builder must not construct body check pairs, Drop checks, operation evidence records, aggregate proof, or proof-store values",
);
assert.doesNotMatch(
    code,
    /\bselfhost_memo_trait_operation_method_body_resolve_result\b|\bSelfhostMemoTraitOperationMethodBodySurfaceState\b/,
    "builder must not run resolver lookup or choose surface completeness",
);
assert.doesNotMatch(
    code,
    /^\s+SelfhostMemoTraitOperationMethodBodyFact\s+/m,
    "builder must not bypass the resolver fact constructor with a direct method-body fact struct expression",
);
assert.doesNotMatch(
    code,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\n]*"(?:name|span|source|path|diagnostic|message|text)")/,
    "accepted builder authority must not use display name, expression span, source text, path, or diagnostic text",
);
const pushBlock = functionBlock(
    source,
    "selfhost_memo_trait_operation_method_body_fact_table_builder_push_hir_root_result",
);
assertOrdered(
    pushBlock,
    [
        "selfhost_memo_trait_operation_method_body_fact_from_hir_root_result module type_id operation root fuel",
        "Result::Ok fact:",
        "selfhost_memo_trait_operation_method_body_table_push table fact",
        "Result::Ok next_table:",
        "Result::Ok next_table",
        "Result::Err table_error:",
        "Result::Err selfhost_memo_trait_operation_method_body_fact_table_builder_table_error table_error",
        "Result::Err producer_error:",
        "selfhost_memo_trait_operation_method_body_table_free table",
        "Result::Err selfhost_memo_trait_operation_method_body_fact_table_builder_producer_error producer_error",
    ],
    "push API must produce fact before table push, wrap table errors, and free table on producer errors",
);
const tableErrorBranch = before(after(pushBlock, "Result::Err table_error:"), "Result::Err producer_error:");
assert.doesNotMatch(
    tableErrorBranch,
    /selfhost_memo_trait_operation_method_body_table_free/,
    "table push error branch must not double-free a table owner already consumed by resolver push",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_table_builder_error_kind_eq"),
    [
        "FactProducerRejected a_producer:",
        "selfhost_memo_trait_operation_method_body_fact_producer_error_kind_eq a_producer b_producer",
        "TableRejected a_table:",
        "selfhost_memo_trait_operation_method_body_resolver_error_kind_eq a_table b_table",
    ],
    "builder error equality must compare nested producer and resolver payloads",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_table_builder_stage0_single_push_len"),
    [
        "selfhost_memo_trait_operation_method_body_table_new",
        "selfhost_memo_trait_operation_method_body_fact_table_builder_push_hir_root_result table module type_id operation root fuel",
        "Result::Ok next_table:",
        "selfhost_memo_trait_operation_method_body_fact_table_builder_stage0_len_and_free next_table",
        "Result::Err error:",
        "Result::Ok Result::Err error",
    ],
    "stage0 single push helper must allocate table, call public builder, free success table through len helper, and preserve builder errors",
);
assert.doesNotMatch(
    code,
    /\b(?:line[_-]?count|doc(?:ument)?[_-]?comment[_-]?(?:length|limit|max)|max[_-]?lines|too[_-]?long|LOC|locLimit|lineLimit)\b/i,
    "method body fact table builder policy must not introduce line-count or doc-comment-length caps",
);
assert.doesNotMatch(
    code,
    /\bunwrap_ok\b|\bunwrap_err\b|unreachable/,
    "builder implementation and smoke helpers must not use unwrap/unreachable shortcuts",
);

console.log("selfhost memo trait operation method body fact table builder contract passed");
