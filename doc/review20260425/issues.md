# NEPLg2.0 実装レビュー Issue 台帳

作成日: 2026-04-25

この台帳は概要のみを持ちます。詳細は各領域別ファイルを正とします。

## 集計

| 領域 | Open | 解決済 |
|---|---:|---:|
| core | 10 | 5 |
| cli | 7 | 1 |
| stdlib | 8 | 4 |
| 合計 | 25 | 10 |

## Core

| ID | 解決済 | 状態 | 優先度 | 種別 | 要約 |
|---|---|---|---|---|---|
| [RV-CORE-001](./core.md#rv-core-001-core-の-no_std-境界が崩れている) | false | open | P1 | architecture | core が `no_std` を掲げながら `std` に依存している |
| [RV-CORE-002](./core.md#rv-core-002-typecheckrs-が巨大化しすぎて責務が分離できていない) | false | open | P1 | architecture | `typecheck.rs` が型推論・名前解決・HIR 生成・trait 処理を抱え込んでいる |
| [RV-CORE-003](./core.md#rv-core-003-reduce_calls-が-on2-化しやすく固定上限で正当な入力を落とす) | true | verified | P0 | performance | 固定上限・全走査・deep clone を除去し、1105 call chain を typecheck 可能に修正済み |
| [RV-CORE-004](./core.md#rv-core-004-overload-解決が候補ごとに-typectx-全体を-clone-している) | true | verified | P0 | performance | `TypeCtx` checkpoint/rollback と mapping-based layout により overload/codegen の全体 clone を除去済み |
| [RV-CORE-005](./core.md#rv-core-005-loader-が-import-clause-を無視して全-import-をフラット結合している) | false | open | P1 | bug | `as name` / selective import が loader の item 結合に反映されていない |
| [RV-CORE-006](./core.md#rv-core-006-通常実行でデバッグ出力が-stderr-へ漏れる) | false | open | P1 | bug | loader などが verbose gate なしに `eprintln!` している |
| [RV-CORE-007](./core.md#rv-core-007-codegen-が診断ではなく-panic-で落ちる経路を多数持つ) | true | verified | P0 | bug | WASM/LLVM backend の explicit panic 経路を diagnostic error に変換し、codegen compile_fail と直接 HIR 回帰テストを追加済み |
| [RV-CORE-008](./core.md#rv-core-008-effect-判定が文字列包含に依存していて不健全) | false | open | P1 | bug | raw body の effect が文字列検索で決まり、純粋性検査が信用できない |
| [RV-CORE-009](./core.md#rv-core-009-moveborrowdrop-が-resource-ir-なしで後付け実装されている) | false | open | P1 | architecture | ownership / borrow / drop が HIR 走査だけで実装されている |
| [RV-CORE-010](./core.md#rv-core-010-name-resolution-が二重化し本パイプラインに統合されていない) | false | open | P2 | architecture | `resolve.rs` と `name_resolve.rs` が分かれ、後者は skeleton のまま |
| [RV-CORE-011](./core.md#rv-core-011-typeexpr-が-span-を保持せず診断位置が失われる) | false | open | P2 | bug | `TypeExpr::span()` が常に dummy を返す |
| [RV-CORE-012](./core.md#rv-core-012-targetprofile-gate-の評価が複数箇所に散っている) | false | open | P2 | architecture | target gate が compiler/typecheck/target_precheck に分散している |
| [RV-CORE-013](./core.md#rv-core-013-参照引数の関数呼び出しが一時-borrow-にならず所有値を固定する) | true | verified | P0 | bug | 参照 parameter の call argument を一時 borrow として評価するよう修正済み |
| [RV-CORE-014](./core.md#rv-core-014-pair-から取り出した-generic-collection-の型が-overload-解決へ伝播しない) | true | verified | P1 | bug | `.Pair` の推論済み tuple 型を保持し、取得した `Vec<T>` の `len` overload が解決できるよう修正済み |
| [RV-CORE-015](./core.md#rv-core-015-深い-hir-を-codegen-pipeline-が再帰処理して-stack-overflow-する) | false | open | P1 | bug | typecheck 後の `--check` / codegen pipeline が深い HIR で native stack overflow する |

## CLI

| ID | 解決済 | 状態 | 優先度 | 種別 | 要約 |
|---|---|---|---|---|---|
| [RV-CLI-001](./cli.md#rv-cli-001---check-がコンパイルせず成功を返す) | true | verified | P0 | bug | `--check` が compile 後に成功可否を返すよう修正済み |
| [RV-CLI-002](./cli.md#rv-cli-002-通常実行で-debug-ログが出力される) | false | open | P1 | bug | CLI が `DEBUG:` を常時出す |
| [RV-CLI-003](./cli.md#rv-cli-003-nepl-cli-test-が-nmd-doctest-を対象にしない) | false | open | P1 | test | Rust CLI の test サブコマンドが `.nepl` だけを集める |
| [RV-CLI-004](./cli.md#rv-cli-004-wasi-fd_write-が-stdout-専用で-stderr-を扱えない) | false | open | P1 | bug | fd 2 が `badf` になる |
| [RV-CLI-005](./cli.md#rv-cli-005-path_open-が-wasi-の-preopen-モデルを実装していない) | false | open | P1 | security | host path を直接 `fs::read` する |
| [RV-CLI-006](./cli.md#rv-cli-006-stdlib-root-がビルド時パスに固定されている) | false | open | P2 | architecture | 配布バイナリで stdlib 解決が壊れやすい |
| [RV-CLI-007](./cli.md#rv-cli-007-llvm-toolchain-条件が既定で-linux--clang-2110-に固定される) | false | open | P2 | bug | LLVM ターゲットの可搬性が低い |
| [RV-CLI-008](./cli.md#rv-cli-008-nodesrc-cli-が未知引数をエラーにしない) | false | open | P3 | test | ドキュメント生成 CLI の typo を検出できない |

## Stdlib

| ID | 解決済 | 状態 | 優先度 | 種別 | 要約 |
|---|---|---|---|---|---|
| [RV-STDLIB-001](./stdlib.md#rv-stdlib-001-allocator-がアドレス-0-のメタデータと最初のブロックを衝突させる) | true | verified | P0 | bug | `alloc_raw` の初回 allocation が heap metadata 後ろから始まるよう修正済み |
| [RV-STDLIB-002](./stdlib.md#rv-stdlib-002-free-list-分割で余りブロックがリストへ戻らない) | true | verified | P0 | bug | split remainder を free list の同じ位置へ戻すよう修正済み |
| [RV-STDLIB-003](./stdlib.md#rv-stdlib-003-所有権を持つ-vecstack-が-copyclone-になっている) | true | verified | P0 | bug | `Vec` / `Stack` の shallow `Copy` / `Clone` を削除し double free パターンを compile_fail 化 |
| [RV-STDLIB-004](./stdlib.md#rv-stdlib-004-collection-free-が要素の-drop-を呼ばない) | false | open | P1 | bug | `Vec<T>` などが要素所有権を解放しない |
| [RV-STDLIB-005](./stdlib.md#rv-stdlib-005-stdio-read_all-が-4096-byte-で切り捨てる) | false | open | P1 | bug | text stdin が固定長で途切れる |
| [RV-STDLIB-006](./stdlib.md#rv-stdlib-006-fscliarg-の主要テストが-skip-されている) | false | open | P1 | test | I/O 系 stdlib の回帰が実行されない |
| [RV-STDLIB-007](./stdlib.md#rv-stdlib-007-str-の-utf-8-保証が実装で守られていない) | false | open | P1 | bug | bytes を検証せず `str` に変換する経路がある |
| [RV-STDLIB-008](./stdlib.md#rv-stdlib-008-self-host-compiler-がプレースホルダのまま) | false | open | P2 | architecture | `stdlib/neplg2` は 17 行の stub 群のみ |
| [RV-STDLIB-009](./stdlib.md#rv-stdlib-009-巨大-stdlib-ファイルが分割されていない) | false | open | P2 | architecture | `math.nepl` / `string.nepl` / `stdio.nepl` が巨大化 |
| [RV-STDLIB-010](./stdlib.md#rv-stdlib-010-resultoption-の-unsafe-helper-が通常コードに広く残っている) | false | open | P2 | bug | `unwrap` / `unwrap_ok` が stdlib 内部で panic 経路を広げている |
| [RV-STDLIB-011](./stdlib.md#rv-stdlib-011-clone-と-collection-read-api-が-by-value-で非-copy-所有型を扱えない) | true | verified | P0 | architecture | `Clone` と `Vec` / `Stack` の read API を borrow-based に移行する前提を追加済み |
| [RV-STDLIB-012](./stdlib.md#rv-stdlib-012-hashkeyhasher-の-clonecopy-capability-が標準-trait-と不整合) | false | open | P1 | architecture | `HashKey` / `Hasher` が独自の clone/copy capability を持ち、標準 `Clone` / `Copy` と不整合 |
