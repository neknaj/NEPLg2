---
id: ISS-20260430T123220209Z-GETTING-STARTED-TUTORIALS-USE-RET-FO-0BE9531F
title: "getting started tutorials use ret for assertion exit code"
area: tutorials
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "tutorials/getting_started/*.n.md"
---

# ISS-20260430T123220209Z-GETTING-STARTED-TUTORIALS-USE-RET-FO-0BE9531F: getting started tutorials use ret for assertion exit code

## 概要

Getting started tutorial doctests already print assertion reports to stdout, but many still use ret: 0 for the runner success value. This keeps ret overloaded as both language return-value expectation and process-style test success.

## 対象

- `tutorials/getting_started/*.n.md`

## 根拠

- `tutorials/getting_started/02_test_harness.n.md` 以降の assertion-style tutorial は `checks_print_report` で stdout report を出している。
- しかし metadata は `ret: 0` のままで、runner success value と言語レベルの戻り値期待が混ざっていた。
- `rg -n "^ret:" tutorials/getting_started` で tutorial 内の残存 `ret:` を確認できる状態だった。

## 問題

Getting started tutorial doctests already print assertion reports to stdout, but many still use ret: 0 for the runner success value. This keeps ret overloaded as both language return-value expectation and process-style test success.

## 影響

Tutorial fixtures do not fully follow the stdout report plus exit_code contract needed for shared Rust/self-host doctest runners.

## 修正方針

Replace tutorial assertion-suite ret: 0 metadata with exit_code: 0 while keeping the existing stdout report expectations.

## 検証

- `rg -n "^ret:" tutorials/getting_started`: no matches
- `rg -n "^exit_code:" tutorials/getting_started`: 22 matches
- `node nodesrc/tests.js -i tutorials/getting_started/02_test_harness.n.md --no-tree -o tmp/tutorial-02-exit-code-agent1.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/tests.js -i tutorials/getting_started/03_values_and_types.n.md -i tutorials/getting_started/13_vec_basics.n.md -i tutorials/getting_started/24_project_byte_output.n.md --no-tree -o tmp/tutorial-representative-exit-code-agent1.json -j 1 --dist web/dist`: total=3, passed=3
- `node nodesrc/tests.js -i tutorials/getting_started --no-tree -o tmp/tutorial-getting-started-exit-code-agent1.json -j 1 --dist web/dist`: 180 秒で timeout。全体 24 件は local 限定実行として重いため、代表 doctest と metadata scan で確認した。
