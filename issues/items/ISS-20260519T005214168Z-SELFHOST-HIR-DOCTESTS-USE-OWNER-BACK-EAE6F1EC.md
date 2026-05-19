---
id: ISS-20260519T005214168Z-SELFHOST-HIR-DOCTESTS-USE-OWNER-BACK-EAE6F1EC
title: "selfhost HIR doctests use owner-backed allocation fields and hide reports"
area: selfhost
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/neplg2/core/hir/hir.nepl, nodesrc/test_selfhost_hir_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260519T005214168Z-SELFHOST-HIR-DOCTESTS-USE-OWNER-BACK-EAE6F1EC: selfhost HIR doctests use owner-backed allocation fields and hide reports

## 概要

Selfhost HIR doc-comment doctests access SelfhostHirModule*Alloc.module through field::get, which is rejected by owner-backed aggregate field access checks; the same doctests call checks_print_report without stdout / exit_code metadata.

## 対象

- `stdlib/neplg2/core/hir/hir.nepl, nodesrc/test_selfhost_hir_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- Focused run of `stdlib/neplg2/core/hir/hir.nepl` failed all 3 doc-comment doctests at compile time with `type.owner_aggregate.field_access_restricted`.
- The failing sites read `SelfhostHirModuleExprAlloc.module`, `SelfhostHirModuleFunctionAlloc.module`, `SelfhostHirModuleChildRangeAlloc.module`, and `SelfhostHirModuleParamRangeAlloc.module` through direct `field::get` in ordinary doctest source.
- This restriction is correct: allocation result wrappers carry `SelfhostHirModule`, which owns multiple `Vec` tables. Ordinary users must not split owner-backed aggregate internals directly.
- The same 3 doctests called `checks_print_report` / `checks_exit_code`, but the metadata did not pin `stdio`, normalized stdout, or `exit_code: 0`.

## 問題

Selfhost HIR doc-comment doctests access SelfhostHirModule*Alloc.module through field::get, which is rejected by owner-backed aggregate field access checks; the same doctests call checks_print_report without stdout / exit_code metadata.

## 影響

HIR module/function/expr/param range regressions are currently compile-blocked or can lose assertion report compatibility between Rust and selfhost runners.

## 修正方針

Add public allocation result accessors that expose Copy ids/ranges by borrow and consume wrappers to recover the module owner, then pin the three HIR doctests with stdio stdout fixtures and source policy regression.

## 検証

Run the HIR source policy contract, focused hir.nepl doctests with --assert-io, source policy regressions, issue check, and diff whitespace check.

## 対応結果

- `SelfhostHirModuleExprAlloc` / `FunctionAlloc` / `ChildRangeAlloc` / `ParamRangeAlloc` に public accessor を追加した。
- Copy な id / range は borrow accessor で読み、`SelfhostHirModule` owner は wrapper を消費する `*_into_module` accessor でのみ取り出すようにした。
- 3 件の doc-comment doctest を direct `field::get` から accessor 利用へ移行した。
- 3 件の doc-comment doctest を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- `nodesrc/test_selfhost_hir_report_contract.js` を追加し、accessor の存在、direct field access 退行、stdout fixture 欠落、`ret:` 代用への退行を source policy で拒否するようにした。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
- [static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md)
