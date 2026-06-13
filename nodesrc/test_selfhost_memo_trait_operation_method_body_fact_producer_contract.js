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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_method_body_fact_producer.nepl";
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
        "# check/module/memo_trait_operation_method_body_fact_producer",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "method body fact producer must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("effect summary と、`memo_trait_operation_method_body_resolver` が読む fact constructor を接続します") &&
        source.includes("method body が必要な operation は `Eq` / `Hash` だけである、という matrix を resolver の public constructor に委譲"),
    "docs must define this module as a narrow effect-summary-to-fact producer",
);
assert.ok(
    source.includes("この module は fact table owner を消費しません") &&
        source.includes("table への追加と duplicate rejection は既存 `selfhost_memo_trait_operation_method_body_table_push` / resolver lookup が担当します"),
    "docs must keep fact production separate from table ownership and duplicate handling",
);
assert.ok(
    source.includes("operation evidence record、method body evidence、Drop evidence、body check pair、aggregate proof status、Resource IR proof、backend artifact、public surface orchestration は作りません"),
    "docs must exclude evidence, body checks, proof status, Resource IR, backend, and public surface orchestration",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_method_body_fact_producer/,
    "method body fact producer must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_method_body_fact_producer/,
    "checker-layer method body fact producer must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_method_body_fact_producer_contract.js"),
    "source policy runner must execute the method body fact producer contract",
);
assertOrdered(
    source,
    [
        "#import \"core/math\" as *",
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_method_body_effect_checker\" as *",
        "#import \"./memo_trait_operation_method_body_resolver\" as *",
    ],
    "method body fact producer must depend on typed HIR root, type ID, operation kind, effect checker, and resolver constructor",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_drop_impl_resolver)/,
    "method body fact producer must not import Resource IR, backend, proof store, artifact, public-surface, impl-table, purity gate, body check resolver, or Drop resolver layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationMethodBodyFactProducerErrorKind:",
        "EffectCheckRejected %SelfhostMemoTraitOperationMethodBodyEffectCheckerErrorKind",
        "FactRejected %SelfhostMemoTraitOperationMethodBodyResolverErrorKind",
        "pub struct SelfhostMemoTraitOperationMethodBodyFactProducerStage0Summary:",
        "pure_eq_fact %Result SelfhostMemoTraitOperationMethodBodyFact SelfhostMemoTraitOperationMethodBodyFactProducerErrorKind",
        "missing_root_rejected %Result SelfhostMemoTraitOperationMethodBodyFact SelfhostMemoTraitOperationMethodBodyFactProducerErrorKind",
    ],
    "producer output and nested errors must be typed payloads",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationMethodBodyFactProducerErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "method body fact producer errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "method body fact producer APIs must return typed Result errors instead of bool/string errors",
);
assert.doesNotMatch(
    code,
    /\b(SelfhostMemoTraitOperationBodyChecks|SelfhostMemoTraitOperationMethodBodyCheck|SelfhostMemoTraitOperationDropCheck|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitOperationMethodBodyEvidence|SelfhostMemoTraitOperationDropEvidence|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|selfhost_memo_trait_operation_method_body_check_new|selfhost_memo_trait_operation_drop_check_new|selfhost_memo_trait_operation_body_check_resolve_result|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_operation_method_body_evidence_new|selfhost_memo_trait_operation_drop_evidence_new|selfhost_memo_trait_aggregate_proof_to_record)/,
    "method body fact producer must not construct body check pairs, Drop checks, operation evidence records, aggregate proof, or proof-store values",
);
assert.doesNotMatch(
    code,
    /^\s+SelfhostMemoTraitOperationMethodBodyFact\s+/m,
    "method body fact producer must not bypass the resolver fact constructor with a direct method-body fact struct expression",
);
assert.doesNotMatch(
    code,
    /\b(?:selfhost_memo_trait_operation_method_body_table_push|selfhost_memo_trait_operation_method_body_resolve_result|SelfhostMemoTraitOperationMethodBodyTable|SelfhostMemoTraitOperationMethodBodySurfaceState)\b/,
    "method body fact producer must not consume table owner, run resolver lookup, or choose surface completeness",
);
assert.doesNotMatch(
    code,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\n]*"(?:name|span|source|path|diagnostic|message|text)")/,
    "accepted method body fact must not use display name, expression span, source text, path, or diagnostic text as authority",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_from_summary_result"),
    [
        "selfhost_memo_trait_operation_method_body_fact_new_result type_id operation summary.effect summary.escape",
        "Result::Ok fact:",
        "Result::Ok fact",
        "Result::Err e:",
        "Result::Err selfhost_memo_trait_operation_method_body_fact_producer_fact_error e",
    ],
    "summary-to-fact API must delegate operation matrix and validation to resolver constructor",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_from_hir_root_result"),
    [
        "selfhost_memo_trait_operation_method_body_effect_check_result module root fuel",
        "Result::Ok summary:",
        "selfhost_memo_trait_operation_method_body_fact_from_summary_result type_id operation summary",
        "Result::Err e:",
        "Result::Err selfhost_memo_trait_operation_method_body_fact_producer_effect_error e",
    ],
    "HIR-root API must run effect checker first and preserve effect checker errors separately",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_producer_error_kind_eq"),
    [
        "EffectCheckRejected a_effect:",
        "selfhost_memo_trait_operation_method_body_effect_checker_error_kind_eq a_effect b_effect",
        "FactRejected a_fact:",
        "selfhost_memo_trait_operation_method_body_resolver_error_kind_eq a_fact b_fact",
    ],
    "producer error equality must compare nested payloads explicitly",
);
assert.ok(
    source.includes("wildcard arm は使いません。error variant が増えた場合はこの equality 境界を明示的に更新します"),
    "producer error equality docs must explicitly forbid wildcard arms",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_producer_fact_error_result_eq"),
    [
        "SelfhostMemoTraitOperationMethodBodyFactProducerErrorKind::EffectCheckRejected _effect_error:",
        "false",
        "SelfhostMemoTraitOperationMethodBodyFactProducerErrorKind::FactRejected fact_error:",
        "selfhost_memo_trait_operation_method_body_resolver_error_kind_eq fact_error expected",
    ],
    "fact-error helper must inspect the nested resolver error payload",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_producer_effect_error_result_eq"),
    [
        "SelfhostMemoTraitOperationMethodBodyFactProducerErrorKind::EffectCheckRejected effect_error:",
        "selfhost_memo_trait_operation_method_body_effect_checker_error_kind_eq effect_error expected",
        "SelfhostMemoTraitOperationMethodBodyFactProducerErrorKind::FactRejected _fact_error:",
        "false",
    ],
    "effect-error helper must inspect the nested effect checker error payload",
);
assert.doesNotMatch(
    code,
    /\b(?:line[_-]?count|doc(?:ument)?[_-]?comment[_-]?(?:length|limit|max)|max[_-]?lines|too[_-]?long|LOC|locLimit|lineLimit)\b/i,
    "method body fact producer policy must not introduce line-count or doc-comment-length caps",
);
assert.doesNotMatch(
    code,
    /\bunwrap_ok\b|\bunwrap_err\b|unreachable/,
    "method body fact producer implementation and smoke helpers must not use unwrap/unreachable shortcuts",
);

console.log("selfhost memo trait operation method body fact producer contract passed");
