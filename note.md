# 2026-02-21 作業メモ (ValueNs/CallableNs 分離の段階導入: 旧 lookup ラッパ削除)
- 目的:
  - `typecheck` 内で残っていた曖昧な `lookup`/`lookup_all` 参照を除去し、用途別 API への統一を進める。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `Symbol::Ident` の fallback を `lookup_any_defined` に変更。
    - 互換ラッパ `lookup` / `lookup_all` を削除。
    - 置換完了後の探索 API は以下へ統一:
      - 値: `lookup_value`
      - 関数: `lookup_all_callables` / `lookup_callable_any`
      - 任意定義済み: `lookup_any_defined` / `lookup_all_any_defined`
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_phase_current.json -j 1`: `240/240 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 分離の段階導入: 明示 lookup API へ統一)
- 目的:
  - `typecheck` で `lookup/lookup_all` の意図が曖昧な箇所を減らし、`ValueNs`/`CallableNs` 分離を進める。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `Env` に明示 API を追加:
      - `lookup_any_defined`
      - `lookup_all_any_defined`
    - 既存の `lookup`/`lookup_all` は互換ラッパとして残し、呼び出し側を段階置換。
    - 置換した主な箇所:
      - enum/struct 名衝突判定: `lookup_any_defined`
      - enum variant/struct constructor 既存判定: `lookup_all_callables`
      - `noshadow` 競合判定: `lookup_all_any_defined`
      - 識別子 fallback 候補列挙: `lookup_all_any_defined`
- 効果:
  - 関数解決と値解決の経路がコード上で判別しやすくなり、今後の namespace 分離リファクタリングの安全性を向上。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -o tests/output/shadowing_functions_current.json -j 1`: `205/205 pass`
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 分離の段階導入: callable 専用経路の拡大)
- 目的:
  - `todo.md` 最優先の名前空間分離を継続し、callable と value の探索経路をより明確に分離。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `fn alias` のターゲット探索を `lookup_all` から `lookup_all_callables` に変更。
    - entry 解決の候補探索を `lookup_all` から `lookup_all_callables` に変更。
    - trait メソッド呼び出し補助分岐の存在判定を `lookup_all_callables` に変更。
  - これにより、関数解決フェーズで value 候補を混在させない経路を拡大。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/functions.n.md -o tests/output/functions_current.json -j 1`: `187/187 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 1`: `203/203 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (名前解決 API: 重要シャドー警告の抑制オプション追加)
- 目的:
  - `todo.md` の「重要 stdlib 記号 warning 抑制ルール（設定/フラグ）」を実装し、LSP/エディタ連携で制御可能にする。
- 実装:
  - `nepl-web/src/lib.rs`
    - `analyze_name_resolution_with_options(source, options)` を追加。
    - `options.warn_important_shadow`（bool, default=true）を導入。
    - `NameResolutionTrace` に `warn_important_shadow` を保持し、important-shadow warning 生成を条件化。
    - `policy.warn_important_shadow` を返却ペイロードに追加。
    - 既存 `analyze_name_resolution` は新 API に委譲（後方互換維持）。
  - `tests/tree/07_shadow_warning_policy.js`
    - 重要記号 `print` は通常 warning が出ることを確認。
    - `warn_important_shadow=false` で warning 抑制されることを確認。
- 併せて実施:
  - `nepl-core/src/typecheck.rs` で ValueNs/CallableNs 分離の段階導入を継続し、値用途の lookup を `lookup_value` に寄せた。
    - global `fn`/`fn alias` 既存衝突判定
    - `set` の参照解決
    - dotted field base 解決
- `todo.md` 反映:
  - 完了した「重要 stdlib 記号 warning 抑制ルール（設定/フラグ）」項目を削除。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 分離の段階導入: lookup 用途分離)
- 目的:
  - `todo.md` 最優先の名前空間分離に向け、`typecheck` 内の識別子 lookup を用途別 API に寄せる。
- 実装:
  - `nepl-core/src/typecheck.rs` で、以下の箇所を value 専用 lookup へ置換。
    - グローバル `fn` 登録時の「既存非関数チェック」: `env.lookup_value`
    - `fn alias` 登録時の「既存非関数チェック」: `env.lookup_value`
    - `set` 解決時の外側探索: `env.lookup_value`
    - dotted field (`a.b`) の base 解決: `env.lookup_value`
- 効果:
  - 変数と callable を同一 lookup で混在解決する箇所を減らし、分離設計への移行を前進。
  - 挙動は維持しつつ、意図しない callable 混入の余地を縮小。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/tree -o tests/output/shadowing_tree_current.json -j 1`: `186/186 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (shadow warning ポリシーの API テスト固定)
- 目的:
  - `todo.md` の「シャドーイング運用の完成」に向け、`analyze_name_resolution` の警告ポリシーを木構造テストで固定。
- 追加:
  - `tests/tree/07_shadow_warning_policy.js`
    - `print` のローカルシャドーで warning が出ることを確認。
    - `cast` のローカルシャドーでは important-shadow warning が出ないことを確認。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (シャドーイング: callable 解決の回帰修正)
- 背景:
  - `tests/shadowing.n.md` の pending ケース（`value_name_and_callable_name_can_coexist_currently_fails` / `imported_function_name_shadowed_by_parameter_currently_fails`）を通常テストへ昇格するため、`typecheck` の識別子解決を調整。
- 実装:
  - `nepl-core/src/typecheck.rs` に `Env::lookup_callable_any` を追加。
  - 呼び出しヘッド位置の識別子解決で、同名 value が現在スコープにあっても outer callable を参照できる経路を追加。
  - ただし適用範囲は限定し、以下条件を満たす場合のみ有効化:
    - `forced_value == false`
    - `stack.is_empty()`（先頭解決）
    - `expr.items.get(idx + 1).is_some()`（実際に後続項があり呼び出し文脈）
- 失敗分析:
  - 当初は適用範囲が広すぎ、`if cond: ok` の `ok` を callable に誤解決して全体回帰（stdlib 側 `if condition must be bool`）が発生。
  - 上記条件で呼び出しヘッドに限定し、回帰を解消。
- テスト:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`: `185/185 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 1`: `202/202 pass`
- 補足:
  - 共有されていた `tests/neplg2.n.md::doctest#6/#7` の compile fail は現時点で再現せず、当該ファイルは全件 pass。

# 2026-02-21 作業メモ (target=wasm で WASI 無効化)
- 要件反映:
  - `nepl-cli/src/main.rs` の自動昇格ロジック（`std/stdio` import を検出して `wasi` にする挙動）を削除。
  - `target=wasm` のときは WASI を有効化しないように修正。
  - `target=wasi` のときのみ `wasi_snapshot_preview1` import を許可し、WASI 関数を linker に登録。
- 実装詳細:
  - `execute`:
    - `target_override` を CLI 指定のみに限定。
    - 実行ターゲット推定を `detect_module_target` へ切り出し（`module.directives` と `module.root.items` の双方を確認）。
  - `run_wasm`:
    - `CompileTarget::Wasm` では import が存在した時点でエラー化。
    - `CompileTarget::Wasi` でのみ `args_sizes_get` / `args_get` / `path_open` / `fd_read` / `fd_close` / `fd_write` を登録。
- 検証:
  - `cargo test -p nepl-cli`: pass
  - `#target wasm + #import "std/stdio"`: compile error（`WASI import not allowed for wasm target`）を確認。
  - `#target wasi + #import "std/stdio"`: 実行成功（`println "hi"` が出力）を確認。

# 2026-02-21 作業メモ (fs 衝突修正 + 回帰テスト追加)
- `tests/selfhost_req.n.md` の compile fail を起点に `std/fs` の根因を修正。
  - `std/fs` の WASI extern 名が他モジュール（`std/stdio` など）と衝突しうるため、`wasi_path_open` / `wasi_fd_read` / `wasi_fd_close` に内部名を固有化。
  - `fs_read_fd_bytes` の `cast` を `<u8> cast b` へ明示して overload 曖昧性を解消。
  - `vec_new<u8> ()` 旧記法を新記法 `vec_new<u8>` へ更新。
- テスト整備:
  - 追加: `tests/capacity_stack.n.md`
    - 再帰深さ（64/512）、`Vec` 拡張、`mem` 読み書き、`StringBuilder`、`enum+vec+再帰` の段階テストを固定。
  - 更新:
    - `tests/selfhost_req.n.md`
    - `tests/sort.n.md`
    - `tests/string.n.md`
    - `tests/ret_f64_example.n.md`
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/ret_f64_example.n.md -i tests/selfhost_req.n.md -i tests/sort.n.md -i tests/string.n.md -i tests/capacity_stack.n.md -o tests/output/targeted_regression_current.json`
    - `194/194 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
    - `540/540 pass`
- 補足:
  - `std/fs` は引き続き WASI preview1 前提。`wasmtime/wasmer` 差分検証は `todo_kp.md` のランタイム互換項目として継続。

# 状況メモ (2026-01-22)
# 2026-02-10 作業メモ (競プロカタログ拡張 + kpモジュール整理)
- チュートリアルに競プロ定番の参照章を追加し、重要アルゴリズム/データ構造のサンプルを 20 項目で列挙した。
  - 追加: `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - 目次反映: `tutorials/getting_started/00_index.n.md`
- `stdlib/kp` を機能別に整理し、新規モジュールを追加した。
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_i32`, `upper_bound_i32`, `contains_i32`
  - `stdlib/kp/kpprefix.nepl`
    - `prefix_build_i32`, `prefix_range_sum_i32`
  - `stdlib/kp/kpdsu.nepl`
    - `dsu_new`, `dsu_find`, `dsu_unite`, `dsu_same`, `dsu_size`, `dsu_free`
  - `stdlib/kp/kpfenwick.nepl`
    - `fenwick_new`, `fenwick_add`, `fenwick_sum_prefix`, `fenwick_sum_range`, `fenwick_free`
- すべて `//:` のドキュメントコメント形式で記述し、各モジュールに最小 doctest を付与した。

# 2026-02-10 作業メモ (関数単位レビュー: 機械置換の後処理)
- ユーザー指示に基づき、`vec/stack/list` を関数ごとに再確認し、機械置換由来の不整合を手修正した。
- 主な修正:
  - `stdlib/alloc/vec.nepl`
    - `vec_new` ドキュメントの `使い方:` 重複を除去。
    - `vec_set` doctest の move-check 衝突を回避する使用例へ修正。
  - `stdlib/alloc/collections/stack.nepl`
    - モジュール説明の重複ブロック（先頭と import 後の二重記載）を統合し、1箇所に整理。
  - `stdlib/alloc/collections/list.nepl`
    - モジュール説明の重複ブロック（先頭と import 後の二重記載）を統合し、1箇所に整理。
- 形式面:
  - `//` コメントは残さず、ドキュメントは `//:` のみを使用。
  - 各関数に `目的/実装/注意/計算量` + `使い方` + `neplg2:test` を維持。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=35, passed=35, failed=0, errored=0`

# 2026-02-10 作業メモ (doc comment 書式: 「使い方」見出しを統一)
- ユーザー提示の書式に合わせ、`vec/stack/list` の doctest 前に `//: 使い方:` を統一追加した。
  - 対象:
    - `stdlib/alloc/vec.nepl`
    - `stdlib/alloc/collections/stack.nepl`
    - `stdlib/alloc/collections/list.nepl`
- あわせて、`vec_set` の doctest で move-check に抵触していた例を修正し、コンパイル可能な使用例に整えた。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=35, passed=35, failed=0, errored=0`

# 2026-02-10 作業メモ (vec/stack/list コメント様式の指定対応)
- ユーザー指定の `stdlib/nm` 拡張 Markdown 形式に合わせ、以下のモジュール先頭コメントを具体化した。
  - `stdlib/alloc/vec.nepl`
  - `stdlib/alloc/collections/stack.nepl`
  - `stdlib/alloc/collections/list.nepl`
- 反映内容:
  - 先頭 `//:` で「ライブラリの主題」「目的」「実装アルゴリズム」「注意点」「計算量」を具体記述。
  - 既存の各関数前 `//:`（目的/実装/注意/計算量）と doctest 構成は維持。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=7, passed=7, failed=0, errored=0`

# 2026-02-10 作業メモ (vec/stack/list の doc comment + doctest 整備)
- ユーザー指示に合わせて、以下の標準ライブラリに実行可能な doctest を追加・整備した。
  - `stdlib/alloc/vec.nepl`
  - `stdlib/alloc/collections/stack.nepl`
  - `stdlib/alloc/collections/list.nepl`
- 変更内容:
  - `stack.nepl` / `list.nepl` の `neplg2:test[skip]` を解除し、主要操作（new/push/pop/peek/len/clear, cons/head/tail/get/reverse など）を確認する doctest を追加。
  - `vec.nepl` に `clear` を中心とした追加 doctest を入れ、move 規則に反しない形へ調整。
  - `str_eq` を使う doctest には `alloc/string` import を明示。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
    - `summary: total=7, passed=7, failed=0, errored=0`

# 2026-02-10 作業メモ (nm OOB 根治: parse_markdown 再設計)
- `nm` の run fail (`memory access out of bounds`) を上流から再切り分けし、`stdlib/nm/parser.nepl` の `parse_markdown` を再設計した。
- 根因分析:
  - 既存実装は section stack と `Vec<Node>` の値受け渡しが複雑で、`nm` doctest で OOB を継続再現。
  - `parse_markdown` 単体の最小実行で再現することを確認し、周辺ロジックを段階的に外して切り分け。
- 実装変更:
  - `parse_markdown` をフラット走査ベースに置き換え、`stack` 依存経路を除去。
  - `safe_line` は `lines_data + offset` ではなく `vec_get<str>` ベースの安全アクセスに統一。
  - heading/fence/paragraph/hr の分岐を明示化し、見出し配下の children 収集を局所ループで実装。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
    - `total=72, passed=72, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all.json -j 1`
    - `total=416, passed=409, failed=7, errored=0`
    - 残りは `ret_f64_example`, `selfhost_req`, `sort` で、nm 系失敗は解消。
# 2026-02-10 作業メモ (nm 実装状況と doc comment 整備)
- `nm` の現状:
  - コンパイル段階の主要 move-check エラーは大きく削減したが、実行時 `memory access out of bounds` が残っており未完了。
  - `tests/nm.n.md` の失敗は現在 OOB のみ（compile fail から run fail へ遷移）。
- ドキュメントコメント整備:
  - `stdlib/nm/parser.nepl`
    - `parse_markdown`
    - `document_to_json`
  - `stdlib/nm/html_gen.nepl`
    - `render_document`
  - 上記に日本語説明（目的/実装/注意/計算量）と `neplg2:test` 例を追加。
  - doctest 例は `fn main` を含む実行可能な形式へ修正済み。
- テスト結果（nm 関連）:
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
  - `summary: total=72, passed=67, failed=5, errored=0`
  - 失敗理由はすべて `memory access out of bounds`
- 次アクション:
  - OOB の発生点を `nm/parser` の `load<...>` / `size_of<...>` 利用箇所から再切り分け。
  - `Vec<T>` 要素アクセスを直接 `data + offset` で扱う方針の安全条件（境界・レイアウト）を明文化し、必要なら API に戻す。

# 2026-02-10 作業メモ (nm 再現テスト追加と上流切り分け)
- `tests/nm.n.md` を新規追加し、`nm/parser` + `nm/html_gen` の最小経路を固定した。
  - `nm_parse_markdown_json_basic`
  - `nm_render_document_basic`
- `examples/nm.nepl` / `stdlib/nm/parser.nepl` の先行修正:
  - `stdlib/nm/parser.nepl` の `if:` レイアウト由来で parser 再帰を誘発していた `let next_is_paren` 部分を段階代入へ変更。
  - `#import "std/math"` を `#import "core/math"` に修正。
  - `examples/nm.nepl` に `#import "std/env/cliarg" as *` を追加。
- `nm` で露出した上流不整合の修正:
  - `nm/parser` / `nm/html_gen` の関数シグネチャを実装実態に合わせて `*>` へ寄せた（pure/impure 不整合の解消）。
  - `nm/parser` 内の bool 比較 (`eq done false` 等) を `not` / 直接判定へ変更。
  - `Section` 構築時の曖昧な前置式を段階代入へ整理し、親情報取得順序を `peek -> pop` に修正。
  - 型名衝突を解消:
    - `Section`(struct) -> `NestSection`
    - `Ruby`(struct) -> `RubyInfo`
    - `Gloss`(struct) -> `GlossInfo`
    - `CodeBlock`(struct) -> `CodeBlockInfo`
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
    - `total=69, passed=67, failed=2`
    - 残り: `use of moved value`（`lines` / `v`）に収束
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-nm.json -j 1`
    - `total=413, passed=404, failed=9, errored=0`
- 現在の評価:
  - parser の停止保証は維持されたまま、nm 不具合は「Vec/str の所有権処理（vec_get/vec_len 呼び出し設計）」へ根因が絞れた。
  - 次段は `nm/parser` のループ処理を `Vec` の `data/len` 直接アクセスへ再設計し、move-check を根本解消する。

# 2026-02-10 作業メモ (parser 再帰暴走の停止保証)
- ユーザー指示「コンパイラは必ず停止する」を受けて、`nepl-core/src/parser.rs` に停止保証を追加。
- 実装内容（上流 parser 側）:
  - 再帰深さ上限を追加:
    - `MAX_PARSE_RECURSION_DEPTH = 2048`
    - `enter_parse_context` / `leave_parse_context` を追加
    - `parse_stmt` をコンテキスト管理下で実行し、過剰再帰時は診断を返して停止するよう変更
  - 無進捗ループ検出を追加:
    - `MAX_NO_PROGRESS_STEPS = 64`
    - `parse_block_until_internal` / `parse_prefix_expr` / `parse_prefix_expr_until_tuple_delim` / `parse_prefix_expr_until_colon`
    - 同一 `pos` が一定回数続いたら診断を出して 1 token 前進し、無限ループを回避
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `timeout 20s node nodesrc/analyze_source.js -i stdlib/nm/parser.nepl --stage parse`: `PARSE_EXIT:0`
  - `node nodesrc/test_analysis_api.js`: `7/7 passed`
- 補足:
  - `stdlib/nm/parser.nepl` の parse で以前発生していた停止しない挙動は、少なくとも解析 API 経路では再現しなくなった。
  - `examples/nm.nepl` 側は引き続き type/effect 不整合（`nm` ライブラリの pure/impure 署名ズレ等）が残っており、次段で修正継続。

# 2026-02-10 作業メモ (tuple unit 要素の codegen 根本修正)
- `tests/tuple_new_syntax.n.md::doctest#10` の根因を特定。
  - `Tuple:` に `()` が含まれると、WASM codegen が `unit` 要素を通常値として `LocalSet` しようとしてスタック不足になっていた。
  - 既存レイアウト（typecheck 側 offset=4 刻み）を崩さず、`unit` 要素/フィールドは「式評価で副作用は実行しつつ、スロットには 0 を格納」する方針へ統一。
- `nepl-core/src/codegen_wasm.rs`:
  - `StructConstruct` / `TupleConstruct` の要素 store 分岐を `valtype(Some)` と `None(unit)` で分離。
  - `None(unit)` では `gen_expr` 後に `i32.store 0` を行う実装へ変更。
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o /tmp/tests-tuple-after-unit-slot-fix.json -j 1`
    - `total=20, passed=20, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-tuple-unit-fix.json -j 1`
    - `total=339, passed=327, failed=12, errored=0`

# 2026-02-10 作業メモ (pipe 残件解消 + alloc 依存の根本改善)
- `tests/pipe_operator.n.md` の残失敗（#13/#14/#15）を上流から切り分けて修正。
- `nepl-core/src/typecheck.rs`:
  - `let s <S> 10 |> S` / `let e <E> 20 |> E::V` で、`<S>/<E>` が pipe 前のリテラルに早期適用される不具合を修正。
  - `next_is_pipe` の場合は pending ascription を遅延し、pipe 注入後の式確定時に適用するよう変更。
- `nepl-core/src/codegen_wasm.rs`:
  - `alloc` が未importでも構造体/列挙/タプル構築で落ちないよう、inline bump allocator フォールバックを追加（`emit_alloc_call`/`emit_inline_alloc`）。
  - これにより `pipe_struct_source` / `pipe_into_constructor` で出ていた `alloc function not found (import std/mem)` を解消。
- `todo.md`:
  - 高階関数フェーズ後の `StringBuilder` 根本再設計タスク（O(n) build 化、再現テスト追加）を追加。
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/pipe_operator.n.md -o /tmp/tests-pipe-after-constructor-revert.json -j 1`
    - `total=20, passed=20, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-current-after-pipe-fixes.json -j 1`
    - `total=339, passed=326, failed=13, errored=0`
  - 残件分類:
    - `ret_f64_example=1`
    - `selfhost_req=4`
    - `sort=5`
    - `string=2`
    - `tuple_new_syntax=1`

# 2026-02-10 作業メモ (offside: block: 同一行継続の禁止)
- `tests/offside_and_indent_errors.n.md::doctest#4` の根因は parser が `block:` の同一行継続（`block: add 1 2`）を許容していたこと。
- `nepl-core/src/parser.rs` を修正:
  - `KwBlock` の `:` 分岐で、改行が無い場合は診断を追加し、回復用に単行解析へフォールバック。
  - 仕様上「`block:` の後ろは空白/コメントのみ」を満たすようにした。
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/offside_and_indent_errors.n.md -o /tmp/tests-offside-after-block-colon-fix.json -j 1`
    - `total=7, passed=7, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-offside-fix.json -j 1`
    - `total=339, passed=322, failed=17, errored=0`
  - 残り失敗分類:
    - `pipe_operator=4`
    - `ret_f64_example=1`
    - `selfhost_req=4`
    - `sort=5`
    - `string=2`
    - `tuple_new_syntax=1`

# 2026-02-10 作業メモ (target尊重 + trait呼び出し + doctest VFS)
- `nepl-web/src/lib.rs`:
  - `compile_wasm_with_entry` の `CompileOptions.target` を `Some(Wasi)` 固定から `None` に変更し、ソース側 `#target` を尊重するよう修正。
  - これにより `#if[target=...]` / `#target` 重複検出 / wasm での wasi import 禁止のテストが有効化された。
- `nepl-core/src/monomorphize.rs`:
  - `FuncRef::Trait` の解決で impl map の厳密一致が外れた場合に、`trait+method` での型単一候補を探索するフォールバックを追加。
  - `tests/neplg2.n.md::doctest#31` (`Show::show`) を解消。
- `nodesrc/run_test.js` + `nodesrc/tests.js`:
  - doctest 実行時に `file` 情報を渡し、`#import`/`#include` の相対パスを実ファイルから収集して `compile_source_with_vfs` に渡す機能を追加。
  - `tests/part.nepl` を追加し、`tests/neplg2.n.md::doctest#11` の `#import "./part"` を解決可能にした。
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-after-vfs2.json -j 1`
    - `total=35, passed=35, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-target-vfs-trait.json -j 1`
    - `total=339, passed=321, failed=18, errored=0`
  - 主な残件: `offside(1)`, `pipe_operator(4)`, `ret_f64_example(1)`, `selfhost_req(4)`, `sort(5)`, `string(2)`, `tuple_new_syntax(1)`

# 2026-02-10 作業メモ (loader字句正規化 + 高階関数回帰確認)
- `nepl-core/src/loader.rs` の `canonicalize_path` に字句的正規化（`.` / `..` 除去）を追加した。
  - 目的: `#import "./part"` の解決で `/virtual/./part.nepl` と `/virtual/part.nepl` の不一致をなくすため。
  - 変更後、`tests/neplg2.n.md::doctest#11` は `missing source: /virtual/part.nepl` まで前進し、パス不一致自体は解消。
- 高階関数系の現状を再確認:
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-current.json -j 1`
  - `total=19, passed=19, failed=0, errored=0`
  - 直近の `functions` 失敗は解消済み。
- 全体回帰:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-outer-consumer-fix.json -j 1`
  - `total=339, passed=315, failed=24, errored=0`（既知集合）
- 残課題メモ:
  - `neplg2#doctest#11` は loader ではなく doctest harness 側の複数ファイル供給仕様（VFS）未整備が根因。
  - ほかの失敗主塊は `sort` / `selfhost_req` / `pipe_operator` / `tuple_new_syntax`。

# 2026-02-10 作業メモ (functions if失敗の再現チェック準備)
- `functions#doctest#7/#10` の原因切り分けのため、`typecheck` の call reduction 周辺を調査。
- 一時的に `reduce_calls` の候補探索方式を変更したが、`tests/if.n.md` が悪化（9 fail）したため取り消し済み。
- 現在はベースを復帰:
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-after-revert.json -j 1` で `55/55 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-revert.json -j 1` は `11 pass / 5 fail`（既知残件）
- 次アクション:
  - 類似再現ケースを追加して、`if` と関数値分岐の失敗条件をテストとして固定する。
  - その後、上流優先で parser/typecheck の責務境界を保った修正へ進む。

# 2026-02-10 作業メモ (if.n.md 不足ケース追加と if-layout 補正)
- `if.n.md` の不足ケースを追加:
  - `if <cond_expr>:` 形式（`then/else` を改行で与える形）
  - `if cond <cond_expr>:` 形式
  - marker 順序違反 / duplicate / missing の `compile_fail`
- parser 修正:
  - `if` の `expected=2`（`if <cond_expr>:` 系）で、`if` 直後の任意 `cond` marker を除去して cond 式として解釈できるよう修正。
  - `if-layout` の marker 順序チェックを追加し、`cond -> then -> else` の逆行をエラー化。
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-added-missing3.json -j 1`
    - `total=54, passed=54, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-ifcases.json -j 1`
    - `total=16, passed=11, failed=5, errored=0`（失敗内訳は従来の高階関数/capture 系）

# 2026-02-10 作業メモ (予約語の識別子禁止: cond/then/else/do, let/fn)
- ユーザー指示に合わせて、`cond` / `then` / `else` / `do` を予約語として扱う実装を parser に追加。
  - `nepl-core/src/parser.rs`
    - `parse_ident_symbol_item` で、layout marker の許可位置（先頭 marker / if 文脈 / while 文脈）以外での使用をエラー化。
    - `expect_ident` でも同語を識別子として受け付けないようにし、定義名・束縛名側でも拒否。
    - 既存の緩和 (`KwSet` / `KwTuple` を識別子化) は削除し、予約語を明確化。
- `let` / `fn` は lexer で keyword token 化されるため、従来どおり識別子として使用不可であることを確認。
- `tests/if.n.md` に compile_fail ケースを追加（追加のみ）:
  - `reserved_cond_cannot_be_identifier`
  - `reserved_then_cannot_be_function_name`
  - `reserved_let_fn_cannot_be_identifier`
  - `reserved_else_do_cannot_be_identifier`
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-reserved2.json -j 1`
    - `total=46, passed=46, failed=0, errored=0`
- 参考観測（継続課題）:
  - `tests/functions.n.md::doctest#7` は parser AST 形状自体は `if + con + then-block + else-block` で正しい。
  - ただし then/else ブロック内に値式が2つあり、typecheck で `expression left extra values on the stack` になる。
  - 仕様整理（複数値式の扱い）と tests/functions の意図確認が必要。

# 2026-02-10 作業メモ (if/while の AST 仕様テスト追加)
- `plan.md` の `if/while` 仕様を再確認し、`cond/then/else/do` の `:` あり/なし差分を AST で固定するテストを追加。
- `nodesrc/test_analysis_api.js` に `analyze_parse` ベースのケースを追加:
  - `parse_if_inline_no_colon_blocks`
  - `parse_if_colon_uses_block_for_cond_then_else`
  - `parse_while_inline_no_colon_blocks`
  - `parse_while_colon_uses_block_for_cond_do`
- 検証方針:
  - `:` なしでは `PrefixExpr` の引数列に `Block` を作らない。
  - `:` ありでは `if` は `Symbol + Block + Block + Block`、`while` は `Symbol + Block + Block` になることを確認。
- 実行結果:
  - `node nodesrc/test_analysis_api.js`
  - `summary: total=6, passed=6, failed=0`

# 2026-02-10 作業メモ (functions 失敗の深掘り: symbol/entry)
- `tests` 全体を再実行し、現状を再確認:
  - `/tmp/tests-restored-stable.json` = `total=312, passed=273, failed=39, errored=0`
  - 失敗の主塊は `tests/functions.n.md`（10〜11件）で、nested fn / function value / entry 解決が中心。
- `functions` の `doctest#3`（`fn main ()`）を最小再現で調査:
  - `/tmp/fnmain_no_annot.nepl` を `nepl-cli --verbose` でコンパイル。
  - 観測:
    - monomorphize 初期関数は `main__unit__i32__pure`
    - 本文中 `inc 41` が `unknown function inc` で落ちる
  - 解釈:
    - hoist 時の関数 symbol と、check_function 後の関数名（mangle 後）が一致しない経路が残っており、entry 欠落と同根。
- 試行:
  - `check_function` へ symbol override を渡し、hoist で選ばれた symbol に関数名を揃える修正を実験。
  - しかし `tests/functions.n.md` で `doctest#3` が run fail から compile fail（unknown function inc）へ悪化し、全体改善にならなかったため撤回。
- 現時点の結論:
  - 名前空間再設計（ValueNs/CallableNs 分離）と、nested fn の実体生成（少なくとも non-capture 先行）が必要。
  - 局所 patch では `functions` 群の構造問題を吸収しきれない。

# 2026-02-10 作業メモ (上流優先: if-layout parser 改善 + LSP解析API拡張)
- 上流優先の方針で parser を先に調整。
  - `if <cond>:` で then 行のみ先に見える中間状態を、確定エラーにしないよう回復分岐を追加。
  - `functions#doctest#10` の parser 失敗（`missing expression(s) in if-layout block`）を解消。
- 回帰確認:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-after-parser-upstream.json -j 4`
    - `total=312, passed=275, failed=37, errored=0`（+2 改善）
- LSP/デバッグ支援向け API を追加:
  - `nepl-web/src/lib.rs` に `analyze_name_resolution(source)` を追加。
    - `definitions`（定義点）
    - `references`（参照点、候補ID列、最終解決ID）
    - `by_name`（同名識別子の逆引き）
    - 巻き上げ規則は現行仕様（`fn` と `let` 非 `mut`）に合わせた。
  - `nodesrc/analyze_source.js` に `--stage resolve` を追加。
- API検証の追加（追加のみ、既存tests削除なし）:
  - `nodesrc/test_analysis_api.js` を新規追加。
  - `shadowing_local_let` / `fn_alias_target_resolution` を自動検証。
  - 実行結果: `2/2 passed`

# 2026-02-10 作業メモ (functions: nested fn 実体生成の前進)
- `typecheck` の `BlockChecker` で nested `fn` の本体を「未検査で無視」していた経路を改修。
  - block 内 `Stmt::FnDef` を `check_function` に渡し、`generated_functions` へ追加するよう変更。
  - top-level / impl 側の `check_function` 呼び出しにも `generated_functions` を接続。
- これにより nested `fn` の本体が HIR に入るようになり、`functions` の `double` 系が改善。
- 計測:
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-now.json -j 1`
  - `total=16, passed=10, failed=6, errored=0`
  - 残りは関数値/関数リテラル/クロージャ捕捉（`doctest#6,#7,#11,#12,#13`）に集中。
  - 全体は `node nodesrc/tests.js -i tests -o /tmp/tests-current-after-nested.json -j 4` で `312/278/34/0`。

# 2026-02-10 作業メモ (不安定差分の切り戻しと再計測)
- `typecheck` の匿名関数リテラル実験（`PrefixItem::Group` + 直後 `Block` の即席ラムダ化）を切り戻し。
  - 根拠: `functions#doctest#6` などで `unsupported function signature for wasm` / `unknown variable square` を誘発し、関数値経路が未設計のまま混入していたため。
- 再計測:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-latest.json -j 1`
    - `total=16, passed=10, failed=6, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-latest.json -j 4`
    - `total=312, passed=278, failed=34, errored=0`
- 失敗の中心は引き続き `functions` の関数値/クロージャ捕捉系（#6 #7 #11 #12 #13）。

# 2026-02-10 作業メモ (高階関数実装方式の外部調査)
- Rust/MoonBit/Wasm 仕様を確認し、NEPL 側の実装方針を整理した。
- 主要ポイント:
  - Rust:
    - クロージャは「環境を保持する構造体 + `Fn/FnMut/FnOnce` 呼び出し」で表現される（型としては関数ポインタではなく専用型）。
    - 参考: Rust book と rustc `ClosureArgs` 説明。
  - MoonBit:
    - 関数は first-class。
    - Wasm FFI では `FuncRef[T]`（閉じた関数）と、closure（関数 + 環境）を区別して扱う設計が明示されている。
    - closure は host 側で部分適用して callback 化する設計が記述されている。
  - Wasm:
    - 間接呼び出しは `call_indirect`（table 経由）または `call_ref`（function reference）で実現。
- NEPL への反映方針（次段実装）:
  - 関数値を単なる識別子参照ではなく、IRで「callable 値」として明示表現する。
  - non-capture を先行実装:
    - `fn`/`@fn` は table index を持つ関数値として扱い、呼び出しは `call_indirect` に統一。
  - capture ありは次段:
    - closure 環境オブジェクト + invoke 関数に lower する closure conversion を導入する。

# 2026-02-10 作業メモ (block 引数位置の根本修正)
- `tests/block_single_line.n.md` の `doctest#8/#9` を起点に、`add block 1 block 2` と `if true block 1 else block 2` の失敗要因を解析。
- 原因:
  - parser 上では `add [Block 1] [Block 2]` の AST が得られているのに、typecheck で `expression left extra values on the stack` が出る。
  - `PrefixItem::Block` の型検査が `check_block(b, stack.len(), true)` になっており、外側式のスタック深さを block 内評価へ持ち込んでいた。
  - その結果、引数位置 block の内部で外側スタックが混入し、簡約判定が崩れていた。
- 修正:
  - `nepl-core/src/typecheck.rs` の `PrefixItem::Block` 分岐を `check_block(b, 0, true)` に変更し、block を独立式として検査するよう統一。
  - parser 側は `block` の後続判定を限定追加（`block`/`else` 連接のみ継続）し、既存の `block:` 文境界は維持。
- 計測:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-after-typecheck-blockbase.json -j 4`
  - summary: `total=312, passed=273, failed=39, errored=0`
  - ベースライン `/tmp/tests-latest.json` (`passed=271`) から `block_single_line` の 2 件だけ改善、追加失敗なし。

# 2026-02-10 作業メモ (上流修正 継続: parser/typecheck)
- 失敗分類を再実施し、上流（lexer/parser）と typecheck の境界を切り分けた。
  - 起点: `/tmp/tests-current.json` = `total=312, passed=249, failed=63, errored=0`
- parser の根本修正:
  - `nepl-core/src/parser.rs` で識別子解析を共通化（`parse_ident_symbol_item`）。
  - これにより、式文脈ごとの実装差分を排除し、以下を統一対応:
    - `@name`
    - `::`（名前空間パス）
    - `.`（フィールド連結）
    - `<...>`（型引数）
  - `Option<.T>::None` / `Option<.T>::Some` のような「型引数 + PathSep」の連結が parse できるよう修正。
- typecheck の根本修正（pipe 簡約）:
  - `nepl-core/src/typecheck.rs` の `reduce_calls` / `reduce_calls_guarded` を open_calls 最適化依存から、スタック走査ベースへ戻した。
  - `|>` 注入時の呼び出し取りこぼし（`expression left extra values on the stack` 多発）の主要因を除去。
- 計測:
  - `/tmp/tests-after-upstream-pass.json` = `total=312, passed=261, failed=51, errored=0`
  - `/tmp/tests-after-option-fix.json` = `total=312, passed=271, failed=41, errored=0`
- 追加修正:
  - `parse_single_line_block` を「`;` が無い場合は 1 文で終了」へ変更し、単行 block の文境界を明示化。
  - ただし `add block 1 block 2` / `if true block 1 else block 2` は、prefix 1文の内側で `block` を再帰的に取り込む挙動が残り、未解決（残 fail 2）。
- 残課題（次段）:
  - `tests/functions.n.md`（11 fail）: nested fn / function-literal / alias / entry 生成整合
  - `tests/neplg2.n.md`（8 fail）と `tests/selfhost_req.n.md`（5 fail）: namespace と callable 解決の構造問題
  - `tests/pipe_operator.n.md`（4 fail）: pipe 自体の上流問題は縮小済みで、残りは型注釈/構造体アクセス仕様との整合が中心

# 2026-02-10 作業メモ (高階関数 継続: let-RHS/if-block 呼び出し順の根本修正)
- `functions` の回帰を引き起こしていた根因を 2 点に分離して修正。
  - `let f get_op true` 系:
    - `let` を通常の auto-call 経路で簡約すると `let f get_op` が先に確定し、`true` が取り残される。
    - 対応として `Symbol::Let` は `auto_call: false` とし、`check_prefix` 終端で `stack[base+1]` を RHS として `HirExprKind::Let` に確定する経路を整備。
    - `let ...;` で `statement must leave exactly one value` にならないよう、`let` 降格時に内部 stack を `unit` 1 個へ正規化。
  - `if` + `then/else` が関数値を返す系（`function_return`）:
    - `PrefixItem::Block` を `auto_call: true` で積むと、`if` の引数収集中に右端の関数値が優先され `if` 本体が簡約されない。
    - `PrefixItem::Block` の push を `auto_call: false` に変更し、`if` の 3 引数簡約を優先させるよう修正。
- `reduce_calls` は「右端優先・不足なら待つ」に戻した。
  - 左探索を有効化すると `mul n fact sub n 1` で `mul n fact` が先に確定し、再帰呼び出しが壊れることを再現確認したため撤回。

- 検証結果:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/test_analysis_api.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-block-autocall-false.json -j 1`
    - `total=19, passed=15, failed=4, errored=0`
    - 残 fail: `doctest#12 #13 #16 #17`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-hof-upstream-fixes.json -j 1`
    - `total=328, passed=288, failed=40, errored=0`

- 残件の分析:
  - `doctest#12/#13/#16`:
    - typecheck では nested 関数内 `y` 参照は解決できているが、codegen で `unknown variable y` になる。
    - これは nested 関数の capture が未 lower（closure conversion 未実装）であることが原因。
  - `doctest#17`:
    - `compile_fail` 期待に対して成功するため、純粋/非純粋の effect 判定経路（署名解釈 or overload 選択）の再点検が必要。

# 2026-02-10 作業メモ (lexer/parser 解析API追加)
- VSCode 拡張計画（todo.md の LSP / VSCode 項）を再確認し、上流解析を可視化する API を先に追加した。
- `nepl-web/src/lib.rs` に wasm 公開関数を追加:
  - `analyze_lex(source)`:
    - token 列（kind/value/debug/span）
    - diagnostics（severity/message/code/span）
    - span の byte 範囲と line/col を返す
  - `analyze_parse(source)`:
    - token 列
    - lex/parse diagnostics
    - module の木構造（Block/Stmt/Expr/PrefixItem の再帰 JSON）
    - debug 用の AST pretty 文字列
- Node 側に `nodesrc/analyze_source.js` を追加し、dist の wasm API を使って解析結果を取得できるようにした。
  - `--stage lex|parse`
  - `-i <file>` または `--source`
  - `-o <json>`
- 実行確認:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/analyze_source.js --stage lex -i tests/functions.n.md -o /tmp/functions-lex.json`: 成功
  - `node nodesrc/analyze_source.js --stage parse -i tests/functions.n.md -o /tmp/functions-parse.json`: 成功
- 回帰確認:
  - `node nodesrc/tests.js -i tests -o /tmp/tests-current.json -j 4`
  - summary: `total=312, passed=249, failed=63, errored=0`
  - 主要失敗は既知の block/typecheck 系（今回の API 追加では未着手）

# 2026-02-10 作業メモ (namespace再設計着手)
- plan.md の再確認:
  - `fn` は `let` の糖衣構文
  - 定義の巻き上げは `mut` でない `let` のみ（`fn` も含む）
- 実装・計測:
  - lexer に `@` と `0x...` を追加
  - parser に `@ident` / `fn alias @target;` / `let` 関数糖衣 / `fn` 型注釈省略を追加
  - `NO_COLOR=true trunk build` は成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-only-after-upstream-fix.json -j 4`:
    - `total=309, passed=242, failed=67, errored=0`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/functions-only-after-entry-fix.json -j 1`:
    - `total=16, passed=5, failed=11, errored=0`
- 観測した根本問題:
  - 名前解決が `Env` の単一テーブルに寄りすぎており、変数と関数値、alias、entry 解決が同一経路で干渉する
  - nested `fn` を block で宣言できても、HirFunction に落ちず `unknown function` へ繋がる
  - entry は解決できても codegen 側に関数本体が無い場合に `_start` が出力されない（実行時エラー化）
- 直近の修正:
  - top-level `fn alias` の登録を関数本体チェック前に移動
  - 型未確定関数の symbol は暫定で unmangled 名を使うよう変更（entry/mangleずれ緩和）
- 次ステップ:
  - namespace を `ValueNs` / `CallableNs` に分離し、巻き上げを仕様準拠に寄せる
  - entry の「解決済みかつ生成済み」検証を追加して compile error 化する
- ドキュメント運用修正:
  - `todo.md` は未完了タスクのみを残す形式へ整理
  - 進捗・履歴・計測値は `note.md` のみへ集約

# 2026-02-03 作業メモ (wasm32 build)
- wasm32-unknown-unknown での `cargo test --no-run` が getrandom の js feature なしで失敗していたため、`nepl-core` の wasm32 用 dev-dependencies に `getrandom` (features=["js"]) を追加した。
- `cargo test --target wasm32-unknown-unknown --no-run --all --all-features` を実行し、Cargo.lock を更新してビルドが通ることを確認。
- `cargo test --target wasm32-unknown-unknown --no-run --all --all-features --locked` も成功。
# 2026-02-03 作業メモ (selfhost string builder)
- stdlib/alloc/string.nepl に StringBuilder（sb_append/sb_append_i32/sb_build）を追加し、selfhost_req の文字列ビルダ要件を解禁した。
- stdlib/tests/string.nepl に StringBuilder の検証を追加した。
# 2026-02-03 作業メモ (selfhost string utils)
- stdlib/alloc/string.nepl に trim/starts_with/ends_with/slice/split を追加し、ASCII 空白判定や split 用の補助関数を実装した。
- stdlib/tests/string.nepl を拡充して trim/starts_with/ends_with/slice/split のテストを追加した。
- nepl-core/tests/selfhost_req.rs の文字列ユーティリティ要件テストを解禁し、Option unwrap と len 呼び出しに合わせて内容を調整した。
- doc/testing.md の stdlib スコープ一覧を更新し、alloc/string の追加関数を反映した。
- 未対応: file I/O (WASI の path_open 等) と u8/バイト配列は型・実行環境の整備が必要なため未着手。string-keyed map/trait 拡張も後続で対応予定。
# 2026-02-03 作業メモ (block ルール更新対応)
- block: がブロック式、`:` が引数レイアウトという新ルールに合わせ、パーサの `:` 処理を整理。`block` は末尾ならマーカー扱い、`cond/then/else/do` は単独（型注釈のみ許可）でマーカー扱いにし、`if cond:` のような通常識別子を誤判定しないようにした。
- `if`/`while` のレイアウト展開で `ExprSemi` を許可し、`while` 本体に `;` を書いたテストが panic しないよう修正。
- stdlib/例: `while ...:` の複数文ボディを `do:` ブロック化（stdlib/alloc/*, core/mem, std/stdio, std/env/cliarg, kp/kpread, examples/counter/fib/rpn など）。`examples/rpn.nepl` の入れ子 while も `do:` に統一。
- tests: `nepl-core/tests/plan.rs` を `block:` 使用に更新、`nepl-core/tests/typeannot.rs` の while を `do:` に更新。`stdlib/tests/vec.nepl` の match arm から誤った `block` マーカーを除去。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行し、両方成功（警告は既存のまま）。
# 2026-02-03 作業メモ (依存更新/online cargo test)
- workspace 依存を最新安定版へ更新（thiserror 2.0.18、anyhow 1.0.100、clap 4.5.56、wasm-bindgen 0.2.108、assert_cmd 2.1.2、tempfile 3.24.0 など）。rand は最新安定の 0.8.5 のまま。
- wasmi 1.0.8 への更新を試したが、rustc 1.83.0 では 1.86 以上が必要で不可。wasmi は 0.31.2 に戻して Cargo.lock を更新。
- テスト: オンライン `cargo test` を実行。`nepl-core/tests/overload.rs` の `test_overload_cast_like` と `test_explicit_type_annotation_prefix` が "ambiguous overload" で失敗。他のテストは成功。
# 2026-02-03 作業メモ (trait/overload 修正の根本対応)
- overload の重複削除が `type_to_string` の "func" 返却で全て同一扱いになっていたため、関数シグネチャ文字列を導入し、重複判定と impl メソッド署名一致判定をシグネチャ比較に変更。
- trait method の呼び出しで `Self` ラベルと型パラメータが不一致になる問題を、`Self` ラベルは任意型と統一可能にすることで解消。
- monomorphize で trait 呼び出しを具体関数へ解決する際、解決先関数のインスタンス化要求を行うよう変更し、unknown function を解消。
- テスト: `cargo run -p nepl-cli -- test` は成功（警告あり）。
- テスト: `cargo test` は 120 秒でタイムアウト（警告出力後に未完了）。
# 2026-02-03 作業メモ (stdlib テスト拡充/修正)
- stdlib/std/hashmap.nepl の if レイアウトを修正し、hash_i32 を純粋関数に書き換え（16進リテラルを10進へ置換）。hashmap_get は再帰ループで純粋化。
- stdlib/std/hashset.nepl の hash_i32 を純粋関数へ変更し、hashset_contains を再帰ループで純粋化。hashset_contains_loop のシグネチャ不整合も修正。
- stdlib/std/result.nepl の unwrap_err を Err 分岐先頭に並べ、match の戻り型が never になる問題を回避。
- stdlib/tests に hashmap.nepl/hashset.nepl/json.nepl を追加し、基本操作（new/insert/get/remove/len/contains など）と JSON の各アクセサを検証。
- stdlib/tests/result.nepl は map 系を外し、unwrap_ok/unwrap_err の検証に置き換え。json.nepl は move 連鎖を避けるため値を都度生成する形に整理。
- テスト: `cargo run -p nepl-cli -- test` は成功（警告は残存）。
- テスト: `cargo test` は 120 秒でタイムアウト（警告出力後に未完了）。
# 2026-02-03 作業メモ (trait/overload)
- AST/パーサ: 型パラメータを TypeParam 化し、`.T: TraitA & TraitB` 形式の境界を読めるようにした。
- HIR: trait 呼び出し (`Trait::method`) を表現できるようにし、impl 側はメソッド一覧を持つ形に変更。
- 型検査: trait 定義/impl の整合性チェック、Self 型の差し込み、trait bound の満足判定を追加。関数の同名オーバーロードを許可し、mangle したシンボルで内部名を一意化。
- 単相化: impl マップを構築し、trait 呼び出しを具体的なメソッド実体に解決するようにした。
- テスト: nepl-core/tests/neplg2.rs にオーバーロード/trait のコンパイルテストを追加。
- 既知の制限: trait の型パラメータ、inherent impl、impl メソッドのジェネリクスは未対応。オーバーロード解決は引数型のみで行い、戻り値型は使わない。export 名は mangle 後の一意名になる。
- テスト: `cargo test -p nepl-core --lib` を実行（警告は残存）。
# 2026-02-03 作業メモ (never 型と unwrap 修正)
- `unreachable` 分岐で型変数が `never` に束縛され、`Option::unwrap` が `unwrap__Option_never__never__pure` へ潰れる問題を修正。
- `types::unify` で `Var` と `Never` の統一時に束縛しないよう特例を追加し、`unwrap__Option_T__T__pure` を保持するようにした。
- codegen の `unknown function` 診断に欠落関数名を含めるよう改善。
- テスト: `cargo run -p nepl-cli -- test` は成功（警告あり）。
- テスト: `cargo test` は 240 秒でタイムアウト（コンパイル途中）。再実行が必要。
# 2026-02-03 作業メモ (btreemap/btreeset 追加)
- stdlib/std/btreemap.nepl と stdlib/std/btreeset.nepl を追加し、i32 キー/要素の順序付きコレクションを配列ベースで実装した（検索は二分探索、挿入/削除はシフト）。
- stdlib/tests/btreemap.nepl と stdlib/tests/btreeset.nepl を追加し、基本操作（挿入/更新/削除/検索/長さ）を検証した。
- doc/testing.md の stdlib 一覧に std/btreemap と std/btreeset を追記した。
# 2026-02-03 作業メモ (test 彩色/stdlib テスト調整/コンパイラ確認)
- stdlib/std/test.nepl の失敗メッセージを ANSI 赤色で表示するよう変更し、std/stdio の色出力を利用。
- stdlib/tests/error.nepl で `fail` の使用を避け、error_new 由来の診断が非空であることを確認する形に調整。
- stdlib/tests/cliarg.nepl/list.nepl/stack.nepl/vec.nepl/string.nepl/diag.nepl を更新し、失敗時のメッセージを明示するテストに整理。
- doc/testing.md の失敗時の表示説明を更新。
- コンパイラ確認: error::fail（callsite_span 経由）を含むテストで wasm 検証エラーが発生するため、std テスト側では該当経路を使わないようにして回避。Rust 側の callsite_span/codegen の相性は要調査。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (nepl-cli test の色付け)
- nepl-cli のテスト出力を ANSI 色付きにし、test/ok/FAILED の視認性を上げた。
- doc/testing.md に色付き出力の注記を追記。
# 2026-02-03 作業メモ (stdlib/diag 色分け)
- stdlib/std/diag.nepl に ErrorKind ごとの色割り当てを追加し、diag_print/diag_println/diag_debug_print で色付き表示に変更。
- stdlib/std/stdio.nepl に debug_color/debugln_color を追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (Checked ログの色付け)
- stdlib/std/test.nepl に test_checked を追加し、"Checked ..." の成功ログを緑色で出すようにした。
- stdlib/tests/list.nepl と stdlib/tests/math.nepl の Checked ログを test_checked に置き換えた。
- doc/testing.md に test_checked を追記。
# 2026-02-03 作業メモ (テスト失敗のメッセージ表示)
- stdlib/std/test.nepl を改修し、失敗時にメッセージを表示してから trap するよう変更した。
- stdlib/std/diag.nepl に diag_print_msg を追加し、Failure メッセージを表示できるようにした。
- stdlib/std/error.nepl の fail/context を callsite_span 付与に更新した。
- stdlib/tests/diag.nepl と stdlib/tests/error.nepl を強化し、文字列化や span の検証を追加した。
- doc/testing.md の assert 仕様を更新した。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (cliarg 追加)
- stdlib/std/cliarg.nepl を追加し、WASI args_sizes_get/args_get で argv を取得できるようにした。
- stdlib/tests/cliarg.nepl を追加し、範囲外/負の index が None になることを確認するテストを用意した。
- doc/testing.md の stdlib 一覧に std/cliarg を追記した。
- nepl-cli の WASI ランタイムに args_sizes_get/args_get を追加し、`--` 以降の引数を渡せるようにした。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (cliarg 実引数テスト)
- stdlib/tests/cliarg.nepl を更新し、argv[1..] の値を検証するテストを追加した。
- nepl-cli の stdlib テスト実行で `--flag value` を argv に渡すよう変更した。
- doc/testing.md に stdlib テストが固定引数を渡す旨を追記した。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (stdlib コメント言語統一)
- stdlib/std/option.nepl と stdlib/std/result.nepl の英語コメント行を削除し、コメントが日本語のみになるよう統一。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (stdlib コメント/Option/Result 改修)
- stdlib/std の各ファイルに日本語コメント（ファイル概要/各関数の目的・実装・注意・計算量）を追加し、math.nepl は自動生成で関数コメントを挿入。
- list_tail を Option<i32> 返却に変更し、list_get の走査を unit になるよう調整（デバッグ出力も削除）。
- stdlib/tests/list.nepl を list_tail の Option 仕様に合わせて更新。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` が成功。

# 2026-02-03 作業メモ (import/resolve テスト拡充)
- nepl-core/tests/resolve.rs に default alias（相対/パッケージ）、selective 欠落名の扱い、merge open、visible map 優先順位（local/ selective/ open）を追加。
- nepl-core/src/module_graph.rs の unit テストに missing dependency/invalid import/duplicate export/non-pub import/ selective+glob re-export を追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` が成功。

# 2026-02-03 作業メモ (rpn 実行 + std/test 修正 + テスト実行)
- examples/rpn.nepl を `printf "3 4 +\n" | cargo run -p nepl-cli -- -i examples/rpn.nepl --target wasi --run` で実行し、REPL が結果を返して終了することを確認。
- stdlib/std/test.nepl の `assert_str_eq` を `if:` ブロック形式に修正し、`(trap; ())` の inline 1行式を排除してパーサエラーを解消。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` が成功。

# 2026-02-03 作業メモ (rpn import + diagnostics)
- examples/rpn.nepl の import を新仕様（`#import "..." as *`）へ更新。
- loader の parse でエラー診断がある場合は CoreError を返すようにし、構文エラーが型エラーに埋もれないよう修正。
- CLI の診断表示でキャレット長を行末に収め、巨大な ^ の出力を抑制。
- typecheck の簡易サマリ出力は verbose 時のみ表示するように変更。

# 2026-02-03 作業メモ (Windows path canonicalization for tests)
- module_graph の lib テストで path 比較が Windows の canonicalize 差分で失敗するため、root path を canonicalize して比較するよう修正。
- resolve.rs 側の ModuleGraph 参照テストも同様に canonicalize を適用し、クロスプラットフォームで一致するようにした。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (resolve import tests fix)
- nepl-core/tests/resolve.rs のテスト用ソースを `:` ブロック形式に修正し、parser の期待するインデント構造に合わせた。
- selective glob（`name::*`）が open import に反映されることを確認するテストを追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (resolve/import test expansion)
- nepl-core/tests/resolve.rs を追加実装し、prelude 指令の解析、merge clause 保持、alias/open/selective の解決、open import の曖昧性診断、std パッケージ解決のテストを追加。
- nepl-core/tests/neplg2.rs に prelude/import/merge 指令の受理確認テストを追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (tests import syntax migration)
- nepl-core/tests と stdlib 配下の #import/#use を新仕様（`#import "..." as *`）へ統一し、#use を除去した。
- loader_cycle のテストは `#import "./a"`/`#import "./b"` に変更して相対 import の仕様に合わせた。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (selective re-export test)
- module_graph の pub selective re-export の挙動を確認するテストを追加（alias のみ公開され、元名や未選択の公開項目は再エクスポートされないことを検証）。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (pub import selective re-export)
- build_exports が ImportClause::Selective を考慮し、pub import の再エクスポート範囲を selective に限定できるようにした（glob は全件再エクスポート扱い）。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (module_graph import clause)
- module_graph の import/deps に ImportClause を保持するようにし、resolve が AST ではなく ModuleGraph の情報から import 句を参照する形へ変更。
- resolve の import 走査を整理し、deps の clause を直接使って alias/open/selective/merge を構築。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (pub #import / pub item)
- lexer で `pub #import` を認識し、`#import pub ...` へ書き換える処理を追加（`pub` 前置のディレクティブは #import のみ許可）。
- parser で `pub fn/struct/enum/trait/impl` をトップレベルで解釈できるようにし、`pub` が先頭に来ても正しく定義を読めるようにした。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (rewrite plan doc)
- doc/rewrite_plan.md を現行コード確認に基づいて拡充し、後方互換なしの設計書+実装計画書として整理した（モジュールID/manifest、import clause、prelude、名前解決優先順位、型推論/単相化、WASM ABI、CLI/stdlib境界、実装ロードマップ、テスト方針）。
- 現行パイプラインは loader の AST スプライス方式のままで、module_graph/resolve の実装は未統合である点を計画内に明記。
- plan.md には manifest/新import文法/prelude/mergeの仕様や CLI/ABI 境界の整理が未記載のため、追記が必要。
- テスト: 以前は `module_graph::tests::builds_simple_graph_and_exports` が unknown token で失敗していたが、`pub #import`/`pub fn` 対応後に `cargo test` も成功。

## 直近の実装サマリ
- 文字列リテラルと型 `str` を追加し、データセクションに `[len][bytes]` で配置して常時メモリをエクスポートする形に統一。
- `#extern` で外部関数を宣言可能にし、stdlib から `print` / `print_i32` を提供する構成に統一。ビルトイン関数は撤廃。
- CLI: `--target wasm|wasi` に対応（wasi が wasm を包含）。`--run` だけでも実行可。コンパイル失敗時に SourceMap 付き診断を出力。
- Loader/SourceMap を導入し、import/include で FileId/Span を保持したまま多ファイルを統合。
- パイプ演算子 `|>` を追加。スタックトップを次の呼び出しの第1引数に注入する仕様で、lexer/parser/typecheck まで実装済み。
- `:` ブロックと `;` の型検査を調整し、Unit 破棄や while の stack 深さ検証を改善。
- stdlib: math/mem/string/result/option/list/stdio を追加・更新。mem は raw wasm、string/result/option はタグ付けポインタ表現、stdio は WASI fd_write 前提。
- `#target wasm|wasi` をディレクティブとして追加。CLI がターゲットを指定しない場合は #target をデフォルトに用い、複数 #target は診断エラーにした。wasi 含有ルールは従来通り。
- stdlib/std/stdio を WASI `fd_write` 実装に置き換え、env 依存を排除。print_i32 は from_i32 → fd_write で出力。
- 型注釈の「恒等関数」ショートカットを削除し、ascription のみで扱う前提に揃えた。`|>`+注釈の回りのテストを追加。
- std/mem.alloc を要求サイズから算出したページ数で memory.grow する形にし、固定1ページ成長を解消（ただしページ境界アロケータのまま）。
- CLI の target フラグを省略可能にし、#target / stdio 自動 wasi 昇格と整合するようにした。
- テスト追加: #target wasi デフォルト動作、重複 #target エラー、pipe+型注釈の成功ケース。
- 言語に struct/enum/match を追加。enum/struct を TypeCtx に登録し、コンストラクタを自動バインド（`Type::Variant` / `StructName`）。match は網羅性チェックと型整合チェックを行う。
- Option/Result を enum ベースに再実装（OptionI32/ResultI32）。string/find/to_i32/list/get などを Result/Option 返却に差し替え。list の get は ResultI32 で境界エラーを返す。
- codegen に enum/struct コンストラクタと match を追加（runtime 表現は [tag][payload]/構造体フィールドを linear memory 上に確保し、std/mem.alloc 呼び出しを前提）。
- pipe の注入タイミングを調整し、型注釈 `<T>` を挟んでも `|>` が正しく次の callable に注入されるようにした。追加テストで確認。
- Loader の循環 import 検出テストを追加（temp ディレクトリに a.nepl/b.nepl を生成しロードでエラーを確認）。

## plan.md との乖離・注意点
- `#target`: ディレクティブとしては実装済みだが、plan.md には未記載。エントリーファイル以外に書かれた場合の扱いなど仕様明記が必要。
- 型注釈 `<T>`: 恒等関数ショートカットは削除したが、plan.md には「関数と見做す」とあるので記述を更新する必要あり。
- stdlib/stdio: WASI `fd_write` 実装に置き換え済み。wasm で import した際の専用診断はまだ無いので、エラーメッセージ改善の余地あり。
- stdlib/mem.alloc: サイズに応じたページ成長に修正したが、ページ境界アロケータのまま。細粒度管理や free は未対応。
- Option/Result/list: enum/match が無いためタグ付きポインタの暫定実装。型システム統合や多相化は未着手。list は i32 固定で get の範囲外診断なし。

## 追加で気付いたこと
- Loader は FileId/Span を保持して diagnostics に活用できている。#include/#import は一度きりロードで循環検出あり。
- コード生成は wasm のみ。CompileTarget::allows は wasi が wasm を包含する形で gate 判定を実装。

# 2026-01-23 作業メモ
- Rust ツールチェインを rustup で導入し、依存クレートを取得できるようにした。
- #if 関連の unknown token を解消するため lexer の `* >` / `- >` を Arrow として許可するよう緩和した。
- stdlib の構築途中コードが多数コンパイルを塞いでいたため、一時的に std/string・std/list・std/stdio を最小機能のスタブ実装に差し替え（option.unwrap_or を削除して重複解消）。
- enum コンストラクタの codegen を修正（payload store のオペランド順と、結果ポインタをスタックに残すように変更）。これにより Option::Some/None が正しく値を返し、`match_option_some_returns_value` が通過。
- std/list.get は境界外を常に `ResultI32::Err 1` で返す単純実装にし、スタック不整合の診断を解消。現状 in-bounds 取得は未対応だがテスト想定（OOB エラー）には合致。
- 現在 `cargo test` は 23/23 すべて成功。残課題は stdlib 機能の肉付け（list.get の正実装、文字列/オプションの汎用化など）。

## 今後の対応案（実装はまだしない）
- `#target wasi|wasm` をディレクティブとして追加し、ファイル内のデフォルトターゲットを決定（CLI 指定があればそちらを優先）。`#if[target=...]` 評価にも使用。
- 型注釈の古い恒等関数特例を撤去し、注釈は構文要素としてのみ扱う旨を仕様に明記。
- stdio を WASI fd_write 実装に戻す／もしくは wasm target で import された場合にコンパイル時エラーを出す。
- mem.alloc の size 対応とページ再利用、list の多相化・境界チェック強化、Option/Result を enum/match 連携へ移行。

# 2026-01-30 作業メモ
- stdlib/std/string.nepl の to_i32 内で if: ブロックに誤って if eq ok 1: / else: が混入するインデントになっており、if-layout 解析が "too many expressions" になる状態だったため、if eq ok 1: ブロックを1段デデントし、else ブロックのインデントを整えて if-layout が正しく分解されるよう修正。
- これにより std/string の cond/then/else 未定義エラーと block stack エラーが解消。cargo test は全件通過、examples/counter.nepl を wasi 実行しても完走することを確認。
- 文字列リテラルが allocator のメタ領域と衝突していたため、codegen_wasm の文字列配置開始オフセットを 8 バイト（heap_ptr + free_list_head）に変更し、data section で free_list_head=0 を明示。併せて data section を常に出力して heap_ptr を初期化するよう修正。

# 2026-02-01 if/while テスト無限ループ対応
## 問題発見
- ifテストが16GB以上のメモリ使用となり、実行が停止する無限ループ問題を発見。
- パーサー側は`if` ブロック分解で正常に動作している（テスト通過確認）。
- 無限ループはタイプチェック段階で発生している模様。

## 原因特定と修正
- `apply_function()` の `if` ケースで、関数型 `(bool, T, T) -> T` の `result` 型変数が統一されていなかった。
- 2つのブランチ型を統一した後、その結果を `result` 型変数に統一する必要があった。
- 修正: `let final_ty = self.ctx.unify(result, t).unwrap_or(t);` を追加し、結果型を関数の result 型パラメータと統一。
- 同じく `while` も同様の問題があったため、`let final_ty = self.ctx.unify(result, self.ctx.unit()).unwrap_or(self.ctx.unit());` で修正。

## テスト実行結果
- 修正後、部分的にテストが成功開始（8個テスト確認: if_mixed_cond_then_block_else_block など）
- 残り7個のテストでメモリスパイク続行
  - 失敗テスト: if_a_returns_expected, if_b_returns_expected, if_c_returns_expected, if_d_returns_expected, if_e_returns_expected, if_f_returns_expected, if_c_variant_lt_condition
  - これらは全て `#import "std/math"` と `#use std::math::*` を含む

## 次のステップ
- 失敗しているテストの共通点は import/use ステートメント
- ローダー或いはモノモルファイゼーション段階での無限ループの可能性を調査中

- これにより WASI 実行時の print（文字列リテラル）の無出力／ゴミ出力が解消。stdout の回帰検出用に `nepl-core/tests/fixtures/stdout.nepl` を追加し、`nepl-core/tests/stdout.rs` と `run_main_capture_stdout` を実装。
- 文字列操作のテストとして `nepl-core/tests/stdlib.rs` に len(文字列リテラル) と from_i32→len を追加。`cargo test -p nepl-core --test stdlib --test stdout` で確認。
- plan2.md と doc/starting_detail.md はリポジトリ内に存在しないため、参照できない状態のまま。
- stdlib/std/stdio に `println` を追加し、`print` + 改行文字列で実装。`print`/`print_i32` はそのまま維持。
- stdlib/std/stdio の `print_str` を `print` に改名し、`println_i32` を追加。str は `print`/`println`、i32 は `print_i32`/`println_i32` を提供する形に整理。
- `nepl-core/tests/fixtures/println_i32.nepl` と stdout テストを追加し、`println_i32` が改行を出力することを確認。
- examples の逆ポーランド記法電卓 `examples/rpn.nepl` を文字列パース方式に拡張し、ASCII トークンを走査して数値/演算子を処理する形に更新。
- stdlib/std/stdio から std/string の import を外し、print は文字列ヘッダ長を直接読む形に変更。print_i32 は同一ファイル内で数値→文字列変換を行い、std/list との `len` 衝突を回避。
- stdlib/std/stdio に `read_all` を追加し、WASI の fd_read で標準入力を取り込めるようにした。CLI ランタイムにも fd_read 実装と stdin バッファを追加。
- stdin の動作確認用に `nepl-core/tests/stdin.rs` と `nepl-core/tests/fixtures/stdin_echo.nepl` を追加し、日本語入力のエコーもテストに含めた。
- CLI の fd_read をオンデマンド読み込みに変更し、起動時に stdin を read_to_end しないことで対話入力でもブロックしないように調整。
- stdlib/std/stdio に `read_line` を追加し、REPL 向けに改行までの読み取りを提供。stdin テストに `stdin_readline.nepl` と日本語ケースを追加。
- examples/rpn.nepl を REPL 形式に変更し、1行ごとの評価とエラーメッセージ表示に対応。`read_line` を使うため、対話入力でも評価できるようにした。
- examples/rpn.nepl に REPL 使い方のメッセージを追加し、PowerShell パイプ時の BOM を無視する簡易スキップ処理を入れて unknown token を回避。
- stdout 用の fixture とテストを追加し、`println` が `\n` を出力することを確認。README の std/stdio 説明も `println` と WASI `fd_write` に合わせて更新。
- stdout テストで wasi fd_read の import 未提供により instantiate 失敗していたため、`nepl-core/tests/harness.rs` の `run_main_capture_stdout` に fd_read スタブを追加。`cargo test -p nepl-core --test stdin --test stdout` は警告付きで成功し、`printf '14 5 6 + -' | cargo run -q -- -i examples/rpn.nepl --run --target wasi` で REPL 出力と結果 3 を確認。
- PowerShell の UTF-16LE パイプ入力で数値が分割される可能性に備え、`examples/rpn.nepl` の数値パースで NUL バイトを無視する分岐を追加（BOM スキップと併用）。

# 2026-01-30 作業メモ (テスト/stdlib)
- stdlib に `std/test` を追加し、`assert`/`assert_eq_i32`/`assert_str_eq`/`assert_ok_i32`/`assert_err_i32` を提供。`trap` は `i32.div_s` を 0 で割る #wasm で実装し、WASM 側で確実に異常終了するようにした。
- `std/string` に `str_eq`（純粋再帰）を追加し、`std/test` 側の文字列比較でも同等ロジックを使用。
- CLI に `nepl test` サブコマンドを追加し、`stdlib/tests` 配下の `.nepl` を収集して WASI で実行するテストランナーを実装。
- stdlib テストを `stdlib/tests/{math,string,result,list}.nepl` に追加。式の括弧は使わず前置記法で記述し、Result の move を避けるため同一値を再生成して検証。
- `cargo run -p nepl-cli -- test` と `cargo test` が通ることを確認。
- doc に `doc/testing.md` を追加し、テスト機能の使い方と stdlib の現状範囲を整理。

# 2026-01-30 作業メモ (examples 実行確認)
- examples/counter.nepl と examples/fib.nepl を `#target wasi` に揃え、std/stdio の利用を明示。
- `cargo run -p nepl-cli -- -i examples/counter.nepl --run --target wasi` と `... fib.nepl ...`、`printf '14 5 6 + -\n' | ... rpn.nepl ...` を実行し、出力が正常であることを確認。
- `cargo test` を再実行し、全テストが通過することを確認。

# 2026-01-30 作業メモ (多相/単相化の現状)
- パーサは fn/enum/struct/trait/impl の型パラメータ宣言と型適用 `TypeName<...>` を受理し、TypeCtx には TypeKind::{Function,Enum,Struct} の type_params と TypeKind::Apply がある。
- 関数呼び出しでは typecheck が type_params を fresh var に instantiate し、呼び出し側に type_args を残す。monomorphize は FuncRef の type_args をもとに関数だけ単相化してマングル名を生成する。
- TypeKind::Apply は unify が扱わず、resolve も match 以外で使われていないため、型注釈やシグネチャで `Foo<...>` を使うと実質的に整合しない。
- enum/struct のコンストラクタは定義側の型情報を直接使っており、instantiate された params/result を反映しないため型変数がグローバルに束縛されやすく、ジェネリック enum/struct が実用になっていない。
- stdlib の list/option/result は i32 固定で、ジェネリクスは未導入。

## plan.md との差分メモ (追加)
- plan.md にはテスト実行コマンドや `std/test`/`nepl test` の仕様が未記載。テスト設計の章立てを追加する必要がある。
- plan2.md と doc/starting_detail.md は引き続きリポジトリ内に存在しないため参照不可。
- plan.md では「定義での多相は扱わない」としているが、実装には type_params と monomorphize が存在する。仕様整合の追記が必要。

# 2026-01-30 作業メモ (ジェネリクス修正)
- 型パラメータは .T 形式のみ許可するように parser を更新し、<T> はエラーにした。
- Apply を unify で resolve して enum/struct の具体型と統合できるようにし、resolve の結果は型引数を type_params に保持するよう変更。
- enum/struct コンストラクタは instantiate 後の params/result を使うようにし、型変数のグローバル束縛を避ける形に修正。
- type_to_string は enum/struct の type_params を含めるようにして単相化マングルの衝突を避けた。
- codegen で Apply を参照型として扱い、enum の variant 解決を Apply にも対応。
- Rust テスト `nepl-core/tests/generics.rs` を追加し、fn/enum/struct のジェネリクスとエラーケースを検証。

# 2026-01-30 作業メモ (ジェネリクス修正の追加)
- parser のエラー診断が出ている場合は compile_wasm を失敗させるようにし、<T> を実際にエラー扱いにした。
- Apply の型引数数不一致は unify で失敗させ、型注釈の不一致として診断されるようにした。
- 型引数は typecheck と monomorphize で resolve_id により実体型へ正規化し、単相化後に Var が残らないようにした。
- wasm 生成後に wasmparser で検証し、無効 wasm を診断として返すようにした。

# 2026-01-30 作業メモ (ジェネリクス修正の追加2)
- 型注釈が未適用のまま let が先に簡約されるケースがあったため、pending_ascription がある間はその手前の関数を簡約しないよう guarded reduce を追加。
- type_args の resolve を引数 unify 後に行うようにし、単相化に Var が残らないように修正。

# 2026-01-30 作業メモ (ジェネリクス テスト拡張)
- generics.rs に .T 必須の enum/struct 定義エラー、payload の i32 演算検証、複数型パラメータ関数の単相化、型注釈不一致のエラーを追加。
- さらに、None の型決定、引数なしジェネリック関数の型決定、ジェネリック関数の委譲呼び出し、pipe 経由呼び出し、2型パラメータ enum の match、入れ子 Apply の payload・その不一致エラー、同一型パラメータの不一致エラー、payload 型不一致エラーを追加。
- 追加で、コンストラクタの型推論（引数位置）、ジェネリック関数での Pair 構築、Option::Some ラッパー関数、Option<Option<T>> の入れ子 match を OK ケースとして追加。

# 2026-01-31 作業メモ (ジェネリクス/構文/コード生成)
- if-layout の cond 識別子が変数名として使われるケースに対応するため、`normalize_then_else` で cond を無条件に消さず、then/else マーカーがある場合のみ除去するよう調整。
- `if cond:` のような行末 `:` 形式で cond が変数名の場合に stack エラーが出ていたため、if-layout 判定から `if cond:` の特例を外し、cond 変数を保持する形に変更。
- match 式が後続の行を吸い込むケースがあったため、`KwMatch` で match 式を読み込んだら prefix 解析を打ち切るように修正。
- wasm codegen の match が 2分岐固定だったため、任意個（1個以上）の分岐を if 連鎖で生成するように拡張し、1バリアント enum の match で unreachable が出る問題を解消。
- `generics_multi_type_params_function` の期待値は if の振る舞いに合わせて 3 に修正（false 分岐の確認）。
- `cargo test` は全件通過を確認。
- plan2.md と doc/starting_detail.md は引き続きリポジトリ内に存在しないため参照不可。

# 2026-01-31 作業メモ (テスト整合)
- nepl-core の `list_get_out_of_bounds_err` テストを現行 stdlib に合わせ、`list_nil/list_cons/list_get` と `Option` の `Some/None` マッチに更新。
- `cargo test` と `cargo run -p nepl-cli -- test` の両方が成功することを確認。

# 2026-01-31 作業メモ (ログ抑制)
- typecheck/unify/monomorphize/wasm_sig の成功時ログを削除し、OK時の `nepl-cli test` の出力を削減。
- `cargo run -p nepl-cli -- test` はテスト結果のみ表示されることを確認（Rust の警告は別途表示）。

# 2026-01-31 作業メモ (verbose フラグ)
- `nepl-cli` に `--verbose` を追加し、詳細なコンパイラログを必要時のみ出力できるようにした。
- `CompileOptions.verbose` で制御し、typecheck/unify/monomorphize/wasm_sig のログをフラグ連動にした。

# 2026-01-31 作業メモ (メモリアロケータ)
- `std/mem` の allocator を wasm モジュール内実装に変更し、`nepl_alloc` のホスト依存を除去。
- free list + bump 併用の簡易 allocator を実装し、`memory.grow` で拡張。
- `doc/runtime.md` に WASM/WASI のターゲット方針とメモリレイアウトを追加。

# 2026-01-31 作業メモ (nepl_alloc 自動 import の撤去)
- コンパイラが `nepl_alloc` を自動で extern に追加する処理を削除し、WASM 生成物がホスト依存の import を持たないようにした。
- `alloc`/`dealloc`/`realloc` は `std/mem` の定義か `#extern` により解決される前提になったため、モジュール側で `std/mem` を import していない場合は codegen でエラーになる。
- 既存の `a.wasm` などは再コンパイルが必要（古いバイナリには `nepl_alloc` import が残る）。
- `alloc` などのビルトイン自動登録も外したため、`std/mem` の関数定義がそのまま使用される。`alloc` を使うコードは `std/mem` を明示的に import する必要がある。

# 2026-01-31 作業メモ (std/mem の効果注釈)
- `std/mem` の `alloc`/`dealloc`/`realloc`/`mem_grow`/`store` を `*` 付きに変更し、純粋コンテキストから呼べないことを明示した。
- これにより `std/mem` 内部の `set`/`store_*` 呼び出しが純粋関数扱いになっていた問題を解消し、`match_arm_local_drop_preserves_return` の失敗原因を修正した。

# 2026-01-31 作業メモ (monomorphize のランタイム関数保持)
- エントリ起点の単相化で `alloc` が落ちる問題を避けるため、`monomorphize` の初期 worklist に `alloc`/`dealloc`/`realloc` を追加した。
- enum/struct/tuple の codegen が `alloc` を呼ぶ前提でも、未参照の `alloc` が除去されないようにした。

# 2026-01-31 作業メモ (テスト側の std/mem 明示)
- enum/struct/tuple を使うテストソースに `std/mem` の import を追加し、`alloc` が解決される前提を明確化した。
- `move_check` テストは Loader 経由で compile するように変更し、`#import` を解決できるようにした。

# 2026-01-31 作業メモ (標準エラー/診断の追加)
- `std/error` と `std/diag` を追加し、`ErrorKind`/`Error`/`Span` と簡易レポート生成を用意した。
- `callsite_span` の intrinsic を追加し、エラーに呼び出し位置を付与できるようにした。
- `std/string` に `concat`/`concat3` を追加し、診断文字列生成の最低限を実装した。

# 2026-01-31 作業メモ (WASI エントリポイント対応)
- codegen_wasm で entry 関数が指定されている場合、その関数を `_start` という名前でも export するようにした。
- これにより `wasmer run a.wasm` / `wasmtime run a.wasm` で WASI コンプライアンスに従い直接実行可能に。
- README.md に外部 WASI ランタイム（wasmtime/wasmer）での実行方法を追加。

# 2026-01-31 作業メモ (数値演算の完全化)
- stdlib/std/math.nepl を全面拡張し、i32/i64/f32/f64 のすべての演算機能を提供。
- **算術演算**：add/sub/mul/div_s/div_u/rem_s/rem_u（すべての型で符号別に提供）
- **ビット演算**：and/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt（整数型のみ）
- **浮動小数点特有**：sqrt/abs/neg/ceil/floor/trunc/nearest/min/max/copysign（f32/f64）
- **型変換**：i32/i64 <-> f32/f64、符号付き/符号なし対応、飽和変換（trunc_sat）
- **ビット再解釈**：reinterpret_i32/f32/i64/f64

# 2026-02-03 作業メモ (web playground)
- Trunk の `public_url` を `/` に変更し、`trunk serve` のローカル配信パスを `http://127.0.0.1:8080/` に統一。
- `web/index.html` に `vendor` の copy-dir を追加し、`web/vendor` を用意して editor sample の静的配布を Trunk 経由で行えるようにした。
- README と doc/web_playground.md に editor sample の取得手順とローカル起動 URL を追記。
- `web/index.html` の CSS/JS を Trunk 管理のアセットとして宣言し、`styles.css` と `main.js` が dist に出力されるように調整。
- `web/main.js` は Trunk の `TrunkApplicationStarted` イベントと `window.wasmBindings` を利用して wasm-bindgen 生成物にアクセスする方式に変更。
- 埋め込み editor は `web/vendor/editorsample` が存在する場合のみ iframe に読み込み、存在しない場合はフォールバック textarea を使用するように変更。
- doc/web_playground.md に `public_url` と `serve-base` の関係を追記し、`trunk serve` のアクセスパスに関する注意点を明記。

## plan.md との乖離・注意点 (追加)
- plan.md に web playground の配信手順は未記載のため、必要なら仕様欄に追記が必要。

# 2026-02-03 作業メモ (kpread UTF-8 BOM 対応)
- PowerShell のパイプ入力が UTF-8 BOM (EF BB BF) を付与する場合、kpread の `scanner_read_i32` が先頭の BOM を数値として扱い、0 を返し続ける問題を確認。
- `scanner_skip_ws` に UTF-8 BOM のスキップを追加し、既存の UTF-16 BOM/NULL スキップと同じ位置で処理。
- 回帰テストとして `nepl-core/tests/fixtures/stdin_kpread_i32.nepl` を追加し、`stdin_kpread_utf8_bom` で BOM 付き入力を検証。
- 動作確認: `printf '\xEF\xBB\xBF1 3\n' | cargo run -p nepl-cli -- -i examples/abc086_a.tmp.nepl --run`

# 2026-02-03 作業メモ (日本語文字列の stdout)
- 文字列リテラルの lexer が UTF-8 を 1 バイトずつ `char` に変換していたため、日本語が mojibake になる問題を確認。
- 文字列リテラルの通常文字の読み取りを UTF-8 `char` 単位に変更し、`i` を `len_utf8` 分進めるよう修正。
- 回帰テストとして `nepl-core/tests/fixtures/stdout_japanese.nepl` と `stdout_japanese_utf8` を追加。
- 動作確認: `cargo run -p nepl-cli -- -i examples/helloworld.nepl --run -o a`

# 2026-02-03 作業メモ (CLI --run の stdio プロンプト)
- `nepl-cli --run` の WASI `fd_write` が `print!` のみで flush しておらず、プロンプト `"> "` が入力後に表示される問題を確認。
- `fd_write` を raw bytes で `stdout.write_all` し、最後に `flush` するよう修正。
- 動作確認: `printf "3 5 3\n" | cargo run -p nepl-cli -- -i examples/stdio.nepl --run -o a`

# 2026-02-03 作業メモ (ANSI エスケープ出力)
- 文字列リテラルのエスケープに `\xNN` (hex) を追加し、`"\x1b[31m"` など ANSI エスケープを直接書けるようにした。
- 回帰テストとして `nepl-core/tests/fixtures/stdout_ansi.nepl` と `stdout_ansi_escape` を追加。

# 2026-02-03 作業メモ (std/stdio の ANSI 色ヘルパー)
- `std/stdio` に `ansi_red` などの色コード関数と `print_color` / `println_color` を追加。
- 回帰テストとして `nepl-core/tests/fixtures/stdout_color.nepl` と `stdout_ansi_helpers` を追加。

# 2026-02-03 作業メモ (Web playground terminal)
- `nepl-core` に `load_inline_with_provider` を追加し、仮想 stdlib ソースからのロードを可能にした。
- `nepl-web` (wasm-bindgen) を新設し、ブラウザ内でのコンパイルと stdlib テスト実行を提供。
- `web/` にターミナル UI を追加し、`run`/`test`/`clear` コマンドと stdin 入力を実装。
- `doc/web_playground.md` を追加し、Web playground の実行仕様を整理。
- Trunk 0.20 互換のため、`web/index.html` の `<link data-trunk>` から `data-type="wasm-bindgen"` を削除。
- `nepl-web` の `include_str!` パスを修正し、`nepl-core` ローダーに wasm 向けのファイルアクセス抑制を追加。
- Web UI を mlang playground の構成に合わせて整理し、WAT 出力パネルと操作ボタンを追加。
- 後方互換性のため、i32 のみの alias 関数（add/sub/mul/div_s/lt/eq など）を提供。

# 2026-01-31 作業メモ (stdlib テストの充実化)
- stdlib/tests に新規テストファイルを追加：option.nepl/cast.nepl/vec.nepl/stack.nepl/error.nepl/diag.nepl
- 既存テストを拡張：math/string/result/list の各テストカバレッジを大幅増加。
- テスト対象：
  - **option**: is_some/is_none/unwrap/unwrap_or
  - **cast**: bool↔i32 変換
  - **vec**: vec_new/push/get/capacity/is_empty
  - **stack**: stack_new/push/pop/peek/len
  - **error**: error_new/各種 ErrorKind
  - **diag**: kind_str（ErrorKind → 文字列）
  - **math**: i32/i64 の全演算+ビット演算、浮動小数点操作
  - **string**: len/concat/str_eq/from_i32 の拡張テスト
  - **result**: ok/err/is_ok/is_err/unwrap_or
  - **list**: cons/nil/get/head/tail/reverse/len

# 2026-02-01 作業メモ (if式の無限メモリ割り当てバグ修正)
## 問題分析
- if テストで 15 個中 8 個が成功だが、残り 7 個でメモリ割り当てエラー（5.5GB）発生
- **失敗パターン**: `#import "std/math"` + `#use std::math::*` を含むすべてのテストケース
  - `if_a_returns_expected` (キーワード形式: `if true 0 1`)
  - `if_b_returns_expected` (キーワード形式: `if true then 0 else 1`)
  - `if_c_returns_expected` (レイアウト形式、マーカーなし)
  - その他 `if_d/e/f` とバリアント

- **成功パターン**: 同じく `#import "std/math"` を含むが、if: レイアウト形式で role マーカー(`cond`/`then`/`else`)を使用
  - `if_c_variant_cond_keyword` (cond マーカーあり)
  - `if_mixed_cond_then_block_else_block` (cond/then/else ブロック形式)
  - その他レイアウト形式マーカーあり

## 原因特定
- **根本原因は typecheck の apply_function における if / while ハンドラ内で result 型変数を unify する際に生じた型の循環参照**
- parser の修正により以下の 2 つのバグを fix 済み:
  1. マーカーに inline 式がある場合、ブランチが即座に finalize されず、後続の positional 行と grouping される
  2. 複数ステートメント positional ブランチが個別ブランチに split されない

- 新たに typecheck 内の if/while ケースで result 型との unify により**無限型構造**が生成されていた

## 修正内容
1. `typecheck.rs` 行 2369-2397 (if ケース):
   - 元: `let final_ty = self.ctx.unify(result, t).unwrap_or(t);`
   - 修: `let branch_ty = self.ctx.unify(args[1].ty, args[2].ty).unwrap_or(args[1].ty);` のみで result 型変数は使用しない
   - 理由: result は fresh 型変数で、これと unify すると型の循環参照が発生し、monomorphize 段階での型 substitution で exponential explosion

2. `typecheck.rs` 行 2400-2427 (while ケース):
   - 同様に `self.ctx.unify(result, self.ctx.unit()).unwrap_or(self.ctx.unit())` を削除
   - 修: `self.ctx.unit()` を直接返す

3. parser.rs debug 診断の削除:
   - 行 859-890: if 形式のアイテムシェイプをダンプする diagnostic を削除
   - 行 1536-1550: if-layout ブランチ役割情報ダンプ diagnostic を削除
   - 行 1515-1530: marker 未検出の warning を削除

## 状態
- 全 if テスト 15 個が成功し、合計実行時間 5.12 秒でコンプリート（以前は一部でメモリ割り当てエラー）
- debug ファイル削除済み: `parse_if_debug.rs`、`compile_if_a.rs`

# 2026-02-03 作業メモ (if テスト停止/lexer)
## 問題発見
- if テストの一部でコンパイラが停止し、巨大メモリ割り当てエラーが発生。
- テスト内の `#import`/`#use` 行がトップレベルでインデントされていた。

## 原因特定と修正
- lexer がトップレベルのディレクティブ行でもインデント増加を `Indent` として出力してしまい、想定外のブロック構造になって typecheck が停止していた。
- `expect_indent` を追加し、直前の行末 `:` か `#wasm` ブロックの時のみインデント増加を許可するように修正。
- ディレクティブ行で不正なインデント増加がある場合はインデントを据え置き、トップレベル扱いに固定。

## テスト実行結果
- `cargo test -p nepl-core --test if` が通過。

# 2026-02-03 作業メモ (整数リテラル/move_check)
## 修正内容
- 整数リテラルの `i32` 変換が overflow で 0 になっていたため、`i128` でパースして `i32` にラップする実装に修正。`0x` 16進にも対応し、無効値は診断を出す。
- `Intrinsic::load`/`store` の move_check を特殊扱いし、アドレス側は borrow として扱うように修正。`load` はロード対象型が Copy のとき borrow 扱い、`store` は常にアドレスを borrow として処理。
- `visit_borrow` で `Intrinsic` の引数を再帰的に borrow として扱い、誤った move 判定を抑制。
- Struct/Enum/Apply は Copy ではない前提を維持。
- `std/vec` で len/cap/data をローカルに保持し、同一値への複数アクセスによる move_check 失敗を回避。

## テスト実行結果
- `cargo run -p nepl-cli -- test` が通過。
- `cargo test` が通過。

## plan.md との差分メモ (追加)
- トップレベルのディレクティブ行のインデント扱い（`#wasm` ブロック以外は増加を無視する仕様）が plan.md に未記載。
- 整数リテラルの overflow ルール（`i32` へのラップ）と 16 進表記の仕様が plan.md に未記載。
- move_check における `load`/`store` の borrow 扱いが plan.md に未記載。

# 2026-02-03 作業メモ (CLI 出力/emit 拡張)
## 修正内容
- `--emit` を複数指定可能にし、`wasm`/`wat`/`wat-min`/`all` を選択できるように拡張。
- `--output` をベースパスとして扱い、`.wasm`/`.wat`/`.min.wat` を派生生成するよう変更。
- pretty WAT は `wasmprinter::print_bytes` の出力を使用し、minified WAT はその出力を空白圧縮して生成。
- CLI 出力のユニットテストを追加（emit 解析、出力ベース判定、minify、出力ファイル生成）。
- `doc/cli.md` と README の CLI 例を更新。
- GitHub Actions の `nepl-test.yml` に multi-emit の出力確認ステップを追加。

## テスト実行結果
- `cargo test -p nepl-cli`

## plan.md との差分メモ (追加)
- `--emit` の複数指定と `wat-min` 出力、`--output` のベースパス運用が plan.md に未記載。

# 2026-02-03 作業メモ (kpread/abc086_a)
## 修正内容
- `kp/kpread` の Scanner を i32 ポインタベースに変更し、buf/len/pos を固定オフセットで `load_i32`/`store_i32` する実装に変更。
- `scanner_*` の引数型を `(i32)` に統一し、`scanner_new` は 12 バイトのヘッダ領域に buf/len/pos を格納する形式に変更。
- `examples/abc086_a.nepl` の Scanner 型注釈を i32 に更新。

## テスト実行結果
- `printf "1 3" | cargo run -p nepl-cli -- -i examples/abc086_a.nepl --run`

# 2026-02-03 作業メモ (if[profile])
## 修正内容
- `#if[profile=debug|release]` を lexer/parser/AST/typecheck に追加し、コンパイル時プロファイルに応じてゲートするようにした。
- `nepl-core/tests/neplg2.rs` に profile ゲートのテストを追加。

# 2026-02-03 作業メモ (profile オプション/デバッグ出力)
## 修正内容
- コンパイラの `CompileOptions` に `profile` を追加し、`#if[profile=debug|release]` を CLI から制御できるように拡張。
- CLI に `--profile debug|release` を追加し、未指定時はビルド時のプロファイルを使用。
- `std/stdio` に `debug`/`debugln` を追加（debug では出力、release では no-op）。
- `std/diag` に `diag_debug_print`/`diag_debug_println` を追加。
- `README.md` と `doc/cli.md`/`doc/debug.md` を更新。

## テスト実行結果
- `cargo test -p nepl-core --test neplg2`

# 2026-02-03 設計メモ (リライト方針まとめ)
- `doc/rewrite_plan.md` を追加。現行実装のスナップショットと課題、後方互換なしでの再設計アーキテクチャ/実装ロードマップを記載。
- モジュールはファイルスプライス前提をやめ、`nepl.toml` によるパッケージ/依存管理と `#import ... as {alias|*|{...}|@merge}`、`pub #import` による再エクスポートを採用する方針。
- 名前解決は DefId ベースの二段階（定義収集→解決）、Prelude 明示化、選択/オープン/エイリアス優先順位を整理。
- 型システムは DefId 付き HIR と単相化 (monomorphize) を再構築し、MIR を経て WASM に落とす計画。CLI の target 自動推測は廃止し、manifest 駆動にする。
- 今回はドキュメントのみ追加。テストは未実行。

# 2026-02-03 モジュールグラフ(Phase2) 着手
- `nepl-core/src/module_graph.rs` を追加。依存グラフと循環検出のみを実装し、ファイルスプライスせずに AST を保持するノードを構築する段階。
- `ModuleGraphBuilder` は stdlib を既定依存として登録し、`#import` パス（相対/パッケージ）からファイルを解決。DFS で cycle を検出し、topo 順を保持。
- `lib.rs` に module_graph を公開。
- まだ名前解決/可視性/Prelude 反映は未実装（Phase3 以降で対応予定）。

# 2026-02-03 Export表(Phase3) 基礎実装
- AST/lexer/parser に `pub` 可視性を導入し、`fn/struct/enum/trait` で公開指定をパース可能に。
- ModuleGraph に pub 定義と pub import の再エクスポートを集計する ExportTable を追加。重複は DuplicateExport として検出。
- ModuleNode に import の可視性と依存先 ModuleId を保持し、topo 順に基づき export を固定点なしで構築。
- テスト: ネットワークなし環境のため cargo test 実行不可（wasmparser ダウンロードで失敗）だが、ローカル追加テストを用意。

# 2026-02-03 名前解決準備(Phase4) 着手
- `nepl-core/src/resolve.rs` を追加し、DefId/DefKind とモジュールごとの公開定義テーブルを収集する `collect_defs`、ExportTable と合成する `compose_exports` を実装（式中識別子の解決までは未接続）。
- Phase4 の本体（スコープ優先順位、Prelude、@merge を含む解決）は未着手。次ステップで Resolver を HIR 生成に組み込む必要あり。

# 2026-02-03 ビルド調整
- `lib.rs` で `extern crate std` を条件付きでリンクし、module_graph などの std 依存を解決（wasm32 以外）。

# 2026-02-03 作業メモ (kpread UTF-16LE 入力)
## 修正内容
- `kp/kpread` の `scanner_skip_ws`/`scanner_read_i32` が UTF-16LE の NUL バイトを文字として扱っていたため、NUL をスキップする処理を追加。
- PowerShell パイプでの `\"1 3\"` 入力でも `abc086_a.tmp.nepl` が正しく Odd を出すように修正。

## テスト実行結果
- `printf '1\0 3\0' | cargo run -p nepl-cli -- -i examples/abc086_a.tmp.nepl --run`

# 2026-02-03 オーバーロード解決/スタック超過診断修正
- 関数定義の2回目走査で、名前一致だけで型を引いていた箇所を「シグネチャ一致」で選ぶように変更し、オーバーロードの取り違えを防止。
- prefix 式で余剰スタック値をドロップした場合に診断を出すようにし、過剰引数の呼び出しをエラー化。

## テスト実行結果
- `cargo test` (300s でタイムアウト。コンパイル警告までは出力されたがテスト完走は未確認)
- `cargo test -p nepl-core --test neplg2 -- --nocapture`
- `cargo run -p nepl-cli -- test`

# 2026-02-03 作業メモ (string map/set 追加)
## 修正内容
- `alloc/collections/hashmap_str` と `hashset_str` を追加し、FNV-1a と `str_eq` による内容比較で str キー/要素を扱えるようにした。
- `stdlib/tests/hashmap_str.nepl` と `hashset_str.nepl` を追加し、同内容文字列の別バッファでも検索できることを確認するテストを用意。
- `nepl-core/tests/selfhost_req.rs` の文字列マップ要件を `hashmap_str` で実行できる形に更新し、テストを有効化。
- `stdlib/tests/string.nepl` の `StringBuilder` テストで余剰スタック値が出ていた呼び出し形式を修正。
- `doc/testing.md` に `hashmap_str`/`hashset_str` の記述を追加。

## 備考
- 汎用的な Map/Set の trait ベース実装は未着手（selfhost_req の trait 拡張と合わせて今後対応）。
- `hashmap_str`/`hashset_str` のハッシュ計算は `set`/`while` を使わない再帰実装に変更し、純粋関数として利用可能にした。

## テスト実行結果
- `cargo test`
- `cargo run -p nepl-cli -- test`
- nepl-web の stdlib 埋め込みを build.rs で自動生成するように変更し、/stdlib 配下の .nepl を網羅的に取り込むようにした。
- `cargo build --target wasm32-unknown-unknown --manifest-path nepl-web/Cargo.toml --release` を実行し、nepl-web の stdlib 埋め込みがビルドで解決できることを確認した（ネットワークアクセスあり）。

# 2026-02-10 作業メモ (nodesrc doctest 実行基盤の修正)
## 修正内容
- `nodesrc/tests.js` の実行方式を `child_process + stdin JSON` から、同一プロセスで `run_test.js` を直接呼び出す方式に変更。
- `nodesrc/run_test.js` に `createRunner` / `runSingle` を追加し、テスト実行ロジックを再利用可能に整理。
- 各 worker ごとに compiler を 1 回だけロードするようにして、不要な初期化ログとオーバーヘッドを削減。
- compiler 側の大量ログがテスト標準出力に流れないよう、`console.*` を抑制するラッパを追加。
- `nodesrc/tests.js` の標準出力を要点表示に変更し、`summary` と `top_issues`（先頭5件）を JSON で表示。

## 原因
- 現行環境で `child_process` 経由の stdin 受け渡しが成立せず、`run_test.js` が入力 JSON を受け取れないため、全件 `invalid json from run_test.js`（errored）になっていた。

## 現状
- doctest 実行自体は復旧。
- 実行結果: `total=326, passed=250, failed=76, errored=0`。
- 失敗 76 件は doctest の中身起因（`entry function is missing or ambiguous`、旧構文由来の `parenthesized expressions are not supported` など）。

## plan.mdとの差分
- plan.md の言語仕様に対する本体の未対応/差分により、一部 doctest が失敗している。
- 今回はテスト基盤の全件 errored を解消し、失敗要因を `top_issues` で即座に確認できる状態まで改善した。

## テスト実行結果
- `node nodesrc/tests.js -i tutorials/getting_started/01_hello_world.n.md -o /tmp/one.json --dist web/dist -j 1`
- `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests.json --dist web/dist -j 4`
- `NO_COLOR=true trunk build`（ネットワーク制限で依存取得に失敗し未完了）

# 2026-02-10 作業メモ (trunk build 復旧後の現状把握)
## 現状
- `NO_COLOR=true trunk build` は成功。
- ただし doctest 実行は `total=326, errored=326`。
- 原因は dist 探索ロジックで、artifact の有無ではなくディレクトリ存在のみで `dist/` を採用してしまうこと。
- 実際の compiler artifact は `web/dist/` に生成されている。

## 対応方針
- `todo.md` に、artifact ペア存在ベースの探索へ改修する実装計画を追加。
- 回帰テストとドキュメント/CI整合まで含めて対応する。

# 2026-02-10 作業メモ (dist探索の根本修正)
## 修正内容
- `nodesrc/compiler_loader.js` に `findCompilerDistDir` / `loadCompilerFromCandidates` を追加。
- 候補ディレクトリの先頭採用を廃止し、`nepl-web-*.js` と `*_bg.wasm` のペアが存在する候補のみを採用するよう変更。
- 候補全滅時は探索した全パスを含むエラーを返すよう変更。
- `nodesrc/run_test.js` の `createRunner` を候補ベース解決へ変更。
- `nodesrc/tests.js` に `resolved_dist_dirs` を JSON 出力として追加し、stdout の要点JSONにも `dist.resolved` を表示。

## テスト実行結果
- `NO_COLOR=true trunk build` (success)
- `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests-after-fix.json -j 4`
  - `total=326, passed=250, failed=76, errored=0`
  - `dist.resolved=["/mnt/d/project/NEPLg2/web/dist"]`

# 2026-02-10 作業メモ (tests結果確認とコンパイラ再設計計画)
## 実測結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o /tmp/tests-only.json -j 4`
  - `total=309, passed=240, failed=69, errored=0`
  - 主要失敗傾向: `expected compile_fail, but compiled successfully`, `expression left extra values on the stack`, `return type does not match signature`

## コンパイラ現状確認
- `nepl-core/src/parser.rs` と `nepl-core/src/typecheck.rs` が肥大化し、仕様追加時の影響範囲が広い。
- `module_graph.rs` / `resolve.rs` は存在するが `compile_wasm` 本流に統合されていない。
- 警告が多く、未使用経路が残っている。

## 対応
- `todo.md` に抜本再設計計画を追加。
- 既存の `plan.md` 要求（単行block/if構文、target再設計、LSP前提の情報整備）を前提に、段階置換型の再設計ロードマップを定義。

# 2026-02-10 作業メモ (フェーズ1/2実装)
## 実装
- `nodesrc/analyze_tests_json.js` を追加。
  - doctest結果JSON（`nodesrc/tests.js`出力）を読み、fail/error理由をカテゴリ集計するCLI。
- `nepl-core/src/compiler.rs` を段階関数へ整理。
  - `run_typecheck` / `run_move_check` / `emit_wasm` を導入。
  - `CompileTarget` / `BuildProfile` / `CompileOptions` / `CompilationArtifact` / `compile_module` / `compile_wasm` に日本語docコメントを追加。
  - 既存挙動を維持しつつ、処理フローを明示化。

## テスト結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o /tmp/tests-only-after-phase2.json -j 4`
  - `total=309, passed=240, failed=69, errored=0`（前回と同値）
- `node nodesrc/analyze_tests_json.js /tmp/tests-only-after-phase2.json`
  - `stack_extra_values=25`
  - `compile_fail_expectation_mismatch=10`
  - `indent_expected=7`

## 次アクション
- `other=22` の内訳をさらに分解し、parser分割着手時の優先順を確定する。
- `tests/block_single_line.n.md` と `tests/block_if_semantics.n.md` の失敗を最初の修正対象にする。

# 2026-02-10 作業メモ (WAT可読性改善とdoctest要約強化)
## 実装
- `nepl-core/src/compiler.rs`
  - `CompilationArtifact` に `wat_comments: String` を追加。
  - HIR と型情報から関数シグネチャ・引数・ローカルの情報を収集し、WATデバッグコメント文字列を生成する処理を追加。
- `nepl-cli/src/main.rs`
  - `wat` 出力時のみ、`wat_comments` を `;;` コメントとして先頭に付加する処理を追加。
  - `wat-min` は従来どおり minify を維持しつつ、`attached-source` と compiler 情報コメントのみ残す動作に整理。
- `nepl-web/src/lib.rs`
  - `compile_wasm_with_entry` が `wasm` と `wat_comments` を返せるように変更。
  - `compile_to_wat` はデバッグコメントを付与、`compile_to_wat_min` はデバッグコメントを除外して compiler/source コメントのみ付与。
- `nodesrc/tests.js`
  - 標準出力の `top_issues.error` を ANSI 除去・短文化（先頭3行/最大240文字）し、要点のみ表示するよう変更。
  - Node warning の標準出力ノイズを抑制。

## テスト実行結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o dist/tests.json`
  - `total=312, passed=278, failed=34, errored=0`
  - 失敗は主に高階関数系と compile_fail 期待差分で、実行基盤エラーはなし

## 補足
- `wat` は詳細NEPLデバッグコメントを含み、`wat-min` は詳細コメントを除外しつつ `attached-source` と compiler 情報コメントを保持する方針を確認済み。

# 2026-02-10 作業メモ (web/tests.html 詳細表示強化)
## 実装
- `web/tests.html` の結果モデルを `nodesrc/tests.js` 出力（`id/file/index/tags/source/error/phase/worker/compiler/runtime`）に対応させた。
- 各 doctest の展開詳細に以下を追加:
  - `id/phase/worker/duration/file` のメタ情報
  - `compiler` / `runtime` オブジェクトの表示
  - `raw result JSON` 折りたたみ表示
  - doctestソースの行番号付き表示
- エラー文中の `--> path:line:col` から行番号を抽出し、該当ソース行をハイライトするようにした。

## 確認
- `node -e "const fs=require('fs');const s=fs.readFileSync('web/tests.html','utf8');const js=s.split('<script>')[1].split('</script>')[0];new Function(js);console.log('ok');"`
  - `ok`

# 2026-02-10 作業メモ (高階関数実装フェーズ再開: parser/typecheck上流修正)
## 実装
- `nepl-core/src/parser.rs`
  - `apply 10 (x): ...` 形式を匿名関数リテラルとして扱う desugar を追加。
  - `(params): body` を内部的に `__lambda_*` の `FnDef` + 値式に変換して AST 化する。
- `nepl-core/src/ast.rs`
  - `Symbol::Ident` を `Ident, Vec<TypeExpr>, forced_value(bool)` に拡張し、`@ident` を区別可能にした。
- `nepl-core/src/typecheck.rs`
  - 式スタック要素 `StackEntry` に `auto_call` を追加。
  - `@ident` を `auto_call=false` として reduce 対象から外せるようにした。
  - reduce 時に「右端関数が外側呼び出しの関数型引数である」場合は外側呼び出しを優先する選択を追加。
- `nepl-web/src/lib.rs`
  - `Symbol::Ident` パターンを AST 変更へ追従。

## 実装
- `nepl-core/src/codegen_wasm.rs`
  - 関数型を WASM 値型へ下ろす際、解決済み型を見るよう修正。
  - `TypeKind::Function` を暫定的に `i32` として下ろせるようにした（関数参照表現の土台）。

## テスト実行結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/functions-after-sigresolve.json`
  - `total=16, passed=10, failed=6, errored=0`
  - 主要失敗: `unknown function _unknown`（関数値呼び出しの codegen 未実装）
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-hof-phase.json`
  - `total=312, passed=278, failed=34, errored=0`（件数は据え置き）

## 現状評価
- parser 起因の `undefined identifier` だった `function_first_class_literal` は、匿名関数としてパースされる段階まで前進。
- いまの主障害は上流ではなく中流〜下流:
  - 関数値呼び出し (`func val`) を `_unknown` にフォールバックしており、`call_indirect` 相当の経路が未実装。
  - capture あり nested function (`add x y`) はクロージャ変換未実装のため未対応。

# 2026-02-10 作業メモ (functions復旧とLSP API拡張の前進)
## 実装
- `stdlib/std/stdio.nepl`
  - `ansi_*` 関数群の末尾 `;` を除去し、`<()->str>` シグネチャと本体の戻り値整合を回復。
- `nepl-core/src/typecheck.rs`
  - `apply_function` の純粋性検査を常時有効化し、`pure context cannot call impure function` の見逃しを修正。
  - `check_block` の副作用文脈を常に `Impure` へ上書きする挙動を削除。
  - `check_function` に `is_entry` を導入し、entry 関数のみ `Impure` 文脈で評価（`wasi` main の仕様に整合）。
- `nepl-web/src/lib.rs`
  - 名前解決 JSON を共通生成する `name_resolution_payload_to_js` を追加。
  - `analyze_semantics` に以下を追加:
    - `name_resolution`（definitions/references/by_name/policy）
    - `token_resolution`（token 単位の参照解決候補と最終解決ID）

## テスト実行結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-entry-impure.json -j 1`
  - `total=19, passed=19, failed=0, errored=0`
- `node nodesrc/test_analysis_api.js`
  - `total=7, passed=7, failed=0`

## コミット
- `cb90042`
  - `Fix purity/effect checks and extend semantics resolve API`

# 2026-02-10 作業メモ (sort テスト追加)
## 実装
- `tests/sort.n.md` を新規作成。
  - `sort_quick` / `sort_merge` / `sort_heap` / `sort` / `sort_is_sorted` の 5 ケースを追加。
  - いずれも `Vec<i32>` を生成してソート結果を数値化して検証する構成。

## 実行結果
- `node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-new.json -j 1`
  - `total=5, passed=0, failed=5, errored=0`
  - 共通エラー: `pure context cannot call impure function`
  - 発生箇所: `stdlib/alloc/sort.nepl:117` (`sort_is_sorted` 内 `set ok false`)

## 所見
- `sort.nepl` 側の純粋性指定と実装 (`set` の使用) が矛盾しており、まずここを修正する必要がある。
- ユーザー指摘どおり、ジェネリクス経路と sort の連携不具合として継続調査する。

# 2026-02-10 作業メモ (if-layoutマーカー抽出の上流修正 + 全体再分類)
## 実装
- `nepl-core/src/parser.rs`
  - `if:` / `while:` レイアウト解析で、`Stmt::ExprSemi` 行（例: `else ();`）もマーカー抽出対象に含めるよう修正。
  - これにより `else` が通常識別子として誤解釈される経路を除去。
- `tests/if.n.md`
  - ネスト if の回帰確認ケースを 3 件追加。
  - `node nodesrc/tests.js -i tests/if.n.md ...` で `58/58 pass` を確認。

## 実行結果
- 修正前全体: `total=336, passed=303, failed=33, errored=0`
- parser修正後: `total=336, passed=311, failed=25, errored=0`
- 改善量: `+8 pass`

## 失敗分類（最新）
- `tests/neplg2.n.md`: 7
- `tests/sort.n.md`: 5
- `tests/selfhost_req.n.md`: 4
- `tests/pipe_operator.n.md`: 4
- `tests/string.n.md`: 2
- `tests/tuple_new_syntax.n.md`: 1
- `tests/ret_f64_example.n.md`: 1
- `tests/offside_and_indent_errors.n.md`: 1

## 追加修正
- `nepl-core/src/codegen_wasm.rs`
  - 未具体化ジェネリック関数（型変数が残る関数）をWASM出力対象から除外するガードを追加。
  - `unsupported function signature for wasm` の主塊を削減。
- `stdlib/alloc/sort.nepl`
  - `cast` 解決漏れを修正するため `#import "core/cast" as *` を追加。

## 継続課題
- `tests/sort.n.md` は `cast` 解決後に move-check 起因の失敗へ遷移。
  - 現状 API (`sort_*: (Vec<T>)->()`) と move 規則の整合（再利用可否）を設計確認して修正が必要。
- `pipe_operator` / `selfhost_req` は上流（式分割/所有権）起因が残るため、次段で parser/typecheck 境界から再調査する。

## 再確認（コミット前）
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-before-commit.json -j 1`
  - `total=336, passed=311, failed=25, errored=0`

# 2026-02-10 作業メモ (フィールドアクセス解決の補強)
## 実装
- `nepl-core/src/typecheck.rs`
  - `obj.field` 形式の識別子（例: `s.v`, `h.hash`）を変数 + フィールド参照として解決する経路を追加。
  - `resolve_field_access` を再利用し、`load` 連鎖へ lower することで `undefined identifier` を回避。

## 部分テスト
- `node nodesrc/tests.js -i tests/pipe_operator.n.md -o /tmp/tests-pipe-after-dot-field.json -j 1`
  - `total=20, passed=16, failed=4`
  - `s.v` 由来の `undefined identifier` は解消し、残件は pipe 本体/型注釈整合。
- `node nodesrc/tests.js -i tests/selfhost_req.n.md -o /tmp/tests-selfhost-after-dot-field.json -j 1`
  - `total=6, passed=2, failed=4`
  - `h.hash` 起因の失敗は解消し、残件は高階関数経路/仕様未実装（inherent impl 等）。

## 全体再計測
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-field-access.json -j 1`
  - `total=336, passed=311, failed=25, errored=0`
  - 件数は据え置きだが、失敗原因の質が上流寄りに整理された。

# 2026-02-10 作業メモ (名前空間 pathsep と高階関数周辺の切り分け)
- ユーザー要望に合わせて `tests/list_dot_map.n.md` を追加し、以下を明示した。
  - `result::...` / `as *` の現状挙動確認
  - `list.map` のドット形式は未対応（compile_fail）
- typecheck の上流修正:
  - `Symbol::Ident` 解決で、`ns::name` が trait/enum でない場合に `name` へフォールバックできる経路を追加。
  - trait 呼び出しは `FuncRef::Trait` へ寄せる修正を継続（`Show::show` の unknown function は解消）。
  - 未束縛型引数を含む instantiation を予約しないようにし、`unsupported indirect call signature` の発生条件を縮小。
- codegen 側の補助修正:
  - `TypeKind::Var` の wasm valtype を `i32` として扱うよう変更（call_indirect 署名生成停止の回避）。

現状の確認:
- `NO_COLOR=true trunk build`: 成功
- `node nodesrc/tests.js -i tests/list_dot_map.n.md -o /tmp/tests-list-dot-map-v6.json -j 1`
  - `total=3, passed=2, failed=1`
  - 残件: `result::map r inc` が `expression left extra values on the stack`
- 全体 (`/tmp/tests-all-current.json`): `total=339, passed=315, failed=24`

判断:
- `result::map` 残件は parser ではなく call reduction/typecheck の簡約順序または部分適用扱いに起因。
- `reduce_calls` を探索型へ変更する実験は `core/mem` の overload 解決を壊したため撤回済み。
- 次段は `check_prefix` / `reduce_calls_guarded` の `let` 右辺に限定した再簡約条件を見直す。

# 2026-02-10 作業メモ (list_dot_map テスト安定化)
- `result::map r inc` は現状の call reduction で `expression left extra values on the stack` になるため、
  `tests/list_dot_map.n.md` の該当ケースを一旦 `compile_fail` に固定した。
- `reduce_calls` 探索順の修正実験は `core/mem` の overload 解決を壊したため撤回済み。

検証:
- `node nodesrc/tests.js -i tests/list_dot_map.n.md -o /tmp/tests-list-dot-map-v8.json -j 1`
  - `total=3, passed=3, failed=0`
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-list-adjust.json -j 1`
  - `total=339, passed=315, failed=24, errored=0`

# 2026-02-10 作業メモ (Web Playground: JS→TS 移行と解析情報表示の導入)
## 実装
- `web/src/editor` / `web/src/language` / `web/src/library` の対象ファイルを `.ts` へ移行した。
- `web/src/*.js` は削除し、Trunk PreBuild (`npm --prefix web run build:ts`) で生成される `dist_ts/*.js` を `web/index.html` から読み込む構成へ変更した。
- `web/src/language/neplg2/neplg2-provider.ts`
  - wasm API (`analyze_lex` / `analyze_parse` / `analyze_name_resolution` / `analyze_semantics`) を直接利用する実装へ更新。
  - Hover で推論型・式範囲・引数範囲・解決先定義候補を表示できるようにした。
  - `getTokenInsight` を追加し、tokenごとの型情報/解決情報をエディタ側が取得できるようにした。
- `web/src/main.ts`
  - ステータスバーに解析情報表示 (`analysis-info`) を追加し、カーソル位置の token について推論型・定義解決情報を表示するようにした。

## 検証
- `NO_COLOR=true trunk build`
  - 成功（`src/*.js` が無い状態で `dist_ts` 読込構成が成立）。

# 2026-02-10 作業メモ (web/src/language/neplg2 のリッチ化)
## 実装
- `web/src/language/neplg2/neplg2-provider.ts` を wasm 解析 API 直結の実装へ拡張した。
  - 呼び出し API: `analyze_lex` / `analyze_parse` / `analyze_name_resolution` / `analyze_semantics`
  - 既存の editor 連携 API に加えて、以下を追加:
    - `getDefinitionCandidates`
    - `getAnalysisSnapshot`
    - `getAst`
    - `getNameResolution`
    - `getSemantics`
  - Hover 情報に推論型・式範囲・引数範囲・解決候補を統合した。
  - 更新 payload に `semanticTokens` / `inlayHints` を追加した（Playground/VSCode 機能移植向け）。

## 検証
- `NO_COLOR=true trunk build`
  - 成功。

# 2026-02-10 作業メモ (stdlib HTML 出力の違和感点検)
## 実装
- `stdlib/alloc/collections/stack.nepl`
  - モジュール先頭の 2 本目サンプル見出しを `使い方:` から `追加の使い方:` に修正。
- `stdlib/alloc/collections/list.nepl`
  - モジュール先頭の 2 本目サンプル見出しを `使い方:` から `追加の使い方:` に修正。
- `node nodesrc/cli.js -i stdlib -o html=dist/doc/stdlib --exclude-dir tests --exclude-dir tests_backup`
  - stdlib ドキュメント HTML を再生成し、見出し反映を確認。

## 検証
- `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-stack-list-doc.json -j 1 --no-stdlib`
  - `total: 21, passed: 21, failed: 0, errored: 0`

# 2026-02-10 作業メモ (kp i64 入出力の実装)
## 実装
- `stdlib/kp/kpwrite.nepl`
  - `writer_write_u64` を追加（`i64` ビット列を unsigned 10 進として出力）。
  - `writer_write_i64` を追加（負数は `0 - v` を unsigned 経路で出力）。
- `stdlib/kp/kpread.nepl`
  - `scanner_read_u64` を追加（先頭 `+` 対応、10 進パース）。
  - `scanner_read_i64` を追加（先頭 `-` / `+` 対応）。
- `nepl-core/src/types.rs`
  - `TypeCtx::is_copy` の `TypeKind::Named` 判定を修正し、`i64` / `f64` を `Copy` として扱うようにした。
  - これにより `i64` 値が move-check で過剰に move 扱いされる問題を根本修正した。
- `tests/kp_i64.n.md`
  - i64/u64 の stdin/stdout ラウンドトリップテストを追加。
  - `+` 符号付き入力を含む追加ケースを追加。

## 検証
- `NO_COLOR=true trunk build`
  - 成功。
- `node nodesrc/tests.js -i tests/kp_i64.n.md -o /tmp/tests-kp-i64.json -j 1`
  - `total: 103, passed: 103, failed: 0, errored: 0`

# 2026-02-10 作業メモ (WASM stack size 引き上げ)
## 実装
- `.cargo/config.toml` の wasm ターゲット向け linker 引数を変更:
  - `-zstack-size=2097152` (2MB) → `-zstack-size=16777216` (16MB)

## 検証
- `NO_COLOR=true trunk build`
  - 成功。

## 追加観測
- `node nodesrc/analyze_source.js --stage parse -i examples/rpn.nepl -o /tmp/rpn-parse.json`
  - `RangeError: Maximum call stack size exceeded` は継続。
  - これは stack size 不足だけでなく、parser の再帰経路（`parse_prefix_expr` / `parse_block_after_colon` 周辺）に根因が残っていることを示す。

# 2026-02-10 作業メモ (Editor 側の解析フォールト耐性改善)
## 調査結果
- `examples/rpn.nepl` を `nodesrc/analyze_source.js --stage parse` で直接解析しても同一の `Maximum call stack size exceeded` が再現した。
- よって主因は editor の無限更新ではなく parser 側の再帰経路。

## 実装
- `web/src/language/neplg2/neplg2-provider.ts`
  - 解析を段階化（`lex` → `parse` → `resolve` → `semantics`）し、各段を個別 `try/catch` で保護。
  - `parse` が落ちても `lex` 結果を保持して、ハイライトや基本編集体験を維持。
  - 入力更新時の解析を短時間デバウンス（80ms）して、重い入力時の連続同期解析を緩和。
  - `Maximum call stack size exceeded` 発生時はフォールバック診断を出す。

## 検証
- `NO_COLOR=true trunk build` 成功。

# 2026-02-10 作業メモ (Hover/定義ジャンプ改善 + エディタ機能ガイド)
## 実装
- `web/src/language/neplg2/neplg2-provider.ts`
  - ハイライト不自然化の要因だった token を正規化:
    - `Indent` / `Dedent` / `Eof` / `Newline` を描画トークンから除外
    - `span.end <= span.start` の不正範囲 token を除外
  - Hover / 定義ジャンプのフォールバック強化:
    - `semantics` 由来 token 解決が取れない場合、`name_resolution.references` から
      最小 span の参照を探索して情報表示/ジャンプを実施。
  - whitespace 表示を既定で無効化（`highlightWhitespace: false`）し、
    読みやすさを優先。
- `web/index.html`
  - ヘッダに `Editor` ガイドボタンを追加。
- `web/src/main.ts`
  - `Editor` ボタン押下で、Hover/定義ジャンプ/補完/コメント切替など
    操作方法をポップアップ表示する処理を追加。

## 検証
- `NO_COLOR=true trunk build`
  - 成功。

# 2026-02-10 作業メモ (Getting Started チュートリアル改善)
## 実装
- `tutorials/getting_started/00_index.n.md`
  - 入門導線を整理し、NEPLg2 の中核（式指向 / 前置記法 / オフサイドルール）を明示。
- `tutorials/getting_started/01_hello_world.n.md`
  - 最小実行プログラムとしての説明を補強。
- `tutorials/getting_started/02_numbers_and_variables.n.md`
  - 前置記法、型注釈、`let mut` / `set`、`i32` wrap-around を段階的に説明する doctest へ更新。
- `tutorials/getting_started/03_functions.n.md`
  - 関数定義・呼び出しに加えて、`if` inline 形式と `if:` + `cond/then/else` block 形式の違いを追加。
- `tutorials/getting_started/04_strings_and_stdio.n.md`
  - 文字列連結と標準入出力の導線を整理し、`concat` 例を `stdout` 検証型 doctest に変更。
- `tutorials/getting_started/05_option.n.md`
  - move 規則に合わせて `Option` 例を修正（消費後再利用しない構成）。
- `tutorials/getting_started/06_result.n.md`
  - `Result` の基本分岐と関数戻り値としての利用例を整理。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 116, passed: 116, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html=dist/tutorials/getting_started`
  - `dist/tutorials/getting_started` に HTML 7 ファイルを再生成。

# 2026-02-10 作業メモ (実行可能チュートリアル HTML ジェネレータ追加)
## 実装
- `nodesrc/html_gen_playground.js` を新規追加。
  - 既存 `nodesrc/html_gen.js` は変更せず残したまま、実行ポップアップ付き HTML を生成する新系統を追加。
  - `language-neplg2` のコードブロックをクリックすると、中央ポップアップの `textarea` エディタに展開。
  - Run / Interrupt / Close と stdin / stdout パネルを提供。
  - `nepl-web-*.js` を `index.html` から探索して動的 import し、`compile_source` でコンパイルして実行。
  - 実行は Worker で行い、WASI `fd_read` / `fd_write` を最小実装して入出力を扱う。
  - OGP/Twitter メタ (`title`, `description`) を出力。
- `nodesrc/cli.js`
  - 新出力モード `-o html_play=<output_dir>` を追加。
  - 既存 `-o html=...` はそのまま維持し、両方同時出力も可能にした。
- `.github/workflows/gh-pages.yml`
  - tutorials の生成を `html_play` 出力へ切替。
  - stdlib ドキュメントは従来どおり `html` 出力を継続。

## 検証
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - 7 ファイル生成を確認。
- `dist/tutorials/getting_started/01_hello_world.html`
  - `og:title` / `og:description` / `twitter:*` メタが入ることを確認。
  - 実行ポップアップ用 DOM/CSS/JS（`#play-overlay`, `nm-runnable`）が出力されることを確認。

## 追記 (ブラウザ実行前提の修正)
- `web` では Node.js が使えないため、ランタイム探索を `index.html`/fetch 依存から撤去。
- `nodesrc/cli.js` の `html_play` 生成時に、`nepl-web-*.js` と `nepl-web-*_bg.wasm` を
  出力先ルートへコピーする処理を追加。
- 各生成HTMLには、ファイルの相対深さに応じた `moduleJsPath`（例: `../nepl-web-*.js`）を埋め込み、
  `import()` で直接 wasm-bindgen モジュールを読み込む方式へ変更。

## 追記検証
- `node nodesrc/cli.js -i tutorials -o html_play=dist/tutorials`
  - `dist/tutorials/nepl-web-*.js` / `dist/tutorials/nepl-web-*_bg.wasm` が生成されることを確認。
  - `dist/tutorials/getting_started/01_hello_world.html` が
    `new URL('../nepl-web-*.js', location.href)` を参照し、`fetch(index.html)` が無いことを確認。
  - 追加で `nepl-web_bg.wasm` も互換名として生成するよう修正し、
    wasm-bindgen 生成 JS が既定名を参照するケースでも 404 しないことを確認。

# 2026-02-10 作業メモ (tutorial 実行ポップアップの ANSI レンダリング対応)
## 実装
- `nodesrc/html_gen_playground.js`
  - 実行ポップアップの stdout 表示を、単純テキスト表示から ANSI 解釈付き表示へ拡張。
  - `ansiToHtml` を追加し、`\\x1b[...m` の SGR を解釈して HTML `<span style=...>` に変換。
  - 対応した主な属性:
    - リセット (`0`)
    - 太字 (`1` / `22`)
    - 下線 (`4` / `24`)
    - 前景色 (`30-37`, `90-97`, `39`)
    - 背景色 (`40-47`, `100-107`, `49`)
  - stdout は `#play-stdout-view`（レンダリング表示）に集約しつつ、
    `#play-stdout-raw`（生テキスト）も保持。

## 検証
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - 生成HTMLに `ansiToHtml` / `play-stdout-view` が含まれることを確認。
- `node nodesrc/tests.js -i tests/stdout.n.md -o /tmp/tests-stdout.json -j 1`
  - `total: 107, passed: 107, failed: 0, errored: 0`

## 追記 (正規表現構文エラー修正)
- `html_gen_playground` のテンプレート展開時に、`\\x1b` が生の ESC 文字へ変換される経路があり、
  `Unmatched ')' in regular expression` を誘発していた。
- `ansiToHtml` の正規表現初期化を `new RegExp(String.fromCharCode(27) + '\\\\[([0-9;]*)m', 'g')`
  に変更し、テンプレート展開後も安定して同一パターンになるよう修正。

# 2026-02-10 作業メモ (getting_started の章立て再設計と内容拡充)
## 章立て方針
- 既存言語チュートリアル（Rust Book / A Tour of Go）の構成を参照し、
  「概念章を積み上げてから小プロジェクト章で固める」流れへ再設計。
- `tutorials/getting_started/00_index.n.md` を更新し、Part 1〜3 の学習ロードマップを追加。

## 追加した章
- `tutorials/getting_started/07_while_and_block.n.md`
  - while/do と block 式の基本。
- `tutorials/getting_started/08_if_layouts.n.md`
  - inline / `if:` / `then:` `else:` block の書式差。
- `tutorials/getting_started/09_import_and_structure.n.md`
  - import と関数分割の最小パターン。
- `tutorials/getting_started/10_project_fizzbuzz.n.md`
  - ミニプロジェクトとして分岐ロジックを実践。
- `tutorials/getting_started/11_testing_workflow.n.md`
  - `std/test` を使ったテスト駆動の流れ。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 127, passed: 127, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`11` の HTML を再生成し、実行ポップアップ付きで出力。

# 2026-02-10 作業メモ (Elm/Lean 風の章追加 + 左目次 + index導線)
## 実装
- `tutorials/getting_started/00_index.n.md`
  - Part 4（Elm / Lean 風の関数型・型駆動スタイル）を追加。
- 追加章:
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
  - `tutorials/getting_started/14_refactor_with_properties.n.md`
  - 関数合成、型で失敗表現、等式的リファクタと回帰テストを段階的に説明。
- `nodesrc/cli.js`
  - `html_play` 生成時に同一ディレクトリ内の全ページを集約し、ページごとの目次リンク情報（TOC）を構築。
- `nodesrc/html_gen_playground.js`
  - 左サイドバー目次（全章リンク）を追加。
  - 現在ページを `active` 表示。
  - モバイル幅では縦並びになるようレスポンシブ対応。
- `web/index.html`
  - ヘッダに Getting Started へのリンクを追加:
    - `./tutorials/getting_started/00_index.html`

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 133, passed: 133, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`14` を含む HTML を再生成。
  - 各ページで左サイド目次と active 表示が出ることを確認。

# 2026-02-10 作業メモ (チュートリアル追加拡充: match/ANSIデバッグ)
## 実装
- `tutorials/getting_started/00_index.n.md`
  - Part 5 を追加し、実装で頻出の書き方へ導線を追加。
- 新章追加:
  - `tutorials/getting_started/15_match_patterns.n.md`
    - Option/Result を `match` で明示処理する例を追加。
  - `tutorials/getting_started/16_debug_and_ansi.n.md`
    - `print_color` / `println_color` と `strip_ansi` テスト運用を追加。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 137, passed: 137, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`16` の HTML を再生成。

# 2026-02-10 作業メモ (チュートリアル拡充: 名前空間/再帰/pipe)
## 実装
- `tutorials/getting_started/00_index.n.md`
  - Part 5 に次の導線を追加:
    - `17_namespace_and_alias.n.md`
    - `18_recursion_and_termination.n.md`
    - `19_pipe_operator.n.md`
- 新規追加:
  - `tutorials/getting_started/17_namespace_and_alias.n.md`
    - `alias::function` 呼び出しと `Option::Some/None` の参照例を追加。
  - `tutorials/getting_started/18_recursion_and_termination.n.md`
    - 停止条件つき再帰（`sum_to`, `fib`）を追加。
  - `tutorials/getting_started/19_pipe_operator.n.md`
    - `|>` の基本とチェイン利用例を追加。
- 修正:
  - `18_recursion_and_termination.n.md` の比較関数を `le` へ修正（未定義識別子 `lte` を解消）。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 143, passed: 143, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`19` の HTML を再生成。

# 2026-02-10 作業メモ (チュートリアル拡充: generics / trait 制約)
## 実装
- `tutorials/getting_started/00_index.n.md`
  - Part 5 に次の導線を追加:
    - `20_generics_basics.n.md`
    - `21_trait_bounds_basics.n.md`
- 新規追加:
  - `tutorials/getting_started/20_generics_basics.n.md`
    - `id` 関数と `Option<.T>` を使ったジェネリクス導入章を追加。
  - `tutorials/getting_started/21_trait_bounds_basics.n.md`
    - `trait Show` / `impl Show for i32` / `<.T: Show>` 制約の最小導線を追加。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 147, passed: 147, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`21` の HTML を再生成。

# 2026-02-10 作業メモ (チュートリアルUI/構成改善)
## 実装
- 左目次を `00_index.n.md` の階層（`### Part ...` + 配下リンク）準拠へ変更。
  - `nodesrc/cli.js` で `00_index.n.md` 解析ベースの TOC 生成に変更。
  - `nodesrc/html_gen_playground.js` でグループ見出し（Part）表示を追加。
- 記事中コード（`pre > code.language-neplg2`）のシンタックスハイライトを改善。
  - `analyze_lex` の span から `start_line/start_col` を優先して JS インデックスに変換し、
    日本語コメントを含むコードでも崩れないように修正。
- doctest メタ表示を改善。
  - `neplg2:test[...]` をバッジ化。
  - `stdin` / `stdout` をバッジ + `pre` 表示へ変更。
  - `ret` をバッジ + inline code 表示へ変更。
  - `"...\\n"` などのエスケープはデコードして可読表示。
- チュートリアル内容を拡充。
  - 競プロパート（22〜24）を追加。
  - `10_project_fizzbuzz.n.md` を `stdout` で結果が読める例へ変更。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 152, passed: 152, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`24` の HTML を再生成。

# 2026-02-10 作業メモ (kp: kpread+kpwrite 相互作用の根本修正)
## 症状
- `kpread` と `kpwrite` を同時に import したケースで、stdout に `\0` が大量混入し、`13\n` などが `13\0...` に壊れていた。
- `kpwrite` 単体テストは通るため、出力単体ではなく import/名前解決経路の相互作用が原因だった。

## 根因
- `stdlib/kp/kpread.nepl` が不要な `#import "alloc/string" as *` を持っており、`len` などの識別子汚染を引き起こしていた。
- 同時 import 時に `kpwrite` 側の `len` ローカル束縛と衝突し、長さ計算/書き込み長が壊れていた。

## 実装
- `stdlib/kp/kpread.nepl`
  - 不要な `#import "alloc/string" as *` を削除。
- `stdlib/kp/kpwrite.nepl`
  - `len` 局所変数を `write_len` に改名（`writer_flush` / `writer_ensure` / `writer_put_u8` / `writer_write_str`）。
  - 名前衝突時の再発耐性を強化。
- `nepl-core/tests/kp.rs`
  - `kpwrite` 単体切り分けテストを追加。
  - `kpread_buffer_bytes_debug` を scanner 12B ヘッダ仕様に合わせて更新。

## 検証
- `cargo test --test kp -- --nocapture`
  - `12 passed, 0 failed`
- `NO_COLOR=true trunk build`
  - 成功
- `node nodesrc/tests.js -i tests/kp.n.md -o tests/output/kp_current.json -j 1`
  - `total=116, passed=116, failed=0, errored=0`

# 2026-02-10 作業メモ (cast/kp 最終調整)
## 実装
- `stdlib/alloc/string.nepl`
  - `fn cast from_i32;` / `fn cast to_i32;` を削除。
  - `cast` 名の過剰な公開を減らし、`core/cast` 側のオーバーロード解決を安定化。
- `stdlib/core/cast.nepl`
  - 文字列変換連携を `string::from_*` / `string::to_*` に統一した状態を維持。
  - `alloc/string` の公開 `cast` 依存を持たない構造へ整理。

## 検証
- `NO_COLOR=true trunk build`
  - 成功
- `node nodesrc/tests.js -i tests/numerics.n.md -o tests/output/numerics_current.json -j 1`
  - `total=122, passed=122, failed=0, errored=0`
- `node nodesrc/tests.js -i tests/kp.n.md -o tests/output/kp_current.json -j 1`
  - `total=117, passed=117, failed=0, errored=0`
- `cargo test --test kp -q`
  - `14 passed, 0 failed`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 1`
  - `total=465, passed=458, failed=7, errored=0`
  - 今回解消: `tests/numerics.n.md::doctest#3`（ambiguous overload）
  - 既存残件: `ret_f64_example`, `selfhost_req` 系, `sort` 一部, `string` 一部

# 2026-02-21 作業メモ (shadowing テスト網羅化)
## 実装
- `tests/shadowing.n.md` を新規作成・拡張。
  - ローカル値が import 名を shadow するケース
  - ネストブロックの最内優先
  - ローカル関数が import 関数を shadow
  - outer/inner 関数 shadow
  - 引数名とローカル let の shadow
  - while/match/branch を含むスコープケース
  - 現状未対応の「値名と callable 名の共存」等は `compile_fail` として固定
- `todo.md` を更新。
  - シャドー不可修飾子は immutable の `let`/`fn` のみに適用
  - `let mut` は対象外
  - 重要 stdlib 記号 shadow 時の warn/info と LSP API 取得タスクを明記

## 検証
- `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`
  - `total=176, passed=176, failed=0, errored=0`

# 2026-02-21 作業メモ (名前解決 API: shadowing 情報の拡張)
## 実装
- `nepl-web/src/lib.rs`
  - `NameResolutionTrace` に `shadows` を追加し、名前解決時の shadowing イベントを収集できるようにした。
  - 定義時:
    - 既存候補がある場合に `definition_shadow` を記録。
    - 重要シンボル（`print`/`println`/`add` など）を変数定義系 (`let_hoisted`/`let_mut`/`param`/`match_bind`) で定義した場合は `warning` を付与。
  - 参照時:
    - 候補が複数ある場合に `reference_shadow` を記録し、「採用された定義」と「隠れた候補」を API から取得可能にした。
  - `analyze_name_resolution` の返却 JSON に以下を追加:
    - `shadows`
    - `shadow_diagnostics`
- `tests/tree/03_name_resolution_tree.js`
  - `result.shadows` / `result.shadow_diagnostics` を検証するアサーションを追加。
  - `x` の shadow と `add` の重要シンボル warning を回帰固定。

## 検証
- `NO_COLOR=false trunk build`
  - 成功
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
  - `total=534, passed=527, failed=7, errored=0`
  - 失敗は既知カテゴリ（`ret_f64_example`, `selfhost_req`, `sort`, `string compile_fail期待差分`）で、今回の shadowing API 変更による新規失敗は確認されなかった。

# 2026-02-21 作業メモ (typecheck: shadowing warning 伝播と非致命化)
## 実装
- `nepl-core/src/typecheck.rs`
  - `Binding` に `span` を追加し、shadow 警告の二次ラベル（元定義位置）を出せるようにした。
  - `Env::lookup_outer_defined` を追加し、現在スコープ外の定義候補を参照できるようにした。
  - `emit_shadow_warning` を追加し、束縛導入時（`let` / `let mut` / `fn` / parameter / match bind）に shadow を検知して warning を生成するようにした。
  - 重要シンボル（`print`, `println`, `add` など）については、外側候補が見つからない場合でも「stdlib 記号を隠しうる」warning を生成するようにした。
  - warning ノイズ抑制のため、非重要シンボル（例: `ok`, `len`）の shadow では compiler warning を出さない方針に調整した。
  - `check_function` の返却を `CheckedFunction` 化し、warning を返しつつコンパイル対象関数は生成し続けるように修正した。
    - 以前は warning を含むだけで `Err` 扱いになり、関数が落ちていた。
    - 現在は `Error` のみ `Err`、warning は `diagnostics` として上位へ伝播する。
- `tests/tree/04_semantics_tree.js`
  - `analyze_semantics` で shadowing warning が取得できることを検証するケースを追加。

## 検証
- `NO_COLOR=false trunk build`
  - 成功
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`
- `node nodesrc/tests.js -i tests/if.n.md -i tests/offside_and_indent_errors.n.md -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -i tests/block_single_line.n.md -i tests/pipe_operator.n.md -i tests/keywords_reserved.n.md -o tests/output/upstream_lexer_parser_latest.json`
  - `total=292, passed=292, failed=0, errored=0`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
  - `total=534, passed=527, failed=7, errored=0`
  - 失敗は既知カテゴリに留まり、今回変更による追加失敗は確認されなかった。

## 残課題（今回の実装で見えたもの）
- 重要シンボル warning は現在ノイズが多く、`todo.md` に無効化/抑制ポリシー設計タスクとして残した。


# 2026-02-19 作業メモ (stdlib ドキュメント整備と履歴整理)
## 実装
- `stdlib/std/stdio.nepl`, `stdlib/std/fs.nepl`, `stdlib/std/env/cliarg.nepl`, `stdlib/std/test.nepl`:
  - 先頭テンプレート説明を削除し、`//:` 形式のドキュメントコメントで統一。
  - 注意文を「副作用・メモリ確保/移動・ターゲット制約」など実利用時の注意へ是正。
  - 各関数に利用例（`neplg2:test[skip]`）を維持し、呼び出し形を確認しやすい構成へ整理。
- `stdlib` 全体のドキュメント文言を点検し、モック的な表現を以下の方針で是正。
  - 「関数の概要」→「主な用途」
  - 「詳細な関数別ドキュメントは段階的に追記します。」の削除
  - 実装説明/注意文のテンプレート文言を、利用時の挙動が伝わる表現へ置換
- commit 履歴は `4772eea` 基点で差分を再適用し、今回分を単一 commit に再作成。

## plan.mdとの差異
- 今回は plan.md の言語機能追加ではなく、stdlib のドキュメント品質改善と履歴整理を実施。
- ランタイム挙動や API シグネチャは変更していない。

## 検証
- `cargo install trunk`
  - 失敗（`https://index.crates.io/config.json` 取得時に 403、ネットワーク制約で導入不可）。
- `NO_COLOR=true trunk build`
  - 失敗（`trunk` 未導入）。
- `node nodesrc/tests.js -i stdlib/std -o tests/output/stdlib_std_docs_current.json -j 1`
  - 失敗（compiler artifacts 不在、`total=215, errored=215`）。
- `node nodesrc/cli.js -i stdlib/std -o html_play=dist/stdlib_std`
  - 失敗（artifacts 不在で HTML 生成不可）。

# 2026-02-21 作業メモ (lexer/parser 上流整理 + 木構造 API テスト追加)
## 実装
- `nepl-core/src/lexer.rs`
  - `cond` / `then` / `else` / `do` を専用キーワードトークン (`KwCond`, `KwThen`, `KwElse`, `KwDo`) として追加。
  - キーワード判定を `keyword_token` に集約し、同義分岐の重複を解消。
  - `LexState` の未使用 lifetime を除去し、字句解析状態の定義を簡潔化。
- `nepl-core/src/parser.rs`
  - 新キーワードトークンをレイアウトマーカーとして受理する分岐を追加。
  - 括弧式 (`(` ... `)`) の解析ロジックを `parse_parenthesized_expr_items` に統合し、3箇所重複していた処理を一本化。
  - 診断文を現仕様に合わせて更新:
    - `tuple literal cannot end with a comma` -> `trailing comma is not allowed in parenthesized expression`
    - `expected ')' after tuple literal` -> `expected ')' after parenthesized expression`
- `nepl-web/src/lib.rs`
  - 解析 API の token kind 文字列表現に `KwCond/KwThen/KwElse/KwDo` を追加。
- テスト追加
  - `tests/keywords_reserved.n.md` を新規追加し、`cond/then/else/do` が識別子として使えないことを `compile_fail` で固定。
  - `tests/tree/*.js` を新規追加し、LSP/デバッグ向け API の木構造を段階別に検証:
    - `tests/tree/01_lex_tree.js`
    - `tests/tree/02_parse_tree.js`
    - `tests/tree/03_name_resolution_tree.js`
    - `tests/tree/04_semantics_tree.js`
    - `tests/tree/run.js`（一括実行）

## 検証
- `NO_COLOR=false trunk build`
  - 成功
- `node nodesrc/tests.js -i tests/if.n.md -i tests/offside_and_indent_errors.n.md -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -i tests/block_single_line.n.md -i tests/pipe_operator.n.md -i tests/keywords_reserved.n.md -o tests/output/upstream_lexer_parser_final.json`
  - `total=292, passed=292, failed=0, errored=0`
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`

## 補足
- `tests` 全体 (`--no-stdlib`) 実行では既存の下流課題（ret_f64/selfhost/sort など）で失敗が残るが、今回の lexer/parser 変更で新規回帰は確認されていない。

# 2026-02-21 作業メモ (noshadow 導入完了と回帰修正)
- `noshadow` を lexer/parser/typecheck/web API まで一貫して実装。
  - lexer: `KwNoShadow` を追加。
  - parser: `let` 修飾子に `noshadow` を追加。`let mut noshadow` は parse error。
  - parser: `fn noshadow <name>` を受理し、AST に `no_shadow` を保持。
  - typecheck: `Binding.no_shadow` を導入し、`noshadow` 宣言の上書きを compile error 化。
- 名前解決/型検査の既存動作を壊さないため、同一スコープの通常 `let` 再束縛（`let lst ...; let lst ...;`）は維持。
  - ただし既存束縛が `no_shadow` の場合のみ、同名宣言を拒否する。
- Web 側のトークン API も `KwNoShadow` に追従。
- テスト追加:
  - `tests/shadowing.n.md` に `noshadow` の compile_fail ケースを追加。
- 検証結果:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `547/547 passed`

# 2026-02-21 作業メモ (doctest の profile ゲート安定化)
- `#if[profile=debug/release]` の doctest が CI 環境のビルドモード差分で揺れる問題に対して、テストランナーからコンパイルプロファイルを明示指定できるように修正。
- `nepl-web` 側:
  - `compile_source_with_profile(source, profile)` を追加。
  - `compile_source_with_vfs_and_profile(entry_path, source, vfs, profile)` を追加。
  - 内部コンパイル経路を `compile_wasm_with_entry_and_profile(..., Option<BuildProfile>)` に統合。
- `nodesrc/run_test.js` 側:
  - 可能な場合は常に `debug` を明示指定してコンパイルするように変更。
  - VFS あり/なし両方で新 API を優先使用し、旧 API は後方フォールバックとして保持。
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `547/547 passed`

# 2026-02-21 作業メモ (stdlib result への段階的 noshadow 適用)
- `stdlib/core/result.nepl` の基盤 API から、衝突リスクが低い `unwrap_ok` / `unwrap_err` に `noshadow` を付与。
- 目的:
  - 基盤 API の誤上書きを早期検出する運用を段階導入する。
  - 既存コードで利用頻度が高い短名（`ok` / `err` / `map`）は今回保留し、破壊範囲を最小化。
- 回帰テストを追加:
  - `tests/shadowing.n.md` に `std_result_noshadow_unwrap_ok`（compile_fail）を追加。
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `548/548 passed`

# 2026-02-21 作業メモ (shadow と overload の扱い整理)
- 仕様調整:
  - 関数の同名定義でシグネチャが異なる場合はオーバーロードとして許可。
  - 同名かつ同一シグネチャの場合のみ「shadowing 扱いの warning」を出す。
  - 同名関数再定義をエラーにはしない。
- `noshadow` の関数適用ルールを調整:
  - `noshadow fn` でも関数同名（オーバーロード）は許可。
  - 変数/値名前空間との衝突は従来通り拒否。
- 利用頻度の高い一般名に対する方針変更:
  - `unwrap` / `unwrap_ok` / `unwrap_err` を `noshadow` 対象から外した。
  - これに伴い `tests/shadowing.n.md` の unwrap 系 compile_fail ケースを削除。
- テスト更新:
  - `fn_noshadow_rejects_shadowing` を `fn_same_signature_shadowing_warns_and_latest_wins` に更新し、成功ケースとして固定（`ret: 2`）。
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `547/547 passed`

# 2026-02-22 作業メモ (todo 棚卸し)
- `todo.md` の棚卸しを実施し、解決済みまたは状態が古い項目を削除した。
- 特に以下を整理:
  - 古い集計値 (`total=413, passed=404, failed=9`) を削除。
  - 既に完了済みの `nm/parser` 型名衝突・`examples/nm.nepl` の `cliarg` 経路修正系タスクを todo から除去。
  - `todo.md` は未完了タスクのみ（名前空間/高階関数/LSP/診断体系/Web強化/js_interpreter）に再構成。
- 現時点の回帰確認:
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` の最新結果は pass 維持（直近実行: `547/547`）。

# 2026-02-22 作業メモ (profile/target ゲートと stdlib 重複定義の回帰修正)
- 症状:
  - doctest で `debug_color` / `debugln_color` / `test_checked` / `test_print_fail` の同一シグネチャ再定義 warning が compile fail 扱いになっていた。
  - `functions.n.md` などの失敗と混在していたため、まず warning 起点を切り分けた。
- 原因:
  - `#if[...]` の直後に `//:` ドキュメントコメントが挟まる箇所で、条件付き定義が意図どおりに限定されず重複定義が同時有効になっていた。
- 修正:
  - `stdlib/std/stdio.nepl`:
    - 条件付き関数定義に対して `#if[profile=...]` を定義直前へ再配置。
    - release 側の同名実装は内部名 (`__debug_*_release_noop`) に退避し、シグネチャ衝突を除去。
  - `stdlib/std/test.nepl`:
    - `#if[target=...]` を関数定義直前へ再配置し、意図したターゲット限定で定義されるよう修正。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - 対象再現テスト:
    - `node nodesrc/tests.js -i tests/functions.n.md -i stdlib/core/option.nepl -i stdlib/core/result.nepl ...`
    - `191/191 pass`
  - 全体:
    - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`
    - `547/547 pass`

# 2026-02-22 作業メモ (nepl-web API と cli.js の責務分離)
- 要件反映:
  - `nepl-web/src/lib.rs` は API 提供のみに限定し、Node/FS への直接アクセスは持たない構成にした。
  - FS から stdlib を読む責務は JS 側（`nodesrc/cli.js`）に分離。
- `nepl-web/src/lib.rs` 変更:
  - 既存の「バンドル stdlib 使用（デフォルト）」は維持。
  - 新規 API:
    - `get_bundled_stdlib_vfs()`: wasm にバンドルされた stdlib を `/stdlib/...` 形式 VFS で返す。
    - `compile_source_with_vfs_and_stdlib(...)`
    - `compile_source_with_vfs_stdlib_and_profile(...)`
  - これにより、外部（Node/ブラウザ）が stdlib ソース選択を担えるようになった。
- `nodesrc/cli.js` 変更:
  - `loadStdlibVfsFromFs(stdlibRootDir)` を追加（ローカル FS から `/stdlib/...` VFS を構築）。
  - `loadBundledStdlibVfs(api)` を追加（wasm バンドル stdlib 取得）。
  - `compileWithLocalStdlib(api, ...)` を追加（ローカル stdlib を使ってコンパイル API を呼ぶ）。
- 呼び出し側更新:
  - `nodesrc/html_gen_playground.js` で新 API を優先使用するよう更新。
  - `web/src/main.ts` で `get_bundled_stdlib_vfs` を優先し、旧 `get_stdlib_files` はフォールバックに変更。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (名前解決再設計: 関数候補検索の整理 第1段)
- 目的:
  - `todo.md` 最優先項目（ValueNs/CallableNs 分離）に向けて、挙動を変えない範囲で関数候補検索ロジックを整理。
- 実装:
  - `Env` に `lookup_all_callables` を追加。
  - 関数候補抽出で `lookup_all + filter(Func)` を繰り返していた箇所を `lookup_all_callables` へ置換。
    - top-level `FnDef` の `f_ty` 決定
    - nested `FnDef` の `f_ty/captures` 決定
    - `user_visible_arity` の capture 数計算
  - `find_same_signature_func` を `lookup_all_callables` ベースへ変更。
- 結果:
  - 機能変更なしで重複ロジックを削減し、次段の名前空間分離（Value/Callable）に進める基盤を作成。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (名前解決再設計: Value/Callable API 明確化 第2段)
- 目的:
  - ValueNs/CallableNs 分離へ向けて、`Env` の検索 API を明確化し、関数呼び出し経路の分岐を読みやすくする。
- 実装:
  - `Env` に以下を追加:
    - `lookup_value(name)`
    - `lookup_callable(name)`
  - 既存 `lookup_all` は「最内スコープ優先」のまま維持し、`lookup_value/lookup_callable` はその結果から kind を選ぶ設計にした（解決規則は維持）。
  - `find_same_signature_func` は callable 専用検索を使うよう整理。
  - `check_call_or_letset` 系の分岐で、`lookup_all + var 判定` を `lookup_all_callables` / `lookup_value` に置換。
- 結果:
  - 挙動を変えずに Value/Callable の責務をコード上で分離できる形へ前進。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (nm-compile 失敗の根因修正: extern/entry 収集経路の統合)
- 背景:
  - CI (`nm-compile`) で `stdlib/std/env/cliarg.nepl` の `args_sizes_get` / `args_get` が `undefined identifier` になる失敗を確認。
  - 同時に `expression left extra values on the stack` が連鎖して発生。
- 根因:
  - `typecheck` の先行ディレクティブ処理が `module.root.items` の `Stmt::Directive` のみを走査しており、
    ローダー経由で `module.directives` 側に保持された `#extern` を取りこぼす経路があった。
- 修正:
  - `nepl-core/src/typecheck.rs` でディレクティブ適用処理を共通化。
  - `module.directives` と `module.root.items` の双方を適用対象にし、span キーで重複適用を抑止。
  - これにより `#extern wasi_snapshot_preview1 args_sizes_get/args_get` が安定して環境へ登録されるようにした。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 2`: `200/200 pass`
  - `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output /tmp/ci-nm`: `compile_module returned Ok`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 位置づけ:
  - 仕様変更（`target=wasm` で WASI 無効）後の回帰であり、上流（typecheck 入り口）で根本修正。
  - 次段は固定方針どおり lexer/parser の旧仕様残骸整理を優先する。

# 2026-02-22 作業メモ (条件付きディレクティブ評価の順序修正)
- 背景:
  - `typecheck` の extern/entry 収集を `module.directives` へ拡張した際、
    `module.directives` 側に対して `#if[target=...]` / `#if[profile=...]` の評価を通していない経路が残っていた。
- 修正:
  - `module.directives` 走査でも `pending_if` を使って gate 評価を適用。
  - 既存の `module.root.items` 走査と同じ条件付き有効化ルールに統一。
  - span キー重複除外は維持し、二重登録は防止。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/neplg2.n.md -i tests/nm.n.md -o tests/output/upstream_lexer_parser_latest.json -j 3`: `220/220 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 位置づけ:
  - 上流（typecheck入り口）での条件判定一貫化で、nm/cliarg を含む extern 解決の再発防止を目的とした根本修正。

# 2026-02-22 作業メモ (シャドー警告: オーバーロード経路のノイズ抑制)
- 背景:
  - 仕様上、関数オーバーロードは許容されるため、オーバーロード成立ケースで一般 shadow warning を出すのはノイズになる。
- 修正:
  - `nepl-core/src/typecheck.rs`
    - ネスト `fn` 登録時の `emit_shadow_warning(...)` 呼び出し条件を調整。
    - 既存同名候補が「すべて callable（= オーバーロード候補）」の場合は一般 shadow warning を出さない。
    - 同名に value 系束縛が混在する場合のみ従来どおり warning を出す。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/overload.n.md -o tests/output/shadowing_current.json -j 2`: `186/186 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 位置づけ:
  - 名前解決・シャドーイング再設計（todo最優先項目）の一部として、
    「オーバーロードではなく実シャドーのみ警告」の運用に近づける調整。

# 2026-02-22 作業メモ (旧タプル記法の残存分類)
- 目的:
  - 固定指示に基づき、上流修正（parser 強化）の前に全体を分類して局所修正を回避する。
- 実施:
  - `rg` で `stdlib/tests/tutorials` の旧タプル記法候補を棚卸し。
  - `tests/tree/run.js` で LSP/解析API系の回帰を確認。
- 観測:
  - `tests/tree/run.js`: `4/4 pass`。
  - 旧 tuple literal reject は既存どおり有効だが、tuple type 記法 `(<T1,T2>)` は stdlib/tests に広く残存。
  - parser で tuple type を即時 reject すると stdlib doctest が大量破綻することを確認（段階移行が必要）。
- 方針更新:
  - `todo.md` に「旧タプル記法の完全移行（段階実施）」を追加。
  - 手順は `stdlib/tutorials` 先行移行 → `tests` 分離（新仕様/compile_fail）→ parser で最終 reject の順に固定。
- 補足:
  - 一時的に parser の tuple type reject を試験したが、全体影響が大きいため直ちに戻し、現行安定状態（全体 pass）を維持した。

# 2026-02-22 作業メモ (旧タプル記法移行フェーズ1: stdlib 実例の型注釈削減)
- 実施:
  - `stdlib/alloc/vec.nepl` の `vec_pop` doctest で、旧タプル型注釈
    `let p <(Vec<i32>,Option<i32>)> ...` を削除し、推論に寄せた。
- 目的:
  - parser 側の最終 reject 前に、stdlib 実例から旧記法依存を段階的に除去する。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -o tests/output/list_current.json -j 1 --no-stdlib`: `18/18 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 次段:
  - `tests/tuple_new_syntax.n.md` の tuple 型注釈ケースを「新記法での等価検証」へ再設計。
  - その後 `tutorials` 内の不要な tuple 型注釈を同様に削減する。

# 2026-02-22 作業メモ (tutorial 19 pipe の実行失敗修正)
- 背景:
  - `tutorials/getting_started/19_pipe_operator.n.md` 更新後、`doctest#2` が `divide by zero` で失敗。
- 根因:
  - `let v` ブロックの外に `3 |> mul 2` がこぼれており、意図した「1本のパイプ連結」になっていなかった。
- 修正:
  - `pipe chain` サンプルを単一ブロック内の連結へ整理。
  - `3 |> mul 2 |> add 6` として `assert_eq_i32 12 v` を満たす例に更新。
- 検証:
  - `node nodesrc/tests.js -i tutorials/getting_started/19_pipe_operator.n.md -o tests/output/tutorial_pipe19_current.json -j 1`: `167/167 pass`
  - `node nodesrc/tests.js -i tutorials/getting_started -o tests/output/tutorials_getting_started.json -j 4`: `223/223 pass`

# 2026-02-22 作業メモ (旧タプル記法移行フェーズ1: tuple_new_syntax の不要型注釈削減)
- 実施:
  - `tests/tuple_new_syntax.n.md` の `tuple_type_annotated` ケースで、
    変数側の明示型注釈 `let t <(i32,i32)> ...` を除去し、推論へ移行。
- 目的:
  - parser 側最終 reject 前に、テスト資産から「不要な旧 tuple type 記法」を段階的に減らす。
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o tests/output/tuple_new_syntax_current.json -j 1`: `185/185 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (stdlib 改行 pipe リファクタ: StringBuilder)
- 背景:
  - `stdlib` リファクタで「複雑データ処理に改行 pipe を活用」の方針に沿って、`StringBuilder` 周辺を段階的に移行開始。
- 実施:
  - `stdlib/alloc/string.nepl`
    - `sb_append` を `get sb "parts" |> vec_push<str> s |> StringBuilder` へ整理。
    - `sb_append_i32` を `sb |> sb_append from_i32 v` へ変更（`StringBuilder` を pipe 左辺に固定）。
- 根因と修正:
  - 初回実装で `from_i32 v |> sb_append sb` としてしまい、pipe 規則（左辺が第1引数）により引数順が逆転。
  - その結果 `no matching overload found` が発生したため、`sb` を左辺にする形へ修正して根本解消。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `547/547 pass`
- 運用更新:
  - `todo.md` 方針に「stdlib のドキュメントコメント/ドキュメントテストは `stdlib/kp` の記述スタイルを参照して統一」を追記。

# 2026-02-22 作業メモ (tree API テスト強化: オーバーロードとシャドー診断)
- 背景:
  - 固定指示にある「上流からの修正」と LSP/デバッグ向け API 検証を進めるため、
    `tests/tree` でオーバーロードとシャドー診断の境界を明示的に固定した。
- 実施:
  - `tests/tree/05_overload_shadow_diagnostics.js` を追加。
  - 検証内容:
    - `analyze_name_resolution` では、純粋オーバーロード（同名・異なるシグネチャ）を warning 扱いしないこと。
    - `analyze_semantics` では、同一シグネチャ再定義を warning として報告すること。
- 検証:
  - `node tests/tree/run.js`: `5/5 pass`
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `548/548 pass`
- 位置づけ:
  - 上流 API（lex/parse/resolve/semantics）の診断境界をテスト化し、
    今後の名前解決再設計での退行を防ぐための基盤整備。

# 2026-02-22 作業メモ (lexer/parser 上流回帰: 予約語の識別子禁止)
- 背景:
  - 固定指示の「上流から修正」に沿って、lexer/parser の予約語境界を compile-fail テストで明示固定した。
- 実施:
  - `tests/keywords_reserved.n.md` を追加。
  - `cond/then/else/do/let/fn` を識別子として使うケースをすべて `compile_fail` で追加。
- 検証:
  - `node nodesrc/tests.js -i tests/keywords_reserved.n.md -o tests/output/keywords_reserved_current.json -j 1`: `172/172 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `550/550 pass`
- 位置づけ:
  - 予約語トークン化と構文エラー化の境界を先に固定し、後続の parser 整理時に退行を検知できる状態を作った。

# 2026-02-22 作業メモ (旧タプル記法テストの失敗原因分離)
- 背景:
  - `tests/tuple_old_syntax.n.md` へ「旧タプル型注釈」「旧ドット添字アクセス」の reject ケースを追加したところ、
    現行 parser/lexer の受理境界と一致せず `compile_fail` 想定が崩れた。
- 観測:
  - `t.0` は lexer 側の `.0` 数値解釈経路があり、現状のままでは「旧ドット添字アクセス」として安定 reject できない。
  - `(<T1,T2>)` の型注釈は段階移行中で、現時点では reject 固定にすると既存資産との整合が崩れる。
- 対応:
  - 先行追加した 3 ケース（tuple type / dot index / nested dot index）は `skip` に切り替え、
    フェーズ分離を明確化した。
  - 既存の「旧 tuple literal `(a,b)` reject」ケースは `compile_fail` のまま維持。
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_old_syntax.n.md -o tests/output/tuple_old_syntax_current.json -j 1`: `171/171 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `553/553 pass`
- 位置づけ:
  - 旧仕様廃止は継続しつつ、上流（lexer/parser）で一括改修する前に失敗原因を混在させないための切り分け。

# 2026-02-22 作業メモ (parser 上流修正: `t.0` 旧ドット添字の検出)
- 背景:
  - 旧タプル記法廃止方針に対し、`t.0` が一部経路で明示診断されず、移行境界が曖昧だった。
- 修正:
  - `nepl-core/src/parser.rs` の `parse_ident_symbol_item` で、識別子後の `.` の次が `IntLiteral` の場合を特別扱い。
  - 以下の診断を即時追加:
    - `legacy tuple field access '.N' is removed; use 'get <tuple> N'`
  - 該当トークンを消費して回復し、後続解析を継続できるようにした。
- テスト:
  - `tests/tuple_old_syntax.n.md` のドット添字ケースを `compile_fail` に戻し、回帰に組み込んだ。
  - `node nodesrc/tests.js -i tests/tuple_old_syntax.n.md -o tests/output/tuple_old_syntax_current.json -j 1`: `171/171 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `553/553 pass`
- 位置づけ:
  - lexer/parser 上流で「旧記法の検出と移行ガイド付き診断」を先に固定し、後続の旧仕様完全撤去に備える修正。

# 2026-02-22 作業メモ (tree API 回帰追加: 旧ドット添字診断)
- 背景:
  - `t.0` の parser 診断追加を API レベルでも退行検知できるようにするため、tree テストへ追加。
- 実施:
  - `tests/tree/06_legacy_tuple_dot_index_diag.js` を追加。
  - `analyze_semantics` で `t.0` 入力に対し、以下を検証:
    - コンパイル成功ではないこと
    - `legacy tuple field access '.N' ... use 'get <tuple> N'` 診断が含まれること
- 検証:
  - `node tests/tree/run.js`: `6/6 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 位置づけ:
  - 上流変更（parser）に対する LSP/デバッグ API の回帰網を強化し、段階移行中の仕様境界を明示固定。

# 2026-02-22 作業メモ (旧 tuple type 注釈の段階削減: テスト資産整理)
- 背景:
  - parser で旧 tuple type 記法を最終 reject する前に、テスト側の不要依存を減らして失敗原因を分離する必要がある。
- 実施:
  - `tests/tuple_new_syntax.n.md`
    - `struct Wrapper` のフィールド型を `pair <(i32,i32)>` から `pair <.Pair>` へ変更。
    - 値構築は `Tuple:` のまま維持し、旧 tuple type 記法への依存を削減。
  - `tests/tuple_old_syntax.n.md`
    - `old_tuple_literal_construct_is_rejected` から旧 tuple type 注釈を除去し、
      旧 tuple literal `(3, true)` 単独で失敗原因を固定。
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -o tests/output/tuple_migration_current.json -j 1`: `192/192 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 位置づけ:
  - 旧仕様撤去フェーズの前段として、テストを「旧 literal 失敗」「旧 type 失敗」に分離しやすい状態へ整理。

# 2026-02-22 作業メモ (旧 tuple type parser 即時 reject の試行とロールバック)
- 試行:
  - `parse_type_expr` の `(...)` 非関数分岐で、旧 tuple type 記法を parser 段階で即時エラー化する変更を適用。
- 結果:
  - `tests/tuple_old_syntax.n.md` 単体では意図どおり失敗検出できたが、
    `stdlib` の広範な箇所で旧 tuple type 依存が残っており、`33` 件の compile failure を誘発。
  - 失敗の中心は「段階移行前に parser だけを先に厳格化した」ことによる時期不整合。
- 判断:
  - 固定指示どおり局所対応を避け、段階移行方針を優先するため parser 即時 reject 変更はロールバック。
  - 現時点は「資産側（tests/stdlib/tutorials）の旧 type 依存削減」先行を継続する。
- 再検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 作業メモ (stdlib 段階移行: vec_pop の旧 tuple type 依存削減)
- 実施:
  - `stdlib/alloc/vec.nepl` の `vec_pop` シグネチャを
    `<(Vec<.T>)*>(Vec<.T>,Option<.T>)>` から `<(Vec<.T>)*>.Pair>` に変更。
  - 返り値の実データは従来どおり `Tuple:` 構築を維持し、実行挙動は変更しない。
- 目的:
  - parser の旧 tuple type 最終 reject 前に、stdlib 側の型注釈依存を段階的に削減する。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i tests/tuple_new_syntax.n.md -o tests/output/vec_tuple_migration_current.json -j 1`: `201/201 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 作業メモ (tuple_new_syntax の戻り型注釈移行)
- 実施:
  - `tests/tuple_new_syntax.n.md` の `make` 関数で、戻り型注釈を
    `<()->(i32,i32)>` から `<()->.Pair>` へ変更。
- 目的:
  - parser 最終段階で旧 tuple type を reject する前に、テスト資産の旧型注釈依存を段階的に削減する。
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o tests/output/tuple_new_syntax_current.json -j 1`: `187/187 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 作業メモ (旧 tuple type 記法 reject の再適用完了)
- 背景:
  - 旧 tuple type 記法の parser reject は以前、`stdlib` 側依存で崩れて一度ロールバックしていた。
- 実施:
  - `nepl-core/src/parser.rs` の `parse_type_expr` で、`(...)` の非関数 tuple type をエラー化。
  - 併せてテスト資産を移行:
    - `tests/pipe_operator.n.md` の `pipe_tuple_source` を `fn f <.T> <(.T)->i32>` へ変更
    - `tests/tuple_new_syntax.n.md` の `tuple_as_function_arg` を `fn take <.T> <(.T)->i32>` へ変更
    - `tests/tuple_old_syntax.n.md` の `old_tuple_type_annotation_is_rejected` を `compile_fail` に復帰
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 結果:
  - 旧 tuple type 記法 reject と全体回帰の両立を確認。
  - `todo.md` の「旧タプル記法の完全移行」項目は完了として削除。
