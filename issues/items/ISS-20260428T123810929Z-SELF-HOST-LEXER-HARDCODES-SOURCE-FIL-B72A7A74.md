---
id: ISS-20260428T123810929Z-SELF-HOST-LEXER-HARDCODES-SOURCE-FIL-B72A7A74
title: "self-host lexer hardcodes source file id in spans"
area: selfhost
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module/loader.nepl"
---

# ISS-20260428T123810929Z-SELF-HOST-LEXER-HARDCODES-SOURCE-FIL-B72A7A74: self-host lexer hardcodes source file id in spans

## 概要

self-host lexer はすべての `SelfhostSourceSpan` を `file_id = 0` で構築している。VFS-backed loader が path を区別できても、token、AST item、diagnostic が source file identity を保持できない。

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module/loader.nepl`

## 根拠

- `stdlib/neplg2/core/syntax/lexer.nepl` の `lex_token_slice` / `lex_structural_token` / diagnostic helper が `source_span_new 0 ...` または `source_span_empty 0 ...` を使っている。
- `doc/neplg2/self_host_plan.md` の S2 は複数 input file と stdlib source を loader で扱う方針であり、span の `file_id` が固定値だと file ごとの診断表示や import graph 追跡ができない。
- `core/infra/text.nepl` は `SelfhostSourceText.file_id` を持つため、lexer/parser 入口が file_id を受け取らない現状は infra 設計とずれている。

## 問題

single source の lexer/parser doctest では発覚しないが、複数 source を VFS から読むと span がすべて同じ file を指す。後続の SourceText line map、import diagnostic、source-span based ID が file identity を失う。

## 影響

multi-file module loading、import diagnostic、SourceText line mapping、後続の source-span based ID が、複数 file 間で正しい location を比較・表示できない。S2 以降で loader を実装しても、診断品質と graph 追跡が根本的に壊れる。

## 修正方針

lexer / parser に file_id aware entry point を追加し、既存の single-source 互換 wrapper は `file_id = 0` として残す。loader は VFS entry の file_id を parser へ渡し、multi-file VFS regression で distinct file_id を確認する。

## 検証

- lexer / parser / module loader doctest を実行する。
- multi-file VFS regression を追加し、loaded AST span が file ごとに異なる `file_id` を保持することを確認する。
- `node nodesrc/issues.js check`
