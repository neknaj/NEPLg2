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
const layout = read("stdlib/core/mem/layout.nepl");
const internal = read("stdlib/core/mem/internal.nepl");
const raw = read("stdlib/core/mem/raw.nepl");
const allocator = read("stdlib/core/mem/allocator.nepl");
const pointer = read("stdlib/core/mem/pointer.nepl");
const pointerAlloc = read("stdlib/core/mem/pointer/alloc.nepl");
const pointerRegion = read("stdlib/core/mem/pointer/region.nepl");
const pointerBulk = read("stdlib/core/mem/pointer/bulk.nepl");
const pointerScalar = read("stdlib/core/mem/pointer/scalar.nepl");
const loader = read("nepl-core/src/loader.rs");

for (const moduleName of ["types", "layout", "pointer"]) {
    assertMatch(
        root,
        new RegExp(`pub\\s+#import\\s+"\\./mem/${moduleName}"\\s+as\\s+\\*`),
        `core/mem facade must re-export mem/${moduleName}`,
    );
}

for (const moduleName of ["internal", "raw", "allocator"]) {
    assertNoMatch(
        root,
        new RegExp(`pub\\s+#import\\s+"\\./mem/${moduleName}"\\s+as\\s+\\*`),
        `core/mem facade must not re-export raw/internal ${moduleName}`,
    );
}

assertNoMatch(root, /^\s*(?:pub\s+)?fn\s+/m, "core/mem facade must not own function bodies");
assertNoMatch(root, /^\s*(?:pub\s+)?struct\s+/m, "core/mem facade must not own data layout");
assertNoMatch(root, /^\s*#(?:wasm|llvmir|intrinsic)\b/m, "core/mem facade must not own raw bodies");

assertMatch(types, /pub\s+struct\s+MemPtr<\.T>:/, "mem/types must own MemPtr");
assertMatch(types, /pub\s+struct\s+RegionToken<\.T>:/, "mem/types must own RegionToken");
assertNoMatch(types, /pub\s+fn\s+mem_ptr_wrap\b/, "mem/types must not expose raw MemPtr construction helper");
assertNoMatch(types, /pub\s+fn\s+mem_ptr_addr\b/, "mem/types must not expose raw MemPtr address helper");
assertNoMatch(types, /pub\s+fn\s+region_new\b/, "mem/types must not expose owner token construction helper");
assertMatch(layout, /pub\s+fn\s+align8\b/, "mem/layout must own public alignment helper");
assertMatch(layout, /pub\s+fn\s+size_of\b/, "mem/layout must own public size_of");
assertMatch(layout, /pub\s+fn\s+align_of\b/, "mem/layout must own public align_of");
assertMatch(internal, /pub\s+fn\s+mem_ptr_wrap\b/, "mem/internal must own MemPtr construction helper");
assertMatch(internal, /pub\s+fn\s+mem_ptr_addr\b/, "mem/internal must own MemPtr raw address helper");
assertMatch(internal, /pub\s+fn\s+region_new\b/, "mem/internal must own RegionToken construction helper");
assertMatch(raw, /pub\s+fn\s+load\s+<\.T>\s+<\(i32\)->\.T>/, "mem/raw must own generic raw load");
assertMatch(raw, /#intrinsic\s+"store"/, "mem/raw must own generic raw store intrinsic");
assertNoMatch(raw, /pub\s+fn\s+size_of\b/, "mem/raw must not own public layout helper");
assertNoMatch(raw, /pub\s+fn\s+align_of\b/, "mem/raw must not own public layout helper");
assertMatch(allocator, /pub\s+fn\s+alloc_raw\b/, "mem/allocator must own raw allocator");
assertMatch(allocator, /pub\s+fn\s+__nepl_rt_alloc\b/, "mem/allocator must own runtime allocator ABI");
for (const moduleName of ["alloc", "region", "bulk", "scalar"]) {
    assertMatch(
        pointer,
        new RegExp(`pub\\s+#import\\s+"\\./pointer/${moduleName}"\\s+as\\s+\\*`),
        `mem/pointer facade must re-export pointer/${moduleName}`,
    );
}
assertNoMatch(pointer, /^\s*(?:pub\s+)?fn\s+/m, "mem/pointer facade must not own function bodies");
assertNoMatch(pointer, /^\s*(?:pub\s+)?struct\s+/m, "mem/pointer facade must not own data layout");
assertNoMatch(pointer, /^\s*#(?:wasm|llvmir|intrinsic)\b/m, "mem/pointer facade must not own raw bodies");
assertMatch(pointerAlloc, /pub\s+fn\s+alloc_ptr\b/, "mem/pointer/alloc must own MemPtr allocation wrapper");
assertMatch(pointerAlloc, /pub\s+fn\s+dealloc_ptr\b/, "mem/pointer/alloc must own MemPtr deallocation wrapper");
assertMatch(pointerRegion, /pub\s+fn\s+region_ptr_at\b/, "mem/pointer/region must own checked region projection");
assertMatch(pointerBulk, /pub\s+fn\s+mem_copy\b/, "mem/pointer/bulk must own checked bulk copy wrapper");
assertMatch(pointerScalar, /pub\s+fn\s+load_i32\b/, "mem/pointer/scalar must own checked scalar load wrapper");

assertNoMatch(
    loader,
    /&\["core",\s*"mem\.nepl"\]/,
    "raw-memory-boundary capability must not remain on core/mem facade",
);
assertNoMatch(
    loader,
    /&\["core",\s*"mem",\s*"types\.nepl"\]/,
    "raw-memory-boundary capability must not remain on mem/types because it only owns public layouts and safe field observers",
);
assertNoMatch(
    loader,
    /&\["core",\s*"mem",\s*"pointer\.nepl"\]/,
    "raw-memory-boundary capability must not remain on mem/pointer facade",
);

for (const rel of [
    '"core", "mem", "internal.nepl"',
    '"core", "mem", "raw.nepl"',
    '"core", "mem", "allocator.nepl"',
    '"core", "mem", "pointer", "alloc.nepl"',
    '"core", "mem", "pointer", "region.nepl"',
    '"core", "mem", "pointer", "bulk.nepl"',
    '"core", "mem", "pointer", "scalar.nepl"',
]) {
    assert(
        loader.includes(`&[${rel}]`),
        `loader raw-memory-boundary table must include exact ${rel}`,
    );
}

for (const [label, text, limit] of [
    ["stdlib/core/mem.nepl", root, 120],
    ["stdlib/core/mem/types.nepl", types, 120],
    ["stdlib/core/mem/layout.nepl", layout, 120],
    ["stdlib/core/mem/internal.nepl", internal, 120],
    ["stdlib/core/mem/raw.nepl", raw, 520],
    ["stdlib/core/mem/allocator.nepl", allocator, 420],
    ["stdlib/core/mem/pointer.nepl", pointer, 120],
    ["stdlib/core/mem/pointer/alloc.nepl", pointerAlloc, 260],
    ["stdlib/core/mem/pointer/region.nepl", pointerRegion, 260],
    ["stdlib/core/mem/pointer/bulk.nepl", pointerBulk, 260],
    ["stdlib/core/mem/pointer/scalar.nepl", pointerScalar, 160],
]) {
    const lines = text.split("\n").length;
    assert(lines <= limit, `${label} has ${lines} lines; split boundary limit is ${limit}`);
}

console.log("stdlib core/mem boundary split policy ok");
