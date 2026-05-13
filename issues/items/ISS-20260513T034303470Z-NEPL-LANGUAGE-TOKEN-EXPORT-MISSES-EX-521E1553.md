---
id: ISS-20260513T034303470Z-NEPL-LANGUAGE-TOKEN-EXPORT-MISSES-EX-521E1553
title: "nepl-language token export misses extern visibility field"
area: tools
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nepl-language/src/lib.rs
---

# ISS-20260513T034303470Z-NEPL-LANGUAGE-TOKEN-EXPORT-MISSES-EX-521E1553: nepl-language token export misses extern visibility field

## 概要

GitHub Actions run 25776828197 fails in Shared bootstrap build because nepl-language token_extra matches TokenKind::DirExtern without the vis field added by pub #extern support.

## 対象

- `nepl-language/src/lib.rs`

## 根拠

- GitHub Actions run `25776828197` の `build / Shared bootstrap build` が `cargo build` 中に停止した。
- error は `nepl-language/src/lib.rs:1167` の `TokenKind::DirExtern { module, name, func, signature }` pattern が新しい `vis` field を列挙していないという `E0027`。
- `pub #extern` support で `TokenKind::DirExtern` は `vis` を持つようになっており、`nepl-web` は同型の token export 追従漏れをすでに修正済みだった。

## 問題

GitHub Actions run 25776828197 fails in Shared bootstrap build because nepl-language token_extra matches TokenKind::DirExtern without the vis field added by pub #extern support.

## 影響

workspace cargo build stops before bootstrap artifacts are uploaded, so compile/test/deploy jobs are skipped and CI cannot validate the pushed main.

## 修正方針

Update nepl-language token export to match TokenKind::DirExtern exhaustively and expose the visibility value in the token detail string, matching the nepl-web contract.

## 検証

cargo check -p nepl-language; cargo build; gh run list/view after push

## 2026-05-13 修正

`nepl-language` の token extra 表示で `TokenKind::DirExtern` の `vis` field を明示的に受け取り、detail string に `vis={vis:?}` を含めるようにした。

これは field を `..` で握りつぶす修正ではなく、extern visibility が editor / language tooling の token detail でも欠落しないようにする追従である。`TokenKind` の構造変更を Rust の pattern exhaustiveness で検出できる状態も維持する。

検証:

- `cargo check -p nepl-language`: passed
- `cargo build`: passed
