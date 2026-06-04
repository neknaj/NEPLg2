#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "..");
const CORE_SRC = path.join(ROOT, "nepl-core", "src");
const PLAN = path.join(ROOT, "doc", "neplg2", "parser_backend_responsibility_split_plan.md");
const RUNNER = path.join(ROOT, "nodesrc", "run_source_policy_regressions.js");

function read(filePath) {
    return fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function assertContains(text, needle, label) {
    assert(text.includes(needle), `${label} must contain ${needle}`);
}

const plan = read(PLAN);
for (const marker of [
    "ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587",
    "## Parser split stages",
    "## Backend split stages",
    "## Monomorphize split stages",
    "P1: token stream / recovery",
    "B2: WASM backend",
    "B3: LLVM backend",
    "M1: trait impl index",
    "Source policy",
]) {
    assertContains(plan, marker, "parser/backend responsibility split plan");
}

const parser = read(path.join(CORE_SRC, "parser.rs"));
const parserDocLine = "//! Parser for NEPLG2 surface syntax (prefix + indentation blocks).";
assert(
    parser.indexOf(`${parserDocLine}\n${parserDocLine}`) === -1,
    "parser.rs must not repeat the module-level documentation line",
);
assertContains(
    parser,
    "mod type_expr;",
    "parser root",
);
assertContains(parser, "use type_expr::parse_type_expr_str;", "parser root");

const wasmRoot = read(path.join(CORE_SRC, "codegen_wasm.rs"));
assertContains(wasmRoot, "mod local_map;", "wasm backend root");
assertContains(wasmRoot, "mod string_data;", "wasm backend root");
assertContains(wasmRoot, "mod aggregate;", "wasm backend root");
assertContains(wasmRoot, "mod enum_helpers;", "wasm backend root");
assertContains(
    read(path.join(CORE_SRC, "codegen_wasm", "local_map.rs")),
    "pub(super) struct LocalMap",
    "wasm local map module",
);
assertContains(
    read(path.join(CORE_SRC, "codegen_wasm", "string_data.rs")),
    "pub(super) struct StringDataLayout",
    "wasm string data module",
);
assertContains(
    read(path.join(CORE_SRC, "codegen_wasm", "aggregate.rs")),
    "pub(super) fn aggregate_field_layout",
    "wasm aggregate module",
);
assertContains(
    read(path.join(CORE_SRC, "codegen_wasm", "enum_helpers.rs")),
    "pub(super) fn enum_variant_payload",
    "wasm enum helper module",
);

const llvmRoot = read(path.join(CORE_SRC, "codegen_llvm.rs"));
assertContains(llvmRoot, "mod type_map;", "llvm backend root");
assertContains(llvmRoot, "mod aggregate;", "llvm backend root");
assertContains(
    read(path.join(CORE_SRC, "codegen_llvm", "type_map.rs")),
    "pub(super) fn llty_for_type",
    "llvm type map module",
);
assertContains(
    read(path.join(CORE_SRC, "codegen_llvm", "aggregate.rs")),
    "pub(super) fn aggregate_field_layout",
    "llvm aggregate module",
);

const monomorphizeRoot = read(path.join(CORE_SRC, "monomorphize.rs"));
assertContains(monomorphizeRoot, "mod trait_identity;", "monomorphize root");
assertContains(monomorphizeRoot, "mod trait_lookup;", "monomorphize root");

const runner = read(RUNNER);
assertContains(
    runner,
    "nodesrc/test_parser_backend_responsibility_policy.js",
    "source policy runner",
);

console.log("parser/backend responsibility policy ok");
