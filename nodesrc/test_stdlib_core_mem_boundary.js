#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function assertMatch(text, pattern, message) {
    assert(pattern.test(text), `${message}: expected ${pattern}`);
}

function assertNoMatch(text, pattern, message) {
    assert(!pattern.test(text), `${message}: forbidden ${pattern}`);
}

const root = read("stdlib/core/mem.nepl");
const types = read("stdlib/core/mem/types.nepl");
const raw = read("stdlib/core/mem/raw.nepl");
const allocator = read("stdlib/core/mem/allocator.nepl");
const pointer = read("stdlib/core/mem/pointer.nepl");
const loader = read("nepl-core/src/loader.rs");

for (const moduleName of ["types", "raw", "allocator", "pointer"]) {
    assertMatch(
        root,
        new RegExp(`pub\\s+#import\\s+"\\./mem/${moduleName}"\\s+as\\s+\\*`),
        `core/mem facade must re-export mem/${moduleName}`,
    );
}

assertNoMatch(root, /^\s*(?:pub\s+)?fn\s+/m, "core/mem facade must not own function bodies");
assertNoMatch(root, /^\s*(?:pub\s+)?struct\s+/m, "core/mem facade must not own data layout");
assertNoMatch(root, /^\s*#(?:wasm|llvmir|intrinsic)\b/m, "core/mem facade must not own raw bodies");

assertMatch(types, /pub\s+struct\s+MemPtr<\.T>:/, "mem/types must own MemPtr");
assertMatch(types, /pub\s+struct\s+RegionToken<\.T>:/, "mem/types must own RegionToken");
assertMatch(types, /pub\s+fn\s+mem_ptr_wrap\b/, "mem/types must own MemPtr construction helper");
assertMatch(raw, /pub\s+fn\s+load\s+<\.T>\s+<\(i32\)->\.T>/, "mem/raw must own generic raw load");
assertMatch(raw, /#intrinsic\s+"store"/, "mem/raw must own generic raw store intrinsic");
assertMatch(allocator, /pub\s+fn\s+alloc_raw\b/, "mem/allocator must own raw allocator");
assertMatch(allocator, /pub\s+fn\s+__nepl_rt_alloc\b/, "mem/allocator must own runtime allocator ABI");
assertMatch(pointer, /pub\s+fn\s+alloc_ptr\b/, "mem/pointer must own MemPtr allocation wrapper");
assertMatch(pointer, /pub\s+fn\s+region_ptr_at\b/, "mem/pointer must own checked region projection");

assertNoMatch(
    loader,
    /&\["core",\s*"mem\.nepl"\]/,
    "raw-memory-boundary capability must not remain on core/mem facade",
);

for (const rel of [
    '"core", "mem", "types.nepl"',
    '"core", "mem", "raw.nepl"',
    '"core", "mem", "allocator.nepl"',
    '"core", "mem", "pointer.nepl"',
]) {
    assert(
        loader.includes(`&[${rel}]`),
        `loader raw-memory-boundary table must include exact ${rel}`,
    );
}

for (const [label, text, limit] of [
    ["stdlib/core/mem.nepl", root, 120],
    ["stdlib/core/mem/types.nepl", types, 120],
    ["stdlib/core/mem/raw.nepl", raw, 520],
    ["stdlib/core/mem/allocator.nepl", allocator, 420],
    ["stdlib/core/mem/pointer.nepl", pointer, 320],
]) {
    const lines = text.split("\n").length;
    assert(lines <= limit, `${label} has ${lines} lines; split boundary limit is ${limit}`);
}

console.log("stdlib core/mem boundary split policy ok");
