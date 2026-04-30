# Lexer / parser / AST review

対象 commit: `f108cebd`

## 概要

`lexer.rs` と `parser.rs` は現行 NEPLg2 構文の正規実装である。char literal、`match`、`#indent`、offside rule、if/while layout、type annotation、raw body、module directives を広く扱っている。

## 進捗

- `TokenKind` は enum で構文要素を表している。
- char literal は lexer 側で `lex_char_literal` / escape / unicode scalar validation を持つ。
- parser は `MatchPattern::CharLiteral` を持ち、match arm で char literal を扱える。
- `#indent` と indentation mismatch は lexer diagnostic code を持つ。
- parser は `match` expression と arms を AST に持ち、typecheck の exhaustiveness へ渡している。

## 良い点

- token / AST kind が enum であり、raw string dispatch より静的検査に向いている。
- char literal の validation が lexer に入り、typecheck match まで接続されている。
- `match` 構文は parser / AST / HIR の段階を通るため、stdlib の finite branch を `match` に寄せる基盤がある。

## 残る問題

- `parser.rs` は約 4000 行で、layout marker、item parser、expr parser、match parser、type parser、recovery helper が同居している。
- parser 内には `unwrap` が多数ある。多くは直前の `peek` や `expect` による invariant だが、selfhost へ移植する際は parser state transition を `Result` / typed recovery で明示した方がよい。
- `parse_import_directive` など directive parsing の一部は文字列 split への依存が残る。selfhost では import spec を lexer/parser の typed structure として早期に作るべきである。

## selfhost への示唆

selfhost parser は `stdlib/neplg2/core/syntax/parser/` 配下で module / item / expr / type / pattern に分ける。Rust `parser.rs` の挙動を parity fixture で追うが、巨大単一 parser をコピーしない。

特に `#indent`、char literal、match arm、block/argument offside rule は、Rust 側と同じ diagnostic taxonomy を使う。finite syntax kind は enum 化し、文字列 keyword 比較の結果を後段で持ち回らない。
