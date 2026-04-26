---
id: ISS-20260426T020003000Z-STDIO-SKIP-TESTS-2E6F0A4B
title: "stdio has many skipped doctests on self-host critical APIs"
area: stdlib
status: open
resolved: false
priority: P1
type: test
created: 2026-04-26
updated: 2026-04-26
target: stdlib/std/stdio.nepl
source: doc/neplg2/pre_selfhost_audit_20260426.md
---

# ISS-20260426T020003000Z-STDIO-SKIP-TESTS-2E6F0A4B: stdio has many skipped doctests on self-host critical APIs

## 概要

`stdlib/std/stdio.nepl` は self-host CLI の入力、diagnostic、progress output に直結するが、27 件の doctest が `neplg2:test[skip]` のまま残っている。
既存の `RV-STDLIB-006` は fs / cliarg の skip を対象としており、stdio の広範な skip は別に管理する必要がある。

## 根拠

- `stdlib/std/stdio.nepl` には `neplg2:test[skip]` が 27 件ある。
- `std/fs` は 5 件、`std/env/cliarg` は 5 件であり、I/O 系の実行可能 coverage が runtime 境界に集中して不足している。
- `ISS-20260426T010003Z-STDIO-RESULT-STDERR-E48B51D0` は Result / stderr interface の設計不足を扱うが、既存 API の test skip を直接閉じない。

## 問題

stdio wrapper と Rust CLI runtime の ABI ずれ、buffer handling、stdout/stderr の取り違えが doctest で検出されない。
セルフホスト compiler の CLI parity では、正常出力と diagnostic 出力を機械的に比較するため、stdio の coverage 不足がそのまま検証不足になる。

## 影響

source file 読み込みエラー、diagnostic 出力、JSON / WAT / WASM artifact 出力の比較が不安定になる。
実装後に CLI smoke test だけで問題が発覚し、原因が stdlib wrapper か runtime host function か切り分けにくくなる。

## 修正方針

test runner に stdin / stdout / stderr fixture と fd error injection を追加し、stdio doctest の skip を段階的に外す。
Result-returning API の新設 issue と合わせ、互換 facade の `print` / `println` と self-host 用 Result API の両方を検証する。

## 検証

- `node nodesrc/tests.js -i stdlib/std/stdio.nepl --no-tree -o tmp/stdio-tests.json -j 1`
- stdout / stderr 分離を確認する CLI JSON fixture。
