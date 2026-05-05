---
id: ISS-20260505T013338193Z-TESTS-STDLIB-IO-MISSING-FILE-OK-BRAN-824A29BA
title: "tests/stdlib/io missing-file Ok branch leaks owned str"
area: stdlib
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "tests/stdlib/io.n.md, stdlib/std/io.nepl"
---

# ISS-20260505T013338193Z-TESTS-STDLIB-IO-MISSING-FILE-OK-BRAN-824A29BA: tests/stdlib/io missing-file Ok branch leaks owned str

## 概要

tests/stdlib/io.n.md の io_fs_missing_file_is_io_error で、ReadStream::Fs missing file の Result::Ok branch が _text を束縛したまま owner を閉じず、Resource IR が resource.raw.ownership_violation を報告する。実行時には Err を期待する branch でも、静的検査では全 arm の owner obligation を満たす必要がある。

## 対象

- `tests/stdlib/io.n.md, stdlib/std/io.nepl`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/io.n.md --no-tree -o tmp/io-nmd-after-streamio-writer-split.json -j 1`: `6 total / 5 passed / 1 failed`
- failure: `tests\stdlib\io.n.md::doctest#3`
- diagnostic: `resource.raw.ownership_violation` / `resource ir owner obligation may leak` for local `_text`

## 問題

tests/stdlib/io.n.md の io_fs_missing_file_is_io_error で、ReadStream::Fs missing file の Result::Ok branch が _text を束縛したまま owner を閉じず、Resource IR が resource.raw.ownership_violation を報告する。実行時には Err を期待する branch でも、静的検査では全 arm の owner obligation を満たす必要がある。

## 影響

stdlib io の regression suite を strict owner gate で実行したとき、意図した missing-file behavior ではなく test fixture の owner leak が先に失敗し、std/io と streamio の実際の回帰を隠す。

## 修正方針

Ok branch でも受け取った str owner を検査用 assertion へ渡して消費する、または Result API 側で不要 owner を明示的に終端できる std/test helper を使う。unreachable 前提や未使用変数で逃がさず、静的検査が通る fixture に直す。

## 検証

node nodesrc/tests.js -i tests/stdlib/io.n.md --no-tree -o tmp/io-nmd-missing-file-owner-leak-fixed.json -j 1 が 6/6 passed になること。

## 2026-05-05 修正メモ

`Result::Ok _text` arm を `Result::Ok text` に変更し、受け取った `str` owner を `concat "io fs read unexpectedly succeeded: " text` で failure message へ組み込むようにした。これにより、実行時には到達しない想定の branch でも、静的検査上は `str` owner が `TestReport` の error message へ移動し、最終的に `checks_exit_code` の消費経路に乗る。

検証:

- `node nodesrc/tests.js -i tests/stdlib/io.n.md --no-tree -o tmp/io-nmd-missing-file-owner-leak-fixed.json -j 1`: `6 total / 6 passed`
