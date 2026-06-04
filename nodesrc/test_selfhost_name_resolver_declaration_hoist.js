#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    NAME_RESOLVER_FACADE,
    NAME_RESOLVER_SPLIT_FILES,
    readNameResolverSource,
    readRepoFile,
} = require("./selfhost_name_resolver_sources");

const repoRoot = path.resolve(__dirname, "..");
const facade = readRepoFile(repoRoot, NAME_RESOLVER_FACADE);
const source = readNameResolverSource(repoRoot);

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

assert.ok(
    NAME_RESOLVER_SPLIT_FILES.includes("stdlib/neplg2/core/resolve/name_resolver/hoist.nepl"),
    "name resolver hoist module must be part of the split source list",
);
assert.match(
    facade,
    /^pub #import "\.\/name_resolver\/hoist" as \*$/m,
    "name resolver facade must re-export the declaration hoist boundary",
);

const kindBlock = functionBlock(source, "selfhost_def_kind_from_module_declaration_kind");
for (const pair of [
    ["Function", "Function"],
    ["Struct", "Struct"],
    ["Enum", "Enum"],
    ["Trait", "Trait"],
]) {
    assert.match(
        kindBlock,
        new RegExp(`SelfhostModuleDeclarationKind::${pair[0]}:[\\s\\S]*some SelfhostDefKind::${pair[1]}`),
        `declaration ${pair[0]} must hoist as def kind ${pair[1]}`,
    );
}
assert.match(
    kindBlock,
    /SelfhostModuleDeclarationKind::Impl:\s*\n\s*none/,
    "impl declarations must not create named scope bindings in the initial hoist boundary",
);

const bindingBlock = functionBlock(source, "selfhost_name_binding_from_declaration_header");
assert.match(
    bindingBlock,
    /SelfhostModuleDeclarationHeadKind::Name:[\s\S]*string_slice::str_slice source head\.span\.start head\.span\.end[\s\S]*selfhost_name_binding_pending name def_kind head\.span/,
    "declaration hoist must derive names from typed head spans, not from whole header text",
);
assert.match(
    bindingBlock,
    /SelfhostModuleDeclarationHeadKind::TypeLabel:\s*\n\s*none/,
    "type-label heads must not be treated as named bindings",
);
assert.doesNotMatch(
    bindingBlock,
    /\bheader\.header_span\b|\bitem\.lexeme\b/,
    "declaration hoist must not reparse header lexemes to recover declaration names",
);

const entryBlock = functionBlock(source, "selfhost_name_scope_hoist_module_declarations");
assert.match(
    entryBlock,
    /selfhost_name_scope_new[\s\S]*selfhost_name_scope_hoist_module_declarations_loop source ast selfhost_module_ast_len ast 0 scope/,
    "declaration hoist entry must build a fresh scope and iterate over the module AST",
);

console.log("selfhost name resolver declaration hoist contract passed");
