# Rust コンパイラ lexer / parser / AST レビュー

確認対象 commit: `3742a1a7 fix(cli): run Resource IR gates for check-only`

## 確認範囲

- `nepl-core/src/lexer.rs`
- `nepl-core/src/parser.rs`
- `nepl-core/src/ast.rs`
- `nepl-core/src/span.rs`
- `nepl-core/src/source_map.rs`
- `nepl-core/tests/{char,doc_comments,parser_debug}.rs`
- `tests/compiler/{lexer_diag,offside_and_indent_errors,match_literal_patterns,match_enum_wildcard_patterns,char_cast,literal_diagnostics}.n.md`

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| lexer token model | indent/dedent、directives、doc comments、raw blocks、char/string literals、keywords を `TokenKind` enum で持つ。 | 良い。char literal は Rust 側に入っている。 |
| lexer diagnostics | `LexerDiagnosticCode` / `ParserDiagnosticCode` を生成時点で付与する helper がある。 | 良い。code-less 化していない。 |
| parser recovery | recursion limit、no-progress recovery、token expected/unexpected diagnostics がある。 | 方向は良い。巨大 file のため境界監査は弱い。 |
| match parsing | variant、wildcard、bool/int/char literal pattern を AST に載せる。 | typecheck の網羅性検査に必要な情報は出ている。 |
| char support | lexer `CharLiteral(u32)`、parser `Literal::Char` / `MatchPattern::CharLiteral` が存在。 | 実装済み。stdlib 連携は stdlib review で確認する。 |
| file size | `parser.rs` 4234 lines、`lexer.rs` 1259 lines、`ast.rs` 408 lines。 | parser は分割 issue 対象。 |

## 良い点

- char literal と char match pattern が lexer/parser/typecheck/codegen の対象に入っている。
- parser diagnostic は `DiagnosticCode::Parser` を直接使い、`.with_code` 後付けへ戻っていない。
- target/profile conditional gate は parser 内では directive として保持し、active 判定は `target_gate` 側へ寄せている。
- `MAX_PARSE_RECURSION_DEPTH` と no-progress recovery があり、異常入力で parser が無限再帰・無限 loop しにくい。

## 問題

### parser が巨大 file のまま

`parser.rs` は 4234 lines あり、declaration parsing、block parsing、prefix expression parsing、match parsing、type expression、extern signature、recovery、token navigation が同居している。typecheck / ResourceIR は分割と source policy が進んだため、parser だけが巨大単一 file のまま残ると、将来の syntax 追加や selfhost parity で同じ複雑さを移植しやすい。

これは `ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587` として追跡する。

### 小さな保守 drift

`parser.rs` 冒頭の module doc line が重複している。動作上の問題ではないが、巨大 file の保守粒度が粗くなっている兆候であり、分割時に整理するべきである。

## 次に確認すること

- selfhost parser が Rust parser の giant-file 構造をそのまま移植せず、token action enum と match dispatch で分割されているか。
- parser source policy を追加する際に、match/type expression/recovery の責務を別 module として固定できるか。
