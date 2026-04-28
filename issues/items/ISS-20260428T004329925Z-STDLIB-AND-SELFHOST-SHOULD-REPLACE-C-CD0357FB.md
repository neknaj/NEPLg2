---
id: ISS-20260428T004329925Z-STDLIB-AND-SELFHOST-SHOULD-REPLACE-C-CD0357FB
title: "stdlib and selfhost should replace character code magic numbers with char literals"
area: stdlib
status: fixed
resolved: true
priority: P2
type: maintenance
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/alloc/string.nepl, stdlib/alloc/encoding/json.nepl, stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, stdlib/std/stdio.nepl, stdlib/std/env/cliarg.nepl, stdlib/platforms/wasix/tui.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib, nodesrc/test_stdlib_match_decision_trees.js"
---

# ISS-20260428T004329925Z-STDLIB-AND-SELFHOST-SHOULD-REPLACE-C-CD0357FB: stdlib and selfhost should replace character code magic numbers with char literals

## 概要

Many existing stdlib and selfhost modules classify ASCII/control bytes with decimal literals even when the branch means a concrete character. Examples include JSON/nm/html escape classifiers, selfhost lexer punctuation, string escape/numeric parsing, stdio sign handling, cliarg C-string tests, byte builder magic bytes, and WASIX TUI escape handling. Once char literals exist, leaving these as numbers will keep the original readability problem and make the new feature underused.

## 対象

- `stdlib/alloc/string.nepl, stdlib/alloc/encoding/json.nepl, stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, stdlib/std/stdio.nepl, stdlib/std/env/cliarg.nepl, stdlib/platforms/wasix/tui.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib, nodesrc/test_stdlib_match_decision_trees.js`

## 関連ドキュメント

- [NEPLg2 stdlib char 整備計画](../../doc/neplg2/char_stdlib_integration_plan.md)

## 根拠

- `stdlib/nm/parser.nepl` は JSON escape classifier で `match ch:` に `92` / `34` / `10` / `13` / `9` / `8` / `12` を使っている。
- `stdlib/nm/html_gen.nepl` は HTML escape classifier で `38` / `60` / `62` / `34` / `39` を使っている。
- `stdlib/neplg2/core/syntax/lexer.nepl` は punctuation / string / comment 周辺の判定で character code を直接扱う。
- `stdlib/std/env/cliarg.nepl` の doctest は C-string bytes を `110` / `101` / `112` / `0` のように書いている。
- `tests/stdlib/byte_builder.n.md` は magic bytes のうち text 部分も `97` / `115` / `109` などの decimal code で検査している。

## 問題

Many existing stdlib and selfhost modules classify ASCII/control bytes with decimal literals even when the branch means a concrete character. Examples include JSON/nm/html escape classifiers, selfhost lexer punctuation, string escape/numeric parsing, stdio sign handling, cliarg C-string tests, byte builder magic bytes, and WASIX TUI escape handling. Once char literals exist, leaving these as numbers will keep the original readability problem and make the new feature underused.

## 影響

Reviewers cannot tell whether a number is a byte value, a length, an offset, a tag, or a character. This is especially harmful in lexer/parser/string code where values like 10, 13, 34, 39, 60, 62, 92, 123, and 125 encode syntax. It also makes match-based finite dispatch less self-documenting.

## 修正方針

After core char support lands, audit stdlib and selfhost code and replace character-code literals with char literals where the value denotes a character. Prioritize escape classifiers and lexer/parser punctuation: use '\n', '\r', '\t', '\\', '\'', '"', '&', '<', '>', '/', '[', ']', '{', '}', '$', '-' instead of decimal codes. Keep numeric literals for sizes, offsets, enum tags, non-text binary formats, and non-printable binary constants where char would mislead. Add focused tests and a static search/regression rule for known character-code comparisons in classifier functions.

## 検証

Run targeted tests for string, json, nm, html_gen, stdio, cliarg, byte_builder, WASIX TUI if available, and stdlib/neplg2 lexer. Add a static test that key classifier functions use char literal match arms or comparisons rather than decimal character codes.

## 解決

- JSON / nm JSON / HTML escape classifier の `match` arm を decimal character code から char literal へ置き換えた。
- selfhost lexer の ASCII digit / alphabet / identifier / trivia / punctuation / string / char literal 判定を char literal に置き換えた。
- `stdlib/alloc/string.nepl` の `str_is_space` を nested `if` から char literal の `match` へ戻し、digit/sign/dot/exponent の byte 判定も char literal で表すようにした。
- stdio の newline/sign/digit 出力、CLI arg C-string doctest、WASIX TUI の ESC/OSC/CSI/newline 判定、byte builder の text magic bytes を char literal 化した。
- binary format の opcode、サイズ、offset、radix、error code、ANSI color offset のように文字ではない値は数値のまま残した。
- `nodesrc/test_stdlib_match_decision_trees.js` に char literal arm の静的回帰を追加し、escape classifier と `str_is_space` が nested `if` に戻らないようにした。

## 検証結果

- `trunk build`
- `node nodesrc/test_stdlib_match_decision_trees.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/char-magic-selfhost-lexer.json -j 1`
- `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/char-magic-byte-builder.json -j 1`
- `node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/char-magic-string.json -j 1`
- `node nodesrc/tests.js -i stdlib/tests/json.n.md --no-tree -o tmp/char-magic-json.json -j 1`
- `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md --no-tree -o tmp/char-magic-cliarg.json -j 1`
- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/char-magic-tui.json -j 1`
- `node nodesrc/tests.js -i tests/stdlib/stdout.n.md --no-tree -o tmp/char-magic-stdout.json -j 1`
- `node nodesrc/tests.js -i tests/stdlib/stdin.n.md --no-tree -o tmp/char-magic-stdin.json -j 1`
- `node nodesrc/tests.js -i stdlib/std/stdio.nepl --no-tree -o tmp/char-magic-stdio-docs.json -j 1`
- `node nodesrc/tests.js -i stdlib/std/env/cliarg.nepl --no-tree -o tmp/char-magic-cliarg-docs.json -j 1`

## 既知の別 issue

- `tests/stdlib/json_typed_values.n.md` は 7 件中 4 件通過、3 件が D3100 raw memory/move check で compile fail。
- `tests/stdlib/nm.n.md` は 5 件が D3100 raw memory/move check で compile fail。
- どちらも char literal 置換の失敗ではなく、[ISS-20260428T003718356Z-NM-PARSER-AND-HTML-GEN-RAW-MEMORY-DE-99175378](./ISS-20260428T003718356Z-NM-PARSER-AND-HTML-GEN-RAW-MEMORY-DE-99175378.md) と同系統の raw memory detour / strict move checking 問題として扱う。
