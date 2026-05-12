---
id: ISS-20260512T210823136Z-COLLECTION-COMPILER-FIXTURES-FAIL-AF-70CD17C5
title: "collection compiler fixtures fail after stdlib API and layout changes"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-13
target: "tests/compiler/neplg2.n.md, tests/compiler/sizeof.n.md"
---

# ISS-20260512T210823136Z-COLLECTION-COMPILER-FIXTURES-FAIL-AF-70CD17C5: collection compiler fixtures fail after stdlib API and layout changes

## 概要

During diagnostic D4 coverage verification, the affected-suite run still failed tests/compiler/neplg2.n.md::doctest#33 with type.overload.type_args_mismatch and tests/compiler/sizeof.n.md::doctest#7 with return value 1 instead of 0. These failures were already present before adding diag_code metadata.

## 対象

- `tests/compiler/neplg2.n.md, tests/compiler/sizeof.n.md`

## 根拠

- `tests/compiler/neplg2.n.md::doctest#33` は現在の `List.get` が `(&List<T>, i32)` を受け取る borrowed observer であるにもかかわらず、古い owner-consuming 形の `get<i32> lst 10` を呼んでいた。そのため overload selection が `type.overload.type_args_mismatch` で失敗していた。
- `tests/compiler/sizeof.n.md::doctest#7` は `Vec<T>` が `VecStorageState` と `MemPtr<T>` を持つ現在の layout へ移行した後も、旧 `size_of<Vec<i32>> == 12` / `size_of<Stack<i32>> == 8` を期待していた。そのため実行時に最初の size check が失敗していた。
- どちらも compiler の type/size_of 実装を弱めるべき問題ではなく、fixture が現在の stdlib API / layout contract に追従していない問題だった。

## 問題

During diagnostic D4 coverage verification, the affected-suite run still failed tests/compiler/neplg2.n.md::doctest#33 with type.overload.type_args_mismatch and tests/compiler/sizeof.n.md::doctest#7 with return value 1 instead of 0. These failures were already present before adding diag_code metadata.

## 影響

The diagnostic metadata change can be verified without new failures, but compiler fixture coverage is not clean. Collection API/layout tests can mask real static-check regressions if stale expectations remain.

## 修正方針

Audit current List get API and collection struct layout, then update fixtures or compiler layout logic according to the current stdlib contract. Do not weaken overload or size_of checks to make the tests pass.

## 検証

Run the two focused doctests and the full tests/compiler/neplg2.n.md and tests/compiler/sizeof.n.md files.

## 対応結果

2026-05-13 に、collection compiler fixture を現在の stdlib contract へ更新した。

- `list_get_out_of_bounds_err` は `get<i32> &lst 10` に変更し、borrowed read 後に `free<i32> lst` で owner を閉じるようにした。これにより List observer が owner を消費しない設計を fixture 側でも明示する。
- `sizeof_collection_structs` は旧固定値ではなく、`Vec<T>` の現在の field layout（`len`、`cap`、`VecStorageState`、`MemPtr<T>`）と `Stack<T>` の現在の field layout（`len`、`cap`、`Vec<Option<T>>`）から expected size を計算するようにした。
- `HashMap` / `HashSet` は現在の hasher parameter 付き型 `HashMap<i32, i32, DefaultHash32>` / `HashSet<i32, DefaultHash32>` を使うようにした。

検証:

- `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 33 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/sizeof.n.md -n 7 --dist web/dist`: pass
- `node nodesrc/tests.js -i tests/compiler/neplg2.n.md --no-tree -o tmp/agent1-collection-fixture-neplg2.json -j 1 --dist web/dist`: 45/45 pass
- `node nodesrc/tests.js -i tests/compiler/sizeof.n.md --no-tree -o tmp/agent1-collection-fixture-sizeof.json -j 1 --dist web/dist`: 9/9 pass
