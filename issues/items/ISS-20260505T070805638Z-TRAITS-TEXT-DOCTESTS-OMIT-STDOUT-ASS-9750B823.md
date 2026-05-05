---
id: ISS-20260505T070805638Z-TRAITS-TEXT-DOCTESTS-OMIT-STDOUT-ASS-9750B823
title: "traits text doctests omit stdout assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: tests/stdlib/traits_text.n.md
---

# ISS-20260505T070805638Z-TRAITS-TEXT-DOCTESTS-OMIT-STDOUT-ASS-9750B823: traits text doctests omit stdout assertion reports

## 概要

traits_text の std/test doctests が checks_print_report を呼ばず、ret: 0 だけで assertion 成功を表している。

## 対象

- `tests/stdlib/traits_text.n.md`

## 根拠

- `tests/stdlib/traits_text.n.md` の `stringify` / `debug_string` doctest は `std/test` の `checks_push` で assertion suite を作るが、`checks_print_report` を呼ばず `ret: 0` だけで成功を表していた。
- 先頭の `clone_add` doctest は `#target core` で `main` の戻り値 `14` を検証する言語戻り値 test なので、stdout report 移行対象ではない。

## 問題

traits_text の std/test doctests が checks_print_report を呼ばず、ret: 0 だけで assertion 成功を表している。

## 影響

Stringify / Debug trait の表示契約が stdout assertion report として固定されず、selfhost runner parity と report format regression を確認できない。

## 修正方針

std/test を使う std target doctests を checks_print_report + stdout fixture + exit_code: 0 へ移行する。core target の言語戻り値 test は ret のまま維持する。

## 検証

tests/stdlib/traits_text.n.md を focused run し、3 doctest が通ることを確認する。

## 対応

- `stringify` doctest を `checks_print_report` + stdout fixture + `exit_code: 0` へ移行した。
- `debug_string` doctest を `checks_print_report` + stdout fixture + `exit_code: 0` へ移行した。
- `clone_add` doctest は core target の戻り値検証として `ret: 14` を維持した。

## 2026-05-05 検証結果

- `node nodesrc/tests.js -i tests/stdlib/traits_text.n.md --no-tree -o tmp/traits-text-report-agent1.json -j 1 --dist web/dist`: total=3, passed=3, failed=0
