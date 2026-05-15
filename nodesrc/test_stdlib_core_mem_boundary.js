#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { implementationLineCount } = require("./source_policy/stdlib_builder_owner");

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
const pointerView = read("stdlib/core/mem/pointer/view.nepl");
const pointerRegion = read("stdlib/core/mem/pointer/region.nepl");
const pointerBulk = read("stdlib/core/mem/pointer/bulk.nepl");
const pointerScalar = read("stdlib/core/mem/pointer/scalar.nepl");

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
assertMatch(types, /pub\s+struct\s+RegionToken<\.T>:\s+raw\s+<i32>\s+size\s+<i32>/, "RegionToken must store raw owner identity directly");
assertNoMatch(types, /pub\s+struct\s+RegionToken<\.T>:[\s\S]*\bptr\s+<MemPtr<\.T>>/, "RegionToken must not store MemPtr as owner state");
assertNoMatch(types, /pub\s+fn\s+mem_ptr_wrap\b/, "mem/types must not expose raw MemPtr construction helper");
assertNoMatch(types, /pub\s+fn\s+mem_ptr_addr\b/, "mem/types must not expose raw MemPtr address helper");
assertNoMatch(types, /pub\s+fn\s+region_new\b/, "mem/types must not expose owner token construction helper");
assertMatch(layout, /pub\s+fn\s+align8\b/, "mem/layout must own public alignment helper");
assertMatch(layout, /pub\s+fn\s+max_alloc_payload_bytes\b/, "mem/layout must define allocator payload upper bound");
assertMatch(layout, /pub\s+fn\s+alloc_payload_fits\b/, "mem/layout must define allocator payload fit predicate");
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
assertMatch(allocator, /not\s+alloc_payload_fits\s+size/, "raw allocator must reject oversized payload before align8");
for (const moduleName of ["view", "region", "bulk", "scalar"]) {
    assertMatch(
        pointer,
        new RegExp(`pub\\s+#import\\s+"\\./pointer/${moduleName}"\\s+as\\s+\\*`),
        `mem/pointer facade must re-export pointer/${moduleName}`,
    );
}
assertNoMatch(
    pointer,
    /pub\s+#import\s+"\.\/pointer\/alloc"\s+as\s+\*/,
    "mem/pointer facade must not re-export low-level allocation wrappers",
);
assertNoMatch(pointer, /^\s*(?:pub\s+)?fn\s+/m, "mem/pointer facade must not own function bodies");
assertNoMatch(pointer, /^\s*(?:pub\s+)?struct\s+/m, "mem/pointer facade must not own data layout");
assertNoMatch(pointer, /^\s*#(?:wasm|llvmir|intrinsic)\b/m, "mem/pointer facade must not own raw bodies");
assert(
    !fs.existsSync(path.join(repoRoot, "stdlib/core/mem/pointer/alloc.nepl")),
    "mem/pointer/alloc must not remain as a direct-import MemPtr owner API",
);
assertMatch(pointerView, /pub\s+fn\s+mem_ptr_add\b/, "mem/pointer/view must own non-owning MemPtr offset view helper");
assertMatch(pointerRegion, /pub\s+fn\s+region_ptr_at\b/, "mem/pointer/region must own checked region projection");
assertMatch(pointerRegion, /pub\s+struct\s+RegionReallocError<\.T>:[\s\S]*region\s+<RegionToken<\.T>>/, "mem/pointer/region must own RegionToken realloc error payload");
assertMatch(pointerRegion, /pub\s+fn\s+realloc_region_bytes_keep\s+<\.T>\s+<\(RegionToken<\.T>,i32\)->Result<RegionToken<\.T>,\s*RegionReallocError<\.T>>>/, "mem/pointer/region must own owner-preserving RegionToken realloc helper");
assertMatch(pointerRegion, /\balloc_region_bytes[\s\S]*allocator::alloc_raw\s+bytes[\s\S]*region_new\s+mem_ptr_wrap<\.T>\s+raw\s+bytes/, "RegionToken allocation must validate raw allocator success inside the owner boundary");
assertMatch(pointerRegion, /\brealloc_region_bytes_keep[\s\S]*not\s+alloc_payload_fits\s+new_size[\s\S]*let\s+old_size\s+<i32>\s+get\s+region\s+"size"[\s\S]*let\s+old_raw\s+<i32>\s+get\s+region\s+"raw"[\s\S]*allocator::realloc_raw\s+old_raw\s+old_size\s+new_size[\s\S]*RegionReallocError<\.T>\s+region/, "RegionToken realloc must validate size and consume the owner raw field directly");
assertNoMatch(pointerRegion, /\b(?:alloc_ptr|realloc_ptr|dealloc_ptr)\b/, "RegionToken owner API must not route allocation through MemPtr owner wrappers");
assertMatch(pointerRegion, /\balign_of<\.U>/, "region_ptr_at must prove target type alignment");
assertMatch(pointerRegion, /\brem_s\s+addr\s+align\s+0\b/, "region_ptr_at must reject unaligned typed addresses");
assertNoMatch(pointerRegion, /alignment は現時点では検査しません/, "region_ptr_at must not delegate alignment proof to callers");
assertMatch(pointerRegion, /\bmax_alloc_payload_bytes\b/, "alloc_region must prove count * size before multiplication");
assertMatch(pointerBulk, /pub\s+fn\s+mem_copy\b/, "mem/pointer/bulk must own checked bulk copy wrapper");
assertMatch(pointerScalar, /pub\s+fn\s+load_i32\b/, "mem/pointer/scalar must own checked scalar load wrapper");

for (const [label, text] of [
    ["mem/internal", internal],
    ["mem/raw", raw],
    ["mem/allocator", allocator],
    ["mem/pointer/view", pointerView],
    ["mem/pointer/region", pointerRegion],
    ["mem/pointer/bulk", pointerBulk],
    ["mem/pointer/scalar", pointerScalar],
]) {
    assertMatch(
        text,
        /\b(?:mem_ptr_wrap|mem_ptr_addr|RegionToken|#intrinsic\s+"(?:load|store)"|alloc_raw|dealloc_raw|realloc_raw|mem_copy|load_i32|store_i32|load_u8|store_u8)\b/,
        `${label} must carry source-level raw memory boundary evidence`,
    );
}

for (const [label, text, limit] of [
    ["stdlib/core/mem.nepl", root, 120],
    ["stdlib/core/mem/types.nepl", types, 120],
    ["stdlib/core/mem/layout.nepl", layout, 180],
    ["stdlib/core/mem/internal.nepl", internal, 120],
    ["stdlib/core/mem/raw.nepl", raw, 520],
    ["stdlib/core/mem/allocator.nepl", allocator, 420],
    ["stdlib/core/mem/pointer.nepl", pointer, 120],
    ["stdlib/core/mem/pointer/view.nepl", pointerView, 120],
    ["stdlib/core/mem/pointer/region.nepl", pointerRegion, 400],
    ["stdlib/core/mem/pointer/bulk.nepl", pointerBulk, 260],
    ["stdlib/core/mem/pointer/scalar.nepl", pointerScalar, 160],
]) {
    const lines = implementationLineCount(text);
    assert(lines <= limit, `${label} has ${lines} lines; split boundary limit is ${limit}`);
}

console.log("stdlib core/mem boundary split policy ok");
