# NEPLg2.0 セルフホスト詳細実装計画

最終更新: 2026-04-26

---

## 1. 目的

NEPLg2.0 の現行 Rust コンパイラを、NEPLg2.0 自身で段階的に再実装する。
この計画の対象は **NEPLg2.0 の self-host compiler** であり、NEPLg3 の仕様実装ではない。

NEPLg3 向けの `doc/neplg3/impl/compiler_structure.md` は、ディレクトリ分割・依存方向・巨大ファイル分割の参考にする。
ただし、NEPLg2.0 では現行の `#import`、angle bracket 型注釈、`fn` 糖衣構文、現行 HIR / WASM / LLVM backend を正とする。

branch、commit、Rust 側 compiler 修正の取り込み、self-host 実装中に発覚した Rust 側 Issue の提出規則は [self_host_execution_plan.md](./self_host_execution_plan.md) を正とする。

---

## 2. 成功条件

| 段階 | 成功条件 |
|---|---|
| S0 | `stdlib/neplg2/` に self-host compiler のディレクトリ骨格と placeholder doctest がある |
| S1 | NEPLg2.0 lexer / parser が `.nepl` ソースを読み、Rust 実装と同じ token / AST JSON を返す |
| S2 | import / module loading が `stdlib/` と複数 input file を解決できる |
| S3 | 型推論、名前解決、overload、trait capability、effect 判定が現行 compiler fixture と一致する |
| S4 | HIR、move / borrow / drop、monomorphize が現行 Rust 実装と同じ判定を返す |
| S5 | self-host WASM backend が主要 smoke fixture を compile し、Rust backend と同じ実行結果を出す |
| S6 | self-host compiler が `stdlib/` と自分自身を compile できる |
| S7 | Rust compiler で作った self-host compiler と、self-host compiler が再生成した compiler の結果を比較できる |

完了判定は「作れる」ではなく、Pass A / Pass B の再現性と既存 regression test で判断する。

---

## 3. 非目標

- NEPLg3 構文、NEPLg3 Resource IR、NEPLg3 module system を先取りしない。
- `nepl-core` の Rust 実装を一気に置き換えない。
- self-host 初期段階で LLVM backend 完全 parity を要求しない。まず WASM / WASI を基準にする。
- `stdlib/neplg3/` を NEPLg2.0 self-host の作業場所として使わない。

---

## 4. 二層構造

| 層 | 場所 | I/O | 責務 |
|---|---|---|---|
| Core | `stdlib/neplg2/core/` | なし | source map、lexer、parser、typecheck、HIR、backend などの純粋 compiler core |
| CLI | `stdlib/neplg2/cli/` | WASI / std | argv、filesystem、stdio/stderr、diagnostic 表示、artifact 書き出し |

`core/` は `std/fs`、`std/stdio`、`std/env/cliarg` に依存しない。
外部入力は `cli/` が `SourceFile` / `VirtualFileSystem` として core へ渡す。

---

## 5. 推奨ディレクトリ構造

`doc/neplg3/impl/compiler_structure.md` の分割を参考にするが、NEPLg2.0 の現行 pipeline に合わせて責務を調整する。

```text
stdlib/neplg2/
    core/
        infra/
            span.nepl             # FileId, SourceSpan, SourceMap
            diag.nepl             # Diagnostic, Severity, Label, Note
            outcome.nepl          # 診断付き Result
            text.nepl             # source text / byte offset / line map

        syntax/
            token.nepl            # Token, TokenKind
            lexer.nepl            # #indent と offside rule を含む tokenizer
            ast/
                module_ast.nepl
                item.nepl
                expr.nepl
                typeexpr.nepl
                pattern.nepl
            parser/
                module_parser.nepl
                item_parser.nepl
                expr_parser.nepl
                type_parser.nepl
                pattern_parser.nepl

        module/
            file_id.nepl           # logical file identity
            import_spec.nepl       # #import / #use / #part の表現
            graph.nepl             # import graph と cycle 検出
            loader.nepl            # VirtualFileSystem から ModuleAst を収集
            stdlib_map.nepl        # /stdlib path と user root の対応

        resolve/
            def_id.nepl
            scope.nepl
            hoist.nepl             # immutable let / fn の巻き上げ
            import_resolver.nepl
            name_resolver.nepl

        ty/
            ty.nepl                # TypeId, TypeKind
            arena.nepl
            unify.nepl
            subst.nepl
            effect.nepl
            layout.nepl            # backend 前の型 layout 情報

        check/
            checker.nepl           # pass orchestration
            expr_reduce.nepl       # NEPLg2.0 reduce_calls
            expr_check.nepl
            decl_check.nepl
            overload.nepl
            trait_check.nepl
            effect_check.nepl
            pattern_check.nepl

        hir/
            hir.nepl
            lower.nepl
            walk.nepl              # deep HIR を iterative に走査する helper

        resource/
            move_state.nepl        # 現行 move_check の状態機械
            borrow.nepl
            drop_plan.nepl
            branch_merge.nepl

        mono/
            instance.nepl
            mono.nepl

        codegen/
            wasm/
                binary.nepl        # byte builder への emit
                section.nepl
                leb128.nepl
                layout.nepl
                intrinsic.nepl
                runtime.nepl
            llvm/
                text.nepl          # LLVM IR text emitter
                layout.nepl
                intrinsic.nepl

        builtins/
            primitives.nepl
            intrinsics.nepl
            prelude.nepl

        pipeline.nepl              # compile_* API
        options.nepl               # CompileOptions, Target, Profile

    cli/
        main.nepl                  # #entry
        args.nepl                  # argv parser
        file_io.nepl               # fs/stdio bridge
        reporter.nepl              # diagnostics to stderr / json
        driver.nepl                # core pipeline 呼び出し
```

---

## 6. NEPLg3 実装設計から採用するもの

| 採用する考え方 | NEPLg2.0 での適用 |
|---|---|
| pipeline stage ごとのディレクトリ分割 | `syntax/`、`module/`、`resolve/`、`ty/`、`check/`、`hir/`、`resource/`、`mono/`、`codegen/` に分割 |
| `infra/diag` の早期整備 | parser / typecheck / codegen の全段で同じ Diagnostic を返す |
| `core` と `cli` の分離 | `core/` は filesystem を持たず、`cli/` が VFS を構築する |
| 巨大 typecheck の分割 | `expr_reduce`、`overload`、`trait_check`、`effect_check`、`lower` を分離 |
| `nm/` を core から外す | self-host compiler core は `.n.md` 抽出を直接持たず、test tool 側に分離する |

---

## 7. NEPLg3 から採用しないもの

| 採用しないもの | 理由 |
|---|---|
| TypePrefixList / kind-directed type application | NEPLg2.0 は angle bracket 型式で構文境界を持つ |
| NEPLg3 の `let name expr` 統一宣言 | NEPLg2.0 は現行 `fn` / `let` / `struct` / `enum` を正とする |
| NEPLg3 Resource IR 完全形 | NEPLg2.0 は現行 move / borrow / drop 仕様を再現することを優先する |
| `nepl-core-g3` 前提の crate 分割 | NEPLg2.0 self-host は `stdlib/neplg2/` に置く |

---

## 8. 実装フェーズ

### S0: 骨格とテスト経路

- `stdlib/neplg2/` を作成する。
- 各 top-level module に `neplg2:test[skip]` ではなく、可能な限り実行可能な小 doctest を置く。
- `nodesrc/tests.js -i stdlib/neplg2 --no-tree` を self-host 用 focused check とする。
- `issues/` の self-host issue を作業単位に紐付ける。

2026-04-26 時点で `stdlib/neplg2/` は Stage 0 の正規ソースツリーとして作成済み。
`core/infra`、`syntax/ast`、`syntax/parser`、`module`、`resolve`、`ty`、`check`、`hir`、`resource`、`mono`、`codegen/wasm`、`codegen/llvm`、`builtins`、`core/pipeline.nepl`、`core/options.nepl`、`cli/` に marker API と実行可能 doctest を置いている。
focused check は次のコマンドを基準にする。

```powershell
node nodesrc/tests.js -i stdlib/neplg2 --no-tree -o tmp/neplg2-selfhost-placeholder.json -j 2
```

### S1: SourceMap / lexer / parser

- `infra/text.nepl` で byte offset、line/column、source file id を扱う。
- `syntax/lexer.nepl` は `#indent` と offside rule を Rust lexer と同じ token stream へ写す。
- `syntax/parser/*` は Rust AST と比較しやすい最小 AST を生成する。
- 受け入れ基準は `tests/compiler/tree/01_lex_tree.js`、`02_parse_tree.js` 相当の JSON parity。

### S2: Module loader

- `cli/file_io.nepl` が input file と stdlib root を読み、`core/module/loader.nepl` へ `VirtualFileSystem` を渡す。
- `#import "core/result" as *`、alias import、selective import、cycle detection を Rust 実装と合わせる。
- filesystem 直結ではなく、core は `path -> source` の map だけを扱う。

### S3: Type / check

- `ty/arena.nepl` で TypeId を安定化する。
- `check/expr_reduce.nepl` に現行 `reduce_calls` を移植する。
- overload は `TypeCtx` checkpoint / rollback 相当を NEPLg2.0 で表現する。
- trait capability と Copy / Clone / Drop 判定を現行 stdlib と合わせる。
- scope、symbol、module、diagnostic のような大規模 mutable table には `BTreeMap` / `BTreeSet` 互換実装を使わない。これらは sorted-array 実装で、更新が O(n) になる。
- 安定順序が必要な出力では、構築中は `HashMap` / `HashSet` を主構造にし、最終出力段階だけ key list を整列する。小さな ordered table に限って `sorted_array_map_*` / `sorted_array_set_*` alias を使い、O(n) 更新を意図として明示する。

### S4: HIR / resource / mono

- `hir/lower.nepl` で AST + type result を HIR へ落とす。
- `resource/move_state.nepl` は現行 `passes/move_check.rs` の state machine を仕様化してから移植する。
- deep HIR traversal は再帰ではなく explicit stack で実装する。
- `mono/` は instance cache と name mangling を分離する。

### S5: WASM backend

- `codegen/wasm/leb128.nepl` と `binary.nepl` を先に作る。
- `ByteBuilder` を使い、raw pointer 直接操作を backend 内に閉じ込めない。
- smoke fixture は `println_i32`、`stdout`、`if`、`function call`、`struct/enum`、`Result` の順に増やす。

### S6: CLI

- `cli/args.nepl` は pure parser とし、raw argv provider と分離する。
- `cli/reporter.nepl` は stderr human diagnostic と JSON output を分ける。
- `cli/driver.nepl` は compile result、exit code、artifact write を一箇所で統合する。

### S7: Bootstrap comparison

1. Rust `nepl-core` が `stdlib/neplg2/cli/main.nepl` を compile する。
2. 生成された self-host compiler が同じ input を compile する。
3. 出力 artifact、diagnostic JSON、exit code を比較する。
4. 差分がある場合は、self-host core の stage ごとの trace JSON で切り分ける。

---

## 9. 不足している外部 interface

| Issue | 不足機能 | self-host への影響 |
|---|---|---|
| [ISS-20260426T010001Z-STDFS-WRITE-B7C4D923](../../issues/items/ISS-20260426T010001Z-STDFS-WRITE-B7C4D923.md) | `std/fs` の write / create / truncate API | `.wasm` や JSON を file に書けない |
| [ISS-20260426T010002Z-STDFS-DIRLIST-C2F93A6E](../../issues/items/ISS-20260426T010002Z-STDFS-DIRLIST-C2F93A6E.md) | directory traversal / path normalization | import graph と stdlib discovery が固定 list に依存する |
| [ISS-20260426T010003Z-STDIO-RESULT-STDERR-E48B51D0](../../issues/items/ISS-20260426T010003Z-STDIO-RESULT-STDERR-E48B51D0.md) | Result 付き stdout/stderr | diagnostics と artifact output を安全に分離できない |
| [ISS-20260426T010004Z-TEXT-UTF8-VALIDATION-F1950B8A](../../issues/items/ISS-20260426T010004Z-TEXT-UTF8-VALIDATION-F1950B8A.md) | UTF-8 checked source loading | lexer/source map が不正 byte sequence で壊れる |
| [ISS-20260426T010005Z-BYTEBUF-BUILDER-WASM-D63A7C11](../../issues/items/ISS-20260426T010005Z-BYTEBUF-BUILDER-WASM-D63A7C11.md) | growable byte builder | wasm emitter が raw memory に依存する |
| [ISS-20260426T010006Z-CLIARG-TESTED-ARGS-A8D0E229](../../issues/items/ISS-20260426T010006Z-CLIARG-TESTED-ARGS-A8D0E229.md) | argv / option parsing の検証 | CLI parity と bootstrap command が不安定になる |

セルフホスト用ソースツリー自体の不足は [ISS-20260426T010000Z-SELFHOST-SOURCE-TREE-A1E5F24C](../../issues/items/ISS-20260426T010000Z-SELFHOST-SOURCE-TREE-A1E5F24C.md) で管理する。

---

## 10. テスト計画

| レイヤ | テスト |
|---|---|
| `infra/` | span / line map / diagnostic label の unit doctest |
| `syntax/` | token JSON / AST JSON parity |
| `module/` | import alias、open import、cycle、missing file |
| `check/` | `tests/compiler/*.n.md` の focused subset |
| `resource/` | move / borrow / drop fixture |
| `codegen/wasm/` | byte vector known output、smoke execution |
| `cli/` | argv parser table test、stdout/stderr split、file output |
| bootstrap | Pass A / Pass B artifact comparison |

CI では最初から全 self-host test を常時実行しない。
S0-S3 は focused job、S4 以降で smoke job、S7 で bootstrap comparison job を追加する。

---

## 11. 作業順序

1. `issues/` の新方式を正にし、旧 review issue を移行する。
2. `stdlib/neplg2/` を再作成し、`infra/` と `syntax/` の最小 doctest を置く。
3. `std/fs` の write / dir / checked text API を実装し、self-host CLI の I/O 前提を固める。
4. lexer / parser parity を作る。
5. module loader と import graph を作る。
6. typecheck / HIR / resource / mono を段階移植する。
7. WASM backend を実装し、binary artifact output を file write へ接続する。
8. CLI を完成させ、bootstrap comparison を CI に追加する。

---

## 12. 記録ルール

- `plan.md` は変更しない。
- 実装状況と planとの差異は `note.n.md` に記録する。
- 未着手タスクは `todo.md` に残し、完了後は削除する。
- self-host blocker は `issues/items/*.md` に作成し、`nodesrc/issues.js index` と `check` を通す。
- branch / commit / Rust 側修正合流の実行規則は [self_host_execution_plan.md](./self_host_execution_plan.md) に従う。
