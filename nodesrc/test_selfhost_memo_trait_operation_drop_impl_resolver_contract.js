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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_drop_impl_resolver.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_drop_impl_resolver",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "Drop impl resolver must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("`Complete` surface で候補が 0 件のときだけ、`DropImplAbsent` を返します") &&
        source.includes("`Missing` surface は `Missing` check、`Unknown` surface は `Unknown` check に畳み、成功済みの no-drop proof にはしません"),
    "docs must forbid mapping incomplete surface lookup misses to DropImplAbsent",
);
assert.ok(
    source.includes("同じ `SelfhostTypeId` の Drop impl fact が 2 件以上ある場合は `RecordDuplicate` として fail-closed に拒否します") &&
        source.includes("record order による first-wins は使いません"),
    "docs must reject duplicate Drop impl facts instead of first-wins",
);
assert.ok(
    source.includes("body module fingerprint、Drop body root id、typed effect kind、typed escape state") &&
        source.includes("HIR payload を走査せず、root id から effect や no-escape を推測しません"),
    "docs must carry body module fingerprint and Drop body root identity while excluding HIR payload traversal as authority",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_drop_impl_resolver/,
    "Drop impl resolver must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_drop_impl_resolver/,
    "checker-layer Drop impl resolver must not be registered in the ty source list",
);
assertOrdered(
    source,
    [
        "#import \"alloc/collections/vec\" as v",
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"./memo_trait_operation_purity_gate\" as *",
    ],
    "Drop impl resolver must depend only on Vec storage, typed effect facts, TypeId, and purity gate check type",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table)/,
    "Drop impl resolver must not import Resource IR, backend, proof store, artifact, canonical-key, public-surface, public-impl-header, evidence-producer, or operation impl table layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationDropImplSurfaceState:",
        "Complete",
        "Missing",
        "Unknown",
        "pub struct SelfhostMemoTraitOperationDropImplFact:",
        "type_id %SelfhostTypeId",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
    ],
    "surface completeness and Drop impl fact must include typed type, module origin, body root, effect, and escape values",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationDropImplResolverErrorKind:",
        "TableAllocFailed %StdErrorKind",
        "RecordPushFailed %StdErrorKind",
        "BodyModuleFingerprintPlaceholder",
        "RecordReadFailed %i32",
        "RecordDuplicate",
    ],
    "Drop impl resolver errors must be typed and payload-carrying where needed",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationDropImplResolverErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "Drop impl resolver errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "Drop impl resolver APIs must return typed Result errors instead of bool/string errors",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_impl_resolve_result"),
    [
        "SelfhostMemoTraitOperationDropImplSurfaceState::Complete:",
        "selfhost_memo_trait_operation_drop_impl_find_loop table type_id 0 none",
        "SelfhostMemoTraitOperationDropImplSurfaceState::Missing:",
        "Result::Ok selfhost_memo_trait_operation_drop_impl_missing_check",
        "SelfhostMemoTraitOperationDropImplSurfaceState::Unknown:",
        "Result::Ok selfhost_memo_trait_operation_drop_impl_unknown_check",
    ],
    "resolver must only scan complete surfaces and must preserve missing/unknown surface state",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_impl_find_finish"),
    [
        "Option::Some check:",
        "Result::Ok check",
        "Option::None:",
        "Result::Ok selfhost_memo_trait_operation_drop_impl_absent_check",
    ],
    "complete-surface lookup miss must be the only path to absent check",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_impl_find_loop"),
    [
        "selfhost_memo_trait_operation_drop_impl_fact_matches fact type_id",
        "Option::Some _existing:",
        "Result::Err SelfhostMemoTraitOperationDropImplResolverErrorKind::RecordDuplicate",
        "Option::None:",
        "selfhost_memo_trait_operation_drop_impl_find_loop table type_id add idx 1 some check",
        "Option::None:",
        "Result::Err SelfhostMemoTraitOperationDropImplResolverErrorKind::RecordReadFailed idx",
    ],
    "lookup must reject duplicate matching facts and must fail on impossible Vec read failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_impl_table_push"),
    [
        "eq fact.body_module_fingerprint 0",
        "selfhost_memo_trait_operation_drop_impl_table_free table",
        "Result::Err SelfhostMemoTraitOperationDropImplResolverErrorKind::BodyModuleFingerprintPlaceholder",
        "Result::Err e:",
        "let error %StdErrorKind field::get e \"error\"",
        "v::free v::vec_push_error_vec e",
        "Result::Err SelfhostMemoTraitOperationDropImplResolverErrorKind::RecordPushFailed error",
    ],
    "table push must recover and free the owner Vec returned by a failed push",
);
assert.doesNotMatch(
    code,
    /source_text|source_span|\bspan\b|lexeme|display_name|diagnostic|module_path|file_path|path_suffix|payload_hash|signature_hash|body_hash/,
    "Drop impl resolver code must not use source text, spans, lexemes, display names, diagnostics, module paths, or hashes as evidence authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "Drop impl resolver policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation Drop impl resolver contract passed");
