---
id: ISS-20260426T230422845Z-BINARYHEAP-FREE-TRAPS-FOR-ZERO-CAPAC-07CE8EC3
title: "BinaryHeap free traps for zero-capacity heap"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "stdlib/alloc/collections/binary_heap.nepl, tests/stdlib/binary_heap_collections.n.md, nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js"
---

# ISS-20260426T230422845Z-BINARYHEAP-FREE-TRAPS-FOR-ZERO-CAPAC-07CE8EC3: BinaryHeap free traps for zero-capacity heap

## 概要

BinaryHeap.with_capacity 0 stores a null data pointer, but free unconditionally calls dealloc_ptr on that pointer and unwraps the Result. A valid zero-capacity heap can therefore trap during normal cleanup. The same module also unwraps internal MemPtr header stores even though the public APIs already use Result for allocation failure.

## 対象

- `stdlib/alloc/collections/binary_heap.nepl, tests/stdlib/binary_heap_collections.n.md, nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js`

## 根拠

- `with_capacity 0` は header の data pointer field に 0 を保存する。
- `free` は capacity を確認せず `dealloc_ptr<.T> data0 ...` を呼び、その `Result` を `uwok` していた。
- `with_capacity` / `push` / `pop` でも、BinaryHeap が所有する header への内部 store を checked MemPtr API で呼び、その `Result` を `uwok` していた。

## 問題

BinaryHeap.with_capacity 0 stores a null data pointer, but free unconditionally calls dealloc_ptr on that pointer and unwraps the Result. A valid zero-capacity heap can therefore trap during normal cleanup. The same module also unwraps internal MemPtr header stores even though the public APIs already use Result for allocation failure.

## 影響

Self-host scheduling or parser work queues may start from zero capacity to avoid allocation until first push. Cleanup should be a no-op for the absent data buffer, not an unreachable trap. The unsafe helpers also keep RV-STDLIB-010 debt in a basic collection.

## 修正方針

Use internal raw header writes for BinaryHeap-owned headers, skip data deallocation when capacity is zero, keep allocation failures as Result errors, and add source/test regressions that prevent unsafe unwraps from returning to the implementation.

## 解決内容

- `heap_store_header_i32` を追加し、BinaryHeap が所有する header field への内部書き込みを raw store に集約した。
- `with_capacity` / `push` / `pop` の header 更新から `uwok store_i32` を削除した。
- `free` は `cap0 > 0` のときだけ data buffer を解放し、capacity 0 の null data pointer は no-op にした。
- `with_capacity 0` の `free` と、capacity 0 からの初回 `push` を `tests/stdlib/binary_heap_collections.n.md` に追加した。
- `nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js` を追加し、実装コードに unsafe unwrap helper が戻らないことを CI source policy で監視するようにした。

## 検証

- `node nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/binary_heap_collections.n.md --no-tree -o tmp/binary-heap-zero-capacity-focused.json -j 1`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/binary_heap.nepl --no-tree -o tmp/binary-heap-zero-capacity-docs.json -j 1`: 6/6 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-binary-heap-zero-capacity.json -j 4`: 284/284 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-binary-heap-zero-capacity.json -j 4`: 416/416 passed
- remote main の `b2b037e core: support owned aggregate field decomposition` を取り込んだ後に `trunk build`: pass
- `node nodesrc/tests.js -i tests/stdlib/binary_heap_collections.n.md --no-tree -o tmp/binary-heap-zero-capacity-after-b2b037e-build.json -j 1`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/binary_heap.nepl --no-tree -o tmp/binary-heap-zero-capacity-docs-after-b2b037e-build.json -j 1`: 6/6 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-binary-heap-zero-capacity-after-b2b037e-build.json -j 4`: 284/284 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-binary-heap-zero-capacity-after-b2b037e-build.json -j 4`: 416/416 passed
