---
id: ISS-20260514T190018299Z-DIAGS-ERROR-OBSERVER-SCANS-VEC-STORA-5ABF687A
title: "Diags error observer scans Vec storage through raw memory"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/diag/error/diags.nepl, nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js"
---

# ISS-20260514T190018299Z-DIAGS-ERROR-OBSERVER-SCANS-VEC-STORA-5ABF687A: Diags error observer scans Vec storage through raw memory

## 概要

alloc/diag/error/diags.nepl keeps diags_has_errors as a raw Vec storage scanner even though Diag is Copy and the existing Vec observer API can read it safely. The module imports core/mem/raw only to compute data_mem_ptr/mem_ptr_addr and load<Diag>.

## 対象

- `stdlib/alloc/diag/error/diags.nepl, nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`

## 根拠

- 修正前の `stdlib/alloc/diag/error/diags.nepl` は `core/mem` / `core/mem/internal` / `core/mem/raw` を import していた。
- `diags_has_errors(&Diags)` は `Vec<Diag>` を借用したあと、`v::data_mem_ptr<Diag>`、`mem_ptr_addr`、`load<Diag>`、`size_of<Diag>` で内部 storage を直接走査していた。
- `Diag` は `Copy` として定義済みであり、error-level 判定は owner move を必要としないため、`v::get<Diag>(&Vec<Diag>, i32)` の Copy-safe observer で十分である。

## 問題

alloc/diag/error/diags.nepl keeps diags_has_errors as a raw Vec storage scanner even though Diag is Copy and the existing Vec observer API can read it safely. The module imports core/mem/raw only to compute data_mem_ptr/mem_ptr_addr and load<Diag>.

## 影響

The Diags owner helper exposes raw storage observation where a safe Copy observer is sufficient. This widens raw-memory-boundary evidence in diagnostic modules and lets future diagnostic queries bypass Vec bounds/storage-state checks.

## 修正方針

Rewrite diags_has_errors to borrow Diags.items and traverse with v::len and v::get<Diag>. Remove core/mem/raw imports from diags.nepl, invert the source policy so Diags owner helpers must not carry raw storage scans, and keep by-value observers closing the Diags owner.

## 検証

Run the diag source policy, stdlib diag/error doctests, issues check, and git diff --check.

## 解決

2026-05-15 に修正済み。

- `stdlib/alloc/diag/error/diags.nepl` から `core/mem` / `core/mem/internal` / `core/mem/raw` import を削除した。
- `diags_has_errors` は borrowed `Vec<Diag>` を `v::len<Diag>` と `v::get<Diag>` で走査し、`Option<Diag>` と `DiagLevel` を exhaustive match する。
- by-value `diags_has_errors(Diags)` は従来通り観測後に `diags_free ds` を呼び、`Diags` owner を閉じる。
- `nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js` は、Diags read-only observer が raw Vec storage scan を再導入しないことを検査するように更新した。

検証:

- `node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib/tests/error.n.md --no-tree -o tmp/agent1-diags-error-observer-vec-boundary-error-tests.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/diag/error/diags.nepl --no-tree -o tmp/agent1-diags-error-observer-vec-boundary-module.json -j 1 --dist web/dist --assert-io`

## 関連

- [ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84](./ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md)
- [ISS-20260514T185052018Z-DIAG-RENDERER-READS-DIAGS-VEC-STORAG-D85114C9](./ISS-20260514T185052018Z-DIAG-RENDERER-READS-DIAGS-VEC-STORAG-D85114C9.md)
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
