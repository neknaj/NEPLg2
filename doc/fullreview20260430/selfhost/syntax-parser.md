# Selfhost Compiler Review: Syntax And Parser

対象 commit: `f108cebd`

## 対象

- `stdlib/neplg2/core/syntax/token.nepl`
- `stdlib/neplg2/core/syntax/lexer.nepl`
- `stdlib/neplg2/core/syntax/ast/module_ast.nepl`
- `stdlib/neplg2/core/syntax/parser/module_parser.nepl`

## 設計評価

token model は `TokenKind` enum と `SelfhostToken` に整理されており、`CharLiteral`、raw backend text、directive token、offside token を持つ。これは selfhost S1 の Rust lexer parity に必要な基盤として妥当である。

lexer は byte scanner と offside token 生成が実装されている。`#indent`、doc comment、char/string literal、raw backend block などが入り、以前の placeholder ではない。一方で `lexer.nepl` は 1200 行超で、keyword / directive / literal / offside / raw block の責務がまだ集中している。S1 parity が進むほど分割が必要になる。

parser は module item stream を作る段階であり、full expression parser ではない。この段階を明記していること自体は良い。しかし `module_parser.nepl` は `TokenKind` を `token_kind_name` で文字列化し、`hash32` の数値 arm で item kind を分類している。これは enum/match による網羅性検査を捨てているため、selfhost の静的検査方針に反する。

## Actions 根拠

Actions run `25157230630` では次の selfhost syntax/parser failure がある。

- `stdlib/neplg2/core/syntax/parser/module_parser.nepl::doctest#1`: timeout
- `stdlib/neplg2/core/module/loader.nepl::doctest#1`: timeout。parser 経路を含む。
- `stdlib/neplg2/core/module/graph.nepl::doctest#1`: timeout。parser / import spec 経路を含む。

timeout は local runtime の観測ではなく、GitHub Actions artifact/log による確認である。

## 良い点

- `TokenKind` は enum で、token kind の finite state を型で持っている。
- `token_kind_name` は Rust `analyze_lex` JSON parity のための表示境界として有用。
- parser raw mode は `SelfhostParserRawMode` enum で表現されている。
- char literal の lexer token は既に入っている。
- parser は raw text outside block を diagnostic にしており、lexer/parser 状態ずれを検出する意図がある。

## 問題

- `module_parser.nepl` の item 分類が `TokenKind -> str -> hash32 -> numeric match -> string equality` になっている。
- この分岐は token enum の追加に対して compile-time の exhaustiveness failure を出せない。
- hash arm の数値が grammar mapping を隠し、review と保守が難しい。
- lexer が大きく、literal scanning / directive scanning / offside stack / raw block を分割しないと S1 parity の差分調査が困難になる。
- parser は module item stream までで、expr/type/pattern parser は未実装である。

## 追加 issue

- `ISS-20260430T141517141Z-SELF-HOST-PARSER-CLASSIFIES-TOKENKIN-645D236B`

## 必要な設計

- parser の分類は `TokenKind` を直接 `match` する。
- `token_kind_name` は JSON / reporter / parity output の境界だけで使う。
- hash/string classification が必要な場合でも、source text の keyword lookup など入力が本当に string である箇所に限定する。
- parser stage は module parser、item parser、expr parser、type parser、pattern parser に分ける。
- Rust parser parity は token JSON だけではなく AST JSON fixture で確認する。

## 進捗状況

- `token.nepl`: 実装中。enum token kind と name mapping あり。
- `lexer.nepl`: 実装中。主要 token と offside rule はあるが巨大化。
- `ast/module_ast.nepl`: 初期実装。module item stream 用。
- `parser/module_parser.nepl`: 初期実装。raw block と top-level item extraction はあるが full parser ではない。

## 判定

S1 は継続して進められる。ただし parser の string/hash dispatch は早めに直すべきである。selfhost が型安全・メモリ安全を担う compiler になる以上、parser の finite state から静的検査を外す設計は残してはいけない。
