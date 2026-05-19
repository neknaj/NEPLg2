---
id: ISS-20260519T021633842Z-NEPL-DOC-COMMENT-REPORT-METADATA-POL-5A293E8D
title: ".nepl doc-comment report metadata policy misses stdout/exit_code regressions"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/**/*.nepl, nodesrc/report_metadata_policy.js, nodesrc/run_source_policy_regressions.js"
---

# ISS-20260519T021633842Z-NEPL-DOC-COMMENT-REPORT-METADATA-POL-5A293E8D: .nepl doc-comment report metadata policy misses stdout/exit_code regressions

## 概要

stdlib の .nepl doc-comment doctest が checks_print_report / checks_exit_code を使っていても、stdout/exit_code/stdio/normalize_newlines metadata を横断的に要求する policy がなく、実行結果や古いサンプルコードの compile failure が fixture に固定されていなかった。

## 対象

- `stdlib/**/*.nepl, nodesrc/report_metadata_policy.js, nodesrc/run_source_policy_regressions.js`

## 根拠

- 2026-05-19 の走査で、active `.nepl` doc-comment report doctest 67 件中 21 件が `checks_exit_code` を使いながら `checks_print_report` 出力、`stdout:`、`exit_code:`、`stdio`、`normalize_newlines` のいずれかを欠いていた。
- `.n.md` 側には横断 policy がある一方、`.nepl` 側は個別 contract のあるファイルだけを監視しており、stdlib module comment の doctest が report metadata なしで残っても検出できなかった。
- metadata を追加して focused run したことで、`stdlib/alloc/diag/error/outcome.nepl::doctest#2` と `stdlib/alloc/io/traits.nepl::doctest#1` の古い import / expression 書き方が compile 不能であることも検出できた。

## 問題

stdlib の .nepl doc-comment doctest が checks_print_report / checks_exit_code を使っていても、stdout/exit_code/stdio/normalize_newlines metadata を横断的に要求する policy がなく、実行結果や古いサンプルコードの compile failure が fixture に固定されていなかった。

## 影響

report 形式や assertion count が壊れても CI が検出できず、selfhost runner へ doctest を移す際に Rust runner と stdout/exit-code contract がずれる。

## 修正方針

.n.md と .nepl の report metadata 判定を共通 helper 化し、.nepl doc-comment doctest にも report print、report-derived exit、stdio/normalize_newlines、stdout fixture、exit_code fixture、ret 不使用を要求する。既存の該当 doctest を stdout report + exit_code へ移行する。

## 検証

node nodesrc/test_nepl_doc_report_metadata_policy.js; node nodesrc/test_nmd_report_metadata_policy.js; changed .nepl report doctests focused run; node nodesrc/issues.js check

## 対応結果

- `nodesrc/report_metadata_policy.js` を追加し、report doctest 判定を `.n.md` / `.nepl` 共通 helper に集約した。
- `nodesrc/test_nmd_report_metadata_policy.js` は共通 helper 利用へ移行し、既存 `.n.md` policy と新規 `.nepl` policy の規則がずれない構造にした。
- `nodesrc/test_nepl_doc_report_metadata_policy.js` を追加し、`.nepl` doc-comment report doctest に以下を必須化した。
  - report を stdout に出すこと。
  - exit code を report result から導くこと。
  - `stdio` と `normalize_newlines` tag を持つこと。
  - `stdout:` と `exit_code:` を fixture として固定すること。
  - `ret:` を exit-code 代用に使わないこと。
- 21 件の `.nepl` doc-comment report doctest を stdout report + `exit_code: 0` に移行した。
- metadata 追加で顕在化した古い doctest の compile failure を修正した。
  - `outcome.nepl::doctest#2`: `core/result` import と明示的な `Result` 値構築へ修正。
  - `traits.nepl::doctest#1`: `core/field` / `core/math` import と、`get` / `add` の型が明確になる一時変数へ修正。
