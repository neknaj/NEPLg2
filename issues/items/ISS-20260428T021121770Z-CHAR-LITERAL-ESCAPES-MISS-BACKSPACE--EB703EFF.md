---
id: ISS-20260428T021121770Z-CHAR-LITERAL-ESCAPES-MISS-BACKSPACE--EB703EFF
title: "char literal escapes miss backspace and form feed"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/lexer.rs, nepl-core/tests/char.rs, tests/compiler/match_literal_patterns.n.md"
---

# ISS-20260428T021121770Z-CHAR-LITERAL-ESCAPES-MISS-BACKSPACE--EB703EFF: char literal escapes miss backspace and form feed

## 概要

The char literal implementation accepts common escapes such as newline and tab, but rejects '\\b' and '\\f' even though the char support issue specified backspace and form-feed escapes. Stdlib JSON/NM escape classifiers cannot use readable char literals for these control bytes and must fall back to hex escapes.

## 対象

- `nepl-core/src/lexer.rs, nepl-core/tests/char.rs, tests/compiler/match_literal_patterns.n.md`

## 根拠

- `nepl-core/src/lexer.rs` の `read_char_escape` は `\n` / `\r` / `\t` / `\0` / quote / backslash / hex / unicode を扱うが、`\b` と `\f` を default error に落としていた。
- `stdlib/alloc/encoding/json.nepl` と `stdlib/nm/parser.nepl` の escape classifier を char literal 化すると、`'\b'` / `'\f'` で `D1210 invalid escape in char literal` が出た。

## 問題

The char literal implementation accepts common escapes such as newline and tab, but rejects '\\b' and '\\f' even though the char support issue specified backspace and form-feed escapes. Stdlib JSON/NM escape classifiers cannot use readable char literals for these control bytes and must fall back to hex escapes.

## 影響

Code that naturally writes '\\b' or '\\f' as char literals fails with D1210 invalid escape in char literal. This blocks stdlib character-code cleanup and leaves the language specification inconsistent with implementation.

## 修正方針

Teach the lexer char escape reader to decode '\\b' as U+0008 and '\\f' as U+000C, then add focused tests covering both escapes in lexer and compiler match paths.

## 検証

Run cargo test -p nepl-core --test char, cargo test -p nepl-core, trunk build, and focused compiler doctests that compile char literal match arms.

## 解決

- `read_char_escape` で `\b` を U+0008、`\f` を U+000C として decode するようにした。
- lexer unit test に `'\b'` / `'\f'` の token value 確認を追加した。
- compiler doctest に `'\b'` / `'\f'` を `i32` 文脈で code point に下げる回帰を追加した。

## 検証結果

- `cargo test -p nepl-core --test char`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/match_literal_patterns.n.md --no-tree -o tmp/char-bf-match-literal-patterns.json -j 1`
- `cargo test -p nepl-core`
- `cargo check`
