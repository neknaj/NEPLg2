#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/syntax/ast/prefix_expr.nepl";
const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
const implementation = source
    .split("\n")
    .filter((line) => !line.startsWith("//:"))
    .join("\n");

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?${kind}\\s+${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${kind} ${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

assert.match(
    source,
    /# prefix_expr[\s\S]*call boundary を決めません[\s\S]*`SelfhostExprPrefixList` は HIR ではありません[\s\S]*計算量/,
    "prefix expression module must document its parser/checker boundary, non-HIR contract, and complexity",
);
assert.match(
    source,
    /pub enum SelfhostExprPrefixItemKind:[\s\S]*TypeAnnotationMarker[\s\S]*FunctionTypeMarker[\s\S]*AtMarker[\s\S]*NamedValue[\s\S]*IntLiteral[\s\S]*BlockMarker/,
    "expression prefix item kind enum must preserve the roles needed by later call reduction",
);
assert.match(
    source,
    /pub struct SelfhostExprPrefixItem:[\s\S]*kind %SelfhostExprPrefixItemKind[\s\S]*token_index %i32[\s\S]*span %SelfhostSourceSpan/,
    "expression prefix items must keep kind, token index, and source span",
);
assert.match(
    source,
    /pub struct SelfhostExprPrefixList:[\s\S]*items %Vec SelfhostExprPrefixItem/,
    "expression prefix list must own a typed item buffer",
);
assert.match(
    source,
    /pub enum SelfhostExprPrefixBuildErrorKind:[\s\S]*MissingExpressionStart[\s\S]*InvalidToken/,
    "builder must reject non-expression starts and invalid expression tokens with typed errors",
);

const fromToken = topLevelBlock(source, "fn", "selfhost_expr_prefix_item_kind_from_token");
for (const [token, role] of [
    ["Percent", "TypeAnnotationMarker"],
    ["KwFn", "FunctionTypeMarker"],
    ["VoidMarker", "VoidMarker"],
    ["At", "AtMarker"],
    ["UnitLiteral", "UnitValue"],
    ["KwLet", "LetMarker"],
    ["KwMatch", "MatchMarker"],
]) {
    assert.match(
        fromToken,
        new RegExp(`TokenKind::${token}:\\s*\\n\\s*some SelfhostExprPrefixItemKind::${role}`),
        `${token} must map to ${role}`,
    );
}

const fromRange = topLevelBlock(source, "fn", "selfhost_expr_prefix_list_from_syntax_range");
assert.match(
    fromRange,
    /not selfhost_expr_prefix_first_token_starts_expr/,
    "builder must use the token predicate to reject ranges that cannot start expressions",
);
assert.match(
    fromRange,
    /selfhost_expr_prefix_list_from_range_loop\s+tokens\s+range\.first_token\s+add\s+range\.first_token\s+range\.token_count/,
    "expression prefix builder must preserve the first token, including % annotations",
);
assert.doesNotMatch(
    implementation,
    /\bSelfhostHir|SelfhostTypeId|SelfhostDefId\b/,
    "prefix expression input must not depend on HIR, TypeId, or DefId allocation",
);
assert.doesNotMatch(
    implementation,
    /selfhost_hir_expr_call|SelfhostHirExprPayload::Call/,
    "prefix expression input must not create resolved HIR call payloads",
);

console.log("selfhost expression prefix contract passed");
