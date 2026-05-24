#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { readHirSource } = require("./selfhost_hir_sources");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const hir = legacyTypeSyntaxView(readHirSource(repoRoot));

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const decl = kind === "fn"
        ? new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s`)
        : new RegExp(`^(?:pub\\s+)?${kind}\\s+${name}`);
    const start = lines.findIndex((line) => decl.test(line));
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

function enumVariants(src, enumName) {
    return topLevelBlock(src, "enum", `${enumName}:`)
        .split("\n")
        .slice(1)
        .map((line) => /^    ([A-Za-z][A-Za-z0-9_]*)(?:\s+<[^>]+>)?$/.exec(line))
        .filter(Boolean)
        .map((match) => match[1]);
}

const exprStruct = topLevelBlock(hir, "struct", "SelfhostHirExpr:");
assert.match(exprStruct, /\bty\s+<SelfhostTypeId>/, "SelfhostHirExpr must keep common type");
assert.match(exprStruct, /\bspan\s+<SelfhostSourceSpan>/, "SelfhostHirExpr must keep common span");
assert.match(exprStruct, /\bpayload\s+<SelfhostHirExprPayload>/, "SelfhostHirExpr must store variant payload");
assert.doesNotMatch(exprStruct, /\bkind\s+<SelfhostHirExprKind>/, "expression kind must be derived from payload");
assert.doesNotMatch(exprStruct, /\b(?:first_child|child_count|name|int_value|bool_value)\s+</, "kind-specific fields must not live on every expression");

assert.deepEqual(
    enumVariants(hir, "SelfhostHirExprPayload"),
    ["Error", "Unit", "BoolLiteral", "I32Literal", "StrLiteral", "Var", "Call", "Block", "If"],
    "expression payload variants must cover the current expression kind set",
);
assert.match(hir, /(?:pub\s+)?struct SelfhostHirCallExpr:[\s\S]*?\bname\s+<str>[\s\S]*?\bargs\s+<SelfhostHirChildRange>/, "call payload must own callee name and argument range");

assert.doesNotMatch(hir, /\bfn\s+selfhost_hir_expr_(?:leaf|with_children)\b/, "flat leaf/children constructors must not remain");
assert.doesNotMatch(hir, /\bselfhost_hir_expr_new\s+SelfhostHirExprKind::/, "flat kind-driven constructor calls must not remain");
assert.doesNotMatch(hir, /\b(SelfhostHirExpr|got_expr|got_parent|expr)\.(?:kind|first_child|child_count|int_value|bool_value)\b/, "callers must use payload accessors instead of flat expression fields");

const kindAccessor = topLevelBlock(hir, "fn", "selfhost_hir_expr_kind");
assert.match(kindAccessor, /<\(&SelfhostHirExpr\)->SelfhostHirExprKind>/, "kind accessor must borrow the expression");
assert.match(kindAccessor, /\bmatch\s+\*field::get_ref\s+expr\s+"payload":/, "kind accessor must match borrowed payload");
for (const variant of enumVariants(hir, "SelfhostHirExprPayload")) {
    assert.match(kindAccessor, new RegExp(`^\\s*SelfhostHirExprPayload::${variant}\\b`, "m"), `kind accessor must handle ${variant}`);
}

const childAccessor = topLevelBlock(hir, "fn", "selfhost_hir_expr_child_range");
assert.match(childAccessor, /<\(&SelfhostHirExpr\)->SelfhostHirChildRange>/, "child range accessor must borrow the expression");
assert.match(childAccessor, /\bmatch\s+\*field::get_ref\s+expr\s+"payload":/, "child range accessor must match borrowed payload");
assert.match(childAccessor, /\bSelfhostHirExprPayload::Call\s+call:/, "call payload must expose args through child range accessor");
assert.match(childAccessor, /\bSelfhostHirExprPayload::Block\s+children:/, "block payload must expose child range");
assert.match(childAccessor, /\bSelfhostHirExprPayload::If\s+branches:/, "if payload must expose branch range");
assert.doesNotMatch(childAccessor, /\bexpr\.(?:first_child|child_count)\b/, "child range accessor must not read flat child fields");

for (const fnName of [
    "selfhost_hir_expr_error",
    "selfhost_hir_expr_unit",
    "selfhost_hir_expr_bool_literal",
    "selfhost_hir_expr_i32_literal",
    "selfhost_hir_expr_str_literal",
    "selfhost_hir_expr_var",
    "selfhost_hir_expr_call",
    "selfhost_hir_expr_block",
    "selfhost_hir_expr_if",
]) {
    const block = topLevelBlock(hir, "fn", fnName);
    assert.match(block, /SelfhostHirExprPayload::/, `${fnName} must construct a payload variant`);
}

console.log("selfhost HIR expression payload regression passed");
