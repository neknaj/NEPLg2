#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/codegen/memo_call_backend_request_table.nepl";
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
        "# codegen/memo_call_backend_request_table",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "request table module must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("通常の `FnValue`、literal、変数参照などは backend cache materialization を要求しないので、error ではなく無視します") &&
        source.includes("PrivateCache / PrivateState、Resource IR proof、proof store、prechecked artifact"),
    "docs must distinguish non-memo ignore behavior from invalid memo fail-closed and forbid private cache/proof/backend work",
);
assert.ok(
    runner.includes('"nodesrc/test_selfhost_memo_call_backend_request_table_contract.js"'),
    "new request table source policy test must be registered in run_source_policy_regressions.js",
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
    ],
    "request table imports must stay at Vec, HIR, identity, effect, type, and request manifest layers",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|proof|memo_trait|PrivateCache|PrivateState|prechecked|wasm|llvm|lower|check\/expr|compiler_known|artifact|serializer|reader)/,
    "request table collector must not import Resource IR, proof store, memo trait proof layers, private cache, prechecked artifact, backend bytes, lower/hir, checker, compiler-known registry, or artifact IO",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoCallBackendRequestTableEntry:",
        "memoized_expr_id %SelfhostHirExprId",
        "request %SelfhostMemoCallBackendRequest",
        "pub struct SelfhostMemoCallBackendRequestTable:",
        "requests %Vec SelfhostMemoCallBackendRequestTableEntry",
    ],
    "request table must be a Vec-backed owner table of occurrence-tagged typed request manifests",
);
const tableBlock = topLevelBlock(source, "struct", "SelfhostMemoCallBackendRequestTable");
assert.doesNotMatch(
    source.slice(source.indexOf(tableBlock), source.indexOf("pub enum SelfhostMemoCallBackendRequestCollectorErrorKind:")),
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendRequestTable/,
    "request table owner must not implement Clone or Copy",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoCallBackendRequestCollectorErrorKind:",
        "RequestTableAllocFailed %StdErrorKind",
        "RequestPushFailed %StdErrorKind",
        "RootExpressionMissing %SelfhostHirExprId",
        "ChildRangeInvalid %SelfhostHirRangeBuildError",
        "ChildIdMissing %i32",
        "ChildExpressionMissing %SelfhostHirExprId",
        "InvalidHirExpr %SelfhostHirExprId",
        "RequestRejected %SelfhostMemoCallBackendRequestRejection",
        "TraversalFuelExhausted %SelfhostHirExprId",
    ],
    "collector error taxonomy must be typed and preserve lower request error kind",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_collector_error_code"),
    /_:/,
    "collector error code helper must not use wildcard fallback",
);
const exprCollector = topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_table_collect_existing_expr_result");
assertOrdered(
    exprCollector,
    [
        "le state.remaining_fuel 0",
        "TraversalFuelExhausted expr_id",
        "let next_fuel %i32 sub state.remaining_fuel 1",
        "match *field::get_ref &expr \"payload\":",
        "SelfhostHirExprPayload::Error:",
        "InvalidHirExpr expr_id",
        "SelfhostHirExprPayload::FnValue _identity:",
        "Result::Ok next_state",
        "SelfhostHirExprPayload::MemoizedFunctionValue _identity:",
        "selfhost_memo_call_backend_request_from_hir_expr_result &expr",
        "selfhost_memo_call_backend_request_table_push request_table expr_id request",
        "RequestRejected SelfhostMemoCallBackendRequestRejection expr_id request_error.kind",
        "SelfhostHirExprPayload::Call call:",
        "selfhost_memo_call_backend_request_table_collect_child_range_result module next_state call.args",
        "SelfhostHirExprPayload::Block children:",
        "SelfhostHirExprPayload::If branches:",
    ],
    "expr collector must consume global fuel, fail on HIR Error payload, ignore ordinary non-memo leaves, accept only memoized payload through request builder, and traverse child ranges",
);
assert.doesNotMatch(
    exprCollector.replace(/SelfhostHirExprPayload::MemoizedFunctionValue[\s\S]*?SelfhostHirExprPayload::Call/, ""),
    /selfhost_memo_call_backend_request_from_hir_expr_result/,
    "request builder must only be called from the MemoizedFunctionValue branch",
);
const childValidator = topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_table_validate_child_range_result");
assertOrdered(
    childValidator,
    [
        "selfhost_hir_child_range_first children",
        "selfhost_hir_child_range_count children",
        "selfhost_hir_module_child_len module",
        "selfhost_hir_child_range_new_bounded_result first_child child_count child_table_len",
        "ChildRangeInvalid e",
    ],
    "child range validator must use bounded HIR range validation before child iteration",
);
const childLoop = topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_table_collect_child_range_loop_result");
assertOrdered(
    childLoop,
    [
        "match selfhost_hir_module_get_child module children idx:",
        "Option::Some child_expr_id:",
        "selfhost_memo_call_backend_request_table_collect_child_expr_result module state child_expr_id",
        "Option::None:",
        "ChildIdMissing idx",
    ],
    "child range loop must use HIR module child accessor and fail-closed on missing child id",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_table_collect_root_expr_result"),
    [
        "match selfhost_hir_module_get_expr module expr_id:",
        "Option::Some expr:",
        "selfhost_memo_call_backend_request_table_collect_existing_expr_result module state expr_id expr",
        "Option::None:",
        "RootExpressionMissing expr_id",
    ],
    "root collector must distinguish missing root expression",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_table_collect_child_expr_result"),
    [
        "match selfhost_hir_module_get_expr module expr_id:",
        "Option::Some expr:",
        "selfhost_memo_call_backend_request_table_collect_existing_expr_result module state expr_id expr",
        "Option::None:",
        "ChildExpressionMissing expr_id",
    ],
    "child collector must distinguish missing child expression",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_table_from_hir_root_result"),
    [
        "match selfhost_memo_call_backend_request_table_new:",
        "Result::Ok table:",
        "SelfhostMemoCallBackendRequestTraversalState table fuel",
        "selfhost_memo_call_backend_request_table_collect_root_expr_result module state root",
        "Result::Ok field::get final_state \"table\"",
    ],
    "public root collector must allocate an owner table, attach global traversal fuel, and return the final table owner",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_table_push"),
    [
        "SelfhostMemoCallBackendRequestTableEntry expr_id request",
        "field::get table \"requests\"",
        "match v::push requests entry:",
        "Result::Ok next_requests:",
        "Result::Ok SelfhostMemoCallBackendRequestTable next_requests",
        "Result::Err e:",
        "let error %StdErrorKind field::get e \"error\"",
        "v::free v::vec_push_error_vec e",
        "RequestPushFailed error",
    ],
    "request table push must preserve owner cleanup and lower StdErrorKind",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_table_push"),
    /^pub\s+fn/m,
    "request table push must stay private so downstream cannot forge accepted request entries without HIR traversal",
);
assert.doesNotMatch(
    code,
    /string_search::str_eq|candidate\.name|diagnostic_symbol|diagnostic_span|memo_call"/,
    "request table collection must not use a memo_call name allow-list, candidate display name, diagnostic symbol, or diagnostic span as authority",
);
assert.doesNotMatch(
    source,
    /line[_-]?count|doc(?:umentation)?[_-]?comment(?:s)?[^\n]*(?:limit|cap|max)|max[_-]?(?:lines|doc)/i,
    "request table source policy must not introduce line-count or doc-comment length limits",
);

console.log("selfhost memo_call backend request table contract ok");
