---
id: ISS-20260514T185052018Z-DIAG-RENDERER-READS-DIAGS-VEC-STORAG-D85114C9
title: "Diag renderer reads Diags Vec storage through raw memory"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/diag/diag.nepl, nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js"
---

# ISS-20260514T185052018Z-DIAG-RENDERER-READS-DIAGS-VEC-STORAG-D85114C9: Diag renderer reads Diags Vec storage through raw memory

## 概要

alloc/diag/diag.nepl is the public diagnostic string renderer, but diags_to_string imports core/mem/raw and scans Diags.items by converting Vec storage to a raw address and calling load<Diag>. This keeps raw storage identity in the renderer instead of using the existing Copy-safe Vec observer boundary.

## 対象

- `stdlib/alloc/diag/diag.nepl, nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`

## 根拠

- 修正前の `stdlib/alloc/diag/diag.nepl` は renderer file でありながら `core/mem` / `core/mem/internal` / `core/mem/raw` を import していた。
- `diags_to_string(&Diags)` は `field::get_ref ds "items"` で `Vec<Diag>` を借用したあと、`data_mem_ptr<Diag>` と `mem_ptr_addr` で raw address を取り出し、`diags_to_string_loop` が `load<Diag>` で直接走査していた。
- `Diag` は `Copy` として定義されており、現行 `Vec` には `.T: Copy` 用の `get<T>(&Vec<T>, i32) -> Option<T>` があるため、renderer が raw storage layout を知る必要はない。

## 問題

alloc/diag/diag.nepl is the public diagnostic string renderer, but diags_to_string imports core/mem/raw and scans Diags.items by converting Vec storage to a raw address and calling load<Diag>. This keeps raw storage identity in the renderer instead of using the existing Copy-safe Vec observer boundary.

## 影響

Safe diagnostic formatting code remains a raw-memory-boundary implementation point, so Stage 6 cannot audit Diags storage separately from rendering. Future renderer changes can accidentally depend on raw Vec layout and bypass Vec.get range/storage-state checks.

## 修正方針

Move diags_to_string to the Copy-safe Vec observer API: keep Diags as the owner of Vec<Diag>, borrow the Vec, use v::len and v::get<Diag> for ordered traversal, remove core/mem/raw imports from the renderer, and add source policy coverage that forbids raw memory evidence in alloc/diag/diag.nepl.

## 検証

Run the diag renderer doctest, the diag source policy, issues check, and git diff --check.

## 解決

2026-05-15 に修正済み。

- `stdlib/alloc/diag/diag.nepl` から `core/mem` / `core/mem/internal` / `core/mem/raw` import を削除した。
- `diags_to_string` は `Diags.items` の borrowed `Vec<Diag>` を受け取り、`v::len<Diag>` と `v::get<Diag>` で順序を保って走査する。
- `diags_to_string_loop` は raw address と `load<Diag>` ではなく、`Option<Diag>` の exhaustive match で traversal を進める。`Diag` の Copy contract は `Vec.get` の trait bound で静的に検査される。
- `nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js` に renderer file の raw memory import / raw Vec storage scan 禁止を追加した。

検証:

- `node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib/tests/diag.n.md --no-tree -o tmp/agent1-diag-renderer-vec-boundary-diag-tests.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/tests/diag.n.md -i stdlib/tests/error.n.md --no-tree -o tmp/agent1-diag-renderer-vec-boundary-diag-error-tests.json -j 1 --dist web/dist --assert-io`

補足:

- `node nodesrc/tests.js -i stdlib/alloc/diag/diag.nepl --no-tree -o tmp/agent1-diag-renderer-vec-boundary-module-doctests.json -j 1 --dist web/dist --assert-io` は runnable doctest が無いため scan error になり、実装検証としては採用していない。

## 関連

- [ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84](./ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md)
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
