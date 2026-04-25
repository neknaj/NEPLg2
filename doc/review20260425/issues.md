# NEPLg2.0 実装レビュー Issue 台帳

作成日: 2026-04-25

この台帳は概要のみを持ちます。詳細は各領域別ファイルを正とします。

## 集計

| 領域 | Open | 解決済 |
|---|---:|---:|
| core | 9 | 9 |
| cli | 8 | 5 |
| stdlib | 9 | 6 |
| examples | 0 | 8 |
| 合計 | 26 | 28 |

## Core

| ID | 解決済 | 状態 | 優先度 | 種別 | 要約 |
|---|---|---|---|---|---|
| [RV-CORE-001](./core.md#rv-core-001-core-の-no_std-境界が崩れている) | false | open | P1 | architecture | core が `no_std` を掲げながら `std` に依存している |
| [RV-CORE-002](./core.md#rv-core-002-typecheckrs-が巨大化しすぎて責務が分離できていない) | false | open | P1 | architecture | `typecheck.rs` が型推論・名前解決・HIR 生成・trait 処理を抱え込んでいる |
| [RV-CORE-003](./core.md#rv-core-003-reduce_calls-が-on2-化しやすく固定上限で正当な入力を落とす) | true | verified | P0 | performance | 固定上限・全走査・deep clone を除去し、1105 call chain を typecheck 可能に修正済み |
| [RV-CORE-004](./core.md#rv-core-004-overload-解決が候補ごとに-typectx-全体を-clone-している) | true | verified | P0 | performance | `TypeCtx` checkpoint/rollback と mapping-based layout により overload/codegen の全体 clone を除去済み |
| [RV-CORE-005](./core.md#rv-core-005-loader-が-import-clause-を無視して全-import-をフラット結合している) | false | open | P1 | bug | `as name` / selective import が loader の item 結合に反映されていない |
| [RV-CORE-006](./core.md#rv-core-006-通常実行でデバッグ出力が-stderr-へ漏れる) | true | verified | P1 | bug | core loader/type string の debug 出力を verbose gate 配下へ移動済み |
| [RV-CORE-007](./core.md#rv-core-007-codegen-が診断ではなく-panic-で落ちる経路を多数持つ) | true | verified | P0 | bug | WASM/LLVM backend の explicit panic 経路を diagnostic error に変換し、codegen compile_fail と直接 HIR 回帰テストを追加済み |
| [RV-CORE-008](./core.md#rv-core-008-effect-判定が文字列包含に依存していて不健全) | false | open | P1 | bug | raw body の effect が文字列検索で決まり、純粋性検査が信用できない |
| [RV-CORE-009](./core.md#rv-core-009-moveborrowdrop-が-resource-ir-なしで後付け実装されている) | false | open | P1 | architecture | ownership / borrow / drop が HIR 走査だけで実装されている |
| [RV-CORE-010](./core.md#rv-core-010-name-resolution-が二重化し本パイプラインに統合されていない) | false | open | P2 | architecture | `resolve.rs` と `name_resolve.rs` が分かれ、後者は skeleton のまま |
| [RV-CORE-011](./core.md#rv-core-011-typeexpr-が-span-を保持せず診断位置が失われる) | false | open | P2 | bug | `TypeExpr::span()` が常に dummy を返す |
| [RV-CORE-012](./core.md#rv-core-012-targetprofile-gate-の評価が複数箇所に散っている) | false | open | P2 | architecture | target gate が compiler/typecheck/target_precheck に分散している |
| [RV-CORE-013](./core.md#rv-core-013-参照引数の関数呼び出しが一時-borrow-にならず所有値を固定する) | true | verified | P0 | bug | 参照 parameter の call argument を一時 borrow として評価するよう修正済み |
| [RV-CORE-014](./core.md#rv-core-014-pair-から取り出した-generic-collection-の型が-overload-解決へ伝播しない) | true | verified | P1 | bug | `.Pair` の推論済み tuple 型を保持し、取得した `Vec<T>` の `len` overload が解決できるよう修正済み |
| [RV-CORE-015](./core.md#rv-core-015-深い-hir-を-check-pipeline-が再帰処理して-stack-overflow-する) | true | verified | P1 | bug | `--check` を artifact 生成から分離し、1105 call chain が check-only path で成功するよう修正済み |
| [RV-CORE-016](./core.md#rv-core-016-深い-hir-を-artifact-codegen-pipeline-が再帰処理して-stack-overflow-する) | true | verified | P1 | bug | artifact 生成側の深い HIR traversal を iterative 化し、1105 call chain の wasm 生成を修正済み |
| [RV-CORE-017](./core.md#rv-core-017-関数値として渡した関数と-lambda-が-backend-到達時に未登録になる) | true | fixed | P0 | bug | concrete 関数の monomorphize でも関数値 / lambda 参照を収集し、D4007 / D4008 の局所回帰を修正済み |
| [RV-CORE-018](./core.md#rv-core-018-nested-aggregate-を-tuple-から取り出すと-2-番目以降の値が壊れる) | false | open | P0 | bug | `Tuple(Vec, Vec)` の 2 番目以降を取り出すと nested aggregate の data pointer / payload が壊れる |

## CLI

| ID | 解決済 | 状態 | 優先度 | 種別 | 要約 |
|---|---|---|---|---|---|
| [RV-CLI-001](./cli.md#rv-cli-001---check-がコンパイルせず成功を返す) | true | verified | P0 | bug | `--check` が compile 後に成功可否を返すよう修正済み |
| [RV-CLI-002](./cli.md#rv-cli-002-通常実行で-debug-ログが出力される) | true | verified | P1 | bug | CLI の内部 debug/progress 出力を verbose gate 配下へ移動済み |
| [RV-CLI-003](./cli.md#rv-cli-003-nepl-cli-test-が-nmd-doctest-を対象にしない) | false | open | P1 | test | Rust CLI の test サブコマンドが `.nepl` だけを集める |
| [RV-CLI-004](./cli.md#rv-cli-004-wasi-fd_write-が-stdout-専用で-stderr-を扱えない) | false | open | P1 | bug | fd 2 が `badf` になる |
| [RV-CLI-005](./cli.md#rv-cli-005-path_open-が-wasi-の-preopen-モデルを実装していない) | false | open | P1 | security | host path を直接 `fs::read` する |
| [RV-CLI-006](./cli.md#rv-cli-006-stdlib-root-がビルド時パスに固定されている) | false | open | P2 | architecture | 配布バイナリで stdlib 解決が壊れやすい |
| [RV-CLI-007](./cli.md#rv-cli-007-llvm-toolchain-条件が既定で-linux--clang-2110-に固定される) | false | open | P2 | bug | LLVM ターゲットの可搬性が低い |
| [RV-CLI-008](./cli.md#rv-cli-008-nodesrc-cli-が未知引数をエラーにしない) | false | open | P3 | test | ドキュメント生成 CLI の typo を検出できない |
| [RV-CLI-009](./cli.md#rv-cli-009-wasm-bindgen-cli-cache-が-rust-cache-の後処理で壊れ-ci-bootstrap-が落ちる) | true | verified | P1 | test | workspace 専用 root と cache 検証を追加し、run `24932659255` の `build` job で bootstrap 成功を確認済み |
| [RV-CLI-010](./cli.md#rv-cli-010-pages-fastfinal-deploy-が同じ-github-pages-artifact-名を使い-final-deploy-が落ちる) | true | verified | P1 | test | fast/final Pages artifact 名を分離し、run `24932659255` で final deploy 成功を確認済み |
| [RV-CLI-011](./cli.md#rv-cli-011-llvm-test-の-full-dual-backend-verification-が-ci-timeout-で-cancelled-になる) | false | open | P1 | test | `llvm-test` の full dual backend verification が 10 分 timeout で cancelled になる |
| [RV-CLI-012](./cli.md#rv-cli-012-trunk-build-が-clean-checkout-で-webexamples-不在により失敗する) | true | verified | P2 | test | `web/examples` を npm prebuild で同期し、ローカル `trunk build` を clean checkout でも通るように修正済み |
| [RV-CLI-013](./cli.md#rv-cli-013-playground-editor-cli-fixture-が-windows-crlf-checkout-で失敗する) | true | verified | P2 | test | fixture source を LF 正規化し、Windows checkout の CRLF で `nodesrc/cli.js` JSON テストが崩れないよう修正済み |

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
| [RV-STDLIB-013](./stdlib.md#rv-stdlib-013-stdlib-collection-doctest-群が所有型-api-移行後の実装とずれている) | false | open | P1 | test | collection doctest が `D3004` / `D3016` / runtime trap で広範囲に失敗し、API と実装の差分が残っている |
| [RV-STDLIB-014](./stdlib.md#rv-stdlib-014-stack-の-更新-api-が-by-value-pop-に偏り所有値の継続利用を阻害する) | true | verified | P1 | architecture | `Stack` に `get_ref` / `pop_ref` を追加し、Copy 要素を借用経由で読み取り・取り出しできるよう修正済み |
| [RV-STDLIB-015](./stdlib.md#rv-stdlib-015-bytevec-操作の-public-api-不足により-example-が-raw-memory-へ依存する) | true | verified | P1 | architecture | `Vec::replace_ref` / `string::byte_at` / `stdio::print_byte` を追加し、byte VM を raw memory なしで書けるよう修正済み |

## Examples

| ID | 解決済 | 状態 | 優先度 | 種別 | 要約 |
|---|---|---|---|---|---|
| [RV-EXAMPLE-001](./examples.md#rv-example-001-rpn-example-が-stackvec-の内部表現と-by-value-api-に依存している) | true | verified | P1 | architecture | `rpn.nepl` を `Stack` / `Vec` の借用 API 中心に書き直し、低レベルメモリ操作と move error を除去済み |
| [RV-EXAMPLE-002](./examples.md#rv-example-002-bf-example-が-raw-memory-と-by-value-stack-pop-に依存している) | true | verified | P1 | architecture | `bf.nepl` を `Vec` / `Stack` / string byte API 中心へ書き直し、raw allocation と move error を除去済み |
| [RV-EXAMPLE-003](./examples.md#rv-example-003-legacy-rpn-example-が-raw-memory-と-typo-名に依存している) | true | verified | P1 | architecture | `rpn_regacy.nepl` を `rpn_legacy.nepl` に改名し、stdlib public API 中心へ書き直し済み |
| [RV-EXAMPLE-004](./examples.md#rv-example-004-basicstools-example-に旧-import--entry-表記が残っている) | true | verified | P2 | maintenance | basics/tools examples の import と entry 関数型表記を現行形へ統一済み |
| [RV-EXAMPLE-005](./examples.md#rv-example-005-rpn_legacy-example-のコメントに変更履歴が残っている) | true | verified | P3 | doc | `rpn_legacy.nepl` のソースコメントから旧ファイル名履歴を除き、利用上の注意へ整理済み |
| [RV-EXAMPLE-006](./examples.md#rv-example-006-nm-example-の-usage-表示が実体名とずれている) | true | verified | P3 | doc | `nm.nepl` の usage 表示を実体名の `nm` に統一済み |
| [RV-EXAMPLE-007](./examples.md#rv-example-007-rpn-example-の先頭構成が-docdoctest-基準から外れている) | true | verified | P3 | doc | `rpn.nepl` の先頭を doctest / 概要コメント / directive の順へ整理済み |
| [RV-EXAMPLE-008](./examples.md#rv-example-008-bf-example-の先頭構成が-docdoctest-基準から外れている) | true | verified | P3 | doc | `bf.nepl` の先頭を doctest / 概要コメント / directive の順へ整理済み |
