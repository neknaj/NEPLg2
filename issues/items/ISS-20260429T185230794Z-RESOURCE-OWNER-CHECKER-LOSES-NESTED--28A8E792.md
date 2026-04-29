---
id: ISS-20260429T185230794Z-RESOURCE-OWNER-CHECKER-LOSES-NESTED--28A8E792
title: "Resource owner checker loses nested collection storage owner from aggregate helper returns"
area: core
status: wontfix
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/src/resource, tests/stdlib/hash_collection_rehash.n.md, stdlib/alloc/collections/hashmap.nepl"
---

# ISS-20260429T185230794Z-RESOURCE-OWNER-CHECKER-LOSES-NESTED--28A8E792: Resource owner checker loses nested collection storage owner from aggregate helper returns

## 概要

HashMap was redesigned to keep storage as owned Vec fields. Focused .n.md rehash tests that build a HashMap in a helper and return it still fail in main with resource.raw.ownership_violation on Temporary(...).Field index 3, even when the caller binds the result and later calls free. The failure shows the strict owner checker is not transferring nested collection storage obligations from an aggregate-returning helper temporary into the receiving local.

## 対象

- `nepl-core/src/resource, tests/stdlib/hash_collection_rehash.n.md, stdlib/alloc/collections/hashmap.nepl`

## 根拠

- 未記入

## 問題

HashMap was redesigned to keep storage as owned Vec fields. Focused .n.md rehash tests that build a HashMap in a helper and return it still fail in main with resource.raw.ownership_violation on Temporary(...).Field index 3, even when the caller binds the result and later calls free. The failure shows the strict owner checker is not transferring nested collection storage obligations from an aggregate-returning helper temporary into the receiving local.

## 影響

Self-host collection tests cannot use natural builder/helper functions for owning aggregate values without false owner leaks. This pushes tests and stdlib code toward awkward manual chains and blocks larger collection regression fixtures unless the checker can prove nested owner transfer across function returns.

## 修正方針

Add ResourceIR regressions for a function returning a struct that contains multiple owned Vec-like fields, bind the return value in the caller, borrow-read it, and free it. Then preserve nested owner obligations from function-call temporaries into let bindings and assignments without weakening leak diagnostics.

## 検証

Re-enable helper-return style in tests/stdlib/hash_collection_rehash.n.md HashMap doctests and run node nodesrc/run_doctest.js -i tests/stdlib/hash_collection_rehash.n.md -n 1 --dist web/dist plus the new ResourceIR focused tests.

## 2026-04-30 再調査結果

この issue は core Resource owner checker の新規不具合として追加したが、再調査により原因は `tests/stdlib/hash_collection_rehash.n.md` 側の direct assertion import 誤りだった。

`std/test::assert_eq_i32` は `TestAssertion` を返す report 集約用 API であり、戻り値を `checks_push` / `run_checks` へ渡さずに捨てると owner obligation が残る。一方、この doctest の用途は即時失敗型の assertion なので `core/test::assert_eq_i32` を使うべきだった。

同じ helper-return 形を `core/test` import で再実行したところ pass したため、`HashMap` の helper return で nested storage owner が失われるという判断は誤りだった。この issue は実装対象から外し、誤検出の調査記録として `wontfix` / `resolved` にする。

検証:

- `node nodesrc/run_test.js` に helper-return HashMap source を渡し、`core/test::assert_eq_i32` import で `HashMap` を helper から返して caller が `free` するケース: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/hash_collection_rehash.n.md -n 1 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/hash_collection_rehash.n.md -n 3 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/hash_collection_rehash.n.md -n 5 --dist web/dist`: pass
