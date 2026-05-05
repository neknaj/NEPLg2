---
id: ISS-20260505T040130265Z-STDIO-ANSI-API-ENUM-MATCH-BDF2FB5E
title: "stdio ANSI 色 API を enum/match で型付けして整理する"
area: stdlib
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-05
updated: 2026-05-05
target: stdlib/std/stdio/ansi.nepl
---

# ISS-20260505T040130265Z-STDIO-ANSI-API-ENUM-MATCH-BDF2FB5E: stdio ANSI 色 API を enum/match で型付けして整理する

## 概要

std/stdio/ansi は root から分離されたが、色や style を個別の文字列返却関数として管理しており、呼び出し側が任意 str を渡せるため静的検査が効きにくい。module も 486 lines あり、同型の doc/function が反復している。

## 対象

- `stdlib/std/stdio/ansi.nepl`

## 根拠

- 未記入

## 問題

std/stdio/ansi は root から分離されたが、色や style を個別の文字列返却関数として管理しており、呼び出し側が任意 str を渡せるため静的検査が効きにくい。module も 486 lines あり、同型の doc/function が反復している。

## 影響

ANSI 色指定の typo や unsupported code が型で表現されず、selfhost/stdlib の方針である enum と match による網羅性検査を活かせない。今後の色追加時に重複実装が増える。

## 修正方針

AnsiStyle / AnsiColor などの enum を設計し、escape code 生成を match に集約する。print_color 系は enum を受け取る typed API に改め、必要なら互換 facade は段階的に削除する。doc と regression で root facade への逆流と網羅的 match を固定する。

## 検証

stdio ansi doctest、stdout 回帰、source policy で enum/match 境界と root 逆流防止を確認する。

## 解決

2026-05-05 に `std/stdio/ansi` を `AnsiStyle` enum ベースへ再設計した。

- `ansi_red` / `ansi_green` / `print_color` / `println_color` のような raw `str` color facade を削除した。
- `AnsiStyle` enum を追加し、`ansi_style_code` の `match` で全 variant を wildcard なしに対応させた。
- `print_style` / `println_style` は `AnsiStyle` を受け取り、任意 `str` escape code を渡せない API にした。
- `debug_color` / `debugln_color` も `AnsiStyle` を受け取るように変更した。
- stdout 回帰、compiler tree fixture、playground hover fixture を typed API に更新した。
- `nodesrc/test_stdlib_stdio_ansi_boundary.js` で enum / wildcard なし match / obsolete raw string facade の再導入禁止を固定した。

検証:

- `node nodesrc/test_stdlib_stdio_ansi_boundary.js`: passed
- `node nodesrc/test_stdlib_stdio_debug_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/std/stdio/ansi.nepl --no-tree -o tmp/stdio-ansi-enum-ansi.json -j 1`: `3 total / 3 passed`
- `node nodesrc/tests.js -i stdlib/std/stdio/debug.nepl --no-tree -o tmp/stdio-ansi-enum-debug.json -j 1`: `8 total / 8 passed`
- `node nodesrc/tests.js -i tests/stdlib/stdout.n.md --no-tree -o tmp/stdio-ansi-enum-stdout.json -j 1`: `7 total / 7 passed`
- `node tests/compiler/tree/run.js`: `20 total / 20 passed`
- playground analysis hover fixture: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: stdio 関連 passed。既存の `owner_summary_variant_paths.rs has 637 lines; responsibility split limit is 380` warning は継続。
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
