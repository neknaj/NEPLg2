---
id: ISS-20260428T000125736Z-SELF-HOST-LEXER-LACKS-OFFSIDE-TOKENS-3BD65FA4
title: "self-host lexer lacks offside tokens and Rust parity fixtures"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/parser/**, tests/stdlib/neplg2_lexer.n.md"
---

# ISS-20260428T000125736Z-SELF-HOST-LEXER-LACKS-OFFSIDE-TOKENS-3BD65FA4: self-host lexer lacks offside tokens and Rust parity fixtures

## 概要

stdlib/neplg2 の lexer は whitespace/comment/identifier/int/string/punctuation/EOF の byte lexer まで進んでいるが、#indent と offside rule による Indent/Dedent token をまだ生成せず、Rust lexer の token JSON と比較する parity fixture もない。

## 対象

- `stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/parser/**, tests/stdlib/neplg2_lexer.n.md`

## 根拠

- `stdlib/neplg2/core/syntax/token.nepl` の `TokenKind` は `Newline` までで、`Indent` / `Dedent` を持たない。
- `stdlib/neplg2/core/syntax/lexer.nepl` は horizontal whitespace と newline を個別 token として扱うだけで、indent stack や `#indent` directive による offside rule を持たない。
- `stdlib/neplg2/README.md` は indent / dedent と Rust lexer JSON parity を後続 issue と記述しているが、open issue としてはまだ分離されていなかった。

## 問題

stdlib/neplg2 の lexer は whitespace/comment/identifier/int/string/punctuation/EOF の byte lexer まで進んでいるが、#indent と offside rule による Indent/Dedent token をまだ生成せず、Rust lexer の token JSON と比較する parity fixture もない。

## 影響

S1 の成功条件である lexer/parser parity に到達できず、parser 実装を開始しても実際の NEPLg2 source の block 構造を安定して読めない。後続の AST JSON 比較も token 境界の不一致に引きずられる。

## 修正方針

TokenKind に Indent/Dedent と directive に必要な token を追加し、Rust lexer と同じ offside rule を self-host lexer に移植する。Rust 側 tree fixture と同じ入力で token JSON を比較する focused test を追加し、その上で parser module を進める。

## 検証

stdlib/neplg2 lexer focused doctest で、indent/dedent、`#indent` 幅更新、indent mismatch diagnostic を確認する。Rust lexer JSON との full parity fixture は residual を分離した issue で扱う。

## 対応結果

- `TokenKind` に `Indent` / `Dedent` / `DirIndentWidth` を追加し、`token_kind_name` と token 判定 helper を網羅的に更新した。
- `lex_all` に indent stack を追加し、行頭 whitespace から `Indent` / `Dedent` を生成するようにした。
- `#indent N` は `DirIndentWidth` token として読み、後続行の offside 幅へ反映するようにした。
- EOF 前に未閉じ indent level を `Dedent` で閉じるようにした。
- 既存 indent stack に存在しない幅へ dedent した場合は `lex.invalid_indentation` を返すようにした。
- nested indent/dedent、`#indent 2`、indent mismatch を `tests/stdlib/neplg2_lexer.n.md` の focused doctest に追加した。

## 残件の分離

Rust lexer JSON との full parity には keyword/directive/doc/raw/mlstr token と比較 harness がさらに必要なため、`ISS-20260428T084929443Z-SELF-HOST-LEXER-NEEDS-FULL-RUST-TOKE-E365D38B` に分離した。

## 実行した検証

- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-offside-fourth.json -j 1`: 8/8 passed
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-offside-focused.json -j 1`: 33/33 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-offside-after-sync.json -j 1`: 8/8 passed
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-offside-after-sync.json -j 1`: 33/33 passed
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-offside-width-after-sync.json -j 1`: 9/9 passed
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-offside-width-after-sync.json -j 1`: 34/34 passed
- `trunk build`: pass after syncing `origin/main` through `6e8dfce`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-offside-after-shadow-gate.json -j 1`: 9/9 passed
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-offside-after-shadow-gate.json -j 1`: 34/34 passed
- `node nodesrc/issues.js check`: pass
