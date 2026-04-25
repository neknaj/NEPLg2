# NEPLg2.0 実装レビュー Issue 台帳

作成日: 2026-04-25

この台帳は概要のみを持ちます。詳細は各領域別ファイルを正とします。

## 集計

| 領域 | Open | 解決済 |
|---|---:|---:|
| core | 12 | 0 |
| cli | 7 | 1 |
| stdlib | 9 | 1 |
| 合計 | 28 | 2 |

## Core

| ID | 解決済 | 状態 | 優先度 | 種別 | 要約 |
|---|---|---|---|---|---|
| [RV-CORE-001](./core.md#rv-core-001-core-の-no_std-境界が崩れている) | false | open | P1 | architecture | core が `no_std` を掲げながら `std` に依存している |
| [RV-CORE-002](./core.md#rv-core-002-typecheckrs-が巨大化しすぎて責務が分離できていない) | false | open | P1 | architecture | `typecheck.rs` が型推論・名前解決・HIR 生成・trait 処理を抱え込んでいる |
| [RV-CORE-003](./core.md#rv-core-003-reduce_calls-が-on2-化しやすく固定上限で正当な入力を落とす) | false | open | P0 | performance | call 縮約が `Vec::remove` と全走査で遅く、1000 回上限で誤診断になり得る |
| [RV-CORE-004](./core.md#rv-core-004-overload-解決が候補ごとに-typectx-全体を-clone-している) | false | open | P0 | performance | overload 解決で `TypeCtx` 全体 clone を多用している |
| [RV-CORE-005](./core.md#rv-core-005-loader-が-import-clause-を無視して全-import-をフラット結合している) | false | open | P1 | bug | `as name` / selective import が loader の item 結合に反映されていない |
| [RV-CORE-006](./core.md#rv-core-006-通常実行でデバッグ出力が-stderr-へ漏れる) | false | open | P1 | bug | loader などが verbose gate なしに `eprintln!` している |
| [RV-CORE-007](./core.md#rv-core-007-codegen-が診断ではなく-panic-で落ちる経路を多数持つ) | false | open | P0 | bug | backend が unsupported HIR を `panic!` で処理している |
| [RV-CORE-008](./core.md#rv-core-008-effect-判定が文字列包含に依存していて不健全) | false | open | P1 | bug | raw body の effect が文字列検索で決まり、純粋性検査が信用できない |
| [RV-CORE-009](./core.md#rv-core-009-moveborrowdrop-が-resource-ir-なしで後付け実装されている) | false | open | P1 | architecture | ownership / borrow / drop が HIR 走査だけで実装されている |
| [RV-CORE-010](./core.md#rv-core-010-name-resolution-が二重化し本パイプラインに統合されていない) | false | open | P2 | architecture | `resolve.rs` と `name_resolve.rs` が分かれ、後者は skeleton のまま |
| [RV-CORE-011](./core.md#rv-core-011-typeexpr-が-span-を保持せず診断位置が失われる) | false | open | P2 | bug | `TypeExpr::span()` が常に dummy を返す |
| [RV-CORE-012](./core.md#rv-core-012-targetprofile-gate-の評価が複数箇所に散っている) | false | open | P2 | architecture | target gate が compiler/typecheck/target_precheck に分散している |

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
| [RV-STDLIB-002](./stdlib.md#rv-stdlib-002-free-list-分割で余りブロックがリストへ戻らない) | false | open | P0 | bug | split remainder が free list から失われる |
| [RV-STDLIB-003](./stdlib.md#rv-stdlib-003-所有権を持つ-vecstack-が-copyclone-になっている) | false | open | P0 | bug | owning handle の shallow copy が double free / aliasing を招く |
| [RV-STDLIB-004](./stdlib.md#rv-stdlib-004-collection-free-が要素の-drop-を呼ばない) | false | open | P1 | bug | `Vec<T>` などが要素所有権を解放しない |
| [RV-STDLIB-005](./stdlib.md#rv-stdlib-005-stdio-read_all-が-4096-byte-で切り捨てる) | false | open | P1 | bug | text stdin が固定長で途切れる |
| [RV-STDLIB-006](./stdlib.md#rv-stdlib-006-fscliarg-の主要テストが-skip-されている) | false | open | P1 | test | I/O 系 stdlib の回帰が実行されない |
| [RV-STDLIB-007](./stdlib.md#rv-stdlib-007-str-の-utf-8-保証が実装で守られていない) | false | open | P1 | bug | bytes を検証せず `str` に変換する経路がある |
| [RV-STDLIB-008](./stdlib.md#rv-stdlib-008-self-host-compiler-がプレースホルダのまま) | false | open | P2 | architecture | `stdlib/neplg2` は 17 行の stub 群のみ |
| [RV-STDLIB-009](./stdlib.md#rv-stdlib-009-巨大-stdlib-ファイルが分割されていない) | false | open | P2 | architecture | `math.nepl` / `string.nepl` / `stdio.nepl` が巨大化 |
| [RV-STDLIB-010](./stdlib.md#rv-stdlib-010-resultoption-の-unsafe-helper-が通常コードに広く残っている) | false | open | P2 | bug | `unwrap` / `unwrap_ok` が stdlib 内部で panic 経路を広げている |
