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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_drop_impl_fact_table_builder.nepl";
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
        "# check/module/memo_trait_operation_drop_impl_fact_table_builder",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "drop impl fact table builder must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("Drop impl body の typed HIR root を 1 件ずつ effect summary に畳み") &&
        source.includes("resolver が所有する `SelfhostMemoTraitOperationDropImplTable` owner へ安全に push"),
    "docs must define the builder as a Drop HIR root to resolver-owned fact-table boundary",
);
assert.ok(
    source.includes("Resource IR no-escape proof なしに pure Drop として成功扱いされません") &&
        source.includes("Drop evidence、operation evidence record、aggregate proof status、Resource IR no-escape proof"),
    "docs must state that the builder does not synthesize Drop evidence or Resource proof",
);
assert.ok(
    source.includes("effect checker が失敗した場合、table owner はまだ resolver push に渡されていないため") &&
        source.includes("table push が失敗した場合、既存 resolver の `selfhost_memo_trait_operation_drop_impl_table_push`"),
    "docs must spell out effect-error cleanup and table-push owner recovery",
);
assertDocBeforeTopLevel(
    source,
    "Clone for SelfhostMemoTraitOperationDropImplFactTableBuilderErrorKind",
    "impl Clone for SelfhostMemoTraitOperationDropImplFactTableBuilderErrorKind:",
);
assertDocBeforeTopLevel(
    source,
    "Copy for SelfhostMemoTraitOperationDropImplFactTableBuilderErrorKind",
    "impl Copy for SelfhostMemoTraitOperationDropImplFactTableBuilderErrorKind:",
);
assertDocBeforeTopLevel(
    source,
    "Clone for SelfhostMemoTraitOperationDropImplFactTableBuilderStage0Summary",
    "impl Clone for SelfhostMemoTraitOperationDropImplFactTableBuilderStage0Summary:",
);
assertDocBeforeTopLevel(
    source,
    "Copy for SelfhostMemoTraitOperationDropImplFactTableBuilderStage0Summary",
    "impl Copy for SelfhostMemoTraitOperationDropImplFactTableBuilderStage0Summary:",
);
assert.ok(
    source.includes("table owner の所有権を複製しません") &&
        source.includes("owner-bearing payload が必要になった場合、この Copy impl を削除") &&
        source.includes("builder が返した table owner は stage0 helper 内で閉じたあとに値へ畳まれます") &&
        source.includes("free が必要な値を summary field として追加してはいけません"),
    "Clone/Copy impl docs must preserve typed-payload-only and no-owner contracts",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_drop_impl_fact_table_builder/,
    "drop impl fact table builder must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_drop_impl_fact_table_builder/,
    "checker-layer drop impl fact table builder must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_drop_impl_fact_table_builder_contract.js"),
    "source policy runner must execute the drop impl fact table builder contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/infra/span\" as *",
        "#import \"neplg2/core/resolve/name_resolver\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"./memo_trait_operation_drop_impl_resolver\" as *",
        "#import \"./memo_trait_operation_method_body_effect_checker\" as *",
    ],
    "builder must depend only on HIR payload, span fixture helpers, effect/type ids, Drop resolver, and effect checker",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_classifier|memo_trait_operation_public_impl_materializer|memo_trait_public_impl_scanner)/,
    "builder must not import Resource IR, backend, proof store, artifact, canonical-key, public-surface, evidence, impl table, purity gate, classifier, materializer, or scanner layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationDropImplFactTableBuilderErrorKind:",
        "EffectCheckRejected %SelfhostMemoTraitOperationMethodBodyEffectCheckerErrorKind",
        "TableRejected %SelfhostMemoTraitOperationDropImplResolverErrorKind",
        "pub struct SelfhostMemoTraitOperationDropImplFactTableBuilderStage0Summary:",
        "pure_push_len %Result i32 SelfhostMemoTraitOperationDropImplFactTableBuilderErrorKind",
        "external_after_pure_push_len %Result i32 SelfhostMemoTraitOperationDropImplFactTableBuilderErrorKind",
        "internal_alloc_check %Result SelfhostMemoTraitOperationDropCheck SelfhostMemoTraitOperationDropImplFactTableBuilderErrorKind",
    ],
    "builder errors and smoke outputs must keep typed nested payloads",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationDropImplFactTableBuilderErrorKind"),
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
    /\b(SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitOperationEvidenceProducerInput|SelfhostMemoTraitOperationMethodBodyEvidence|SelfhostMemoTraitOperationDropEvidence|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_operation_evidence_producer_input_new|selfhost_memo_trait_operation_method_body_evidence_new|selfhost_memo_trait_operation_drop_evidence_new|selfhost_memo_trait_aggregate_proof_to_record)/,
    "builder must not construct operation evidence, Drop evidence, aggregate proof, or proof-store values",
);
assert.doesNotMatch(
    code,
    /\bSelfhostMemoTraitOperationDropCheckKind::DropImplAbsent\b|\bSelfhostMemoTraitOperationDropEvidence::(?:NoDropRequired|PureDrop)\b/,
    "builder must not synthesize DropImplAbsent, NoDropRequired, or PureDrop",
);
assert.doesNotMatch(
    code,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\n]*"(?:name|span|source|path|diagnostic|message|text)")/,
    "accepted input authority must not use call names, expression spans, source text, path, or diagnostic text",
);
const pushBlock = functionBlock(source, "selfhost_memo_trait_operation_drop_impl_fact_table_builder_push_hir_root_result");
assertOrdered(
    pushBlock,
    [
        "selfhost_memo_trait_operation_method_body_effect_check_result module root fuel",
        "Result::Ok summary:",
        "selfhost_memo_trait_operation_drop_impl_fact_from_summary type_id body_module_fingerprint root summary",
        "selfhost_memo_trait_operation_drop_impl_table_push table fact",
        "Result::Ok next_table:",
        "Result::Ok next_table",
        "Result::Err table_error:",
        "Result::Err selfhost_memo_trait_operation_drop_impl_fact_table_builder_table_error table_error",
        "Result::Err effect_error:",
        "selfhost_memo_trait_operation_drop_impl_table_free table",
        "Result::Err selfhost_memo_trait_operation_drop_impl_fact_table_builder_effect_error effect_error",
    ],
    "push API must run effect checker, convert summary to fact, push through resolver table API, and free table only on effect-check rejection",
);
const tableErrorBranch = before(after(pushBlock, "Result::Err table_error:"), "Result::Err effect_error:");
assert.doesNotMatch(
    tableErrorBranch,
    /selfhost_memo_trait_operation_drop_impl_table_free/,
    "table push rejection branch must not double-free a table owner already consumed by resolver push",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_impl_fact_from_summary"),
    [
        "selfhost_memo_trait_operation_drop_impl_fact_new type_id body_module_fingerprint root summary.effect summary.escape",
    ],
    "summary-to-fact helper must preserve module origin, root, effect, and escape without masking",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_impl_fact_table_builder_error_kind_eq"),
    [
        "EffectCheckRejected a_effect:",
        "selfhost_memo_trait_operation_method_body_effect_checker_error_kind_eq a_effect b_effect",
        "TableRejected a_table:",
        "selfhost_memo_trait_operation_drop_impl_resolver_error_kind_eq a_table b_table",
    ],
    "builder error equality must compare nested effect-checker and resolver payloads",
);
assert.ok(
    functionBlock(source, "selfhost_memo_trait_operation_drop_impl_fact_table_builder_stage0_push_and_resolve").includes(
        "SelfhostMemoTraitOperationDropImplSurfaceState::Complete",
    ),
    "stage0 internal allocation check must resolve a pushed fact through the Drop resolver rather than reading table internals",
);
assert.ok(
    source.includes("SelfhostEffectKind::InternalAlloc") &&
        source.includes("SelfhostEffectEscapeState::NotApplicable"),
    "stage0 must keep InternalAlloc without Resource no-escape masking",
);

console.log("selfhost memo trait operation drop impl fact table builder contract ok");
