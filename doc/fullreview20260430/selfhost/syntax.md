# selfhost syntax review

確認対象 commit: `31291b37 fix(core): add parser backend responsibility policy`

## 確認対象

- `stdlib/neplg2/core/syntax/token.nepl`
- `stdlib/neplg2/core/syntax/lexer.nepl`
- `stdlib/neplg2/core/syntax/ast/module_ast.nepl`
- `stdlib/neplg2/core/syntax/parser/module_parser.nepl`
- `tests/stdlib/neplg2_lexer.n.md`
- `nodesrc/test_selfhost_lexer_rust_parity.js`

## 良い点

`TokenKind` は Rust compiler の token 語彙に近い enum として管理され、`token_kind_name` も exhaustive match で stable string を返している。char literal、string literal、doc comment、raw wasm/llvmir text、directive token、indent/dedent まで含まれている。

lexer は `#indent` のような文字列判定で `alloc/string/search::str_starts_with_at` を使っており、以前問題になった byte-by-byte 文字列比較には戻っていない。

module parser は `SelfhostParserRawMode` と `SelfhostParserTokenAction` を enum として持ち、raw text の pending/active state や token action を match で扱っている。parser 側の raw mode model は lexer より良い。

module AST は import directive など S1/S2 に必要な item を保持し、import graph が parser 内部へ直接依存しすぎない境界を作っている。

## 問題とリスク

lexer の raw block state はまだ `i32` sentinel である。`raw_mode` / `pending_raw_mode` が `0/1/2` を持ち、`lex_token_pending_raw_mode` は数値を返し、`lex_raw_kind` は `raw_mode == 1` 以外を `LlvmIrText` に倒す。これは unexpected value を静的検査で拒否できない。

directive 分類は有限集合にもかかわらず deep nested `if` chain になっている。keyword classifier や CLI args classifier のような hash/key + match + string verification に寄せれば、directive 追加時の見落としを source policy で固定しやすい。

`token.nepl` は predicate helper が TokenKind 全 variant を列挙する巨大 match を複数持つ。これは static-check friendly ではあるが、file size と重複の観点では責務分割が必要になる可能性が高い。巨大 file split open issue の selfhost 入力として扱う。

module parser はまだ executable module の full AST ではない。S1/S2 の import/module item model としてはよいが、expression/statement/function/type syntax へ進むと Rust parser の巨大 file 問題を selfhost に移植する危険がある。

## 追加 issue

- `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B`
  - raw mode を `SelfhostLexerRawMode` enum へ移行する。
  - directive 分類を finite classifier と source policy へ移す。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `syntax/token.nepl` | TokenKind enum と stable name match。 | 良いが巨大化注意。 |
| `syntax/lexer.nepl` | Rust token parity に必要な char/raw/indent/directive がある。 | raw mode と directive classifier の再設計が必要。 |
| `syntax/ast/module_ast.nepl` | module item / import directive AST。 | S2向けには十分。full AST は未実装。 |
| `syntax/parser/module_parser.nepl` | module item stream parser、raw block mode enum。 | parser raw modeは良い。full parser化時に分割必須。 |
| lexer parity tests | Rust/selfhost parity harness がある。 | raw/directive regression を拡張する。 |

## 推奨対応

- lexer raw state を enum/record 化し、unexpected raw mode が token kind fallback にならないようにする。
- directive classifier を keyword classifier と同じ設計へ移し、source policy で deep nested directive chain の再導入を拒否する。
- parser 拡張時は Rust `parser.rs` の巨大構造をコピーせず、module/directive/item/expression/type/parser-recovery を分割する。
- char/string/raw text の fixture は Rust lexer JSON と selfhost lexer output の kind/span parity を継続的に比較する。
