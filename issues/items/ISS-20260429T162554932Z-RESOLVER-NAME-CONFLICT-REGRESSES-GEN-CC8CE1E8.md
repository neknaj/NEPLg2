---
id: ISS-20260429T162554932Z-RESOLVER-NAME-CONFLICT-REGRESSES-GEN-CC8CE1E8
title: "Generics and shadowing doctest fixtures are stale after prelude and std/test updates"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "tests/compiler/generics.n.md, tests/compiler/shadowing.n.md"
---

# ISS-20260429T162554932Z-RESOLVER-NAME-CONFLICT-REGRESSES-GEN-CC8CE1E8: Generics and shadowing doctest fixtures are stale after prelude and std/test updates

## 概要

On current main after MemPtr explicit Clone merge, `node nodesrc/tests.js -i tests/compiler/generics.n.md --no-tree -o tmp/memptr-explicit-clone-generics-after-merge.json -j 1 --dist web/dist` reported total=24 passed=16 failed=8. The failing generics doctests compile-failed early with `resolve.item.name_conflict`. The wider `tests/compiler` run also reported `tests/compiler/shadowing.n.md::doctest#22` as expected compile_fail, but compiled successfully.

Investigation showed these were stale fixture problems rather than a resolver implementation bug:

- `generics.n.md` imported `core/mem` or `core/math` with open imports while also defining a local `Option`. `core/math -> core/field -> core/mem -> core/option` brings the stdlib `Option` into the fixture's item set, so the local `Option` definition legitimately conflicts.
- The `std_test_noshadow_same_signature_redefinition_is_error` fixture still redefined the old `assert_eq_i32` signature returning `Result<(), str>`. Current `std/test` returns `TestAssertion`, so the fixture was no longer same-signature and correctly compiled.

## 対象

- `tests/compiler/generics.n.md, tests/compiler/shadowing.n.md`

## 根拠

- 未記入

## 問題

Generics and shadowing fixture code had not been updated for the current prelude/import graph and `std/test` assertion API. The tests therefore failed for reasons unrelated to the behavior they were meant to cover.

## 影響

The compiler doctest gate was not clean, and real generic/typecheck coverage was hidden behind stale test setup. Self-host work relies on generics and shadowing semantics, so these fixtures must stay aligned with current stdlib names and signatures.

## 修正方針

Keep generics fixtures independent from default prelude/open stdlib imports by using `#no_prelude`, removing unnecessary `core/mem` open imports, and avoiding local `Option` names in the few cases that intentionally import `core/math`. Update the shadowing fixture to redefine the current `std/test` `assert_eq_i32` same-signature overload returning `TestAssertion`.

## 検証

node nodesrc/tests.js -i tests/compiler/generics.n.md --no-tree -o tmp/generics-fixture-cleanup6.json -j 1 --dist web/dist; node nodesrc/tests.js -i tests/compiler/shadowing.n.md --no-tree -o tmp/shadowing-fixture-cleanup3.json -j 1 --dist web/dist

## 解決

2026-04-30:

- `tests/compiler/generics.n.md` の各 doctest に `#no_prelude` を追加し、prelude 経由の stdlib item が generic fixture の local item と衝突しないようにした。
- 不要な `core/mem` open import を削除した。
- `core/math` が必要な fixture は alias import にし、`m::add` を使うようにした。
- `core/math` import が必要かつ local `Option` を定義していた fixture は、generic 型の検証目的を保ったまま `LocalOption` に改名した。
- `tests/compiler/shadowing.n.md` の `std_test_noshadow_same_signature_redefinition_is_error` を、現在の `std/test` と同じ `(i32, i32) -> TestAssertion` signature の再定義へ更新した。

## 解決時の検証

- `node nodesrc/tests.js -i tests/compiler/generics.n.md --no-tree -o tmp/generics-fixture-cleanup6.json -j 1 --dist web/dist`: total=24, passed=24
- `node nodesrc/tests.js -i tests/compiler/shadowing.n.md --no-tree -o tmp/shadowing-fixture-cleanup3.json -j 1 --dist web/dist`: total=27, passed=27
- `node nodesrc/tests.js -i tests/compiler/generics.n.md -i tests/compiler/shadowing.n.md --no-tree -o tmp/generics-shadowing-fixture-cleanup-final.json -j 1 --dist web/dist`: total=51, passed=51
- `node nodesrc/tests.js -i tests/compiler --no-tree -o tmp/compiler-after-generics-shadowing-cleanup.json -j 4 --dist web/dist`: total=649, passed=635, failed=14。残りは `drop_overwrite` の `resource.borrow.assign_during_shared` と ResourceIR owner obligation 系で、generics/shadowing fixture 由来の失敗は解消。
