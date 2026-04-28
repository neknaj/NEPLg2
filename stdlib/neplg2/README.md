# NEPLg2 Self-Host Compiler

`stdlib/neplg2/` は NEPLg2.0 の現行 Rust コンパイラを NEPLg2.0 自身で再実装するための正規ソースツリーです。

このツリーは NEPLg3 の設計実験ではありません。`doc/neplg3/impl/compiler_structure.md` の分割方針を参考にしつつ、構文、型注釈、import、HIR、WASM/LLVM backend は現行 NEPLg2.0 を正とします。

## 層

- `core/`: filesystem、stdio、argv に依存しない純粋な compiler core。
- `cli/`: WASI / stdlib interface を使い、入力、diagnostic 表示、artifact 書き出しを担当する CLI 層。

## Stage 0 Skeleton / S1 Foundation

Stage 0 では各 pipeline stage の所有境界だけを固定し、各ファイルに実行可能な最小 doctest を置きます。実処理の移植は `selfhost/s0-infra-span-diag` 以降の issue で、依存順を崩さず追加します。

S1 の最初の基盤として、`core/infra/span.nepl` は byte offset ベースの `SelfhostSourceSpan` を持ち、`core/infra/text.nepl` は `SelfhostSourceText`、line start table、byte offset から line / column への変換を提供します。`core/infra/diag.nepl` と `core/infra/outcome.nepl` は、parser / resolver / checker / backend が共有する diagnostic value と diagnostic-carrying Result を提供します。`core/syntax/token.nepl` は `TokenKind` / `SelfhostToken` を定義します。`core/syntax/lexer.nepl` は whitespace、comment、identifier、integer literal、string literal、主要 punctuation、`#indent`、offside `Indent` / `Dedent`、EOF、lexical diagnostic を扱う byte lexer です。Rust lexer JSON との full parity harness は `ISS-20260428T084929443Z-SELF-HOST-LEXER-NEEDS-FULL-RUST-TOKE-E365D38B` で進めます。

## S2 Module Layer

`core/module/loader.nepl` は filesystem 非依存の in-memory VFS と single-module load entry を提供します。`core/module/import_spec.nepl` は parser が保持した import directive を `SelfhostImportSpec` に変換し、`core/module/graph.nepl` は root module から import closure を構築して missing module と cycle を `SelfhostDiagnostic` として返します。stdlib root / user root の path mapping は後続の `stdlib_map` stage で追加します。

## 検証

```powershell
node nodesrc/tests.js -i stdlib/neplg2 --no-tree -o tmp/neplg2-selfhost-placeholder.json -j 2
node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-foundation-focused.json -j 1
node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_text.n.md --no-tree -o tmp/neplg2-source-text-focused.json -j 1
node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_diag_outcome.n.md --no-tree -o tmp/neplg2-diag-outcome-focused.json -j 1
node nodesrc/tests.js -i tests/stdlib/neplg2_module_graph.n.md --no-tree -o tmp/neplg2-module-graph-focused.json -j 1
```
