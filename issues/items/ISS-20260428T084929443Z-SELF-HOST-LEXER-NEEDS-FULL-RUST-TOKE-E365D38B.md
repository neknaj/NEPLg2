---
id: ISS-20260428T084929443Z-SELF-HOST-LEXER-NEEDS-FULL-RUST-TOKE-E365D38B
title: "self-host lexer needs full Rust token JSON parity harness"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md, nodesrc/analyze_source.js"
---

# ISS-20260428T084929443Z-SELF-HOST-LEXER-NEEDS-FULL-RUST-TOKE-E365D38B: self-host lexer needs full Rust token JSON parity harness

## 概要

Indent/Dedent と #indent 幅更新は self-host lexer に入ったが、Rust lexer の analyze_lex JSON と完全比較するには keywords、directive token、doc/raw/mlstr token、newline span などを含む token model parity と比較 harness がまだ不足している。

## 対象

- `stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md, nodesrc/analyze_source.js`

## 根拠

- `ISS-20260428T000125736Z-SELF-HOST-LEXER-LACKS-OFFSIDE-TOKENS-3BD65FA4` で offside `Indent` / `Dedent` と `#indent` 幅更新は追加した。
- ただし現行 self-host lexer は `fn` などの keyword を `Identifier` として扱う段階で、Rust lexer の `KwFn` などとは token kind が一致しない。
- `#entry` / `#target` / `#import` / doc comment / raw wasm/llvm / mlstr など、Rust lexer JSON と比較するには追加 token kind と normalized comparison harness が必要。

## 問題

Indent/Dedent と #indent 幅更新は self-host lexer に入ったが、Rust lexer の analyze_lex JSON と完全比較するには keywords、directive token、doc/raw/mlstr token、newline span などを含む token model parity と比較 harness がまだ不足している。

## 影響

S1 parser parity を確認する段階で token kind と span の差分が混ざり、parser 側の不一致と lexer 側の不一致を切り分けにくい。

## 修正方針

self-host TokenKind を Rust lexer token set へ段階的に合わせ、Rust analyze_lex JSON と self-host lexer output を同一 fixture で比較する nodesrc harness を追加する。offside token の focused doctest は維持し、full parity は別 issue として進める。

## 検証

Rust analyze_lex JSON と self-host lexer output の normalized token stream 比較を、#entry/#target/#import/#indent、keyword、literal、doc/raw/mlstr、nested block source で実行する。

## 対応結果

- `TokenKind` を Rust `analyze_lex` JSON の kind 名へ合わせ、`Ident` / `Eof` / `Star` / `Arrow` / keyword / directive / doc / mlstr / raw text token を self-host 側へ追加した。
- `token_kind_name` は `KwFn`、`DirEntry`、`DocComment` など Rust JSON と同じ表記を返すようにした。
- `lex_next` に keyword 分類、`#entry` / `#target` / `#import` / `#use` / `#if[...]` / `#capability` / `#prelude` / `#no_prelude` / `#intrinsic` などの directive 分類、`DocComment`、`MlstrLine`、`Pipe`、`PathSep`、`At`、`Ampersand`、`Equals`、`Minus`、float / bool / hex int literal を追加した。
- `Newline` span を Rust lexer と同じ 0 byte span に寄せ、走査の次 offset は `lex_all_loop` 側で進めるようにした。
- `tests/stdlib/neplg2_lexer.n.md` の lexer 回帰テストへ `ret: 0` を追加し、チェック失敗が runner で検知されるようにした。
- directive / keyword / literal / doc / mlstr の self-host lexer focused doctest を追加した。
- `nodesrc/test_selfhost_lexer_rust_parity.js` を追加し、Rust `analyze_lex` JSON の normalized token stream を directive / keyword / literal / doc / mlstr / raw block fixture で固定した。

## 残件の分離

- `#wasm:` / `#llvmir:` 後の raw block 本文を self-host lexer が `WasmText` / `LlvmIrText` として生成するには、token kind 追加とは別に pending raw mode と base indent state が必要である。
- この stateful raw block 対応は `ISS-20260428T102223821Z-SELF-HOST-LEXER-RAW-BLOCK-STATE-IS-M-6F637EE2` に分離した。

## 実行した検証

- `trunk build`: pass
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-after-rebase-final.json -j 1`: 11/11 passed
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-lexer-parity-after-rebase-final.json -j 1`: 36/36 passed
- `node nodesrc/test_selfhost_lexer_rust_parity.js`: pass（2 fixtures / 79 tokens）
- `node nodesrc/issues.js check`: pass（files=251）
- `git diff --check HEAD`: pass（CRLF warning only）
