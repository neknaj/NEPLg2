#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl";
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
        "# codegen/memo_call_backend_private_cache_proof_gate",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "private-cache proof gate module must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("caller が渡す proof table をそのまま authority にせず") &&
        source.includes("non-executable summary") &&
        source.includes("proof record / proof table / proof table writer / gate 本体は module-private") &&
        source.includes("stable artifact sidecar index"),
    "docs must state that caller proof tables are not direct authority, success is not executable backend output, table writes are private in phase 1, and index optimization is a later contract-preserving change",
);
assert.ok(
    runner.includes('"nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js"'),
    "private-cache proof gate source policy test must be registered in run_source_policy_regressions.js",
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
    "private-cache proof gate imports must stay at Vec, HIR, identity, effect, type, request manifest, and request table layers",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|proof\/|memo_trait|PrivateCache|PrivateState|prechecked|wasm|llvm|lower|check\/expr|compiler_known|artifact|serializer|reader|neplobj|neplproof)/,
    "private-cache proof gate must not import Resource IR, proof store, memo trait proof layers, private cache/state implementation, prechecked artifacts, backend bytes, checker, compiler-known registry, or artifact IO",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+.*SelfhostMemoCallBackendRequestTable/m,
    "private-cache proof gate must not expose a public API that accepts a caller-supplied request table as authority",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoCallBackendPrivateCacheProofKey:",
        "memoized_expr_id %SelfhostHirExprId",
        "source_function_def_id %SelfhostDefId",
        "function_ty %SelfhostTypeId",
        "root_expr_id %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "request_kind %SelfhostMemoCallBackendRequestKind",
        "source_effect %SelfhostEffectKind",
        "type_arg_count %i32",
        "proof_kind %SelfhostMemoCallBackendPrivateCacheProofKind",
        "proof_schema_version %i32",
        "struct SelfhostMemoCallBackendPrivateCacheProofTable:",
        "records %Vec SelfhostMemoCallBackendPrivateCacheProofRecord",
    ],
    "private-cache proof key must bind memoized occurrence, source def, function type, root expr, body module fingerprint, request kind, effect, type arg count, proof kind, and schema, and proof table must be Vec-backed",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheProof(?:Status|Record|Table)\b/,
    "request evidence status, record, and table must stay private until a producer-owned Resource proof boundary exists",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_proof_(?:key_new|record_new|table_new|table_free|table_len)\b/m,
    "proof key/table constructors and owner operations must not be public accepted-path building blocks",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_proof_gate_from_hir_root_result\b/m,
    "request-evidence gate must stay module-private until a producer-owned Resource proof boundary owns the proof table",
);
const tableArea = source.slice(
    source.indexOf("struct SelfhostMemoCallBackendPrivateCacheProofTable:"),
    source.indexOf("pub struct SelfhostMemoCallBackendPrivateCacheProofGateSummary:"),
);
assert.doesNotMatch(
    tableArea,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheProofTable/,
    "private-cache proof table owner must not implement Clone or Copy",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoCallBackendPrivateCacheProofGateErrorKind:",
        "RequestCollectionFailed %SelfhostMemoCallBackendRequestCollectorErrorKind",
        "RequestEntryMissing %i32",
        "RequestExpressionMissing %SelfhostHirExprId",
        "RequestRecheckRejected %SelfhostMemoCallBackendRequestRejection",
        "RequestIdentityMismatch %SelfhostHirExprId",
        "BodyModuleFingerprintPlaceholder",
        "ProofTableAllocFailed %StdErrorKind",
        "ProofRecordPushFailed %StdErrorKind",
        "ProofRecordReadFailed %i32",
        "ProofMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ProofRefuted %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ProofDuplicate %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ProofOrphan %SelfhostMemoCallBackendPrivateCacheProofKey",
        "Stage0FixtureAllocFailed %StdErrorKind",
    ],
    "private-cache proof gate error taxonomy must keep collection, HIR recheck, placeholder fingerprint, proof table, missing/refuted/duplicate/orphan, and fixture failures distinct",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_error_code"),
    /_:/,
    "private-cache proof gate error code helper must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_request_table_from_hir_root_result module root fuel",
        "Result::Ok table:",
        "selfhost_memo_call_backend_private_cache_proof_gate_from_collected_table_result module table root body_module_fingerprint proofs",
        "Result::Err e:",
        "RequestCollectionFailed e",
    ],
    "public gate must build the request table internally from HIR root and wrap collector errors",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_from_collected_table_result"),
    [
        "selfhost_memo_call_backend_request_table_len &table",
        "selfhost_memo_call_backend_private_cache_proof_gate_recheck_loop_result module &table 0 request_count",
        "selfhost_memo_call_backend_private_cache_proof_gate_apply_loop_result &table proofs root_expr_id body_module_fingerprint 0 request_count",
        "selfhost_memo_call_backend_request_table_free table",
        "Result::Ok SelfhostMemoCallBackendPrivateCacheProofGateSummary request_count proven_count",
        "Result::Err e:",
        "selfhost_memo_call_backend_request_table_free table",
    ],
    "collected-table helper must recheck entries, apply proof records, and close the internal request table on success and errors",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_recheck_entry_result"),
    [
        "selfhost_hir_module_get_expr module entry.memoized_expr_id",
        "selfhost_memo_call_backend_request_from_hir_expr_result &expr",
        "selfhost_memo_call_backend_private_cache_proof_gate_request_record_matches &entry rebuilt",
        "RequestIdentityMismatch entry.memoized_expr_id",
        "RequestRecheckRejected SelfhostMemoCallBackendRequestRejection entry.memoized_expr_id request_error.kind",
        "RequestExpressionMissing entry.memoized_expr_id",
    ],
    "private-cache proof gate must re-read HIR payload for each request entry and re-run the typed request builder",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_request_record_matches"),
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
    "private-cache proof gate recheck must compare typed request fields instead of diagnostic metadata",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_key_eq"),
    [
        "a.memoized_expr_id",
        "b.memoized_expr_id",
        "selfhost_def_id_eq a.source_function_def_id b.source_function_def_id",
        "selfhost_type_id_eq a.function_ty b.function_ty",
        "a.root_expr_id",
        "b.root_expr_id",
        "eq a.body_module_fingerprint b.body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_request_kind_eq a.request_kind b.request_kind",
        "selfhost_effect_kind_eq a.source_effect b.source_effect",
        "eq a.type_arg_count b.type_arg_count",
        "selfhost_memo_call_backend_private_cache_proof_kind_eq a.proof_kind b.proof_kind",
        "eq a.proof_schema_version b.proof_schema_version",
    ],
    "proof key equality must use occurrence, source def, function type, root expr, body module fingerprint, request kind, source effect, type arg count, proof kind, and schema version",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_key_from_entry_result"),
    [
        "eq body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "selfhost_memo_call_backend_private_cache_proof_key_new entry.memoized_expr_id entry.request.source_function_def_id entry.request.function_ty root_expr_id body_module_fingerprint entry.request.request_kind entry.request.source_effect entry.request.type_arg_count",
    ],
    "proof key construction must reject placeholder body module fingerprints and derive key material from request entry, request kind/effect/type arguments, and root origin",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_lookup_loop"),
    [
        "selfhost_memo_call_backend_private_cache_proof_table_len proofs",
        "selfhost_memo_call_backend_private_cache_proof_lookup_finish key found",
        "field::get_ref proofs \"records\"",
        "v::get records idx",
        "selfhost_memo_call_backend_private_cache_proof_key_eq record.key key",
        "ProofDuplicate key",
        "some record.status",
        "ProofRecordReadFailed idx",
    ],
    "proof lookup must scan records by exact key, reject duplicate matching records, and fail on missing table entry",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_status_result"),
    [
        "SelfhostMemoCallBackendPrivateCacheProofStatus::RequestEvidenceProven:",
        "Result::Ok unit",
        "SelfhostMemoCallBackendPrivateCacheProofStatus::RequestEvidenceRefuted:",
        "ProofRefuted key",
    ],
    "proof status fold must accept only request-evidence Proven and reject request-evidence Refuted",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_table_push"),
    /^pub\s+fn/m,
    "private-cache proof table push must stay private until a producer-owned Resource proof boundary exists",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_validate_orphan_loop"),
    [
        "v::get records idx",
        "selfhost_memo_call_backend_private_cache_proof_gate_request_table_contains_key_loop table root_expr_id body_module_fingerprint record.key 0 request_count",
        "Result::Ok found:",
        "found",
        "ProofOrphan record.key",
        "ProofRecordReadFailed idx",
    ],
    "proof gate must reject proof records that do not correspond to a request entry from the current root",
);
assert.doesNotMatch(
    code,
    /ProofDeferred|Deferred|ProofReady|StableKeyReady|Result::Ok\s+.*ProofMissing|Result::Ok\s+.*ProofRefuted|PrivateCacheProofUnavailable/,
    "private-cache proof gate must not model missing/refuted proof as a successful ready/deferred plan or reuse the preflight proof-unavailable error",
);
assert.doesNotMatch(
    code,
    /diagnostic_symbol|diagnostic_span|string_search::str_eq|candidate\.name|memo_call"/,
    "private-cache proof gate must not use display symbol, diagnostic span, candidate name, string search, or memo_call string literal as authority",
);
assert.doesNotMatch(
    code,
    /cache_lookup|cache_insert|CacheAlloc|CacheDrop|Wasm|LLVM|wasm_|llvm_|sealed|backend_bytes|neplobj|neplproof/,
    "private-cache proof gate must not create executable cache operations, backend bytes, sealed representation, or persistent artifact IO",
);
assert.doesNotMatch(
    source,
    /line[_-]?count|doc(?:umentation)?[_-]?comment(?:s)?[^\n]*(?:limit|cap|max)|max[_-]?(?:lines|doc)/i,
    "private-cache proof gate source policy must not introduce line-count or doc-comment length limits",
);

console.log("selfhost memo_call backend private cache proof gate contract ok");
