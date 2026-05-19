---
id: ISS-20260519T010217734Z-SELFHOST-NAME-RESOLVER-DOCTESTS-USE--3A40ADE1
title: "selfhost name resolver doctests use owner-backed add result fields and hide reports"
area: selfhost
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/neplg2/core/resolve/name_resolver.nepl, nodesrc/test_selfhost_name_resolver_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260519T010217734Z-SELFHOST-NAME-RESOLVER-DOCTESTS-USE--3A40ADE1: selfhost name resolver doctests use owner-backed add result fields and hide reports

## 概要

Selfhost name resolver doc-comment doctests access SelfhostNameScopeAddResult.scope through field::get, which is rejected by owner-backed aggregate field access checks; the same doctests call checks_print_report without stdout / exit_code metadata.

## 対象

- `stdlib/neplg2/core/resolve/name_resolver.nepl, nodesrc/test_selfhost_name_resolver_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- Focused run of `stdlib/neplg2/core/resolve/name_resolver.nepl` failed both doc-comment doctests at compile time with `type.owner_aggregate.field_access_restricted`.
- The failing sites read `SelfhostNameScopeAddResult.scope` through direct `field::get` in ordinary doctest source.
- `SelfhostNameScopeAddResult` carries the `SelfhostNameScope` owner, so direct field projection would bypass the public name-scope API boundary. The owner aggregate field gate should stay strict.
- The same doctests called `checks_print_report` / `checks_exit_code`, but the metadata did not pin `stdio`, normalized stdout, or `exit_code: 0`.

## 問題

Selfhost name resolver doc-comment doctests access SelfhostNameScopeAddResult.scope through field::get, which is rejected by owner-backed aggregate field access checks; the same doctests call checks_print_report without stdout / exit_code metadata.

## 影響

Name resolver shadowing and kind-filter regressions are compile-blocked or can lose assertion report compatibility between Rust and selfhost runners.

## 修正方針

Add public add-result accessors that expose the Copy DefId by borrow and consume the wrapper to recover the scope owner, then pin the name resolver doctests with stdio stdout fixtures and source policy regression.

## 検証

Run the name resolver source policy contract, focused name_resolver.nepl doctests with --assert-io, source policy regressions, issue check, and diff whitespace check.

## 対応結果

- `selfhost_name_scope_add_result_def_id` を追加し、borrow から Copy な `SelfhostDefId` だけを読む public accessor を用意した。
- `selfhost_name_scope_add_result_into_scope` を追加し、wrapper を消費して `SelfhostNameScope` owner を取り出す public accessor を用意した。
- 2 件の doc-comment doctest と `selfhost_name_resolver_stage0` を direct `field::get` から accessor 利用へ移行した。
- 2 件の doc-comment doctest を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- `nodesrc/test_selfhost_name_resolver_report_contract.js` を追加し、accessor の存在、direct field access 退行、stdout fixture 欠落、`ret:` 代用への退行を source policy で拒否するようにした。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
- [static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md)
