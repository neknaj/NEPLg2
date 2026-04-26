---
id: ISS-20260426T121038912Z-STRING-LITERALS-CONTAINING-ARE-PARSE-F8AD3CED
title: "string literals containing // are parsed as comments"
area: core
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: nepl-core/src/lexer.rs
---

# ISS-20260426T121038912Z-STRING-LITERALS-CONTAINING-ARE-PARSE-F8AD3CED: string literals containing // are parsed as comments

## 概要

`"a/./b//c"` のように `//` を含む文字列リテラルを stdlib doctest に追加すると、lexer が文字列内の `//` を comment 開始として扱い、`D1209 unterminated string literal` になる。

## 対象

- `nepl-core/src/lexer.rs`

## 根拠

- `fs_normalize_relative` の回帰テストで `"a/./b//c"` を使ったところ、文字列リテラル内であるにもかかわらず `//c"` 以降が comment として扱われた。
- 同じテストは `"a/./b/"` へ避けると parse できるため、path 正規化ではなく lexer の comment 判定が原因と判断した。

## 問題

comment scanning が string state を見ていないため、文字列リテラル内の `//` まで comment delimiter として解釈される。

## 影響

path や URL など、`//` を自然に含む文字列をソース上で安全に表現できない。
stdlib や compiler のテストが不自然な回避表現を使うことになり、実際の入力を直接固定できなくなる。

## 修正方針

lexer の comment scanning を string-state aware にし、`//` は文字列リテラル外でだけ comment delimiter として扱う。
path と URL のような `//` を含む文字列リテラルの lexer/parser 回帰テストを追加する。

## 検証

`"a/./b//c"` と `"https://example.test/a"` が文字列として保持されることを、Rust lexer test または compiler doctest で確認する。

## 解決内容

- `nepl-core/src/lexer.rs` の comment 検出を `line.find("//")` から string-state aware な scanner へ置き換えた。
- scanner は `"` で文字列状態へ入り、文字列内の escape sequence を読み飛ばすため、`"https://..."` や `"a//b"` の `//` を comment として扱わない。
- `//` が文字列外に出た場合は従来どおり line comment / doc comment として扱う。
- `nepl-core/tests/string.rs` に path と URL の string literal token を確認する回帰テストを追加した。
- `tests/stdlib/fs.n.md` の path normalization test を `"a/./b//c"` に戻し、不自然な回避 literal を削除した。

## 検証結果

- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test string test_string_literal_keeps_double_slash -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test string -- --nocapture`: 23 passed
- `cargo test -p nepl-core --test doc_comments -- --nocapture`: 3 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/stdlib/fs.n.md -i stdlib/std/fs.nepl --no-tree -o tmp/fs-string-comment-focused.json -j 1`: `total=14`, `passed=14`, `failed=0`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-string-comment.json`: 13/13 passed
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
