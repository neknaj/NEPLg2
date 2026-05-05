---
id: ISS-20260505T073434026Z-CORE-MEM-ALLOCATOR-METADATA-DOCTEST--3D5EEF97
title: "core mem allocator metadata doctest fails Resource IR cell initialization"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: stdlib/core/mem.nepl
---

# ISS-20260505T073434026Z-CORE-MEM-ALLOCATOR-METADATA-DOCTEST--3D5EEF97: core mem allocator metadata doctest fails Resource IR cell initialization

## 概要

stdlib/core/mem.nepl doctest#3 reads allocator metadata with load_i32 0 and load_i32 4 after alloc_raw, but Resource IR reports resource.cell.uninit for those absolute raw loads.

## 対象

- `stdlib/core/mem.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib/core/mem.nepl --no-tree -o tmp/core-mem-fill-report-agent1.json -j 1 --dist web/dist`: total=6, passed=5, failed=1。
- 失敗は未変更の `stdlib/core/mem.nepl::doctest#3` で、`load_i32 0` と `load_i32 4` が `resource.cell.uninit` と診断された。
- `node nodesrc/run_doctest.js -i stdlib/core/mem.nepl -n 3 --dist web/dist` でも同じ compile failure を再現した。

## 問題

stdlib/core/mem.nepl doctest#3 reads allocator metadata with load_i32 0 and load_i32 4 after alloc_raw, but Resource IR reports resource.cell.uninit for those absolute raw loads.

## 影響

The full core mem doctest file fails even though the public memset_u8/fill_i32 report migration doctests pass, and allocator metadata introspection bypasses the memory model boundary expected by the static checker.

## 修正方針

Replace the doctest with public observable allocator invariants or explicitly model allocator metadata initialization in the compiler; do not weaken Resource IR raw-load initialization checks.

現時点の判断:

- Resource IR の raw-load initialized check は緩めない。
- doctest が allocator 内部 metadata を absolute address で直接読む設計が、メモリモデル境界として不適切かを優先して検討する。
- compiler 側で allocator metadata の初期化を特別扱いする場合でも、通常ユーザーコードの任意 absolute raw load を許可する形にはしない。

## 対応結果

- `load_i32 0` / `load_i32 4` による allocator metadata introspection を doctest から削除した。
- 代わりに `alloc_raw` を2回呼び、返された pointer が非ゼロ領域にあり、2つ目が1つ目より後ろに割り当てられることを public observable invariant として確認する形にした。
- それぞれの領域へ `store_i32` した値を `load_i32` で読み戻し、raw allocation が別々の cell を提供していることを確認した。
- Resource IR の raw-load initialized check は変更していない。
- doctest を `std/test` report 形式にし、4件の assertion を stdout fixture として固定した。

## 検証

- `node nodesrc/run_doctest.js -i stdlib/core/mem.nepl -n 3 --dist web/dist`: passed, stdout=`Checked [ok,ok,ok,ok]`
- `node nodesrc/tests.js -i stdlib/core/mem.nepl --no-tree -o tmp/core-mem-after-metadata-agent1.json -j 1 --dist web/dist`: total=6, passed=6
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
