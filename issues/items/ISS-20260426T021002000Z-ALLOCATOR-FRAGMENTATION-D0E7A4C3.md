---
id: ISS-20260426T021002000Z-ALLOCATOR-FRAGMENTATION-D0E7A4C3
title: "stdlib allocator does not coalesce free blocks"
area: stdlib
status: open
resolved: false
priority: P1
type: performance
created: 2026-04-26
updated: 2026-04-26
target: stdlib/core/mem.nepl
source: doc/neplg2/pre_selfhost_performance_audit_20260426.md
---

# ISS-20260426T021002000Z-ALLOCATOR-FRAGMENTATION-D0E7A4C3: stdlib allocator does not coalesce free blocks

## 概要

`stdlib/core/mem.nepl` の allocator は free list を first-fit で探索し、`dealloc_raw` は解放ブロックを free list 先頭へ挿入するだけである。
隣接 free block の coalescing がないため、総空き容量が十分でも大きい連続領域を確保できず、memory growth が進む可能性がある。

## 根拠

- `mem.nepl:272` の `alloc_raw` は free list を線形探索して合うブロックを探す。
- `mem.nepl:379` の `dealloc_raw` は block size と next pointer を書き、free list head に挿入するだけで隣接 block を調べない。
- file header は `alloc_raw/free` が free list 探索で O(n) になり得ることを記述している。

## 問題

self-host compiler は token、AST、HIR、diagnostic、temporary string / Vec を大量に確保・破棄する。
coalescing がないと短命オブジェクトの churn で heap が断片化し、長い source や複数 file compile で memory growth と O(n) free list scan が増える。

## 影響

WASI 上の self-host CLI で大きなプロジェクトを処理すると、実際には解放済み memory があるのに `alloc_raw` が grow を繰り返す可能性がある。
また、free list が長くなるほど allocation latency が不安定になり、performance regression の原因を compiler pass と allocator のどちらに求めるべきか切り分けにくくなる。

## 修正方針

free list を address order で管理し、`dealloc_raw` 時に前後隣接 block を coalesce する。
最低限、coalescing できる block を結合する regression test を追加し、fragmentation pattern で追加 page growth が不要になることを確認する。

## 検証

- 小ブロックを複数確保して連続解放し、その合計サイズの大ブロックを再確保できる doctest。
- allocation 回数と `mem_size` の増加を確認する stress fixture。
