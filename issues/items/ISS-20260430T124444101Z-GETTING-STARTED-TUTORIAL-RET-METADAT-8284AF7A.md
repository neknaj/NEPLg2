---
id: ISS-20260430T124444101Z-GETTING-STARTED-TUTORIAL-RET-METADAT-8284AF7A
title: "getting_started tutorial ret metadata migration lacks regression policy"
area: tutorials
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: "nodesrc/test_tutorial_getting_started_current_style.js, tutorials/getting_started/**/*.n.md"
source: issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md
---

# ISS-20260430T124444101Z-GETTING-STARTED-TUTORIAL-RET-METADAT-8284AF7A: getting_started tutorial ret metadata migration lacks regression policy

## 概要

The getting_started tutorial was migrated from ret: 0 to exit_code: 0 for assertion-style doctests, but the tutorial style policy does not fail if ret: metadata is reintroduced. The migration can regress without source-policy coverage.

## 対象

- `nodesrc/test_tutorial_getting_started_current_style.js, tutorials/getting_started/**/*.n.md`

## 根拠

- `ISS-20260430T123220209Z-GETTING-STARTED-TUTORIALS-USE-RET-FO-0BE9531F` で `tutorials/getting_started` の `ret: 0` は `exit_code: 0` へ移行済みである。
- しかし `nodesrc/test_tutorial_getting_started_current_style.js` は chapter list や禁止コード pattern は見ていたが、`ret:` metadata の再混入は検出していなかった。
- `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` の方針では、process / WASI / selfhost CLI の終了可否は `exit_code:` で表し、`ret:` は言語レベルの戻り値検証に限定する。

## 問題

The getting_started tutorial was migrated from ret: 0 to exit_code: 0 for assertion-style doctests, but the tutorial style policy does not fail if ret: metadata is reintroduced. The migration can regress without source-policy coverage.

## 影響

Self-host and Rust doctest runners can drift back to ambiguous return-value-as-exit-code fixtures, weakening the stdout assertion report contract for beginner tutorial examples.

## 修正方針

Extend the getting_started current-style source policy to reject ret: metadata in tutorial .n.md files, keeping process success/failure under exit_code: and stdout report fixtures.

## 検証

Run node nodesrc/test_tutorial_getting_started_current_style.js, node nodesrc/issues.js check, and git diff --check.

## 対応結果

`nodesrc/test_tutorial_getting_started_current_style.js` に `^ret:` metadata 禁止を追加した。getting_started tutorial に `ret:` が戻ると source policy が失敗するため、stdout report + `exit_code:` の tutorial contract を維持できる。

検証:

- `node nodesrc/test_tutorial_getting_started_current_style.js`: passed
- `rg -n '^ret:' tutorials/getting_started`: no matches
