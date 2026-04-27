---
id: ISS-20260427T163710082Z-STD-TEST-LOADS-VEC-RESULT-FROM-RAW-T-BDF60069
title: "std/test loads Vec<Result> from raw temp multiple times"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/std/test.nepl, tests/stdlib/byte_builder.n.md"
---

# ISS-20260427T163710082Z-STD-TEST-LOADS-VEC-RESULT-FROM-RAW-T-BDF60069: std/test loads Vec<Result> from raw temp multiple times

## 概要

`checks_print_machine` と `finish_checks` は `Vec<Result<(),str>>` を temporary raw memory に退避した後、同じ raw place から `checks_summary` / `checks_has_err` / 最終 read-back のために複数回 by-value load している。raw memory ownership 検査が正確になると、各 `load<Vec<Result<(),str>>> checks_mem` は同じ owner を再生成するため `D3100` になる。

## 対象

- `stdlib/std/test.nepl, tests/stdlib/byte_builder.n.md`

## 根拠

- `stdlib/std/test.nepl:705` / `707` / `712` で `checks_print_machine` が同じ `checks_mem` から `Vec<Result<(),str>>` を複数回 load している。
- `stdlib/std/test.nepl:777` / `779` でも `finish_checks` が同じ pattern を持つ。
- `tests/stdlib/byte_builder.n.md` の 3 doctest は、この helper 経由で `error[D3100]: use of moved raw memory place: checks_mem` になった。
- compiler 側では raw aggregate field read の誤検出と branch merge の悪化は解消済みであり、この D3100 は non-Copy `Vec` owner の by-value load が複数回行われている実際の所有権違反として残る。

## 問題

`checks_summary` と `checks_has_err` が `Vec<Result<(),str>>` を by-value で受け取るため、呼び出し側は raw temp から Vec owner を何度も取り出している。これは shallow owner の複製であり、最後に返す owner と helper に渡した owner が同じ backing storage を指し得る。

## 影響

`byte_builder` doctest と、この std/test helper を使う test path が compile-time D3100 で停止する。検査を迂回すると、同じ Vec storage に対する複数 owner が作られ、drop/free 設計の前提を壊す。

## 修正方針

`std/test` helper を、Vec owner を複数回 by-value load しない形へ設計し直す。候補は以下。

- `len` / `data` を一度だけ取り出して summary / has_err loop へ渡す。
- observer helper を参照または raw slice 相当で表現し、Vec owner は最後まで 1 つだけ保持する。
- どうしても by-value helper を維持する場合は、owner を線形に受け渡して返す API に変える。

## 検証

stdlib 修正後に `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/std-test-vec-result-raw-temp.json -j 1` を実行し、`D3100` が消えることを確認する。
