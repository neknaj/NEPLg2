---
id: ISS-20260426T020003000Z-STDIO-SKIP-TESTS-2E6F0A4B
title: "stdio has many skipped doctests on self-host critical APIs"
area: stdlib
status: verified
resolved: true
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

## 対応

- `stdlib/std/stdio.nepl` の skipped doctest をすべて実行可能な stdin / stdout / ret 付き fixture に置き換えた。
- `print` / `println` / `print_i32` / `println_i32` / `read_all` / `read_line` の標準 I/O 経路を doctest で直接確認するようにした。
- ANSI helper と color output helper は実際の escape sequence を stdout で比較するようにした。
- debug 系 API は debug profile の出力を確認し、release profile 側の no-op 実装が public 名 `debug` / `debug_color` / `debugln` / `debugln_color` として解決できるように修正した。
- `tests/compiler/tree/19_stdio_release_debug_noop.js` を追加し、release profile で debug no-op symbol が stdlib import から解決できることを固定した。

## 検証結果

- `rg -n "neplg2:test\\[skip\\]" stdlib/std/stdio.nepl`: no matches
- `node nodesrc/tests.js -i stdlib/std/stdio.nepl --no-tree -o tmp/stdio-executable-doctests.json -j 1`: total=28, passed=28
- `node nodesrc/tests.js -i stdlib/std/stdio.nepl --with-tree --no-stdlib -o tmp/stdio-executable-doctests-tree.json -j 1`: total=48, passed=48
- `node nodesrc/tests.js -i tests/stdlib/stdio_result_stderr.n.md -i tests/stdlib/stdio_read_all.n.md -i tests/stdlib/stdout.n.md -i tests/stdlib/streamio.n.md -i tests/stdlib/io.n.md --no-tree -o tmp/stdio-related-tests.json -j 2`: total=30, passed=30
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdio-executable-stdlib-full.json -j 4`: total=404, passed=404
- `cargo fmt --all --check`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-stdio-executable.json`: 13/13 passed
- `node nodesrc/issues.js index` / `node nodesrc/issues.js check`: pass
