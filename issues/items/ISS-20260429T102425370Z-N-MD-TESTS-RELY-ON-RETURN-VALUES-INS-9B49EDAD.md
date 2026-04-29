---
id: ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD
title: ".n.md tests rely on return values instead of stdout assertion reports"
area: TEST
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "tests/**/*.n.md, stdlib/**/*.nepl, nodesrc/tests.js, nodesrc/run_doctest.js, stdlib/std/test.nepl"
---

# ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD: .n.md tests rely on return values instead of stdout assertion reports

## 概要

`.n.md` の assertion 系 test が `main` の返す `i32` だけで可否を表し、stdout に検査内容を出さないケースが多い。

`main` の `i32` は runner では exit code 相当として扱えるが、失敗時に「どの assertion が、どの expected/actual で落ちたか」を fixture の期待値として確認できない。selfhost でも同じ `.n.md` を使うには、assertion report を stdout に出し、exit code は可否だけを表す運用に統一する必要がある。

## 対象

- `tests/**/*.n.md, stdlib/**/*.nepl, nodesrc/tests.js, nodesrc/run_doctest.js, stdlib/std/test.nepl`

## 根拠

- 2026-04-29 時点の調査では、`tests` / `tutorials` / `stdlib` の doctest 1481 件中、`ret:` を持つものは 719 件、stdout 期待値を持つものは 98 件である。
- `ret:` だけで stdout/stderr を持たない doctest は 710 件ある。
- `std/test`、`checks_exit_code`、`assert_*` などを使う assertion 系 doctest は 227 件ある。そのうち 116 件は `ret:` だけで、stdout report を期待していない。
- `checks_exit_code` は 171 箇所で使われている一方、`checks_print_report` は 101 箇所であり、検査結果の表示を伴わない assertion suite が残っている。
- `nodesrc/tests.js` と `nodesrc/run_doctest.js` は stdout が未指定の `std/test` case について `FAIL:` 行を検出する ad hoc な保険を持つが、これは成功時の詳細 report を仕様として固定するものではない。

## 問題

- exit code 相当の `i32` は 0/1 しか表現せず、失敗内容の情報量が不足する。
- `ret:` が「言語仕様としての戻り値期待」と「テスト成功/失敗の exit code 期待」を兼ねており、`.n.md` manifest の意味が曖昧である。
- stdout report を fixture に固定しないため、Rust runner と selfhost runner の assertion 表示、集約順、failure formatting の差異を検出できない。
- `std/test` を使っていても `checks_print_report` を呼ばない test があり、CI 上の失敗時に詳細確認がしにくい。

## 影響

- selfhost runner へ `.n.md` を共通利用するとき、Rust 側と selfhost 側が同じ exit code を返しても、失敗 detail や report format の互換性が確認できない。
- test failure の原因調査が runner log や local reproduction に依存し、`.n.md` 単体を読んでも期待される assertion report が分からない。
- 将来 `ret:` の意味を拡張すると、言語戻り値 test と exit code test が衝突する。

## 修正方針

- `.n.md` manifest に `exit_code:` を追加し、process / WASI / selfhost CLI の終了可否は `exit_code:` で表す。`ret:` は言語レベルの戻り値を検証する場合に限定する。
- assertion suite は stdout に deterministic な report を出す。標準形は `std/test` の report helper を通し、最後に `test_report_exit_code` 相当の helper で 0/1 を返す。
- `std/test` を import する assertion-style doctest について、stdout report なしの `ret:` だけ運用を runner か lint で検出する。
- 既存 fixture は一括置換ではなく、`std/test` 再設計後の安定 API に合わせて段階的に移行する。
- `core` target のように stdout を持たない層は、assertion report 必須の対象から分ける。core-only の primitive semantics は `ret:` または compile diagnostic で扱い、std stdout report と混同しない。

## 検証

- `parser.ts` / `parser.js` の metadata parser が `exit_code:` と `diag_code:` を保持する regression を追加する。
- `nodesrc/tests.js` と `nodesrc/run_doctest.js` が同じ expectation logic で `exit_code` / `stdout` / `stderr` / `diag_code` を検査することを確認する。
- `std/test` を使う代表 fixture を stdout report + exit code 期待に移行し、失敗時に stdout diff で assertion detail が見えることを確認する。
