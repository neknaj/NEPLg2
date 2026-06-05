---
id: ISS-20260604T034124436Z-RAW-MEMORY-ALLOCATOR-AND-COLLECTION--C8833FBA
title: "raw memory allocator and collection mutation APIs are exposed as pure functions"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: "stdlib/core/mem/raw.nepl, stdlib/core/mem/allocator.nepl, stdlib/alloc/collections/vec/storage/api.nepl"
---

# ISS-20260604T034124436Z-RAW-MEMORY-ALLOCATOR-AND-COLLECTION--C8833FBA: raw memory allocator and collection mutation APIs are exposed as pure functions

## 概要

Subagent audit found raw memory, allocator, and collection owner mutation APIs where the source-level effect contract did not clearly distinguish compiler-known internal proof boundaries from externally observable owner mutation. This conflicted with plan.md pure/impure function separation and the Zenn policy that side effects must be surfaced by the type system or by an explicit static proof boundary.

## 対象

- `stdlib/core/mem/raw.nepl`
- `stdlib/core/mem/allocator.nepl`
- `stdlib/core/mem/pointer/*.nepl`
- `stdlib/alloc/collections/vec/storage/*.nepl`
- `stdlib/alloc/collections/vec/mutation/*.nepl`
- `stdlib/alloc/collections/vec/transform/*.nepl`
- `stdlib/alloc/collections/*` owner cleanup APIs
- `stdlib/alloc/diag/error/*.nepl`

## 根拠

- Zenn article policy requires pure/impure boundaries to be statically checked, and says side-effecting platform/resource behavior must not be hidden in core logic.
- `doc/compare/memory_model.md` treats internal allocation as a compiler-managed memory-model operation that may remain surface-pure only when the private allocation identity cannot escape.
- Collection cleanup and owner-consuming mutation are externally observable ownership lifecycle operations, so their source signature must carry `impure fn`.

## 問題

Raw memory and allocator helpers were not documented as compiler-known Resource IR proof boundaries. Separately, Vec mutation/cleanup, Vec transform APIs that allocate or release intermediate owners, diagnostic wrappers that free owned `Diags`, and collection cleanup surfaces were still exposed or tested as pure even though they consume owners and close storage.

## 影響

Pure stdlib functions could appear to call owner mutation or cleanup without an impure boundary, weakening static effect checks and hiding resource lifecycle effects in collection APIs.

## 修正方針

Keep raw memory and allocator primitives as compiler-known proof boundaries where Resource IR rejects ordinary source use and private allocation identity does not escape. Move externally observable ownership-changing collection operations to `impure fn`, including Vec push/pop/clear/free, Vec transform APIs that allocate or clean intermediate owners, by-value diagnostic cleanup wrappers, and collection `free` helpers. Update source-policy regressions so the impure contract is fixed.

## 検証

Implemented and verified:

- `node nodesrc/test_stdlib_core_mem_boundary.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_stdlib_list_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`
- focused individual source-policy tests for BinaryHeap, Queue/Deque, RingBuffer, Stack, AdjacencyMatrix, BitSet, DisjointSet, SegmentTree, SparseSet, BloomFilter, CountingBloomFilter, Fenwick, BTreeMap/BTreeSet, HashMap, and HashSet
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/push.nepl -i stdlib/alloc/collections/vec/mutation/pop.nepl -i stdlib/alloc/collections/vec/mutation/cleanup.nepl --no-tree -o tmp/raw-memory-effect-boundary-focused.json -j 1`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/map.nepl -i stdlib/alloc/collections/vec/transform/filter/select.nepl -i stdlib/alloc/collections/vec/transform/filter/partition/build.nepl -i stdlib/alloc/collections/vec/transform/filter/partition/view.nepl -i stdlib/alloc/collections/vec/transform/prefix.nepl --no-tree -o tmp/raw-memory-effect-boundary-transform.json -j 1`
- `node nodesrc/tests.js -i stdlib/tests/vec.n.md -i tests/stdlib/vec_collections.n.md -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/raw-memory-effect-boundary-vec-suite.json -j 1`

`run_source_policy_regressions.js --warn-only` still reports the unrelated existing warnings in `nodesrc/test_resource_gate_order.js` and `nodesrc/test_diagnostic_code_first_boundary.js`.
