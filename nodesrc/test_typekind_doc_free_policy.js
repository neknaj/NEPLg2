#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const assert = require("assert");

const root = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(root, relPath), "utf8").replace(/\r\n/g, "\n");
}

function balancedBlock(source, startNeedle, label) {
    const start = source.indexOf(startNeedle);
    assert(start >= 0, `${label} must exist`);
    return balancedBlockAt(source, start, label);
}

function balancedBlockAt(source, start, label) {
    const open = source.indexOf("{", start);
    assert(open >= 0, `${label} must open a block`);

    let depth = 0;
    for (let i = open; i < source.length; i += 1) {
        const ch = source[i];
        if (ch === "{") {
            depth += 1;
        } else if (ch === "}") {
            depth -= 1;
            if (depth === 0) {
                return source.slice(open + 1, i);
            }
        }
    }

    throw new Error(`${label} block is not balanced`);
}

const types = read("nepl-core/src/types.rs");
const ast = read("nepl-core/src/ast.rs");
const hir = read("nepl-core/src/hir.rs");

const typeKind = balancedBlock(types, "pub enum TypeKind", "TypeKind");
const enumVariant = balancedBlock(typeKind, "Enum", "TypeKind::Enum");
const structVariant = balancedBlock(typeKind, "Struct", "TypeKind::Struct");

assert(
    !/\bdoc\s*:/.test(enumVariant),
    "TypeKind::Enum must not carry doc comments; docs belong to AST/HIR metadata, not type identity",
);
assert(
    !/\bdoc\s*:/.test(structVariant),
    "TypeKind::Struct must not carry doc comments; docs belong to AST/HIR metadata, not type identity",
);
assert(
    /pub\s+doc:\s+Option<String>/.test(ast),
    "AST declarations must keep source documentation metadata",
);
assert(
    /pub\s+doc:\s+Option<String>/.test(hir),
    "HIR declarations must keep source documentation metadata",
);

for (const relPath of walkRustFiles("nepl-core")) {
    const source = read(relPath);
    const pattern = /TypeKind::(?:Enum|Struct)\s*\{/g;
    for (const match of source.matchAll(pattern)) {
        const block = balancedBlockAt(source, match.index, `${relPath} TypeKind construction`);
        assert(
            !/\bdoc\s*:/.test(block),
            `${relPath} must not pass doc metadata into TypeKind nominal construction`,
        );
    }
}

console.log("TypeKind documentation separation policy ok");

function walkRustFiles(relDir, out = []) {
    const absDir = path.join(root, relDir);
    for (const entry of fs.readdirSync(absDir, { withFileTypes: true })) {
        const childRel = path.join(relDir, entry.name).replace(/\\/g, "/");
        if (entry.isDirectory()) {
            walkRustFiles(childRel, out);
        } else if (entry.isFile() && entry.name.endsWith(".rs")) {
            out.push(childRel);
        }
    }
    return out.sort();
}
