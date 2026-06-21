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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_method_body_effect_checker.nepl";
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
        "# check/module/memo_trait_operation_method_body_effect_checker",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "method body effect checker must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("method body fact は、この summary を受け取る後続境界") &&
        source.includes("この module は fact table、operation evidence、aggregate proof status を作りません"),
    "docs must keep HIR effect summary generation separate from fact/evidence/proof construction",
);
assert.ok(
    source.includes("`InternalAlloc` はこの module では `Pure` に mask しません") &&
        source.includes("Resource IR no-escape proof がない call payload 由来の `InternalAlloc` は `NotApplicable` escape のまま残し"),
    "docs must not allow this checker to mask InternalAlloc without Resource IR no-escape proof",
);
assert.ok(
    source.includes("同じ subtree が複数箇所から参照される場合の memoization") &&
        source.includes("後から追加できる最適化"),
    "docs must separate later traversal optimization from current semantic boundary",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_method_body_effect_checker/,
    "method body effect checker must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_method_body_effect_checker/,
    "checker-layer method body effect checker must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_method_body_effect_checker_contract.js"),
    "source policy runner must execute the method body effect checker contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
    ],
    "method body effect checker must consume typed HIR and typed effect IDs",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_operation_purity_gate|memo_trait_operation_method_body_resolver|memo_trait_operation_body_check_resolver|memo_trait_operation_drop_impl_resolver)/,
    "method body effect checker must not import Resource IR, backend, proof store, artifact, public-surface, impl-table, purity gate, body resolver, body check resolver, or Drop resolver layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationMethodBodyEffectSummary:",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
        "pub enum SelfhostMemoTraitOperationMethodBodyEffectCheckerErrorKind:",
        "ExpressionMissing %i32",
        "ChildExpressionMissing %i32",
        "FuelExhausted",
        "ErrorExpressionUnsupported",
    ],
    "checker output and errors must be typed payloads",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationMethodBodyEffectCheckerErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "method body effect checker errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "method body effect checker APIs must return typed Result errors instead of bool/string errors",
);
assert.doesNotMatch(
    code,
    /\b(SelfhostMemoTraitOperationMethodBodyFact|SelfhostMemoTraitOperationBodyChecks|SelfhostMemoTraitOperationMethodBodyCheck|SelfhostMemoTraitOperationDropCheck|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitOperationMethodBodyEvidence|SelfhostMemoTraitOperationDropEvidence|selfhost_memo_trait_operation_method_body_fact_new_result|selfhost_memo_trait_operation_method_body_check_new|selfhost_memo_trait_operation_drop_check_new|selfhost_memo_trait_operation_body_check_resolve_result|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_operation_method_body_evidence_new|selfhost_memo_trait_operation_drop_evidence_new)/,
    "method body effect checker must not construct method body facts, body check pairs, Drop checks, or operation evidence records",
);
assert.doesNotMatch(
    code,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\\n]*\"(?:name|span|source|path|diagnostic|message|text)\")/,
    "accepted effect summary must not use call display name, expression span, source text, path, or diagnostic text as authority",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_call_effect_summary"),
    [
        "SelfhostEffectKind::Pure:",
        "SelfhostEffectKind::InternalAlloc:",
        "SelfhostEffectEscapeState::NotApplicable",
        "SelfhostEffectKind::PrivateState:",
        "SelfhostEffectEscapeState::NotApplicable",
        "SelfhostEffectKind::PrivateCache:",
        "SelfhostEffectEscapeState::NotApplicable",
        "SelfhostEffectKind::UnsafeMemory:",
        "SelfhostEffectKind::ExternalIo:",
        "SelfhostEffectKind::Nondet:",
    ],
    "call effect summary must preserve all current effect variants",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_effect_check_payload_result"),
    [
        "SelfhostHirExprPayload::Error:",
        "Result::Err SelfhostMemoTraitOperationMethodBodyEffectCheckerErrorKind::ErrorExpressionUnsupported",
        "SelfhostHirExprPayload::MemoizedFunctionValue _identity:",
        "Result::Ok selfhost_memo_trait_operation_method_body_effect_summary_pure",
        "SelfhostHirExprPayload::Call call:",
        "selfhost_memo_trait_operation_method_body_call_effect_summary call.effect",
        "selfhost_memo_trait_operation_method_body_effect_fold_children_loop module call.args 0 fuel call_summary",
        "SelfhostHirExprPayload::Block children:",
        "SelfhostHirExprPayload::If branches:",
    ],
    "payload fold must explicitly handle Error, memoized function values, calls, blocks, and if expressions",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_summary_combine"),
    [
        "selfhost_memo_trait_operation_method_body_stronger_effect a.effect b.effect",
        "SelfhostEffectKind::Pure:",
        "SelfhostEffectKind::InternalAlloc:",
        "selfhost_memo_trait_operation_method_body_escape_combine a.escape b.escape",
        "SelfhostEffectKind::PrivateState:",
        "selfhost_memo_trait_operation_method_body_escape_combine a.escape b.escape",
        "SelfhostEffectKind::PrivateCache:",
        "selfhost_memo_trait_operation_method_body_escape_combine a.escape b.escape",
        "SelfhostEffectKind::UnsafeMemory:",
        "SelfhostEffectEscapeState::NotApplicable",
        "SelfhostEffectKind::ExternalIo:",
        "SelfhostEffectEscapeState::NotApplicable",
        "SelfhostEffectKind::Nondet:",
        "SelfhostEffectEscapeState::NotApplicable",
    ],
    "summary combine must only keep escape state for internal/private no-escape effects",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_effect_check_expr_result"),
    [
        "le fuel 0",
        "SelfhostMemoTraitOperationMethodBodyEffectCheckerErrorKind::FuelExhausted",
        "selfhost_hir_module_get_expr module expr_id",
        "SelfhostMemoTraitOperationMethodBodyEffectCheckerErrorKind::ExpressionMissing selfhost_hir_expr_id_index expr_id",
    ],
    "expression checker must fail closed on fuel exhaustion and missing root expression",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_effect_fold_children_loop"),
    [
        "selfhost_hir_module_get_child module children idx",
        "selfhost_memo_trait_operation_method_body_effect_check_expr_result module child_id fuel",
        "selfhost_memo_trait_operation_method_body_summary_combine acc child_summary",
        "SelfhostMemoTraitOperationMethodBodyEffectCheckerErrorKind::ChildExpressionMissing idx",
    ],
    "child fold must read the typed HIR child table and fail closed on malformed ranges",
);
assert.doesNotMatch(
    code,
    /\b(?:line[_-]?count|doc(?:ument)?[_-]?comment[_-]?(?:length|limit|max)|max[_-]?lines|too[_-]?long|LOC|locLimit|lineLimit)\b/i,
    "method body effect checker policy must not introduce line-count or doc-comment-length caps",
);
assert.doesNotMatch(
    code,
    /\bunwrap_ok\b|\bunwrap_err\b|unreachable/,
    "method body effect checker implementation and smoke helpers must not use unwrap/unreachable shortcuts",
);
