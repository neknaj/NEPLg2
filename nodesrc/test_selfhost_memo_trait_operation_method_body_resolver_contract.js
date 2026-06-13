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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_method_body_resolver.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_method_body_resolver",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "method body resolver must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("`Eq` / `Hash` は method body purity を必要とし") &&
        source.includes("`Copy` / `Drop` は method body を必要としない"),
    "docs must fix the operation requirement matrix",
);
assert.ok(
    source.includes("`Complete` surface で `Eq` / `Hash` の候補が 0 件の場合は `Missing` check") &&
        source.includes("complete でない lookup miss を成功扱いしません"),
    "docs must preserve missing and unknown instead of treating lookup miss as pure success",
);
assert.ok(
    source.includes("record order による first-wins は使いません") &&
        source.includes("`selfhost_memo_trait_operation_method_body_fact_new_result` は `Copy` / `Drop` fact を拒否します"),
    "docs must reject first-wins duplicate handling and invalid method-body operations",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、HIR、Resource IR、backend artifact、proof store record を authority にしません"),
    "docs must exclude source/display/diagnostic/module path/HIR/Resource/backend/proof-store authority",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_method_body_resolver/,
    "method body resolver must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_method_body_resolver/,
    "checker-layer method body resolver must not be registered in the ty source list",
);
assertOrdered(
    source,
    [
        "#import \"alloc/collections/vec\" as v",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_purity_gate\" as *",
    ],
    "method body resolver must depend only on Vec storage, typed effect facts, TypeId, operation kind, and purity gate check type",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_operation_drop_impl_resolver)/,
    "method body resolver must not import HIR, Resource IR, backend, proof store, artifact, canonical-key, public-surface, public-impl-header, producer, impl table, or Drop resolver layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationMethodBodySurfaceState:",
        "Complete",
        "Missing",
        "Unknown",
        "pub struct SelfhostMemoTraitOperationMethodBodyFact:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
    ],
    "surface completeness and method body fact must be typed values",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationMethodBodyResolverErrorKind:",
        "TableAllocFailed %StdErrorKind",
        "RecordPushFailed %StdErrorKind",
        "RecordReadFailed %i32",
        "RecordDuplicate",
        "UnexpectedMethodOperation",
    ],
    "method body resolver errors must be typed and payload-carrying where needed",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationMethodBodyResolverErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "method body resolver errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "method body resolver APIs must return typed Result errors instead of bool/string errors",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_operation_required"),
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
    "operation requirement matrix must require method bodies only for Eq and Hash",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_fact_new_result"),
    [
        "selfhost_memo_trait_operation_method_body_operation_required operation",
        "Result::Ok selfhost_memo_trait_operation_method_body_fact_new_unchecked type_id operation effect escape",
        "Result::Err SelfhostMemoTraitOperationMethodBodyResolverErrorKind::UnexpectedMethodOperation",
    ],
    "fact constructor result must reject Copy and Drop method body facts",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_resolve_result"),
    [
        "selfhost_memo_trait_operation_method_body_operation_required operation",
        "SelfhostMemoTraitOperationMethodBodySurfaceState::Complete:",
        "selfhost_memo_trait_operation_method_body_find_loop table type_id operation 0 none",
        "SelfhostMemoTraitOperationMethodBodySurfaceState::Missing:",
        "Result::Ok selfhost_memo_trait_operation_method_body_missing_check",
        "SelfhostMemoTraitOperationMethodBodySurfaceState::Unknown:",
        "Result::Ok selfhost_memo_trait_operation_method_body_unknown_check",
        "Result::Ok selfhost_memo_trait_operation_method_body_not_required_check",
    ],
    "resolver must scan complete surfaces only for required method operations and must return NotRequired for Copy/Drop",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_find_finish"),
    [
        "Option::Some check:",
        "Result::Ok check",
        "Option::None:",
        "Result::Ok selfhost_memo_trait_operation_method_body_missing_check",
    ],
    "complete-surface lookup miss must become Missing and not Present",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_find_loop"),
    [
        "selfhost_memo_trait_operation_method_body_fact_matches fact type_id operation",
        "Option::Some _existing:",
        "Result::Err SelfhostMemoTraitOperationMethodBodyResolverErrorKind::RecordDuplicate",
        "Option::None:",
        "selfhost_memo_trait_operation_method_body_find_loop table type_id operation add idx 1 some check",
        "Option::None:",
        "Result::Err SelfhostMemoTraitOperationMethodBodyResolverErrorKind::RecordReadFailed idx",
    ],
    "lookup must reject duplicate matching facts and fail on impossible Vec read failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_table_push"),
    [
        "Result::Err e:",
        "let error %StdErrorKind field::get e \"error\"",
        "v::free v::vec_push_error_vec e",
        "Result::Err SelfhostMemoTraitOperationMethodBodyResolverErrorKind::RecordPushFailed error",
    ],
    "table push must recover and free the owner Vec returned by a failed push",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_method_body_resolver_error_kind_eq"),
    [
        "TableAllocFailed a_alloc:",
        "RecordPushFailed a_push:",
        "RecordReadFailed a_idx:",
        "eq a_idx b_idx",
        "RecordDuplicate:",
        "UnexpectedMethodOperation:",
    ],
    "error equality must be exhaustive and compare payloads",
);
assert.ok(
    source.includes("wildcard arm は使いません。error variant が増えた場合はこの equality 境界を明示的に更新します"),
    "error equality docs must explicitly forbid wildcard arms",
);
assert.doesNotMatch(
    code,
    /source_text|source_span|\bspan\b|lexeme|display_name|diagnostic|module_path|file_path|path_suffix|payload_hash|signature_hash|body_hash/,
    "method body resolver code must not use source text, spans, lexemes, display names, diagnostics, module paths, or hashes as evidence authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "method body resolver policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation method body resolver contract passed");
