---
id: ISS-20260428T210703348Z-STREAMIO-AND-SELF-HOST-TEXT-STILL-US-B53BD33B
title: "streamio and self-host text still use decimal character codes"
area: stdlib
status: fixed
resolved: true
priority: P2
type: maintenance
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/std/streamio.nepl, stdlib/std/fs.nepl, stdlib/neplg2/core/infra/text.nepl, nodesrc/test_stdlib_match_decision_trees.js"
---

# ISS-20260428T210703348Z-STREAMIO-AND-SELF-HOST-TEXT-STILL-US-B53BD33B: streamio and self-host text still use decimal character codes

## 概要

After the char literal migration issue was closed, streamio scanner whitespace/digit/sign handling and self-host SourceText line trimming still compare decimal ASCII codes such as 10, 13, 32, 45, 48, 57, 69, and 101 where the value denotes a concrete character.

## 対象

- `stdlib/std/streamio.nepl, stdlib/std/fs.nepl, stdlib/neplg2/core/infra/text.nepl, nodesrc/test_stdlib_match_decision_trees.js`

## 根拠

- `stdlib/std/streamio.nepl` は scanner の空白、符号、digit、指数 marker を `32` / `10` / `13` / `9` / `45` / `43` / `48` / `57` / `101` / `69` で判定していた。
- `stdlib/neplg2/core/infra/text.nepl` は line end trimming で LF/CR を `10` / `13` の match arm として扱っていた。
- `stdlib/std/fs.nepl` は host path 禁止文字の `\` / `:` を `92` / `58` で判定していた。
- 既存の char migration 静的テストは JSON/NM/string/selfhost lexer などの代表関数だけを監視しており、streamio / source text の再発を検出できなかった。

## 問題

After the char literal migration issue was closed, streamio scanner whitespace/digit/sign handling and self-host SourceText line trimming still compare decimal ASCII codes such as 10, 13, 32, 45, 48, 57, 69, and 101 where the value denotes a concrete character.

## 影響

The char cleanup can regress silently in scanner and source text code. Reviewers cannot distinguish character semantics from numeric sizes or offsets, and future parser/self-host work may copy the decimal-code style back into new modules.

## 修正方針

`streamio` に `stream_scanner_is_leading_skip_byte`、`stream_scanner_is_token_separator`、`stream_scanner_is_ascii_digit`、`stream_scanner_digit_value`、`stream_scanner_is_exponent_marker` を追加し、scanner / numeric parser / writer digit 出力の ASCII 判定を char literal ベースへ寄せました。

`SourceText` の LF/CR trimming と `std/fs` の host path 禁止文字判定も char literal に置き換えました。

`nodesrc/test_stdlib_match_decision_trees.js` を拡張し、streamio の空白・指数 classifier と SourceText の newline match arm が decimal code に戻らないよう静的に監視します。

## 検証

- `node nodesrc\test_stdlib_match_decision_trees.js`: pass
- `node nodesrc\tests.js -i stdlib\neplg2\core\infra\text.nepl --no-tree -o tmp\char-magic-source-text.json -j 1`: total=1 passed=1
- `node nodesrc\tests.js -i tests\stdlib\streamio.n.md --no-tree -o tmp\char-magic-streamio.json -j 1`: total=14 passed=14
- `node nodesrc\tests.js -i stdlib\std\fs.nepl --no-tree -o tmp\char-magic-fs.json -j 1`: total=7 passed=7
- `node nodesrc\issues.js check`: pass, files=319
