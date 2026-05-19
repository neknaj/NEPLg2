---
id: ISS-20260519T004049268Z-VEC-DROP-LAST-DOC-COMMENT-DOCTEST-PR-9BA3CF28
title: "vec drop_last doc-comment doctest prints checks report without stdout fixture"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/alloc/collections/vec/mutation/pop.nepl, nodesrc/test_stdlib_vec_pop_doc_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260519T004049268Z-VEC-DROP-LAST-DOC-COMMENT-DOCTEST-PR-9BA3CF28: vec drop_last doc-comment doctest prints checks report without stdout fixture

## 概要

The drop_last doc-comment doctest in stdlib/alloc/collections/vec/mutation/pop.nepl calls checks_print_report and checks_exit_code, but the doc test metadata does not pin stdout / exit_code and still uses the legacy checks_* report path.

## 対象

- `stdlib/alloc/collections/vec/mutation/pop.nepl, nodesrc/test_stdlib_vec_pop_doc_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `drop_last` の doc-comment doctest は `Vec` owner を消費して末尾を捨て、次の owner を返すという `Vec` mutation の所有境界を説明する代表例である。
- 旧実装は `checks_print_report` / `checks_exit_code` を呼んでいたが、manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなく、`drop_last` 後の `len == 1` という観測結果が fixture として固定されていなかった。
- selfhost runner と Rust runner の互換性を確認するには、単なる exit status ではなく、どの assertion がどの expected / actual で成功したかを stdout に残す必要がある。

## 問題

The drop_last doc-comment doctest in stdlib/alloc/collections/vec/mutation/pop.nepl calls checks_print_report and checks_exit_code, but the doc test metadata does not pin stdout / exit_code and still uses the legacy checks_* report path.

## 影響

Vec owner-preserving drop_last behavior can regress without a fixture-checked assertion label for the resulting length and cleanup path.

## 修正方針

Migrate the drop_last doc-comment doctest to a named TestReport stdout fixture with exit_code metadata, and add a source policy contract rejecting ret-only or legacy checks_* regression.

## 検証

Run the Vec pop source policy contract, focused pop.nepl doctest, source policy regressions, issue check, and diff whitespace check.

## 対応結果

- `drop_last` doc-comment doctest を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- `vec_drop_last_keeps_owner` という named `TestReport` を使い、`drop_last length` assertion の expected / actual を stdout に固定した。
- `Vec` owner の cleanup は report 出力前に維持し、テスト移行で所有境界を弱めないようにした。
- `nodesrc/test_stdlib_vec_pop_doc_report_contract.js` を追加し、`ret:` 代用、stdout fixture 欠落、旧 `checks_*` 形式、cleanup 順序の退行を source policy で拒否するようにした。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
