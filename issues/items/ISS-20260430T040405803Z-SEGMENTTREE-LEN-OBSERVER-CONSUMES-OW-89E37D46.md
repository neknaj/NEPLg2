---
id: ISS-20260430T040405803Z-SEGMENTTREE-LEN-OBSERVER-CONSUMES-OW-89E37D46
title: "SegmentTree len observer and free contract leave owner obligations unresolved"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/segment_tree.nepl, stdlib/tests/segment_tree.n.md, tests/stdlib/segment_tree_collections.n.md, nodesrc/test_stdlib_segment_tree_borrowed_observers.js"
---

# ISS-20260430T040405803Z-SEGMENTTREE-LEN-OBSERVER-CONSUMES-OW-89E37D46: SegmentTree len observer and free contract leave owner obligations unresolved

## 概要

SegmentTree.len is a read-only observer but takes SegmentTree by value. Calling len moves the owner even though the API only reads n, so callers cannot keep using or freeing the tree after observing its length. When tests were updated to explicitly free observed owners, Resource IR also exposed that SegmentTree.free read the data owner through field::get_ref and therefore did not discharge the owner obligation.

## 対象

- `stdlib/alloc/collections/segment_tree.nepl, stdlib/tests/segment_tree.n.md, tests/stdlib/segment_tree_collections.n.md, nodesrc/test_stdlib_segment_tree_borrowed_observers.js`

## 根拠

- `stdlib/alloc/collections/segment_tree.nepl` had `fn len <(SegmentTree)->i32>` even though the implementation only read field `n`.
- The module also kept a duplicate `len_ref`, leaving two observer surfaces where only the borrowed one is sound under the current owner model.
- Existing doctests and `.n.md` tests queried SegmentTree values without explicitly closing the owner afterward.
- After changing tests to call `free`, focused doctests failed with `resource.owner.leak` for the `data` field, showing that `free` itself borrowed the owner field instead of consuming it.

## 問題

SegmentTree.len consumed `SegmentTree` for an O(1) read-only observation, and SegmentTree.free did not prove to Resource IR that the `data` owner field was moved into `dealloc_raw`. The combination made tests unable to express the intended contract: observe through a borrow, then close the same owner.

## 影響

The current contract hides owner leaks in doctests and user code. Under the mandatory memory-safety model, read-only collection observers must borrow owners, and cleanup functions must consume owner fields so Resource IR can distinguish observation, transfer, and deallocation.

## 修正方針

Change SegmentTree.len to borrow &SegmentTree, remove the redundant observer surface, make free consume the `data` owner field, update doctests/tests to observe through borrows and explicitly free owners, and add source-policy regression coverage.

## 検証

Run SegmentTree doctests, stdlib/tests/segment_tree.n.md, tests/stdlib/segment_tree_collections.n.md, source-policy regressions, and issue checks.

確認済み:

- `node nodesrc/test_stdlib_segment_tree_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/segment_tree.nepl --no-tree -o tmp/segment-tree-borrowed-len-doctests.json -j 1` (`total=5`, `passed=5`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/segment-tree-borrowed-len-stdlib-tests.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/segment-tree-borrowed-len-collections-tests.json -j 1` (`total=2`, `passed=2`, `failed=0`)

## 修正内容

- `SegmentTree.len` を `&SegmentTree` receiver に変更し、by-value の `len_ref` 重複 surface を削除した。
- `SegmentTree.free` を `field::get_ref` ではなく `field::get` で `data` owner field を消費してから `dealloc_raw` する実装へ変更した。
- doctest / stdlib test / collection test を、`len &st` / `sum_range &st` のあとに同じ owner を `free` する形へ更新した。
- `nodesrc/test_stdlib_segment_tree_borrowed_observers.js` を追加し、by-value observer と重複 `len_ref` の再発を検出するようにした。
- 既存の SegmentTree source-policy に、`free` が owner field を borrow-read しないことの検査を追加した。
