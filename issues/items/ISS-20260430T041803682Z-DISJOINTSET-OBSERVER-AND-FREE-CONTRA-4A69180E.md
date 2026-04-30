---
id: ISS-20260430T041803682Z-DISJOINTSET-OBSERVER-AND-FREE-CONTRA-4A69180E
title: "DisjointSet observer and free contract leave owner obligations unresolved"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/disjoint_set.nepl, stdlib/tests/disjoint_set.n.md, tests/stdlib/disjoint_set_collections.n.md, nodesrc/test_stdlib_disjoint_set_borrowed_observers.js, nodesrc/test_stdlib_disjoint_set_no_unsafe_unwraps.js"
---

# ISS-20260430T041803682Z-DISJOINTSET-OBSERVER-AND-FREE-CONTRA-4A69180E: DisjointSet observer and free contract leave owner obligations unresolved

## 概要

DisjointSet.len consumes the owner while len_ref remains as a duplicate borrowed surface, tests query DisjointSet values without closing owners, and DisjointSet.free reads parent/sizes through field::get_ref so Resource IR cannot prove the owner fields were deallocated.

## 対象

- `stdlib/alloc/collections/disjoint_set.nepl, stdlib/tests/disjoint_set.n.md, tests/stdlib/disjoint_set_collections.n.md, nodesrc/test_stdlib_disjoint_set_borrowed_observers.js, nodesrc/test_stdlib_disjoint_set_no_unsafe_unwraps.js`

## 根拠

- `stdlib/alloc/collections/disjoint_set.nepl` had `fn len <(DisjointSet)->i32>` while also keeping `len_ref` as a borrowed duplicate surface.
- Existing doctests and `.n.md` tests called `find` / `same` / `size` and then returned without closing the `DisjointSet` owners.
- `DisjointSet.free` read `parent` and `sizes` via `field::get_ref`, so Resource IR still reported both array owner fields as live after `free`.
- Focused DisjointSet doctests failed strict owner checking with `resource.owner.leak` before this fix.

## 問題

DisjointSet.len consumes the owner while len_ref remains as a duplicate borrowed surface, tests query DisjointSet values without closing owners, and DisjointSet.free reads parent/sizes through field::get_ref so Resource IR cannot prove the owner fields were deallocated.

## 影響

DisjointSet doctests and collection tests fail strict Resource IR owner checking with parent/sizes leaks. Read-only observation and cleanup are not represented as separate owner events.

## 修正方針

Change len to borrow &DisjointSet, remove len_ref, make free consume parent/sizes owner fields, update doctests/tests to free observed owners, and add source-policy regression coverage.

## 検証

Run DisjointSet doctests, stdlib/tests/disjoint_set.n.md, tests/stdlib/disjoint_set_collections.n.md, source-policy regressions, and issue checks.

確認済み:

- `node nodesrc/test_stdlib_disjoint_set_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_disjoint_set_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/disjoint_set.nepl --no-tree -o tmp/disjoint-set-borrowed-observers-doctests.json -j 1` (`total=6`, `passed=6`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md --no-tree -o tmp/disjoint-set-borrowed-observers-stdlib-tests.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/disjoint_set_collections.n.md --no-tree -o tmp/disjoint-set-borrowed-observers-collections-tests.json -j 1` (`total=3`, `passed=3`, `failed=0`)

## 修正内容

- `DisjointSet.len` を `&DisjointSet` receiver に変更し、重複 `len_ref` を削除した。
- `DisjointSet.free` を `field::get` で `parent` / `sizes` owner fields を消費してから `dealloc_raw` する実装へ変更した。
- doctest / stdlib test / collection test を、borrowed query 後に同じ owner を `free` する形へ更新した。
- `nodesrc/test_stdlib_disjoint_set_borrowed_observers.js` を追加し、by-value `len` / `len_ref` / owner cleanup 漏れの再発を検出するようにした。
- 既存の DisjointSet source-policy に、`free` が owned array fields を borrow-read しないことの検査を追加した。
