---
id: ISS-20260426T010004Z-TEXT-UTF8-VALIDATION-F1950B8A
title: "source loading requires UTF-8 validation before producing str"
area: selfhost
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/std/text.nepl, stdlib/std/fs.nepl, stdlib/std/stdio.nepl, stdlib/std/streamio.nepl, stdlib/std/io.nepl, tests/stdlib/text_utf8.n.md, tests/stdlib/stdio_read_all.n.md"
source: doc/neplg2/self_host_plan.md
---

# ISS-20260426T010004Z-TEXT-UTF8-VALIDATION-F1950B8A: source loading requires UTF-8 validation before producing str

## 概要

self-host parser は source text を `str` として扱うため、filesystem / stdin から得た byte sequence が UTF-8 として妥当であることを境界で保証する必要がある。

## 対象

- `stdlib/std/fs.nepl`
- `stdlib/std/stdio.nepl`
- `stdlib/alloc/string.nepl`
- `stdlib/neplg2/core/syntax/source_text.nepl`

## 根拠

- `fs_bytes_to_string` は UTF-8 検証を行わないと明記している。
- `stdio` の text read も byte sequence の妥当性を caller へ報告しない。

## 問題

不正 UTF-8 を `str` として扱うと、lexer の byte offset / char boundary / diagnostic span が壊れる。
source map と parser recovery が、文字列が妥当であるという暗黙前提に依存する。

## 影響

不正な入力ファイルで compiler panic、誤った span、または silent corruption が起きる可能性がある。
セルフホスト compiler の source loader が Rust 実装より弱くなり、diagnostic parity を満たせない。

## 修正方針

`alloc/string` または `std/text` に `utf8_validate` / `bytes_to_utf8_str` を追加し、`Result<str, StdErrorKind>` を返す。
`fs_read_to_string_checked` を追加し、self-host source loading は checked API のみを使う。
unchecked 変換は binary protocol や既存互換用として名前で明示する。

## 検証

- 不正 UTF-8 byte sequence を含む file read が Err になる test。
- 日本語 source と multi-byte span の lexer diagnostic が一致する test。
- `fs_bytes_to_string` 既存互換と checked API の使い分け doctest。

## 対応結果

`std/text.nepl` を追加し、UTF-8 leading byte を `TextUtf8LeadKind` enum に分類して `match` で sequence 長ごとの検証を行う checked conversion を実装した。
検証では continuation byte 単体、overlong sequence、surrogate range、4 byte sequence の境界を扱い、invalid byte sequence は `StdErrorKind::InvalidUtf8` として返す。

`std/fs` には `fs_bytes_to_utf8_string_result` と `fs_read_to_string_checked` を追加し、invalid UTF-8 を errno 84 (ILSEQ 相当) に変換するようにした。
既存の `fs_bytes_to_string_result` / `fs_read_to_string` は互換用の unchecked API として残し、コメント上でも source text には checked API を使うことを明示した。

`std/stdio` には `stdio_read_all_bytes_result` と `stdio_read_all_text_result` を追加し、stdin read の allocation / memory helper / fd_read failure を Result で返せるようにした。
既存の `stdio_read_all_bytes` と `read_all` は互換 facade として失敗時だけ空値へ丸める形に整理した。
`std/streamio` と `std/io` の text read 経路は checked conversion に接続し、`ReadStream::Fs` / `ReadStream::Bytes` から invalid byte sequence が unchecked `str` へ入らないようにした。

`stdlib/neplg2/core/syntax/source_text.nepl` は現行 tree にまだ存在せず、`module/loader.nepl` も Stage 0 marker のため実 filesystem loader を持たない。
そのため今回は source loading が使う stdlib 境界 API を固定し、self-host loader 実装時には `fs_read_to_string_checked` / `stdio_read_all_text_result` を使う前提を明確にした。

## 検証結果

- `trunk build`: pass, warnings なし
- `node nodesrc/tests.js -i stdlib/std/text.nepl -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/text-utf8-focused-final.json -j 1`: 5/5 passed
- `node nodesrc/tests.js -i tests/stdlib/stdio_read_all.n.md -i tests/stdlib/stdin.n.md --no-tree -o tmp/text-utf8-stdio-final.json -j 1`: 7/7 passed
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i tests/stdlib/io.n.md --no-tree -o tmp/text-utf8-streamio-io-final.json -j 1`: 19/19 passed
- `node nodesrc/tests.js -i tests/stdlib/fs.n.md -i stdlib/std/fs.nepl --no-tree -o tmp/text-utf8-fs-final.json -j 1`: 14/14 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/text-utf8-stdlib-full-final.json -j 4`: 406/406 passed
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-text-utf8-final.json`: 13/13 passed
