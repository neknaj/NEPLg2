---
id: ISS-20260604T042500000Z-GUI-TUI-DOCTEST-REPORT-CONTRACT-GAPS-4D6B9A0E
title: "GUI and TUI doctests still use ret-only or stale report contracts"
area: stdlib
status: fixed
resolved: true
priority: P1
type: doc
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/core/gui, stdlib/alloc/gui, stdlib/std/gui, tests/stdlib/gui_*.n.md, tests/stdlib/features_tui.n.md"
---

# ISS-20260604T042500000Z-GUI-TUI-DOCTEST-REPORT-CONTRACT-GAPS-4D6B9A0E: GUI and TUI doctests still use ret-only or stale report contracts

## 概要

GUI / TUI 関連の doctest と integration-style `.n.md` tests には、canonical `std/test` `TestReport` stdout ではなく `ret:` only、`Checked` 風の出力、または簡易的な成功判定だけで契約を固定しているものが残っている。Zenn 記事の doc comment 方針では、simple / typical example が動くことに加えて、契約として観測したい値を report として明示する必要がある。

## 対象

- `stdlib/core/gui`
- `stdlib/alloc/gui`
- `stdlib/std/gui`
- `tests/stdlib/gui_*.n.md`
- `tests/stdlib/features_tui.n.md`

## 根拠

- `rg -n "ret:|Checked|checks_exit_code" stdlib/core/gui stdlib/alloc/gui stdlib/std/gui tests/stdlib` で GUI/TUI 関連の旧式 report contract が残る。
- `tests/stdlib/gui_core.n.md`、`tests/stdlib/gui_diff.n.md`、`tests/stdlib/features_tui.n.md` などは GUI/TUI contract の中心なので、戻り値だけでは layout / event / render / effect の観測点が不足する。

## 問題

ret-only doctest は、出力される値が 0 かどうかだけを固定し、何を検査したかを stdout contract として残さない。GUI/TUI は状態、event、layout、render command、host effect の複数軸を持つため、TestReport で assertion label と期待値を残さないと、仕様変更時に契約との差分が追跡しづらい。

## 影響

GUI/TUI 標準ライブラリの再設計・再実装時に、The Elm Architecture 風の update/view/effect 分離、event routing、render command、dirty region、backend boundary のどれが壊れたかを doctest から読み取れない。Web/native/embedded backend をまたぐときの regression 検出も弱くなる。

## 修正方針

GUI/TUI 関連 doctest を module ごとに整理し、ret-only や `Checked` 風出力を canonical `TestReport` stdout へ置き換える。各 report は assertion label、期待値、実測値を明示し、simple example と typical example を分ける。runtime / backend 差が絡む挙動は doctest とは別に cfg-test-style regular tests へ移す。

## 検証

- `rg -n "ret:|Checked|checks_exit_code" stdlib/core/gui stdlib/alloc/gui stdlib/std/gui tests/stdlib`
- focused GUI/TUI doctests
- `node nodesrc/run_source_policy_regressions.js --warn-only`

## 解決

- `stdlib/core/gui`、`stdlib/alloc/gui`、`stdlib/std/gui` の GUI module doctest を canonical `std/test` `TestReport` stdout contract へ移行した。
- `tests/stdlib/gui_*.n.md` と `tests/stdlib/features_tui.n.md` の旧 `ret:` / `Checked` / `checks_*` 契約を `test_report_new` / `test_report_push` / `test_report_print_stdout` / `test_report_exit_code` へ移行した。
- `core/gui` 本体には `std/test` 依存を入れず、doc / integration test の実行 target だけを `std` にして report contract を固定した。
- `features_tui` report contract policy を canonical `TestReport` stdout 監視へ更新した。
