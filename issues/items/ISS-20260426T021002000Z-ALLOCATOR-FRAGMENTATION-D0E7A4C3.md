---
id: ISS-20260426T021002000Z-ALLOCATOR-FRAGMENTATION-D0E7A4C3
title: "stdlib allocator does not coalesce free blocks"
area: stdlib
status: verified
resolved: true
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

## 対応

`dealloc_raw` を free list 先頭挿入から address order insertion に変更し、挿入した block が next / prev と隣接していれば total size と next pointer を更新して結合するようにした。
`alloc_raw` の first-fit / split は維持し、split 後の remainder は元 block の位置に残すことで address order を保つ。
`tests/stdlib/allocator_coalesce.n.md` を追加し、next 方向結合、prev 方向結合、page 末尾付近の fragmentation pattern で `mem_size` が増えないことを確認した。

## 検証

- `node nodesrc/tests.js -i stdlib/core/mem.nepl -i tests/stdlib/allocator_coalesce.n.md --no-tree -o tmp/allocator-coalesce.json -j 1`: 9/9 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/core/mem.nepl -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/hashmap.nepl -i stdlib/alloc/collections/hashset.nepl -i tests/stdlib/mem_fill.n.md -i tests/stdlib/capacity_stack.n.md -i tests/stdlib/allocator_coalesce.n.md --no-tree -o tmp/allocator-coalesce-suite-final.json -j 1`: 86/86 passed
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-allocator-coalesce.json`: 13/13 passed
