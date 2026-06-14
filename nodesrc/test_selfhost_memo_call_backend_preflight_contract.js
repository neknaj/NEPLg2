#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/codegen/memo_call_backend_preflight.nepl";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";
const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
const runner = fs.readFileSync(path.join(repoRoot, runnerRelPath), "utf8").replace(/\r\n/g, "\n");

function stripDocComments(src) {
    return src
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const declaration = kind === "fn"
        ? new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s`)
        : new RegExp(`^(?:pub\\s+)?${kind}\\s+${name}:`);
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

const code = stripDocComments(source);

assertOrdered(
    source,
    [
        "# codegen/memo_call_backend_preflight",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "preflight module must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("public table を直接 authority にせず") &&
        source.includes("PrivateCache proof boundary") &&
        source.includes("typed error で fail-closed"),
    "docs must state that public request tables are not authority and non-empty requests fail closed until proof boundary exists",
);
assert.ok(
    runner.includes('"nodesrc/test_selfhost_memo_call_backend_preflight_contract.js"'),
    "new preflight source policy test must be registered in run_source_policy_regressions.js",
);
assertOrdered(
    source,
    [
        "#import \"alloc/collections/vec\" as v",
        "#import \"core/field\" as field",
        "#import \"core/math\" as *",
        "#import \"core/option\" as *",
        "#import \"core/result\" as *",
        "#import \"core/traits/copy\" as *",
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/infra/span\" as *",
        "#import \"neplg2/core/resolve/name_resolver\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty\" as *",
        "#import \"./memo_call_backend_request\" as *",
        "#import \"./memo_call_backend_request_table\" as *",
    ],
    "preflight imports must stay at Vec, HIR, identity, effect, type, request manifest, and request table layers",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|proof|memo_trait|PrivateCache|PrivateState|prechecked|wasm|llvm|lower|check\/expr|compiler_known|artifact|serializer|reader|neplobj|neplproof)/,
    "preflight must not import Resource IR, proof store, memo trait proof layers, private cache/state implementation, prechecked artifacts, backend bytes, checker, compiler-known registry, or artifact IO",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+.*SelfhostMemoCallBackendRequestTable/m,
    "preflight must not expose a public API that accepts a caller-supplied request table as authority",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoCallBackendPreflightErrorKind:",
        "RequestCollectionFailed %SelfhostMemoCallBackendRequestCollectorErrorKind",
        "RequestEntryMissing %i32",
        "RequestExpressionMissing %SelfhostHirExprId",
        "RequestRecheckRejected %SelfhostMemoCallBackendRequestRejection",
        "RequestIdentityMismatch %SelfhostHirExprId",
        "PrivateCacheProofUnavailable %SelfhostHirExprId",
        "StableArtifactKeyUnavailable %SelfhostHirExprId",
        "Stage0FixtureAllocFailed %StdErrorKind",
    ],
    "preflight error taxonomy must keep collection errors, HIR recheck errors, proof absence, stable key absence, and fixture allocation distinct",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_preflight_error_code"),
    /_:/,
    "preflight error code helper must not use wildcard fallback",
);
const recheckEntry = topLevelBlock(source, "fn", "selfhost_memo_call_backend_preflight_recheck_entry_result");
assertOrdered(
    recheckEntry,
    [
        "selfhost_hir_module_get_expr module entry.memoized_expr_id",
        "selfhost_memo_call_backend_request_from_hir_expr_result &expr",
        "selfhost_memo_call_backend_preflight_request_record_matches &entry rebuilt",
        "RequestIdentityMismatch entry.memoized_expr_id",
        "RequestRecheckRejected SelfhostMemoCallBackendRequestRejection entry.memoized_expr_id request_error.kind",
        "RequestExpressionMissing entry.memoized_expr_id",
    ],
    "preflight must re-read HIR payload for each request entry and re-run the typed request builder",
);
const recordMatch = topLevelBlock(source, "fn", "selfhost_memo_call_backend_preflight_request_record_matches");
assertOrdered(
    recordMatch,
    [
        "stored.request_kind",
        "rebuilt.request_kind",
        "selfhost_def_id_eq stored.source_function_def_id rebuilt.source_function_def_id",
        "selfhost_type_id_eq stored.function_ty rebuilt.function_ty",
        "selfhost_effect_kind_eq stored.source_effect SelfhostEffectKind::Pure",
        "selfhost_effect_kind_eq rebuilt.source_effect SelfhostEffectKind::Pure",
        "eq stored.type_arg_count 0",
        "eq rebuilt.type_arg_count 0",
    ],
    "preflight recheck must compare typed request fields instead of diagnostic metadata",
);
const publicEntrypoint = topLevelBlock(source, "fn", "selfhost_memo_call_backend_preflight_from_hir_root_result");
assertOrdered(
    publicEntrypoint,
    [
        "selfhost_memo_call_backend_request_table_from_hir_root_result module root fuel",
        "Result::Ok table:",
        "selfhost_memo_call_backend_preflight_from_collected_table_result module table",
        "Result::Err e:",
        "RequestCollectionFailed e",
    ],
    "public preflight must build the request table internally from HIR root and wrap collector errors",
);
const collected = topLevelBlock(source, "fn", "selfhost_memo_call_backend_preflight_from_collected_table_result");
assertOrdered(
    collected,
    [
        "selfhost_memo_call_backend_request_table_len &table",
        "selfhost_memo_call_backend_preflight_recheck_loop_result module &table 0 request_count",
        "eq request_count 0",
        "selfhost_memo_call_backend_request_table_free table",
        "Result::Ok SelfhostMemoCallBackendPreflightSummary request_count",
        "selfhost_memo_call_backend_preflight_block_non_empty_result &table",
        "selfhost_memo_call_backend_request_table_free table",
        "Result::Err e:",
        "selfhost_memo_call_backend_request_table_free table",
    ],
    "collected-table helper must recheck all entries and close the owner table on success, blocked non-empty request, and recheck error",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_preflight_block_non_empty_result"),
    [
        "selfhost_memo_call_backend_request_table_get_entry table 0",
        "PrivateCacheProofUnavailable first_entry.memoized_expr_id",
        "RequestEntryMissing 0",
    ],
    "non-empty request table must fail closed with private-cache proof unavailable instead of returning an accepted plan",
);
assert.doesNotMatch(
    code,
    /ProofDeferred|Deferred|ProofReady|StableKeyReady|Result::Ok\s+SelfhostMemoCallBackendPreflightSummary\s+(?!request_count)|Result::Ok\s+.*PrivateCacheProofUnavailable/,
    "preflight must not model missing proof as a successful ready/deferred plan",
);
assert.doesNotMatch(
    code,
    /diagnostic_symbol|diagnostic_span|string_search::str_eq|candidate\.name|memo_call"/,
    "preflight must not use display symbol, diagnostic span, candidate name, string search, or memo_call string literal as authority",
);
assert.doesNotMatch(
    source,
    /line[_-]?count|doc(?:umentation)?[_-]?comment(?:s)?[^\n]*(?:limit|cap|max)|max[_-]?(?:lines|doc)/i,
    "preflight source policy must not introduce line-count or doc-comment length limits",
);

console.log("selfhost memo_call backend preflight contract ok");
