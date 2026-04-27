---
id: ISS-20260427T050328482Z-NM-INLINE-PARSER-VEC-ALLOCATION-FAIL-817BA561
title: "nm inline parser が Vec allocation failure を unwrap_ok で trap する"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/nm/parser.nepl, tests/stdlib/nm.n.md, nodesrc/test_stdlib_nm_parser_no_inline_unwraps.js"
source: ISS-20260425T000000Z-RV-STDLIB-010-BF35FCBB
---

# ISS-20260427T050328482Z-NM-INLINE-PARSER-VEC-ALLOCATION-FAIL-817BA561: nm inline parser が Vec allocation failure を unwrap_ok で trap する

## 概要

stdlib/nm/parser.nepl の parse_inlines は Inline/Gloss 用 Vec の生成と push を unwrap_ok で処理し、allocation failure を AST の失敗値や既存 facade へ戻せない。

## 対象

- `stdlib/nm/parser.nepl, tests/stdlib/nm.n.md, nodesrc/test_stdlib_nm_parser_no_inline_unwraps.js`

## 根拠

parse_inlines は v::new<Inline>、v::push<Inline>、v::new<str>、v::push<str> をすべて unwrap_ok で剥がしている。

## 問題

nm parser は docs/self-host 周辺の Markdown 処理で使う基盤だが、入力サイズや allocation pressure で parser が診断可能な値を返す前に trap する。

## 影響

Gloss/Ruby/Math inline を含む nm document の解析が、OOM や Vec grow failure に対して graceful に中断できず、RV-STDLIB-010 の unsafe helper debt が残る。

## 修正方針

parse_inlines の Vec new/push を match で扱う。失敗時は既存 Vec owner が失われるため空 Vec sentinel に切り替えて解析を止め、既存 parse_inlines facade は trap しない失敗値を返す。source policy regression で inline parser への unwrap_ok 再導入を防ぐ。

## 解決内容

- `InlinePushRes` / `StrPushRes` を追加し、Vec owner と push 成否を同時に返せるようにした。
- `nm_inline_empty_vec` / `nm_str_empty_vec` を追加し、allocation/grow failure 時に consumed owner を再利用しない空 Vec sentinel を明示した。
- `nm_push_inline` / `nm_push_str` を追加し、`v::push` の `Err` を `ok=false` と空 Vec sentinel へ変換するようにした。
- `parse_inlines` の `v::new<Inline>` / `v::new<str>` / `v::push<Inline>` / `v::push<str>` から `unwrap_ok` を除去し、失敗時は `failed=true` で scan を止めるようにした。
- `nodesrc/test_stdlib_nm_parser_no_inline_unwraps.js` を追加し、inline parser への unsafe unwrap helper 再導入を source policy で固定した。
- CI/source policy と `doc/testing.md` に新しい guard を登録した。

## 検証

- `node nodesrc/test_stdlib_nm_parser_no_inline_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl --no-tree -o tmp/nm-parser-inline-allocation-docs.json -j 1`: 3/3 passed
- `node nodesrc/tests.js -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-parser-inline-allocation-focused.json -j 1`: 5/5 passed
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-parser-inline-allocation-suite.json -j 1`: 10/10 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-nm-parser-inline-allocation.json -j 4`: 305/305 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-nm-parser-inline-allocation.json -j 4`: 418/418 passed
