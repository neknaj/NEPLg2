# NEPLg2.0 実装レビュー Issue 台帳

作成日: 2026-04-25

この台帳は概要のみを持ちます。詳細は各領域別ファイルを正とします。

## 集計

| 領域 | Open | 解決済 |
|---|---:|---:|
| core | 3 | 26 |
| cli | 3 | 11 |
| stdlib | 15 | 9 |
| examples | 0 | 13 |
| 合計 | 21 | 59 |

## Core

| ID | 解決済 | 状態 | 優先度 | 種別 | 要約 |
|---|---|---|---|---|---|
| [RV-CORE-001](./core.md#rv-core-001-core-の-no_std-境界が崩れている) | true | verified | P1 | architecture | `SourceMap` と host module 境界を分離し、`wasm32v1-none` で core を check 可能に修正済み |
| [RV-CORE-002](./core.md#rv-core-002-typecheckrs-が巨大化しすぎて責務が分離できていない) | false | open | P1 | architecture | `typecheck.rs` が型推論・名前解決・HIR 生成・trait 処理を抱え込んでいる |
| [RV-CORE-003](./core.md#rv-core-003-reduce_calls-が-on2-化しやすく固定上限で正当な入力を落とす) | true | verified | P0 | performance | 固定上限・全走査・deep clone を除去し、1105 call chain を typecheck 可能に修正済み |
| [RV-CORE-004](./core.md#rv-core-004-overload-解決が候補ごとに-typectx-全体を-clone-している) | true | verified | P0 | performance | `TypeCtx` checkpoint/rollback と mapping-based layout により overload/codegen の全体 clone を除去済み |
| [RV-CORE-005](./core.md#rv-core-005-loader-が-import-clause-を無視して全-import-をフラット結合している) | true | verified | P1 | bug | 未修飾 lookup を import clause 可視性で filter し、alias / selective / open import の main pipeline 挙動を修正済み |
| [RV-CORE-006](./core.md#rv-core-006-通常実行でデバッグ出力が-stderr-へ漏れる) | true | verified | P1 | bug | core loader/type string の debug 出力を verbose gate 配下へ移動済み |
| [RV-CORE-007](./core.md#rv-core-007-codegen-が診断ではなく-panic-で落ちる経路を多数持つ) | true | verified | P0 | bug | WASM/LLVM backend の explicit panic 経路を diagnostic error に変換し、codegen compile_fail と直接 HIR 回帰テストを追加済み |
| [RV-CORE-008](./core.md#rv-core-008-effect-判定が文字列包含に依存していて不健全) | true | verified | P1 | bug | raw body の direct call target を宣言済み effect で判定し、文字列包含依存を除去済み |
| [RV-CORE-009](./core.md#rv-core-009-moveborrowdrop-が-resource-ir-なしで後付け実装されている) | false | open | P1 | architecture | ownership / borrow / drop が HIR 走査だけで実装されている |
| [RV-CORE-010](./core.md#rv-core-010-name-resolution-が二重化し本パイプラインに統合されていない) | false | open | P2 | architecture | `resolve.rs` と `name_resolve.rs` が分かれ、後者は skeleton のまま |
| [RV-CORE-011](./core.md#rv-core-011-typeexpr-が-span-を保持せず診断位置が失われる) | true | verified | P2 | bug | `TypeExpr::Spanned` で型式 span を保持し、impl target と call reduction 診断の dummy span を解消済み |
| [RV-CORE-012](./core.md#rv-core-012-targetprofile-gate-の評価が複数箇所に散っている) | true | verified | P2 | architecture | target/profile gate evaluator を集約し、未知 gate を `InvalidConditionalGate` 診断に修正済み |
| [RV-CORE-013](./core.md#rv-core-013-参照引数の関数呼び出しが一時-borrow-にならず所有値を固定する) | true | verified | P0 | bug | 参照 parameter の call argument を一時 borrow として評価するよう修正済み |
| [RV-CORE-014](./core.md#rv-core-014-pair-から取り出した-generic-collection-の型が-overload-解決へ伝播しない) | true | verified | P1 | bug | `.Pair` の推論済み tuple 型を保持し、取得した `Vec<T>` の `len` overload が解決できるよう修正済み |
| [RV-CORE-015](./core.md#rv-core-015-深い-hir-を-check-pipeline-が再帰処理して-stack-overflow-する) | true | verified | P1 | bug | `--check` を artifact 生成から分離し、1105 call chain が check-only path で成功するよう修正済み |
| [RV-CORE-016](./core.md#rv-core-016-深い-hir-を-artifact-codegen-pipeline-が再帰処理して-stack-overflow-する) | true | verified | P1 | bug | artifact 生成側の深い HIR traversal を iterative 化し、1105 call chain の wasm 生成を修正済み |
| [RV-CORE-017](./core.md#rv-core-017-関数値として渡した関数と-lambda-が-backend-到達時に未登録になる) | true | fixed | P0 | bug | concrete 関数の monomorphize でも関数値 / lambda 参照を収集し、D4007 / D4008 の局所回帰を修正済み |
| [RV-CORE-018](./core.md#rv-core-018-nested-aggregate-を-tuple-から取り出すと-2-番目以降の値が壊れる) | true | verified | P0 | bug | named generic aggregate の storage layout 解決を修正し、`Tuple(Vec, Vec)` の 2 番目以降を正しく copy できるよう修正済み |
| [RV-CORE-019](./core.md#rv-core-019-generic-wrapper--nested-generic-enum-の期待型伝播が-typenomatchingoverload-になる) | true | verified | P1 | bug | generic wrapper / nested generic enum の型引数汚染を防ぎ、`TypeNoMatchingOverload` を修正済み |
| [RV-CORE-020](./core.md#rv-core-020-pipe-左辺の部分適用が-d3013-になり-rust-test-と-doctest-の状態が不整合) | true | verified | P2 | bug | pipe 左辺の退避範囲を未完了 callable の直近引数式に限定し、`pipe_nested_pipes` / `pipe_in_if` の skip を解除済み |
| [RV-CORE-021](./core.md#rv-core-021-neplg2nmd-の-overload-arity-fixture-が-rust-test-と不整合) | true | verified | P2 | test | overload arity doctest を現行 Rust test と同じ `D3005` compile_fail 期待へ修正済み |
| [RV-CORE-022](./core.md#rv-core-022-github-actions-24940960078-で-compiler-doctest-が広範囲に回帰している) | true | verified | P0 | bug | run `24940960078` 由来の compiler doctest 回帰を分離修正し、`tests/compiler` 474件greenを確認済み |
| [RV-CORE-023](./core.md#rv-core-023-raw_body_precheck-の-unsupported-signature-fixture-が-zero-sized-unit-引数対応後の仕様とずれている) | true | verified | P2 | test | `raw_body_precheck` の D4002 fixture を現在も未対応の `never` 戻り値 signature へ更新済み |
| [RV-CORE-024](./core.md#rv-core-024-move_check-の-local-borrow-fixture-が-last-use-borrow-release-後の仕様とずれている) | true | verified | P2 | test | 未使用参照は move を阻害せず、後続使用される参照だけ D3051 で拒否するよう move doctest を整理済み |
| [RV-CORE-025](./core.md#rv-core-025-move_effect-の-copy-fixture-が標準-clone-signature-変更後の仕様とずれている) | true | verified | P2 | test | `move_effect.n.md` の標準 `Clone` impl を `(&Self)->Self` へ更新し、26件green化済み |
| [RV-CORE-026](./core.md#rv-core-026-overloadnmd-に同名arity違いを許可する旧fixtureが残っている) | true | verified | P2 | test | `overload.n.md` の arity 違い overload 5件を D3005 期待へ更新済み |
| [RV-CORE-027](./core.md#rv-core-027-llvm-top-level-llvmir-entry-が-hir-関数としてしか解決されない) | true | verified | P0 | bug | top-level `#llvmir` の `define @entry` を `#entry` 定義として扱い、LLVM smoke の D3092 を修正済み |
| [RV-CORE-028](./core.md#rv-core-028-pipe-左辺の完結した-open-call-が途中で分断される) | true | verified | P0 | bug | `unwrap_ok new 32 |> ...` のような完結済み呼び出し全体を pipe 左辺として扱うよう修正済み |
| [RV-CORE-029](./core.md#rv-core-029-pipe-左辺の試験簡約が-open-call-を再構築せず-nullary-call-を分断する) | true | verified | P0 | bug | `unwrap_ok new |> ...` のような nullary call を含む完結済み pipe 左辺を正しく単一値へ簡約するよう修正済み |

## CLI

| ID | 解決済 | 状態 | 優先度 | 種別 | 要約 |
|---|---|---|---|---|---|
| [RV-CLI-001](./cli.md#rv-cli-001---check-がコンパイルせず成功を返す) | true | verified | P0 | bug | `--check` が compile 後に成功可否を返すよう修正済み |
| [RV-CLI-002](./cli.md#rv-cli-002-通常実行で-debug-ログが出力される) | true | verified | P1 | bug | CLI の内部 debug/progress 出力を verbose gate 配下へ移動済み |
| [RV-CLI-003](./cli.md#rv-cli-003-nepl-cli-test-が-nmd-doctest-を対象にしない) | false | open | P1 | test | Rust CLI の test サブコマンドが `.nepl` だけを集める |
| [RV-CLI-004](./cli.md#rv-cli-004-wasi-fd_write-が-stdout-専用で-stderr-を扱えない) | true | verified | P1 | bug | fd 1/2 の共通 iovec 読み取りと stderr immediate flush を実装済み |
| [RV-CLI-005](./cli.md#rv-cli-005-path_open-が-wasi-の-preopen-モデルを実装していない) | true | verified | P1 | security | fd 3 preopen root、read-only rights、relative path sandbox 検証を実装済み |
| [RV-CLI-006](./cli.md#rv-cli-006-stdlib-root-がビルド時パスに固定されている) | true | verified | P2 | architecture | `--stdlib-root` / `NEPL_STDLIB_ROOT` / 実行ファイル相対 / build fallback の解決順を実装済み |
| [RV-CLI-007](./cli.md#rv-cli-007-llvm-toolchain-条件が既定で-linux--clang-2110-に固定される) | false | open | P2 | bug | LLVM ターゲットの可搬性が低い |
| [RV-CLI-008](./cli.md#rv-cli-008-nodesrc-cli-が未知引数をエラーにしない) | true | verified | P3 | test | unknown argument / value missing を usage error exit code 2 に修正済み |
| [RV-CLI-009](./cli.md#rv-cli-009-wasm-bindgen-cli-cache-が-rust-cache-の後処理で壊れ-ci-bootstrap-が落ちる) | true | verified | P1 | test | workspace 専用 root と cache 検証を追加し、run `24932659255` の `build` job で bootstrap 成功を確認済み |
| [RV-CLI-010](./cli.md#rv-cli-010-pages-fastfinal-deploy-が同じ-github-pages-artifact-名を使い-final-deploy-が落ちる) | true | verified | P1 | test | fast/final Pages artifact 名を分離し、run `24932659255` で final deploy 成功を確認済み |
| [RV-CLI-011](./cli.md#rv-cli-011-llvm-test-の-full-dual-backend-verification-が-ci-timeout-で-cancelled-になる) | false | open | P1 | test | `llvm-test` の full dual backend verification が 10 分 timeout で cancelled になる |
| [RV-CLI-012](./cli.md#rv-cli-012-trunk-build-が-clean-checkout-で-webexamples-不在により失敗する) | true | verified | P2 | test | `web/examples` を npm prebuild で同期し、ローカル `trunk build` を clean checkout でも通るように修正済み |
| [RV-CLI-013](./cli.md#rv-cli-013-playground-editor-cli-fixture-が-windows-crlf-checkout-で失敗する) | true | verified | P2 | test | fixture source を LF 正規化し、Windows checkout の CRLF で `nodesrc/cli.js` JSON テストが崩れないよう修正済み |
| [RV-CLI-014](./cli.md#rv-cli-014-llvm-smoke-test-が存在しない-fixture-path-を指して-0件成功扱いになる) | true | fixed | P0 | test | LLVM smoke の input path を `tests/compiler/llvm_target.n.md` に修正し、明示input 0件収集を error 扱いに修正済み |

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
| [RV-STDLIB-016](./stdlib.md#rv-stdlib-016-stack-push-が所有権を消費し-example-で-panic-helper-回避を妨げる) | true | verified | P1 | architecture | `Stack::push_ref` を追加し、Copy 要素を借用 stack へ追加して失敗時も handle を保持できるよう修正済み |
| [RV-STDLIB-017](./stdlib.md#rv-stdlib-017-vec-に固定長初期化-api-がなく-example-が-panic-helper-に依存する) | true | verified | P1 | architecture | `Vec::filled` を追加し、固定長 buffer/table を `Result` で作れるよう修正済み |
| [RV-STDLIB-018](./stdlib.md#rv-stdlib-018-streamio-の-wasi-doctest-が-trait-bound-不一致と出力破損で失敗する) | false | open | P1 | bug | `tests/stdlib/streamio.n.md` が `D3069` / `D3006` と stdout の binary layout 混入で 5 件失敗 |
| [RV-STDLIB-019](./stdlib.md#rv-stdlib-019-collection-doctest-の値ブロック末尾セミコロンが戻り値を-unit-にしている) | true | verified | P0 | test | collection doctest の `let x <T>:` 値ブロック末尾 `;` を削除し、対象4ファイルの doctest 34件を green 化済み |
| [RV-STDLIB-020](./stdlib.md#rv-stdlib-020-fenwicksegmenttree-doctest-が-d3016-expression-left-extra-values-で失敗する) | false | open | P0 | test | Fenwick / SegmentTree doctest が D3016 で 14件失敗している |
| [RV-STDLIB-021](./stdlib.md#rv-stdlib-021-vec-sort-doctest-が-overload-解決不一致で失敗する) | false | open | P1 | test | `vec/sort.nepl` doctest が D3021 / D3006 で失敗している |
| [RV-STDLIB-022](./stdlib.md#rv-stdlib-022-hashmap-doctest-にインデント不整合が残っている) | false | open | P1 | test | `hashmap.nepl::doctest#3` が D1206 indentation error で失敗している |
| [RV-STDLIB-023](./stdlib.md#rv-stdlib-023-hashmaphashset-の文字列-key-runtime-test-が-memory-oob-と-return-mismatch-で失敗する) | false | open | P0 | bug | HashMap / HashSet の string key runtime tests が OOB / mismatch で失敗している |
| [RV-STDLIB-024](./stdlib.md#rv-stdlib-024-deserialize-doctest-の-match-arm-が-result-と-unit-で不一致になる) | false | open | P1 | test | `deserialize.nepl::doctest#1` が D3045 match arm type mismatch で失敗している |

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
| [RV-EXAMPLE-009](./examples.md#rv-example-009-rpn_legacy-example-が-stack-push-失敗を-unwrap_ok-で-panic-させる) | true | verified | P1 | architecture | `rpn_legacy.nepl` の stack 初期化・push を `match` / `push_ref` へ移行し、`unwrap_ok` 依存を除去済み |
| [RV-EXAMPLE-010](./examples.md#rv-example-010-rpn-example-が-stack-push-失敗を-unwrap_ok-で-panic-させる) | true | verified | P1 | architecture | `rpn.nepl` の stack 初期化・push を `match` / `push_ref` へ移行し、`unwrap_ok` 依存を除去済み |
| [RV-EXAMPLE-011](./examples.md#rv-example-011-bf-example-が-vecstack-初期化を-unwrap_ok-で-panic-させる) | true | verified | P1 | architecture | `bf.nepl` の Vec/Stack 初期化を `filled` / `push_ref` / `match` へ移行し、`unwrap_ok` 依存を除去済み |
| [RV-EXAMPLE-012](./examples.md#rv-example-012-stdio-example-が-utf-8-標準入力を回帰確認していない) | true | verified | P3 | test | `stdio.nepl` に UTF-8 入力の doctest を追加し、説明を ASCII / UTF-8 入力へ整理済み |
| [RV-EXAMPLE-013](./examples.md#rv-example-013-helloworld-example-だけが標準ヘッダの-indent-を欠いている) | true | verified | P3 | maintenance | `helloworld.nepl` に `#indent 4` を追加し、examples の標準ヘッダを統一済み |
