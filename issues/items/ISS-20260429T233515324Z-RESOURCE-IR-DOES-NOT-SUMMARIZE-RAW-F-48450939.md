---
id: ISS-20260429T233515324Z-RESOURCE-IR-DOES-NOT-SUMMARIZE-RAW-F-48450939
title: "Resource IR does not summarize raw fill helpers as initialized cell writes"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/src/resource, stdlib/core/mem.nepl"
---

# ISS-20260429T233515324Z-RESOURCE-IR-DOES-NOT-SUMMARIZE-RAW-F-48450939: Resource IR does not summarize raw fill helpers as initialized cell writes

## 概要

stdlib/core/mem.nepl doctests that call memset_u8 or fill_i32 and then load from the same allocation fail with resource.cell.uninit. Resource IR sees the later RawMemoryLoadCell but does not preserve the caller-visible initialized-cell transition performed by the raw fill helper.

## 対象

- `nepl-core/src/resource, stdlib/core/mem.nepl`

## 根拠

- `node nodesrc/run_doctest.js -i stdlib/core/mem.nepl -n 5 --dist web/dist` が `memset_u8` 後の `load_u8 add p 0` / `load_u8 add p 7` で `resource.cell.uninit` を報告した。
- `node nodesrc/run_doctest.js -i stdlib/core/mem.nepl -n 6 --dist web/dist` が `fill_i32` 後の `load_i32 add p 0` / `load_i32 add p 12` で `resource.cell.uninit` を報告した。
- どちらも `alloc_raw` で得た storage に compiler-owned `core/mem` helper が書き込んだ後の Copy load であり、true load-before-store ではない。
- RawMemoryLoadCell gate を弱めるのではなく、raw fill/write helper の caller-visible initialized-cell transition を Resource IR に表現する必要がある。

## 問題

stdlib/core/mem.nepl doctests that call memset_u8 or fill_i32 and then load from the same allocation fail with resource.cell.uninit. Resource IR sees the later RawMemoryLoadCell but does not preserve the caller-visible initialized-cell transition performed by the raw fill helper.

## 影響

Valid initialized raw memory use through compiler-owned core/mem helpers is rejected under the RawMemoryLoadCell gate. This blocks stdlib/core/mem documentation tests and any self-host or stdlib code that relies on fill helpers before reading initialized Copy cells.

## 修正方針

Add Resource IR lowering or function summaries for raw write/fill helpers so caller-visible raw byte/i32 writes mark the corresponding cells initialized, while retaining the existing diagnostics for overwriting live non-Copy cells and for true load-before-store.

## 検証

Add focused Resource IR regressions for alloc_raw -> memset_u8/fill_i32 -> load_u8/load_i32, run stdlib/core/mem.nepl doctests #5 and #6, and keep move_effect raw overwrite compile_fail cases passing.

## 発見経緯

`ISS-20260429T231611047Z-STD-TEST-ASSERTION-DISCARD-SOURCE-PO-B9226736` の std/test report 移行を検証中に発見した。該当 doctest は assertion report 形式には移行済みだが、compile phase で `resource.cell.uninit` になる。

`memset_u8 p 8 65` / `fill_i32 p 4 42` の後に同じ allocation から `load_u8` / `load_i32` するため、言語上は Copy cell が初期化済みとして扱われるべきである。Resource IR は true load-before-store を拒否し続ける必要があるが、compiler-owned `core/mem` raw fill helper の効果を caller 側の initialized-cell transition として表現できていない。

## 関連

- 親 issue: `ISS-20260425T000000Z-RV-CORE-009-58589A3F`
- 発見元 issue: `ISS-20260429T231611047Z-STD-TEST-ASSERTION-DISCARD-SOURCE-PO-B9226736`
- 関連計画: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 Resource check
