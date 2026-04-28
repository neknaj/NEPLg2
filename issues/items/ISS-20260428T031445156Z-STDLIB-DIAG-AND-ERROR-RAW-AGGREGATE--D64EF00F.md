---
id: ISS-20260428T031445156Z-STDLIB-DIAG-AND-ERROR-RAW-AGGREGATE--D64EF00F
title: "stdlib diag and error raw aggregate detours fail under strict move checking"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/alloc/diag/error.nepl, stdlib/alloc/diag/diag.nepl, stdlib/core/result.nepl, stdlib/tests/diag.n.md, stdlib/tests/error.n.md"
---

# ISS-20260428T031445156Z-STDLIB-DIAG-AND-ERROR-RAW-AGGREGATE--D64EF00F: stdlib diag and error raw aggregate detours fail under strict move checking

## 概要

Latest strict move checking rejects diag/error helpers that store Diag or aggregate Result-like values in raw memory and repeatedly load fields from the same non-Copy raw place. stdlib/tests/diag.n.md and stdlib/tests/error.n.md now fail with D3100 moved raw memory / deallocating raw memory containing non-Copy values.

## 対象

- `stdlib/alloc/diag/error.nepl, stdlib/tests/diag.n.md, stdlib/tests/error.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/tests --no-tree -o tmp/stdlib-tests-static-followup-20260428.json -j 1` で `stdlib/tests/diag.n.md::doctest#1/#2` と `stdlib/tests/error.n.md::doctest#1/#2/#3` が D3100 になった。
- 主な失敗は `use of moved raw memory place: d_mem`、`deallocating raw memory place containing non-Copy value: notes_mem`、`deallocating raw memory place containing non-Copy value: items_mem`、`use of moved raw memory place: r0_mem`。
- `Diag` や diagnostic container を raw memory に置いて field を繰り返し `load` する実装が、最新の strict move checking で non-Copy aggregate の二重 move / live payload dealloc として表面化している。

## 問題

Latest strict move checking rejects diag/error helpers that store Diag or aggregate Result-like values in raw memory and repeatedly load fields from the same non-Copy raw place. stdlib/tests/diag.n.md and stdlib/tests/error.n.md now fail with D3100 moved raw memory / deallocating raw memory containing non-Copy values.

## 影響

stdlib diagnostic helpers are part of error reporting and self-host infrastructure. Leaving them as raw aggregate detours blocks clean stdlib verification and encourages weakening D3100 instead of removing unsafe aggregate decomposition patterns.

## 修正方針

Replace raw aggregate detours with borrowed field projection or owned decomposition that does not repeatedly load non-Copy aggregates from raw memory. Keep the move checker strict and add focused diag/error doctest regressions.

## 対応結果

- `StdErrorKind` / `DiagLevel` / `Span` / `DiagKind` / `Diag` に Copy/Clone capability を明示し、診断値本体を `Vec<Diag>` の要素として安全に観察できる軽量値へ寄せた。
- `Diag.notes` / `Diag.help` は owning `Vec<str>` ではなく、改行済み text block として保持する形へ変更した。これにより `Diag` 本体を non-Copy owner の集合体にせず、`Diags` の backing store 走査でも D3100 を誘発しない。
- `diag_with_span` / `diag_with_source` / `diag_add_note` / `diag_add_help` / `diag_to_string` / `diags_to_string` / `diags_len` / `diags_has_errors` から raw aggregate detour を削除し、`field::get_ref` と Copy field read に置き換えた。
- `Outcome` / `Diag` の観察用参照 overload を追加し、`stdlib/tests/error.n.md` の raw temp fixture を通常 API 呼び出しに置き換えた。

## 検証

Run node nodesrc/tests.js -i stdlib/tests/diag.n.md -i stdlib/tests/error.n.md --no-tree -o tmp/stdlib-diag-error-after-fix.json -j 1 and node nodesrc/issues.js check.

## 実施した検証

- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/tests/diag.n.md -i stdlib/tests/error.n.md -i stdlib/alloc/diag/error.nepl -i stdlib/alloc/diag/diag.nepl --no-tree -o tmp/diag-all-after-dead-helper-removal.json -j 1`: `total=7`, `passed=7`
- `node nodesrc/tests.js -i stdlib/core/result.nepl --no-tree -o tmp/result-after-stderrorkind-copy.json -j 1`: `total=7`, `passed=7`
- `node nodesrc/tests.js -i stdlib/tests --no-tree -o tmp/stdlib-tests-diag-error-after-fix-j4.json -j 4`: `total=80`, `passed=80`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-all-diag-error-after-fix.json -j 4`: 15 分で timeout。partial JSON は `completed_results=0/422` のため、完走結果は得られていない。
