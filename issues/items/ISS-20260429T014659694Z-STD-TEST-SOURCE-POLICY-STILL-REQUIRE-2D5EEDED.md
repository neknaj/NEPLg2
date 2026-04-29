---
id: ISS-20260429T014659694Z-STD-TEST-SOURCE-POLICY-STILL-REQUIRE-2D5EEDED
title: "std/test source policy still requires removed Vec accumulator"
area: nodesrc
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js, stdlib/std/test.nepl"
---

# ISS-20260429T014659694Z-STD-TEST-SOURCE-POLICY-STILL-REQUIRE-2D5EEDED: std/test source policy still requires removed Vec accumulator

## 概要

GitHub Actions Source policy regressions fail because nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js still requires stdlib/std/test.nepl to import alloc/collections/vec and define Vec<Result<(),str>> accumulator helpers even though std/test has migrated to the Checks value accumulator.

## 対象

- `nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js, stdlib/std/test.nepl`

## 根拠

- 未記入

## 問題

GitHub Actions Source policy regressions fail because nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js still requires stdlib/std/test.nepl to import alloc/collections/vec and define Vec<Result<(),str>> accumulator helpers even though std/test has migrated to the Checks value accumulator.

## 影響

CI build job fails before full doctest/rust jobs run, and the policy now encourages reintroducing the raw Vec<Result<(),str>> backing-store design that Resource IR raw-memory gates intentionally removed.

## 修正方針

Update the std/test source policy to assert the current Checks accumulator contract, forbid the old Vec<Result<(),str>> raw-scan helpers, and keep unsafe unwrap bans.

## 検証

Run node nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js, node nodesrc/issues.js check, and the same local focused tests if policy changes touch no compiler semantics.

- `node nodesrc\test_stdlib_std_test_no_unsafe_unwraps.js`: pass
- `node nodesrc\issues.js check`: pass
- `git diff --check`: pass

## 対応結果

`nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js` の std/test policy を、旧 `Vec<Result<(),str>>` accumulator から現行の `Checks` value accumulator へ更新した。`alloc/collections/vec` import、`Vec<Result<(),str>>`、`checks_empty_vec`、旧 loop helper、raw `load<Result<(),str>>` の再導入を禁止し、`Checks` の fields、Copy 実装、allocation-free `checks_empty`、`checks_single_error`、value 更新型の `checks_push`、`finish_checks` を確認する。

CI の Source policy regressions を進めると、別件として `passes/move_check/provenance.rs` の行数上限超過と Resource checker responsibility policy の import 文字列不一致が残ることも確認した。これらは本 issue とは別の root cause として分離する。
