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

`core/module/loader.nepl` は filesystem 非依存の in-memory VFS と single-module load entry を提供します。`core/module/import_spec.nepl` は parser が保持した import directive を `SelfhostImportSpec` に変換し、`core/module/stdlib_map.nepl` は stdlib root / user root から VFS logical path を解決します。`core/module/graph.nepl` は root module から import closure を構築して missing module と cycle を `SelfhostDiagnostic` として返し、`selfhost_build_module_graph_with_path_map` では `core/result` と `./util` のような import を同じ graph に載せます。

## S3 Type Layer

`core/ty/ty.nepl` は `SelfhostTypeId`、`SelfhostTypeKind`、`SelfhostTypeArena` を提供します。primitive type、`i64` / `f64` の named numeric primitive、function type を arena-local stable id として登録し、function type の引数列は arena の argument table に集約します。`selfhost_type_arena_types_equal` は同じ arena 内の valid `TypeId` を構造比較し、unify / overload / checker が record inspection を重複実装しないための入口です。struct / enum / type variable / effect / layout は、Rust 実装との parity fixture を作りながら後続 issue で追加します。

## S4 Mono Foundation

`core/mono/mono.nepl` は generic instance の元定義を表す `SelfhostMonoDefId`、type argument table の範囲を表す `SelfhostMonoTypeArgRange`、cache lookup 用の `SelfhostMonoInstanceKey`、instance table index の `SelfhostMonoInstanceId` を提供します。`selfhost_mono_instance_key_seed` は name mangling / cache bucket 用の deterministic seed を返します。実際の cache storage、trait impl lookup、HIR 複製は後続 issue で追加します。

## S6 CLI Boundary

`cli/args/types.nepl` は CLI driver / reporter / parser が共有する `SelfhostCliTarget`、`SelfhostCliEmitSet`、`SelfhostCliOptions` などの public option 型を提供します。`cli/args/classify.nepl` は argv token / option value の有限集合を hash dispatch と enum `match` で分類します。`cli/args/emit.nepl` は `--emit` の artifact set 合成、comma 区切り parser、emit 判定 helper を提供します。`cli/args/options.nepl` は parser state から `SelfhostCliOptions` を構築し、CLI option model を core compile option へ変換します。`cli/args/parse.nepl` は argv index 走査と usage error state machine を担当し、`cli/args/predicates.nepl` は target/error enum predicate を担当します。`cli/args.nepl` は既存 import path の compatibility facade として `args/types`、`args/emit`、`args/options`、`args/parse`、`args/predicates` を `pub #import` で再 export します。`cli/reporter.nepl` は core diagnostic を human stderr text と compact JSON に変換し、Result 付き stdio API で stdout/stderr を分離します。`cli/driver.nepl` は VFS と parsed options を受け取り、core pipeline の root load result を exit code と diagnostics に正規化します。`cli/file_io.nepl` は `std/fs` 依存を閉じ込め、root source の checked read から VFS への登録と text / binary artifact write を担当します。driver の artifact slice では、この file_io boundary を呼び出すだけにします。

## 検証

```powershell
node nodesrc/tests.js -i stdlib/neplg2 --no-tree -o tmp/neplg2-selfhost-placeholder.json -j 2
node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-foundation-focused.json -j 1
node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_text.n.md --no-tree -o tmp/neplg2-source-text-focused.json -j 1
node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_diag_outcome.n.md --no-tree -o tmp/neplg2-diag-outcome-focused.json -j 1
node nodesrc/tests.js -i tests/stdlib/neplg2_module_graph.n.md --no-tree -o tmp/neplg2-module-graph-focused.json -j 1
node nodesrc/tests.js -i tests/stdlib/neplg2_stdlib_map.n.md --no-tree -o tmp/neplg2-stdlib-map-focused.json -j 1
node nodesrc/tests.js -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/neplg2-type-arena-focused.json -j 1
node nodesrc/tests.js -i tests/stdlib/neplg2_mono.n.md --no-tree -o tmp/neplg2-mono-focused.json -j 1
node nodesrc/tests.js -i tests/stdlib/selfhost_cli_file_io.n.md --no-tree -o tmp/selfhost-cli-file-io-focused.json -j 1
```
