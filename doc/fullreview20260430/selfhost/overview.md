# Selfhost Compiler Review: Overview

対象 commit: `f108cebd`

## 結論

`stdlib/neplg2/` は S0 の placeholder tree からは明確に前進しており、S1 lexer/parser、S2 in-memory module loader、CLI 境界の一部は実装済みである。ただし、selfhost compiler 全体としてはまだ bootstrap 実装を開始できる段階ではない。S3 型検査、S4 HIR/resource/mono、S5 backend は受け皿と smoke API が中心で、Rust compiler と同等の安全性判定を持っていない。

今すぐ進めてよい範囲は、S1/S2 の Rust parity fixture、typed diagnostic、source map、module graph、CLI I/O 境界である。S3 以降は Rust 側の Resource IR / diagnostic enum / match exhaustiveness 方針を正として、旧 move_check や HIR special-case を selfhost へコピーしないことが必須である。

## Actions 根拠

review の test 状況は local 実行ではなく GitHub Actions を根拠にした。

- Actions run: `25157230630`
- 対象 commit: `f108cebdf72289251b5d9f90c0fd7de4ca591e6e`
- 全体 conclusion: failure
- `stdlib-test`: `415 total / 232 passed / 173 failed / 10 errored`
- selfhost 関連 failure: `stdlib/neplg2/` で 19 件

selfhost 関連の failure は次の傾向である。

- timeout: `cli/args/options.nepl`, `cli/args/parse.nepl`, `cli/driver.nepl`, `core/module/graph.nepl`, `core/module/loader.nepl`, `core/module/stdlib_map.nepl`, `core/pipeline.nepl`, `core/syntax/parser/module_parser.nepl`
- owner / Resource IR: `cli/file_io.nepl`, `cli/reporter.nepl`, `core/hir/hir.nepl`, `core/infra/diag.nepl`, `core/module/import_spec.nepl`, `core/resolve/name_resolver.nepl`, `core/ty/ty.nepl`

このため、現状は「selfhost 部品は存在するが CI の stdlib doctest は green ではない」と判断する。

## S0-S7 進捗

| Stage | 判定 | 根拠 |
|---|---|---|
| S0 tree / smoke | 実装済み | `stdlib/neplg2/core` と `stdlib/neplg2/cli` が存在し、各 module に stage0 / doctest がある |
| S1 lexer / parser | 部分実装 | `token.nepl`, `lexer.nepl`, `module_parser.nepl`, `module_ast.nepl` があるが、parser は full expression parser ではない |
| S2 module loader | 部分実装 | VFS, import spec, stdlib map, module graph があるが Actions では module doctest timeout が残る |
| S3 type / check | 初期実装 | `ty.nepl` は arena と primitive/function model を持つが `check/checker.nepl` は stage0 smoke に近い |
| S4 HIR / resource / mono | 初期実装 | HIR flat table と mono key はあるが `resource/move_state.nepl` は未実装相当 |
| S5 backend | 未着手相当 | `codegen/wasm/binary.nepl` と `codegen/llvm/text.nepl` は marker API の段階 |
| S6 CLI | 部分実装 | args, reporter, file_io, driver はあるが artifact build / bootstrap compile へは未接続 |
| S7 bootstrap comparison | 未着手 | Pass A / Pass B 比較 job はない |

## 進捗状況

- `stdlib/neplg2/core/infra`: typed diagnostic / span / source text / outcome の基盤は実装中。
- `stdlib/neplg2/core/syntax`: token enum と byte lexer は実装が厚いが、parser は item stream 段階。
- `stdlib/neplg2/core/module`: VFS / import / path map / graph はあるが、large graph 向け lookup と Actions timeout が残る。
- `stdlib/neplg2/core/resolve`: name scope の table はあるが、full Rust resolve parity は未達。
- `stdlib/neplg2/core/ty`: TypeId arena と primitive/function record はあるが、unify / subst / effect / layout は未分離。
- `stdlib/neplg2/core/check`: checker orchestration は未完成。
- `stdlib/neplg2/core/hir`: flat HIR table はあるが lowering は未実装。
- `stdlib/neplg2/core/resource`: move/borrow/drop の authority は未実装。
- `stdlib/neplg2/core/mono`: instance key model の初期実装。
- `stdlib/neplg2/core/codegen`: WASM / LLVM backend は未実装相当。
- `stdlib/neplg2/cli`: pure args parser と reporter 境界はあるが、実 artifact pipeline は未完成。

## 新規 issue

review 中に、parser が `TokenKind` を文字列化して `hash32` の数値 arm で分岐している問題を確認した。これは enum/match による網羅性検査を捨てる設計なので、次の issue を追加した。

- `ISS-20260430T141517141Z-SELF-HOST-PARSER-CLASSIFIES-TOKENKIN-645D236B`

## selfhost 実装開始可否

開始してよい作業:

- lexer/parser の Rust token / AST JSON parity
- source map、diagnostic、module loader、stdlib map、CLI args/reporter の completion
- `.n.md` 共有 fixture を selfhost stage runner が消費できる形式への整理
- stdlib string / hash / fs / stdio の selfhost 用不足 API 整備

まだ開始すべきでない作業:

- selfhost S3 以降の独自 typecheck 設計
- Resource IR を介さない move/borrow/drop checker
- raw memory helper を前提にした builder / ByteBuf / string backend
- Rust 側 diagnostic ID redesign と異なる selfhost diag ID 体系

## 優先順

1. Actions 上の selfhost timeout と owner failure を分類し、stdlib 側 owner issue と selfhost 側設計 issueに分離する。
2. `module_parser` の string/hash dispatch を `TokenKind` direct match へ直す。
3. parser / module / CLI の stage0 doctest を `.n.md` stdout report policy に合わせる。
4. S3 以降は Rust Resource IR の最終設計が固まった範囲から移植する。
