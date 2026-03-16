# NEPLg2.1 コンパイラ構成設計

最終更新: 2026-03-17

---

## 1. 設計目標

### 1.1 現行（NEPLg2.0）の問題点

| ファイル | 行数 | 問題 |
|---|---:|---|
| `typecheck.rs` | 8,871 | 型推論・trait 検査・effect 判定・HIR 生成が全部混在 |
| `compiler.rs` | 933 | パイプライン統合とターゲットゲートパーサが同居 |
| `resolve.rs` / `name_resolve.rs` | 279 / 55 | 役割重複。両方ともスタブ状態 |
| `module_graph.rs` / `loader.rs` | 583 / 738 | ファイル解決・パース・依存グラフが混在 |
| `hir.rs` | 194 | `TupleConstruct` など NEPLg2.1 廃止構文が残る |
| `passes/` | 3ファイル | Resource IR パスが存在しない |
| `nm.rs` | 548 | NM フォーマットパーサがコアコンパイラに同居 |
| トップレベル全体 | 20+ ファイル | フラット構造で関係が見えない |

Resource IR（ownership/borrow/region/drop の中間表現）が存在しないことは NEPLg2.1 の中核仕様（`doc/2.1spec/compiler.md`）と直接矛盾する。

### 1.2 NEPLg2.1 での方針

- **パイプラインステージ = ディレクトリ階層**。依存方向がディレクトリ構造から読み取れる。
- **1 ファイルの目安は 800 行以下**。巨大 `typecheck.rs` は役割ごとに分割する。
- **Resource IR を第一級モジュールとして配置**。`resource/` ディレクトリに独立させる。
- **型表現（`ty/`）と型検査（`check/`）を分離**。型の定義と「その型を検査するロジック」は別。
- **モジュールシステムの 3 層（物理・構文・論理）を`module/` 内の別ファイルで表現**。
- **`nm/` はコアコンパイラから切り離す**（ツールチェーン補助）。
- **セルフホスト（`stdlib/neplg2/`）との命名パリティを保つ**。Rust 側のディレクトリ名が自然に NEPL 側のモジュール名に対応する。

---

## 2. ブートストラップ Rust コンパイラ (`nepl-core-2.1`)

### 2.1 ファイル構成

```
nepl-core-2.1/
    src/
        lib.rs                  # crate root、公開 API の re-export

        // ── 共通基盤 ────────────────────────────────────
        infra/
            mod.rs
            span.rs             # SourceSpan, FileId
            source.rs           # SourceFile, SourceMap
            diag/
                mod.rs
                diag.rs         # Diagnostic, Severity, DiagLabel, DiagNote
                ids.rs          # DiagnosticId 定数一覧
                outcome.rs      # Outcome<T, E>（診断付き Result）
                emit.rs         # テキスト出力（人間向け表示）

        // ── フロントエンド ───────────────────────────────
        syntax/
            mod.rs
            token.rs            # Token, TokenKind
            lexer.rs            # インデント aware トークナイザ（オフサイド規則）
            ast/
                mod.rs
                item.rs         # Item（let fn / struct / enum / trait / impl / use / merge）
                expr.rs         # Expr（前置記法・block・if/while/match・let/set・borrow）
                pattern.rs      # Pattern（コンストラクタ・リテラル・ワイルドカード・OR・@）
                typexpr.rs      # TypeExpr（juxtaposition・fn/fn*・%TypeExpr）
                module_ast.rs   # ModuleAst（ファイル単位 AST）・FileHeader（#module/#entry/#part）
            parser/
                mod.rs
                item_parser.rs  # 宣言パーサ（let fn / struct / enum / trait / impl）
                expr_parser.rs  # 式パーサ（前置 juxtaposition・block・if/while/match）
                type_parser.rs  # 型式パーサ（kind-directed アルゴリズム）
                pat_parser.rs   # パターンパーサ

        // ── モジュールシステム ──────────────────────────
        module/
            mod.rs
            physical.rs         # 物理層：ファイルパス解決・ハッシュ計算
            syntax_graph.rs     # 構文層：AST + merge/use 辺の DAG
            logical.rs          # 論理層：Canonical Module Path・merge 結合後モジュールツリー
            use_resolver.rs     # use/pub use パス解決・循環検出（pub use cycle）
            cache.rs            # パースキャッシュ・セマンティックキャッシュ

        // ── 名前解決 ────────────────────────────────────
        resolve/
            mod.rs
            def_id.rs           # DefId, DefKind, DefTable
            scope.rs            # スコープチェーン（ネスト・巻き上げ対応）
            item_resolver.rs    # トップレベル item 解決・定義テーブル構築
            use_import.rs       # use 文の item 引き当て・エクスポートテーブル
            variant_resolver.rs # EnumType::Variant / bare variant 解決（type context 判定）

        // ── 型システム ──────────────────────────────────
        ty/
            mod.rs
            ty.rs               # TypeId, TypeKind（Prim / Fn / Struct / Enum / Apply / Var …）
            arena.rs            # TypeArena（型インターン）
            kind.rs             # Kind（*、* -> * …）と kind 推論
            unify.rs            # 単一化・型変数インスタンス化
            subst.rs            # 型変数代入（Substitution）
            effect.rs           # Effect（Pure/Impure）・InternalAlloc / ExternalIO 分類

        // ── 型検査 ──────────────────────────────────────
        check/
            mod.rs
            checker.rs          # 型検査オーケストレーション・ContextStack
            expr_check.rs       # 式の型検査（juxtaposition 境界確定・call arity 解決）
            pat_check.rs        # パターン型検査・網羅性検査（match exhaustiveness）
            decl_check.rs       # 宣言検査（fn / struct / enum / trait / impl）
            hoist.rs            # let fn 巻き上げ（相互再帰対応）
            overload.rs         # オーバーロード解決 3 段階（制約フィルタ→期待型→修飾名）
            trait_check.rs      # impl 一意性・Orphan Rule・Global Coherence
            effect_check.rs     # Pure/Impure 伝播・InternalAlloc Escape Analysis

        // ── HIR ─────────────────────────────────────────
        hir/
            mod.rs
            hir.rs              # HirModule, HirExpr, HirPattern, HirItem
            lower.rs            # AST + 型検査結果 → HIR lowering
            pretty.rs           # デバッグ表示

        // ── Resource IR（NEPLg2.1 新規）────────────────
        resource/
            mod.rs
            ir.rs               # ResourceIr ノード
                                #   move x → y
                                #   borrow_shared x → b
                                #   borrow_unique x → b
                                #   region_new ρ / region_alloc ρ n / region_end ρ
                                #   drop x
                                #   io_open/write/close
            lower.rs            # HIR → Resource IR lowering
            ownership.rs        # use-after-move 検査（診断 5001）
            borrow.rs           # Borrow check / NLL last-use（診断 5007/5008）
            region.rs           # Region Inference（Pure persistent value の bulk free）
            drop_elab.rs        # Drop Elaboration（Owned/Linear の自動 drop 挿入）
            linear.rs           # Linear resource 消費検査（診断 5005/5006）

        // ── パス ────────────────────────────────────────
        passes/
            mod.rs
            target_gate.rs      # #if[target=...] / #target 評価
            codegen_pre.rs      # コード生成前整合検査

        // ── 単相化 ──────────────────────────────────────
        mono/
            mod.rs
            mono.rs             # 単相化メインロジック
            instance.rs         # インスタンス化キャッシュ・重複排除

        // ── コード生成 ──────────────────────────────────
        codegen/
            mod.rs
            wasm/
                mod.rs
                emit.rs         # Wasm バイナリ出力（wasm-encoder）
                intrinsic.rs    # Wasm 組み込み命令
                runtime.rs      # runtime helper 関数注入
                layout.rs       # 型の Wasm メモリレイアウト
            llvm/
                mod.rs
                emit.rs         # LLVM IR テキスト出力
                intrinsic.rs    # LLVM 組み込み命令
                layout.rs       # 型のネイティブレイアウト

        // ── 組み込み ────────────────────────────────────
        builtins/
            mod.rs
            primitives.rs       # プリミティブ型・演算（i32/u8/f32/bool/str/unit/never）
            intrinsics.rs       # コンパイラ組み込み操作（load/store/mem_copy 等）
            prelude.rs          # 暗黙 prelude（core::prelude から自動 use されるもの）

        // ── パイプライン API ────────────────────────────
        pipeline.rs             # compile_xxx 公開関数・パイプライン統合・エラー集約
        options.rs              # CompileOptions, BuildProfile, CompileTarget

        // ── ツールチェーン補助（コアコンパイラとは分離）──
        nm/
            mod.rs
            parser.rs           # .n.md NM 拡張 Markdown パーサ
            extractor.rs        # doctest・テストケース抽出
```

### 2.2 依存方向

```
infra
  ↑
syntax  →  module
  ↑           ↑
resolve  ←  (module)
  ↑
ty
  ↑
check
  ↑
hir
  ↑
resource
  ↑
passes  →  mono  →  codegen/{wasm, llvm}
                        ↑
                    builtins

pipeline.rs（全体を統合）
nm/（独立。pipeline からは参照しない）
```

上位ほど下位に依存する（矢印 = 依存先）。`nm/` は `pipeline.rs` の上流に位置するが、コアコンパイラ本体からは参照しない独立ツール。

### 2.3 ファイルサイズの目安

| モジュール | 推奨上限 | 根拠 |
|---|---:|---|
| `lexer.rs` | 1,000 行 | 複雑なインデント規則があるため |
| `parser/*_parser.rs` 各 | 800 行 | 役割単位で分割済み |
| `ty/unify.rs` | 600 行 | 単一化アルゴリズムに集中 |
| `check/expr_check.rs` | 800 行 | 現 `typecheck.rs` の主要部分を切り出し |
| `check/checker.rs` | 400 行 | オーケストレーションのみ |
| `resource/*` 各 | 500 行 | 解析フェーズごとに独立 |
| `codegen/wasm/emit.rs` | 1,200 行 | wasm エンコードの複雑さを許容 |
| `codegen/llvm/emit.rs` | 1,000 行 | LLVM IR テキスト生成 |

---

## 3. CLI クレート (`nepl-cli`)

```
nepl-cli/
    src/
        main.rs             # CLI エントリポイント・引数パース・exit コード
        args.rs             # コマンドライン引数定義（clap または手書き）
        file_io.rs          # ソースファイル読み込み・出力ファイル書き込み
        runner/
            mod.rs
            wasm_runner.rs  # WASM/WASI 実行（wasmtime / wasm3）
            llvm_runner.rs  # clang 呼び出し・native バイナリ生成
        repl.rs             # 将来の REPL サポート（Phase 後半）
```

`nepl-cli` は `nepl-core-2.1` に依存する。WASI / OS 依存の操作はすべてここに集中させ、`nepl-core-2.1` は `no_std` ＋ `extern crate alloc` を維持する。

---

## 4. セルフホストコンパイラ (`stdlib/neplg2/`)

セルフホストは NEPLg2.1 自身で書かれたコンパイラ。Rust 側のクレート構造と**ディレクトリ名を一致させる**ことで、移植の対応関係を明確にする。

```
stdlib/neplg2/
    core/                   # 純粋コンパイラコア（Wasm、WASI 不使用）
        #module             # anchor ファイル
        infra/
            span.nepl       # SourceSpan, FileId（core::pair/option に依存）
            diag.nepl       # Diagnostic, DiagnosticId, Outcome
        syntax/
            token.nepl      # Token, TokenKind
            lexer.nepl      # インデント aware トークナイザ
            ast/
                item.nepl
                expr.nepl
                pattern.nepl
                typexpr.nepl
                module_ast.nepl
            parser/
                item_parser.nepl
                expr_parser.nepl
                type_parser.nepl
                pat_parser.nepl
        module/
            physical.nepl
            syntax_graph.nepl
            logical.nepl
            use_resolver.nepl
            cache.nepl
        resolve/
            def_id.nepl
            scope.nepl
            item_resolver.nepl
            variant_resolver.nepl
        ty/
            ty.nepl
            arena.nepl
            kind.nepl
            unify.nepl
            effect.nepl
        check/
            checker.nepl
            expr_check.nepl
            pat_check.nepl
            decl_check.nepl
            trait_check.nepl
            effect_check.nepl
        hir/
            hir.nepl
            lower.nepl
        resource/
            ir.nepl
            ownership.nepl
            borrow.nepl
            region.nepl
            drop_elab.nepl
            linear.nepl
        passes/
            target_gate.nepl
            codegen_pre.nepl
        mono/
            mono.nepl
            instance.nepl
        codegen/
            wasm/
                emit.nepl
                layout.nepl
            llvm/
                emit.nepl
                layout.nepl
        builtins/
            primitives.nepl
            intrinsics.nepl
        pipeline.nepl
        options.nepl

    cli/                    # WASI CLI（ファイル I/O・プロセス操作を含む）
        #entry
        args.nepl
        file_io.nepl
        runner.nepl
```

`core/` は Pure 関数のみで構成し、WASI syscall を一切含まない。`cli/` が WASI を通じて外界と接続し、`core/` の `pipeline.nepl` を呼び出す。これは現行の `nepl-core`（no_std）と `nepl-cli`（std）の分離を NEPL 言語で再現したもの。

---

## 5. 現行ファイルと NEPLg2.1 ファイルの対応表

| 現行（nepl-core/src/） | NEPLg2.1（nepl-core-2.1/src/） | 変更内容 |
|---|---|---|
| `span.rs` | `infra/span.rs` | 移動のみ |
| `diagnostic.rs` + `diagnostic_ids.rs` | `infra/diag/diag.rs` + `ids.rs` | 統合・Outcome 追加 |
| `error.rs` | `infra/diag/diag.rs` に統合 | CoreError → Diagnostic に一本化 |
| `log.rs` | 廃止（eprintln! → Diag emitter へ） | 11 行のみ、不要 |
| `lexer.rs` | `syntax/lexer.rs` | 移動のみ |
| `ast.rs` | `syntax/ast/` (分割) | TypeExpr → `typexpr.rs`、Expr → `expr.rs`、Pattern → `pattern.rs` |
| `parser.rs` | `syntax/parser/` (分割) | 役割別に 4 ファイルへ |
| `module_graph.rs` + `loader.rs` | `module/` (分割) | 3 層モデルに再設計 |
| `resolve.rs` + `name_resolve.rs` | `resolve/` (統合・拡張) | variant 解決追加 |
| `types.rs` | `ty/ty.rs` + `ty/arena.rs` | Tuple 廃止、ResourceUsage 追加 |
| `effects.rs` | `ty/effect.rs` | InternalAlloc/ExternalIO 分類を充実 |
| `typecheck.rs` (8,871 行) | `check/` (7 ファイルに分割) | 役割別分割が最大の変更 |
| `hir.rs` | `hir/hir.rs` | TupleConstruct 廃止 |
| `compiler.rs` | `pipeline.rs` + `options.rs` | ターゲットゲートは `passes/target_gate.rs` へ |
| `builtins.rs` | `builtins/primitives.rs` | 分割・整理 |
| `runtime_helpers.rs` | `codegen/wasm/runtime.rs` | Wasm 固有のため移動 |
| `wasm_shared.rs` | `codegen/wasm/layout.rs` + `emit.rs` に統合 | |
| `codegen_wasm.rs` | `codegen/wasm/emit.rs` | 移動・整理 |
| `codegen_llvm.rs` | `codegen/llvm/emit.rs` | 移動・整理 |
| `monomorphize.rs` | `mono/mono.rs` + `mono/instance.rs` | 分割 |
| `passes/move_check.rs` | `resource/ownership.rs` に統合 | Resource IR の一部として再実装 |
| `passes/drop_insertion.rs` | `resource/drop_elab.rs` に統合 | Resource IR の一部として再実装 |
| `passes/codegen_precheck.rs` | `passes/codegen_pre.rs` | 移動 |
| `target_precheck.rs` | `passes/target_gate.rs` に統合 | |
| `nm.rs` | `nm/parser.rs` + `nm/extractor.rs` | コアから分離 |
| ―（存在しない） | `resource/ir.rs` / `borrow.rs` / `region.rs` / `linear.rs` | NEPLg2.1 新規 |

---

## 6. パイプライン統合（`pipeline.rs`）

`pipeline.rs` は各ステージを順に呼び出す統合関数を提供する。

```
pub fn compile(opts: CompileOptions, source_map: &SourceMap)
    -> Result<CompilationArtifact, Vec<Diagnostic>>

内部処理順:
  1. syntax::lexer          トークナイズ
  2. syntax::parser         パース → ModuleAst
  3. module::               物理→構文→論理の 3 層解決
  4. resolve::              DefId 付与・スコープ構築・use 引き当て
  5. ty:: + check::         型推論・型検査・effect 検査・trait 検査
  6. hir::lower             AST + 型情報 → HIR
  7. resource::             Resource IR 生成・ownership/borrow/region/drop 検査
  8. passes::               target gate 評価・codegen 前検査
  9. mono::                 単相化
 10. codegen::{wasm,llvm}   コード生成
```

各ステージは独立した `Result<_, Vec<Diagnostic>>` を返す。エラーがあっても可能な限り後段まで続行して診断をまとめる（エラー回復）。

---

## 7. 移行戦略

NEPLg2.1 コンパイラは現行 `nepl-core` を置き換えるのではなく、**並行して新規クレート `nepl-core-2.1` として開発する**。

| フェーズ | 内容 |
|---|---|
| Stage 1 | `infra/` + `syntax/` + `module/` の骨格を作る。現行 lexer/parser をリファクタして移植 |
| Stage 2 | `resolve/` + `ty/` の基礎を移植。effect 分類を充実させる |
| Stage 3 | `check/` を `typecheck.rs` から分割移植。巨大ファイルを解消 |
| Stage 4 | `hir/` + `resource/` を新規実装。Resource IR を導入 |
| Stage 5 | `mono/` + `codegen/` を移植。既存テストが通ることを確認 |
| Stage 6 | `nepl-core` を `nepl-core-2.1` に切り替え。テスト全通過を確認してから旧クレートを廃止 |

セルフホスト (`stdlib/neplg2/`) はブートストラップコンパイラが Stage 4 以降（Resource IR 利用可能）になってから本格着手する。
