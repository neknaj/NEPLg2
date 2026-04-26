---
id: ISS-20260426T010004Z-TEXT-UTF8-VALIDATION-F1950B8A
title: "source loading requires UTF-8 validation before producing str"
area: selfhost
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: stdlib/std/fs.nepl
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
