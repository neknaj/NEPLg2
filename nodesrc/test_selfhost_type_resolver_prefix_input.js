#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { readTypeResolverSource } = require("./selfhost_type_resolver_sources");

const repoRoot = path.resolve(__dirname, "..");
const source = readTypeResolverSource(repoRoot);

function topLevelBlock(src, kind, name) {
    const pattern = new RegExp(`^(?:pub\\s+)?${kind}\\s+${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b[\\s\\S]*?(?=\\n(?:pub\\s+)?(?:struct|enum|fn|impl)\\s+|\\n#|\\n//: neplg2:test|\\n$)`, "m");
    const match = src.match(pattern);
    assert.ok(match, `missing top-level ${kind} ${name}`);
    return match[0];
}

assert.match(
    source,
    /pub enum SelfhostTypePrefixItemKind:[\s\S]*FunctionMarker[\s\S]*VoidMarker[\s\S]*NamedType/,
    "type resolver input must distinguish function marker, void marker, and named types",
);
assert.match(
    source,
    /pub struct SelfhostTypePrefixItem:[\s\S]*kind %SelfhostTypePrefixItemKind[\s\S]*token_index %i32[\s\S]*span %SelfhostSourceSpan/,
    "type prefix items must keep kind, token index, and source span",
);
assert.match(
    source,
    /pub enum SelfhostTypePrefixBuildErrorKind:[\s\S]*MissingAnnotationMarker[\s\S]*InvalidToken/,
    "type prefix list builder must reject ranges that are not parser-provided % annotations",
);
assert.doesNotMatch(
    source,
    /\bSelfhostPrimitiveTypeKind:[\s\S]*\bVoid\b/,
    "type resolver must not introduce void as a primitive type",
);
assert.doesNotMatch(
    source,
    /\bSelfhostTypeId\b[\s\S]{0,80}\bselfhost_type_prefix_list_from_syntax_range\b/,
    "prefix input construction must not allocate TypeId during the resolver input slice",
);

const fromToken = topLevelBlock(source, "fn", "selfhost_type_prefix_item_kind_from_token");
assert.match(
    fromToken,
    /TokenKind::VoidMarker:\s*\n\s*some SelfhostTypePrefixItemKind::VoidMarker/,
    "void token must become a dedicated void marker item",
);
assert.match(
    fromToken,
    /TokenKind::UnitLiteral:\s*\n\s*some SelfhostTypePrefixItemKind::NamedType/,
    "unit token must remain a normal named type item",
);

const fromRange = topLevelBlock(source, "fn", "selfhost_type_prefix_list_from_syntax_range");
assert.match(
    fromRange,
    /not selfhost_type_prefix_first_token_is_percent/,
    "builder must require the parser-provided range to start at the % marker",
);
assert.match(
    fromRange,
    /selfhost_type_prefix_list_from_range_loop\s+tokens\s+add\s+range\.first_token\s+1/,
    "builder must skip the % marker when creating resolver input items",
);

console.log("selfhost type resolver prefix input contract passed");
