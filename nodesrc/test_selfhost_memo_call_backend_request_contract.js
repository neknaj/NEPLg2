#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/codegen/memo_call_backend_request.nepl";
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
        : new RegExp(`^(?:pub\\s+)?${kind}\\s+${name}`);
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
        "# codegen/memo_call_backend_request",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "memo_call backend request module must document purpose, stable contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("codegen / backend はこの leaf を通常の `FnValue` と同じ扱いに戻してはいけません") &&
        source.includes("typed request manifest"),
    "docs must define the request as a typed backend-input manifest, not backend bytes",
);
assert.ok(
    source.includes("diagnostic_symbol` と `diagnostic_span` は backend symbol / diagnostic 用 metadata") &&
        source.includes("accepted 判定、cache namespace、proof authority は `source_function_def_id`、function type、source effect、type argument evidence"),
    "docs must reject symbol/span/name authority for acceptance, proof, and cache namespace",
);
assert.ok(
    runner.includes('"nodesrc/test_selfhost_memo_call_backend_request_contract.js"'),
    "new selfhost source policy test must be registered in run_source_policy_regressions.js",
);
assertOrdered(
    source,
    [
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
    ],
    "request module imports must stay at HIR identity, span, DefId, effect, and type evidence layers",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|proof|memo_trait|PrivateCache|PrivateState|prechecked|wasm|llvm|lower|check\/expr|compiler_known|artifact|serializer|reader)/,
    "request module must not import Resource IR, proof store, memo trait proof layers, private cache, prechecked artifact, backend bytes, lower/hir, checker, or compiler-known primitive layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoCallBackendRequestKind:",
        "MemoCall",
    ],
    "request kind must be a typed enum rather than a string or numeric tag",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoCallBackendRequest:",
        "request_kind %SelfhostMemoCallBackendRequestKind",
        "source_function_def_id %SelfhostDefId",
        "function_ty %SelfhostTypeId",
        "source_effect %SelfhostEffectKind",
        "type_arg_count %i32",
        "diagnostic_symbol %str",
        "diagnostic_span %SelfhostSourceSpan",
    ],
    "request record must keep typed request kind, authority fields, and diagnostic-only metadata with explicit names",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoCallBackendRequestErrorKind:",
        "UnsupportedExprKind",
        "NonMemoizedExpressionUnsupported",
        "FnValueUnsupported",
        "CallUnsupported",
        "MissingDefId",
        "GenericTypeArgumentsUnsupported",
        "ImpureSourceFunction",
        "ExpressionTypeMismatch",
    ],
    "request errors must keep typed fail-closed variants",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_error_kind_eq"),
    /_:/,
    "request error equality must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_from_identity_result"),
    [
        "match identity.def_id:",
        "Option::Some def_id:",
        "not selfhost_hir_function_value_identity_is_monomorphic &identity",
        "GenericTypeArgumentsUnsupported",
        "not selfhost_effect_kind_eq identity.effect SelfhostEffectKind::Pure",
        "ImpureSourceFunction",
        "SelfhostMemoCallBackendRequest SelfhostMemoCallBackendRequestKind::MemoCall def_id identity.function_ty identity.effect identity.type_arg_count identity.symbol span",
        "Option::None:",
        "MissingDefId",
    ],
    "identity request builder must require DefId, monomorphic identity, Pure effect, and copy typed fields into the request",
);
const exprRequest = topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_from_hir_expr_result");
assertOrdered(
    exprRequest,
    [
        "let expr_ty %SelfhostTypeId *field::get_ref expr \"ty\"",
        "match *field::get_ref expr \"payload\":",
        "SelfhostHirExprPayload::Error:",
        "SelfhostHirExprPayload::Unit:",
        "SelfhostHirExprPayload::BoolLiteral _value:",
        "SelfhostHirExprPayload::I32Literal _value:",
        "SelfhostHirExprPayload::F32Literal _value:",
        "SelfhostHirExprPayload::StrLiteral _value:",
        "SelfhostHirExprPayload::Var _identity:",
        "SelfhostHirExprPayload::FnValue _identity:",
        "SelfhostHirExprPayload::MemoizedFunctionValue identity:",
        "not selfhost_type_id_eq expr_ty identity.function_ty",
        "ExpressionTypeMismatch",
        "selfhost_memo_call_backend_request_from_identity_result identity span",
        "SelfhostHirExprPayload::Call _call:",
        "SelfhostHirExprPayload::Block _children:",
        "SelfhostHirExprPayload::If _branches:",
    ],
    "HIR expression request builder must explicitly reject every non-memoized payload and accept only MemoizedFunctionValue",
);
assert.match(
    exprRequest,
    /SelfhostHirExprPayload::FnValue _identity:[\s\S]*FnValueUnsupported/,
    "FnValue must not be accepted as a memo_call backend request",
);
assert.match(
    exprRequest,
    /SelfhostHirExprPayload::Call _call:[\s\S]*CallUnsupported/,
    "Call must not be accepted as a memo_call backend request",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_request_identity_matches"),
    /diagnostic_symbol|diagnostic_span/,
    "request identity matcher must not use diagnostic metadata as authority",
);
assert.doesNotMatch(
    code,
    /string_search::str_eq|candidate\.name|memo_call"/,
    "backend request acceptance must not use a memo_call name allow-list or candidate display name",
);
assert.doesNotMatch(
    source,
    /line[_-]?count|doc(?:umentation)?[_-]?comment(?:s)?[^\\n]*(?:limit|cap|max)|max[_-]?(?:lines|doc)/i,
    "request source policy must not introduce line-count or doc-comment length limits",
);

console.log("selfhost memo_call backend request contract ok");
