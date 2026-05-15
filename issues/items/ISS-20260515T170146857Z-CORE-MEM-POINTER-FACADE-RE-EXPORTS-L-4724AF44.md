---
id: ISS-20260515T170146857Z-CORE-MEM-POINTER-FACADE-RE-EXPORTS-L-4724AF44
title: "core/mem pointer facade re-exports low-level alloc_ptr owner wrappers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-16
target: "stdlib/core/mem.nepl; stdlib/core/mem/pointer.nepl; stdlib/core/mem/pointer/view.nepl; nodesrc/test_stdlib_core_mem_boundary.js"
---

# ISS-20260515T170146857Z-CORE-MEM-POINTER-FACADE-RE-EXPORTS-L-4724AF44: core/mem pointer facade re-exports low-level alloc_ptr owner wrappers

## 概要

core/mem and mem/pointer safe facades re-exported alloc_ptr/realloc_ptr/dealloc_ptr, so ordinary safe imports could obtain MemPtr-returning allocation owner APIs even though MemPtr is now a non-owning pointer view.

## 対象

- `stdlib/core/mem.nepl; stdlib/core/mem/pointer.nepl; stdlib/core/mem/pointer/view.nepl; nodesrc/test_stdlib_core_mem_boundary.js`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6 は `MemPtr<T>` を non-owning pointer projection に固定し、free obligation owner を owner token / storage state 側へ分離する方針である。
- `stdlib/core/mem/pointer.nepl` が `./pointer/alloc` を public re-export していたため、`#import "core/mem" as *` だけで `alloc_ptr<T> -> Result<MemPtr<T>, str>` / `realloc_ptr<T>` / `dealloc_ptr<T>` に到達できた。
- `mem_ptr_add` は non-owning offset view helper であり、alloc/free obligation owner wrapper と同じ file に置くと pointer view 操作と owner boundary API の責務が混ざる。
- 親 issue: [ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686](./ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686.md)

## 問題

core/mem and mem/pointer safe facades re-exported alloc_ptr/realloc_ptr/dealloc_ptr, so ordinary safe imports could obtain MemPtr-returning allocation owner APIs even though MemPtr is now a non-owning pointer view.

## 影響

The public surface contradicted Stage 6 MemPtr = non-owning pointer discipline and forced Resource IR to keep MemPtr owner-carrier compatibility at the safe API boundary.

## 修正方針

Stop re-exporting pointer/alloc from mem/pointer, move mem_ptr_add into a separate non-owning pointer/view module, and make stdlib scratch implementations import pointer/alloc explicitly.

## 検証

Add source policy and compile-fail regressions proving core/mem does not expose alloc_ptr while core/mem pointer doctests still pass through RegionToken owner APIs.

## 対応結果

- `stdlib/core/mem/pointer.nepl` は `./pointer/alloc` を再公開せず、`view` / `region` / `bulk` / `scalar` の safe facade に縮小した。
- `mem_ptr_add<T>` は `stdlib/core/mem/pointer/view.nepl` へ移し、storage owner の確保・再確保・解放 API と分離した。ordinary source からの offset view 利用は既存の `resource.raw.memory_outside_boundary` compile_fail doctest で固定した。
- `stdlib/core/mem.nepl` / `pointer.nepl` / `scalar.nepl` の public doctest は `alloc_ptr` ではなく `alloc_region` / `region_ptr` / `dealloc_region` を使う形へ移行した。失敗 branch でも `RegionToken` を解放するようにし、owner leak を隠さない。
- 低レベル scratch 実装と fixture は、必要箇所だけ `core/mem/pointer/alloc` を明示 import するようにした。これはまだ Stage 6 の移行中境界であり、親 issue の direct import / token API 移行は open のまま継続する。
- `stdlib/alloc/io/bytebuf.nepl` の `io_bytebuf_region_ptr` が旧 `RegionToken.ptr` field を読んでいた stale 実装を、`region_ptr` projection へ修正した。

## 検証結果

- `node nodesrc/test_stdlib_core_mem_boundary.js`: pass
- `node nodesrc/test_stdlib_documentation_contract.js`: pass
- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-core-mem-safe-root-memory-safety.json -j 1 --dist web/dist --assert-io`: 39 passed
- `node nodesrc/tests.js -i stdlib/core/mem.nepl -i stdlib/core/mem/pointer.nepl -i stdlib/core/mem/pointer/alloc.nepl -i stdlib/core/mem/pointer/view.nepl -i stdlib/core/mem/pointer/scalar.nepl -i stdlib/core/mem/internal.nepl --no-tree -o tmp/agent1-core-mem-safe-root-doctests.json -j 1 --dist web/dist --assert-io`: 15 passed

## 残件

`core/mem/pointer/alloc` を直接 import すると `MemPtr<T>` owner wrapper API へ到達できるため、親 issue は閉じない。次段階では stdlib scratch buffer を `RegionToken` / `OwnedBytes` / `OwnedBuffer` に寄せ、direct import 可能な low-level API 自体を compiler-owned boundary へ閉じる。
