---
id: ISS-20260426T221226565Z-BTREEMAP-AND-BTREESET-INSERT-TRAP-ON-F1DACB01
title: "BTreeMap and BTreeSet insert trap on grow allocation failure"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/btreeset.nepl, stdlib/tests/btreemap.n.md, stdlib/tests/btreeset.n.md, nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js"
---

# ISS-20260426T221226565Z-BTREEMAP-AND-BTREESET-INSERT-TRAP-ON-F1DACB01: BTreeMap and BTreeSet insert trap on grow allocation failure

## 概要

BTreeMap.insert and BTreeSet.insert return Result but use unwrap_ok when the backing sorted-array grows. If allocation fails, grow returns Diag::OutOfMemory and insert immediately traps instead of returning Err.

## 対象

- `stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/btreeset.nepl, stdlib/tests/btreemap.n.md, stdlib/tests/btreeset.n.md, nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js`

## 根拠

- `stdlib/alloc/collections/btreemap.nepl` の `insert` は `grow<.K,.V> hm` の結果を `unwrap_ok` していた。
- `stdlib/alloc/collections/btreeset.nepl` の `insert` は `btreeset_grow<.T> set0` の結果を `unwrap_ok` していた。
- どちらの grow helper も `Diag::OutOfMemory` を返せる `Result` API なので、public `insert` 側で `Err` を値として返す必要がある。

## 問題

BTreeMap.insert and BTreeSet.insert return Result but use unwrap_ok when the backing sorted-array grows. If allocation fails, grow returns Diag::OutOfMemory and insert immediately traps instead of returning Err.

## 影響

Self-host compiler support code can use sorted-array collections for small ordered tables, but allocation failure during insert becomes unreachable. That violates the stdlib Result policy and hides RV-STDLIB-010 unsafe helper debt behind a public Result API.

## 修正方針

Move the post-growth insertion body into helpers that operate on an already-capable collection, and make public insert match grow. Propagate Err directly instead of using unwrap_ok.

## 解決内容

- `btreemap_insert_ready` / `btreeset_insert_ready` を追加し、capacity が足りている collection への lower_bound / shift / store を helper に分離した。
- public `insert` は満杯時に `grow` / `btreeset_grow` を `match` し、`Result::Err d` を `err<..., Diag> d` としてそのまま返すようにした。
- grow 境界の 9 件目 insert regression を `stdlib/tests/btreemap.n.md` / `stdlib/tests/btreeset.n.md` に追加した。
- `nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js` を追加し、BTreeMap/BTreeSet の insert/grow 経路に `unwrap_ok` / `uwok` / `unreachable` が戻らないことを CI source policy で監視するようにした。

## 検証

- `node nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-tree -o tmp/btree-insert-grow-focused.json -j 1`: 8/8 passed
- `node nodesrc/tests.js -i tests/stdlib/btree_array_cost.n.md --no-tree -o tmp/btree-array-cost-after-grow-result.json -j 1`: 6/6 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap.nepl -i stdlib/alloc/collections/btreeset.nepl --no-tree -o tmp/btree-insert-grow-docs.json -j 1`: 15/15 passed
- `node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/pipe-collections-after-btree-grow-result.json -j 1`: 8/8 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-btree-grow-result.json -j 4`: 282/282 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-btree-grow-result.json -j 4`: 416/416 passed
- remote main の `590722d core: retain aggregate construction borrows` を取り込んだ後に `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-tree -o tmp/btree-insert-grow-after-remote-build.json -j 1`: 8/8 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap.nepl -i stdlib/alloc/collections/btreeset.nepl --no-tree -o tmp/btree-insert-grow-docs-after-remote-build.json -j 1`: 15/15 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-btree-grow-after-remote-build.json -j 4`: 282/282 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-btree-grow-after-remote-build.json -j 4`: 416/416 passed
- remote main の `a5db7f4 core: preserve match payload borrow origins` へ rebase した後に `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-tree -o tmp/btree-insert-grow-after-a5db-build.json -j 1`: 8/8 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap.nepl -i stdlib/alloc/collections/btreeset.nepl --no-tree -o tmp/btree-insert-grow-docs-after-a5db-build.json -j 1`: 15/15 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-btree-grow-after-a5db-build.json -j 4`: 282/282 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-btree-grow-after-a5db-build.json -j 4`: 416/416 passed
- remote main の `c4c25d0 core: lower LLVM reference address operations` へ rebase した後に `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-tree -o tmp/btree-insert-grow-after-c4c25d0-build.json -j 1`: 8/8 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap.nepl -i stdlib/alloc/collections/btreeset.nepl --no-tree -o tmp/btree-insert-grow-docs-after-c4c25d0-build.json -j 1`: 15/15 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-btree-grow-after-c4c25d0-build.json -j 4`: 282/282 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-btree-grow-after-c4c25d0-build.json -j 4`: 416/416 passed
- remote main の `795bede core: emit LLVM allocator helper dependencies` へ rebase した後に `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-tree -o tmp/btree-insert-grow-after-795bede-build.json -j 1`: 8/8 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap.nepl -i stdlib/alloc/collections/btreeset.nepl --no-tree -o tmp/btree-insert-grow-docs-after-795bede-build.json -j 1`: 15/15 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-btree-grow-after-795bede-build.json -j 4`: 282/282 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-btree-grow-after-795bede-build.json -j 4`: 416/416 passed
