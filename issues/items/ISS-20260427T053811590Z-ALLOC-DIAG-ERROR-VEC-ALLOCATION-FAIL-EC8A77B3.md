---
id: ISS-20260427T053811590Z-ALLOC-DIAG-ERROR-VEC-ALLOCATION-FAIL-EC8A77B3
title: "alloc/diag/error が Vec allocation failure を unwrap_ok で trap する"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/diag/error.nepl, stdlib/tests/error.n.md, nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js"
---

# ISS-20260427T053811590Z-ALLOC-DIAG-ERROR-VEC-ALLOCATION-FAIL-EC8A77B3: alloc/diag/error が Vec allocation failure を unwrap_ok で trap する

## 概要

Diag/Diags constructors and add helpers allocate Vec values through unwrap_ok, so allocation or grow failure traps inside the diagnostic value model instead of returning a diagnostic-safe fallback or Result.

## 対象

- `stdlib/alloc/diag/error.nepl, stdlib/tests/error.n.md, nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`

## 根拠

- `diag_new` は `notes` / `help` の `Vec<str>` 生成を `unwrap_ok v::new<str>` で行っていた。
- `diag_add_note` / `diag_add_help` は `Vec<str>` への追記を `unwrap_ok v::push<str>` で行っていた。
- `diags_new` / `diags_one` / `diags_push` は `Vec<Diag>` の生成と追記を `unwrap_ok` していた。

## 問題

Diag/Diags constructors and add helpers allocate Vec values through unwrap_ok, so allocation or grow failure traps inside the diagnostic value model instead of returning a diagnostic-safe fallback or Result.

## 影響

Self-host lexer/parser/typecheck diagnostics depend on alloc/diag/error. Under memory pressure, the error-reporting path can fail by trap and hide the original compiler failure.

## 修正方針

Replace implementation unwrap_ok use with explicit Result matches. Keep existing facade APIs by using empty Vec sentinels or unchanged diagnostic fallbacks on allocation failure, and add a source policy regression for alloc/diag/error implementation code.

## 検証

- `node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/diag/error.nepl --no-tree -o tmp/diag-error-allocation-docs.json -j 1`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/tests/error.n.md --no-tree -o tmp/diag-error-allocation-focused.json -j 1`: 3/3 passed
- `node nodesrc/issues.js check`: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-diag-error-allocation.json -j 4`: 305/305 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-diag-error-allocation.json -j 4`: 418/418 passed

## 解決内容

- `StrVecPushRes` / `DiagVecPushRes` を追加し、push 後の owner と成否を同時に返せるようにした。
- `diag_empty_str_vec` / `diag_empty_diag_vec` を追加し、allocation failure 時に consumed owner を再利用しない空 Vec sentinel を明示した。
- `diag_push_str_vec` / `diag_push_diag_vec` を追加し、`v::push` の `Err` を trap ではなく `ok=false` と空 Vec sentinel に変換した。
- `diag_new` / `diags_new` は `v::new` を `match` し、失敗時は空 Vec sentinel を使うようにした。
- `diag_add_note` / `diag_add_help` / `diags_one` / `diags_push` から implementation `unwrap_ok` を除去した。
- `nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js` を追加し、CI/source policy と `doc/testing.md` に登録した。
