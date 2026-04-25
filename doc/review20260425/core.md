# Core レビュー

作成日: 2026-04-25

対象: `nepl-core/src/**`

## レビュー範囲

確認した主要ファイルは次の通りです。

| 区分 | ファイル |
|---|---|
| パイプライン | `lib.rs`, `compiler.rs`, `loader.rs`, `module_graph.rs`, `target_precheck.rs` |
| frontend | `lexer.rs`, `parser.rs`, `ast.rs`, `nm.rs` |
| 型・名前解決 | `types.rs`, `typecheck.rs`, `resolve.rs`, `name_resolve.rs`, `effects.rs` |
| HIR / pass | `hir.rs`, `passes/move_check.rs`, `passes/drop_insertion.rs`, `passes/codegen_precheck.rs` |
| codegen | `codegen_wasm.rs`, `codegen_llvm.rs`, `wasm_shared.rs`, `runtime_helpers.rs`, `monomorphize.rs` |
| 診断 | `diagnostic.rs`, `diagnostic_ids.rs`, `error.rs`, `span.rs`, `log.rs` |

行数上の最大リスクは `typecheck.rs` 8759 行、`parser.rs` 3891 行、`codegen_llvm.rs` 3528 行、`codegen_wasm.rs` 2339 行です。設計書 `doc/2.1impl/compiler_structure.md` が指摘している巨大ファイル問題は、現行コードにもそのまま残っています。

## 総評

NEPLg2.0 の core は、動く経路を増やすために型検査・名前解決・HIR 生成・trait 解決・effect 判定・一部の codegen 前処理が密結合しています。特に `typecheck.rs` の hot path は、全走査、`Vec::remove`、`TypeCtx` clone、`BTreeMap` 線形探索を多用しており、「コンパイラが異常に遅い」原因候補として優先的に分解・測定すべきです。

また、`core` は no_std 境界、診断、import semantics、Resource IR のいずれも計画とずれており、局所パッチではなく pipeline stage ごとの分割が必要です。

## RV-CORE-001: core の no_std 境界が崩れている

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: architecture
- 対象: `nepl-core/src/lib.rs`, `nepl-core/src/compiler.rs`, `nepl-core/src/typecheck.rs`, `nepl-core/src/codegen_wasm.rs`

### 根拠

- `nepl-core/src/lib.rs:1`: crate root は `#![no_std]`。
- `nepl-core/src/compiler.rs:2`: `extern crate std;` が unconditional。
- `nepl-core/src/typecheck.rs:3`: `extern crate std;` が unconditional。
- `nepl-core/src/typecheck.rs:9`: `std::path::Path` を core 側で直接使用。
- `nepl-core/src/codegen_wasm.rs:4`: wasm backend も `extern crate std;` を持つ。

### 問題

`plan.md` と `doc/self_host.md` は compiler core を WASI なしの純粋 core として扱う方針ですが、現行 core は `std` に直接依存しています。`nepl-core` を wasm32/no_std 対象へ切り出す前提が崩れ、CLI と core の責務分離も不明確です。

### 影響

core をブラウザ / WASM / self-host bootstrap で再利用する際に、host I/O や path 処理が混入します。将来の `stdlib/neplg2/core` へ移植するときも、どこまでが pure compiler core なのか判別しにくくなります。

### 修正方針

`SourceMap` の path 表示、debug output、host filesystem access を core API の外に出します。core は `alloc` までに限定し、CLI / web / test harness が path と I/O adapter を渡す構成に分けます。

### 検証

`nepl-core` を `wasm32-unknown-unknown` または同等の no_std 条件でビルドする CI job を追加します。

## RV-CORE-002: typecheck.rs が巨大化しすぎて責務が分離できていない

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: architecture
- 対象: `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/typecheck.rs`: 8759 行。
- `nepl-core/src/typecheck.rs:566`: instantiation cache を型検査本体が直接持つ。
- `nepl-core/src/typecheck.rs:7777`: `Env` / `Scope` 実装も同じファイル内。
- `nepl-core/src/typecheck.rs:8354`: `LabelEnv` / `StringTable` など補助構造も同居。

### 問題

型推論、trait capability、overload、名前探索、HIR lowering、effect 判定、import alias 補助、diagnostic recovery が 1 ファイルに集中しています。局所修正が別機能へ波及しやすく、性能改善も profiling point を切りにくい状態です。

### 影響

既知バグの根本原因を追いにくく、修正のたびに回帰範囲が広がります。`doc/2.1impl/compiler_structure.md` で示されている `check/`, `ty/`, `resolve/`, `hir/` 分割方針との乖離も大きいです。

### 修正方針

短期的には `Env`、overload 解決、prefix reduction、trait impl 検査を別 module へ切り出します。中期的には AST flat list から HIR への lowering を型検査結果と分離し、`resolve` / `ty` / `check` の依存方向を固定します。

### 検証

分割ごとに既存 `tests/compiler/**` と stdlib doctest を走らせ、ファイル移動だけの段階では出力 JSON の差分がないことを確認します。

## RV-CORE-003: reduce_calls が O(n^2) 化しやすく固定上限で正当な入力を落とす

- 解決済: true
- 状態: verified
- 優先度: P0
- 種別: performance
- 対象: `nepl-core/src/typecheck.rs`, `nepl-core/tests/call_reduction.rs`, `tests/compiler/tree/19_call_reduction_large_prefix.js`

### 根拠

- 旧実装の `reduce_calls` は `max_iterations = 1000` の固定上限で停止していた。
- 旧実装の `reduce_calls` / `reduce_calls_guarded` は各 iteration で stack を後ろから全走査していた。
- 旧実装は引数取り出しに `stack.remove(func_pos + 1)`、callee 取り出しに `stack.remove(func_pos)` を使っていた。
- guarded reduction 側にも同じ固定上限と全走査 / middle remove が重複していた。

### 問題

prefix expression の縮約が「全走査して middle remove」を繰り返すため、長い式や overload が多い式で O(n^2) 以上になりやすいです。さらに iteration 上限が 1000 固定なので、入力が正しくても `TypeCallReductionLimitExceeded` になる可能性があります。

対応中に、縮約上限を外すだけでは深い HIR の後段 traversal / codegen で native stack overflow が残ることも確認しました。この残件は `RV-CORE-015` として分離し、ここでは `reduce_calls` と typecheck/semantics 段階の根本原因を修正対象にしています。

### 影響

コンパイル時間が式長と候補数に対して急激に増えます。stdlib の巨大な関数や tutorial の複雑な式で体感速度が悪化し、CI timeout の原因にもなります。

### 修正方針

`reduce_calls` / `reduce_calls_guarded` を共通の縮約ループへ統合し、`open_calls` を末尾候補スタックとして使うようにしました。通常の prefix chain では毎 iteration の全 stack 走査を行わず、callee と引数は連続範囲の `drain` で取り出すため、`Vec::remove` の繰り返しを避けます。

固定の 1000 回上限は削除し、0-arity などで stack 長が縮まらない場合だけ状態キーで進捗なしを検出する方式にしました。また、縮約済みの深い HIR を毎回 clone していた通常 call の引数生成と `check_prefix` 戻り値生成を move ベースに変更し、長い chain の typecheck が再帰 clone で stack overflow しないようにしました。型 ID 解決の HIR 式走査も明示スタックに置き換えています。

### 検証

確認済み:

- `cargo check -p nepl-core`
- `cargo test -p nepl-core --test call_reduction`
- `trunk build`
- `node tests/compiler/tree/run.js` (`total=19`, `passed=19`, `failed=0`)
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`caseCount=13`, `passedCount=13`, `failedCount=0`)

追加した fixture:

- `nepl-core/tests/call_reduction.rs`: 1105 個の prefix call chain が typecheck できることを確認。
- `tests/compiler/tree/19_call_reduction_large_prefix.js`: wasm API の semantics 経由で 1105-call chain が `ok=true` になることを JSON 出力で確認。

未解決の後段問題:

- `cargo run -p nepl-cli -- -i tmp/rv-core-003-large.nepl --check --target core` は、typecheck 後の compile pipeline で native stack overflow になる。この codegen / `--check` 側の深い HIR traversal は `RV-CORE-015` で追跡します。

## RV-CORE-004: overload 解決が候補ごとに TypeCtx 全体を clone している

- 解決済: true
- 状態: verified
- 優先度: P0
- 種別: performance
- 対象: `nepl-core/src/types.rs`, `nepl-core/src/typecheck.rs`, `nepl-core/src/codegen_llvm.rs`, `nepl-core/src/codegen_wasm.rs`, `nepl-core/tests/typectx_checkpoint.rs`

### 根拠

- `nepl-core/src/typecheck.rs:6497`: overload candidate ごとに `let mut tmp_ctx = self.ctx.clone();`。
- `nepl-core/src/typecheck.rs:7234`: 関数値解決でも `TypeCtx` clone。
- `nepl-core/src/codegen_llvm.rs:2976` 付近: layout 計算で `types.clone()` を使う。
- `nepl-core/src/codegen_wasm.rs:75` 以降: generic Apply の storage 計算で `ctx.clone()` を使う。

### 問題

`TypeCtx` は arena と型変数状態を持つため、clone は候補数と型数に比例して重くなります。overload が stdlib 全体に広がるほど、1 call の型解決で大量の一時 arena clone が発生します。

### 影響

「コンパイラが異常に遅い」問題の主要候補です。特に `add`, `eq`, `len`, `get`, `push` のような多重定義名が頻出する stdlib / tutorial で悪化します。

### 修正方針

候補検査用の checkpoint/rollback を `TypeCtx` に実装し、候補ごとの arena 全体 clone を除去しました。型変数束縛と named type table の変更は undo log で戻し、copy/drop trait target と copy trait flag は checkpoint の長さ・値へ復元します。これにより、候補検査中の一時的な `instantiate` / `unify` / `substitute` が外側の型文脈へ漏れません。

overload candidate 走査と関数値引数の候補照合は、`TypeCtx::clone()` ではなく checkpoint 上の一時変更として評価し、各候補の終了時に rollback します。codegen の generic `Apply` layout 計算は、`TypeCtx` を clone して `substitute` する方式をやめ、type parameter mapping を引き回して storage size / align / field offset を計算する形に変更しました。

### 検証

確認済み:

- `cargo check -p nepl-core`
- `cargo test -p nepl-core --test typectx_checkpoint` (`2 passed`)
- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-rv-core-004.json -j 1` (`total=45`, `passed=45`, `failed=0`)
- `trunk build`
- `node tests/compiler/tree/run.js` (`total=19`, `passed=19`, `failed=0`)
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`caseCount=13`, `passedCount=13`, `failedCount=0`)

追加した fixture:

- `nepl-core/tests/typectx_checkpoint.rs`: checkpoint が型変数束縛、一時 arena entry、named type table、copy/drop trait target、copy trait flag を復元することを確認。

補足:

- `cargo test -p nepl-core --test overload` は、HEAD から分けた baseline worktree でも同じ 3 件が失敗する既存状態でした。RV-CORE-004 の回帰判定には、現行の `tests/compiler/overload.n.md` doctest を使用しています。

## RV-CORE-005: loader が import clause を無視して全 import をフラット結合している

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `nepl-core/src/loader.rs`, `nepl-core/src/parser.rs`, `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/parser.rs:458`: `ImportClause` は `DefaultAlias`, `Alias`, `Open`, `Selective`, `Merge` まで parse している。
- `nepl-core/src/loader.rs:436`: `Directive::Import { path, .. }` として clause を捨てている。
- `nepl-core/src/loader.rs:457`: imported module の root items を clause に関係なく親へ push。
- `nepl-core/src/typecheck.rs:8310`: qualified alias map は後段で別途作っているが、loader の item 可視性は制御していない。

### 問題

`#import "x" as name` や `as { a }` の構文があっても、loader は対象 module の item をすべて親 module に混ぜます。alias / selective import は型検査の一部補助にしか効かず、未選択 symbol が見えてしまう可能性があります。

### 影響

名前衝突、意図しない overload 候補増加、import 順依存の挙動が起きます。性能面でも、使っていない module の item まで常に型検査候補になります。

### 修正方針

loader は AST を物理的に結合せず、module graph と export/import table を構築するだけにします。可視性は `resolve` stage で clause に従って解決し、HIR には canonical symbol を渡します。

### 検証

`as name` で未修飾参照が失敗するテスト、selective import で未選択 symbol が失敗する compile_fail テスト、open import の曖昧性テストを追加します。

## RV-CORE-006: 通常実行でデバッグ出力が stderr へ漏れる

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: bug
- 対象: `nepl-core/src/loader.rs`, `nepl-core/src/types.rs`, `nepl-cli/src/main.rs`

### 根拠

- `nepl-core/src/loader.rs:166`: `load_inline_with_provider` が常時 `eprintln!`。
- `nepl-core/src/loader.rs:271`: `load_from_contents_with` が path/canon を出す。
- `nepl-core/src/loader.rs:320`: `load_file` が常時 loading log を出す。
- `nepl-core/src/typecheck.rs:1965`: diagnostics summary を stderr 出力する経路がある。
- `nepl-core/src/types.rs:1527`: `type_to_string` cycle で常時 `eprintln!`。

### 問題

verbose option と関係なく stderr が汚染されます。compiler を library として使う web / test runner / CLI で、stderr を診断やプログラム出力として扱う経路と衝突します。

### 影響

doctest の `stderr` 比較、JSON 出力、CI log が不安定になります。大量 import ではログ出力そのものも性能劣化要因になります。

### 修正方針

core から直接 `eprintln!` を排除し、`Diagnostic` または injectable logger に統一します。debug-only log は `crate::log::is_verbose()` を通し、default は完全に無出力にします。

### 対応

loader の `[Loader]` 出力を `loader_log!` に集約し、`crate::log::is_verbose()` が有効な場合だけ stderr へ出すようにしました。`type_to_string` の cycle 検出ログも verbose gate 配下へ移動し、通常の型文字列化で stderr を汚染しないようにしました。

CLI は loader を呼ぶ前に `nepl_core::log::set_verbose(cli.verbose)` を設定するようにし、`--verbose` が core loader にも伝播するようにしました。CLI 自身の `DEBUG:` 出力は既存の `RV-CLI-002` として別 issue で扱います。

### 検証

確認済み:

- `cargo fmt --all --check`
- `cargo check -p nepl-core`
- `cargo build -p nepl-cli`
- `target/debug/nepl-cli.exe --check -i tmp/rv-core-003-large.nepl --target core` で `[Loader]` 出力 0 件
- `target/debug/nepl-cli.exe --verbose --check -i tmp/rv-core-003-large.nepl --target core` で `[Loader]` 出力あり
- `cargo check --workspace`
- `trunk build`
- `node tests/compiler/tree/run.js`: `total=19`, `passed=19`, `failed=0`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: `13/13 passed`

## RV-CORE-007: codegen が診断ではなく panic で落ちる経路を多数持つ

- 解決済: false
- 状態: open
- 優先度: P0
- 種別: bug
- 対象: `nepl-core/src/codegen_wasm.rs`, `nepl-core/src/codegen_llvm.rs`

### 根拠

- `nepl-core/src/codegen_wasm.rs:476`: unsupported extern signature で `panic!`。
- `nepl-core/src/codegen_wasm.rs:504`: unsupported function signature で `panic!`。
- `nepl-core/src/codegen_wasm.rs:1123`: string literal lookup failure で `panic!`。
- `nepl-core/src/codegen_wasm.rs:1140`: unknown variable で `panic!`。
- `nepl-core/src/codegen_llvm.rs:1391`: unknown variable で `panic!`。
- `nepl-core/src/codegen_llvm.rs:2863`: unsupported intrinsic で `panic!`。

### 問題

compiler はユーザー入力に対して診断を返すべきですが、backend は内部不整合を `panic!` で処理しています。precheck が完全でない場合、通常のコンパイルエラーではなくプロセス異常終了になります。

### 影響

CLI / web / doctest runner が compiler crash と compile error を区別できません。クラッシュ時には span も失われ、根本原因の追跡が難しくなります。

### 修正方針

codegen API を `Result<CodegenResult, Vec<Diagnostic>>` に寄せ、panic 経路をすべて `DiagnosticId::Codegen...` 付きエラーに置き換えます。precheck は「panic を防ぐ安全網」ではなく「早い診断」の位置づけにします。

### 対応

`codegen_wasm::generate_wasm` を `Result<CodegenResult, Vec<Diagnostic>>` に変更し、unsupported signature、raw wasm parse error、missing return、unknown variable / function / intrinsic、string literal table mismatch、field selector mismatch、aggregate lower 非対応を `Diagnostic` として返すようにしました。これにより `codegen_wasm.rs` の production 経路から explicit `panic!` / `unwrap()` / `expect()` を除去しました。

LLVM backend は `LlvmCodegenError::CodegenDiagnostic` を追加し、raw body mismatch、unknown variable / function / function value、unsupported intrinsic、残りの lowering invariant failure を diagnostic error として返すようにしました。`get_field` / `set_field` は WASM / LLVM の precheck supported intrinsic list に追加し、precheck と backend の supported set がずれて誤診断になる問題も同時に修正しました。

### 検証

unsupported intrinsic、unknown field selector、invalid raw wasm を `tests/compiler/codegen_diagnostics.n.md` の compile_fail として固定しました。precheck を bypass した backend 直接呼び出しについては `nepl-core/tests/codegen_diagnostics.rs` で unsupported function signature、unknown variable、missing string literal が panic ではなく diagnostic を返すことを確認しました。

## RV-CORE-008: effect 判定が文字列包含に依存していて不健全

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: bug
- 対象: `nepl-core/src/effects.rs`, `nepl-core/src/typecheck.rs`, `nepl-core/tests/effects.rs`

### 根拠

- `nepl-core/src/effects.rs:57`: `marker_is_impure_io` が `text.contains(m)`。
- `nepl-core/src/effects.rs:69`: raw body effect は行文字列の contains で判定。
- `nepl-core/src/effects.rs:77`: `HirBody::Block(_)` は raw body 判定では Pure。

### 問題

raw wasm / LLVM IR の effect を文字列検索で推定しています。コメントや別名に `fd_write` が含まれれば false positive になり、逆に wrapper 名や syscall 経由は missed detection になります。

### 影響

pure 関数から外部 I/O が呼べる、または pure な raw body が impure と誤判定されるなど、effect system の信頼性が落ちます。

### 修正方針

raw body は明示 effect annotation を必須にするか、extern / intrinsic 宣言に effect を持たせて call graph で伝播します。文字列検索は診断補助に限定します。

### 対応

raw body の effect 推定から行文字列の `contains` 判定を削除し、raw wasm / LLVM IR の direct call target だけを抽出するようにしました。コメント内の `fd_write` や、`fd_write_like` のように impure marker を部分文字列として含むだけの名前は effect 判定に使いません。

抽出した call target は、まず NEPL 側の宣言済み callable / extern symbol の effect と照合します。同じ target に impure な宣言があれば impure とし、pure 宣言だけなら pure として扱います。宣言が見つからない場合に限り、`fd_write` などの既知 WASI I/O marker と完全一致する intrinsic fallback で impure 判定します。LLVM の `llvm.*` intrinsic は compiler intrinsic として pure 扱いにしました。

### 検証

確認済み:

- `cargo test -p nepl-core --test effects` (`5 passed`)
- `cargo fmt --all --check`
- `cargo check --workspace`
- `trunk build`
- `node tests/compiler/tree/run.js` (`total=19`, `passed=19`, `failed=0`)
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`13/13 passed`)

既存残件:

- `cargo test -p nepl-core` で見つかった first-class function / lambda wasm codegen の失敗は `RV-CORE-017` として分離し、後続修正で解決済みです。
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-stdlib --no-tree -o tmp/rv-core-008-move-effect.json` は `23/26` です。raw body effect の `fd_write` compile_fail は維持されていますが、既存の D3090 impl method signature mismatch が `doctest#5` から `doctest#7` で残っています。

## RV-CORE-009: move/borrow/drop が Resource IR なしで後付け実装されている

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: architecture
- 対象: `nepl-core/src/passes/move_check.rs`, `nepl-core/src/passes/drop_insertion.rs`, `nepl-core/src/compiler.rs`

### 根拠

- `nepl-core/src/compiler.rs:374`: drop 挿入後に monomorphize し、その後 move check。
- `nepl-core/src/passes/move_check.rs:35`: 変数状態は `BTreeMap<String, Vec<VarState>>`。
- `nepl-core/src/passes/move_check.rs:616`: `AddrOf` は borrow として状態を変える。
- `nepl-core/src/passes/drop_insertion.rs:91`: scope exit で trait drop call を後付け生成。

### 問題

Resource IR がなく、HIR 木を直接走査して所有権状態を推測しています。borrow の lifetime release、field move、auto drop と user drop の整合、branch merge が局所状態で処理されており、仕様化しづらいです。

### 影響

false positive / false negative の両方が起きやすく、stdlib の owning collection と組み合わせたときに double free や use-after-move を防ぎきれません。

### 修正方針

`doc/2.1impl/compiler_structure.md` の方針どおり HIR 後に Resource IR を導入し、move、borrow、region、drop elaboration を別 pass に分けます。drop 挿入前に ownership を確定し、auto drop は Resource IR 上で挿入します。

### 検証

branch/loop/borrow/field move/drop trait の compile_fail と should_panic を整理し、Resource IR dump の snapshot test を追加します。

## RV-CORE-010: name resolution が二重化し本パイプラインに統合されていない

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: architecture
- 対象: `nepl-core/src/resolve.rs`, `nepl-core/src/name_resolve.rs`, `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/resolve.rs:1`: DefId / export table の scaffolding。
- `nepl-core/src/name_resolve.rs:1`: 別の name resolution skeleton。
- `nepl-core/src/name_resolve.rs:48`: `resolve_names` は diagnostics 空で返すだけ。
- `nepl-core/src/typecheck.rs:7777`: 実際の値・関数探索は `typecheck.rs` 内の `Env` が担う。

### 問題

名前解決の意図は `resolve.rs` と `name_resolve.rs` にありますが、本 pipeline は `typecheck.rs` 内 Env に依存しています。DefId を持つ canonical resolution になっていません。

### 影響

import alias、overload、shadowing、qualified path の責務が分散し、同名関数や module boundary のバグが出やすくなります。

### 修正方針

`name_resolve.rs` は削除または `resolve/` へ統合し、DefId を AST/HIR lowering の入力にします。`typecheck.rs` の Env は型検査用 local binding に限定します。

### 検証

cross-file import、qualified alias、shadowing、ambiguous open import の tree test を DefId snapshot として追加します。

## RV-CORE-011: TypeExpr が span を保持せず診断位置が失われる

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: bug
- 対象: `nepl-core/src/ast.rs`, `nepl-core/src/parser.rs`, `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/ast.rs:41`: `TypeExpr::span()` が常に `Span::dummy()`。
- `nepl-core/src/parser.rs`: 型式 parsing の各所で `Span::dummy()` fallback が多い。
- `nepl-core/src/typecheck.rs:5547`: call reduction limit diagnostic も `Span::dummy()`。

### 問題

型注釈や型式のエラー位置を正確に出せません。dummy span は `<unknown>:1:1` のような診断に化け、修正箇所が分からなくなります。

### 影響

ユーザー体験が悪く、compile_fail テストで span を固定しにくくなります。内部バグの triage でも、問題の型式を特定しづらいです。

### 修正方針

`TypeExpr` を enum + span wrapper にするか、各 variant に span を持たせます。parser は型式の開始・終了 span を保存し、typecheck diagnostic では必ず元の型式 span を使います。

### 検証

型注釈 mismatch、trait bound mismatch、generic arity mismatch の `diag_span` テストを追加します。

## RV-CORE-012: target/profile gate の評価が複数箇所に散っている

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: architecture
- 対象: `nepl-core/src/compiler.rs`, `nepl-core/src/target_precheck.rs`, `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/compiler.rs:44`: target gate parser は compiler に定義。
- `nepl-core/src/target_precheck.rs:54`: `gate_allows` は compiler の parser を呼ぶ。
- `nepl-core/src/typecheck.rs:8976`: typecheck 側にも target gate helper がある。
- `nepl-core/src/target_precheck.rs:285`: precheck は codegen 前に別途実行される。

### 問題

target/profile gate の有効 statement 判定が複数箇所に散っています。未知 gate は `false` に丸められやすく、診断になる箇所と静かに無効化される箇所が分かれます。

### 影響

`#if[target=...]` を含む stdlib で、target ごとに実行される関数 body がずれる可能性があります。raw body precheck と typecheck の見ている active statements が一致しないと、backend panic に繋がります。

### 修正方針

gate evaluation を `passes/target_gate` 相当へ一本化し、AST から inactive item を明示的に除外した lowered module を作ります。未知 gate は warning ではなく diagnostic error にします。

### 検証

target gate の boolean expression、unknown gate、profile gate、raw body selection の matrix test を追加します。

## RV-CORE-013: 参照引数の関数呼び出しが一時 borrow にならず所有値を固定する

- 解決済: true
- 状態: verified
- 優先度: P0
- 種別: bug
- 対象: `nepl-core/src/passes/move_check.rs`, `tests/compiler/move_check.n.md`, `stdlib/alloc/collections/vec.nepl`, `stdlib/alloc/collections/stack.nepl`

### 根拠

- `nepl-core/src/passes/move_check.rs`: 通常の関数呼び出し引数は `visit_expr` で処理され、`AddrOf` が `visit_borrow` によりスコープ終端までの borrow として記録される。
- `stdlib/alloc/collections/vec.nepl`: `len_ref &v` の後に `push v ...` する doctest が、`Vec` から `Copy` を外した時点で `cannot move out of shared borrowed value` になる。
- `stdlib/alloc/collections/stack.nepl`: `peek_ref &stk` / `len_ref &stk` も同じ構造です。

### 問題

`len_ref &v` のように参照を関数呼び出しへ渡すだけの式でも、move checker は `v` をスコープ終端まで shared borrow として扱います。参照引数の呼び出しは式の評価中だけの一時 borrow であるべきですが、現状では borrow が解放されないため、非 Copy 所有値を読み取った後に移動・更新できません。

### 影響

`Vec` / `Stack` を正しく非 Copy にすると、borrow-based read API を追加しても、その後の `push` / `free` / move が拒否されます。所有権を安全にした stdlib API と現在の borrow checker が噛み合わず、`RV-STDLIB-003` の根本修正を妨げます。

### 修正方針

move checker で call target の parameter type を参照し、parameter が `&T` / `&mut T` の場合は対応する引数を一時 borrow として評価するよう修正しました。`&x` は呼び出し式の評価中だけ borrow し、呼び出し後の `x` の所有権状態を `Valid` のまま保ちます。非参照引数の by-value move と、永続的な local borrow (`let r &x`) は従来どおり区別します。

### 検証

`tests/compiler/move_check.n.md` に「非 Copy 値を参照引数へ渡した後に move できる」回帰テストを追加しました。既存の「local に保持した shared borrow 中の move は引き続き拒否される」compile_fail と合わせて確認しています。

確認済み:

- `cargo check -p nepl-core`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md --no-tree -o tmp/move-check-rv-core-013.json -j 1` (`total=14`, `passed=14`, `failed=0`)
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`caseCount=13`, `passedCount=13`, `failedCount=0`)

## RV-CORE-014: Pair から取り出した generic collection の型が overload 解決へ伝播しない

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: bug
- 対象: `nepl-core/src/typecheck.rs`, `tests/compiler/overload.n.md`, `stdlib/alloc/collections/vec.nepl`

### 根拠

- `stdlib/alloc/collections/vec.nepl::doctest#28`: `partition` の戻り値 `.Pair` から `let evens get parts 0` で取り出した後、`len<i32> evens` が `error[D3006]: no matching overload found` になる。
- 同じ箇所へ明示型注釈を付けると、過去の調査では overload error ではなく codegen の internal panic 経路へ進んだ。
- `RV-CORE-004` は overload 解決の clone 過多を扱うが、この件は `.Pair` field access 結果の型伝播と overload 候補選択の正当性問題です。

### 問題

`.Pair` のような generic tuple 相当の値から `get` で取り出した collection の型情報が、その後の overloaded function call に十分伝播していませんでした。原因は field accessor ではなく、関数本体で `.Pair` が実際の tuple 型へ推論された後、関数チェック終了時の snapshot 復元でその束縛まで破棄していた点でした。結果として `Vec<i32>` であるべき値に対して `len<i32>` の候補を選べず、正常な stdlib doctest が compile error になっていました。

### 影響

`partition` のように複数の collection を返す API を利用したコードで、明示型注釈なしに後続 API を呼べません。型注釈で回避しようとしても backend panic 経路へ進む可能性があり、診断品質と codegen 安全性の両方に影響します。

### 修正方針

関数チェック成功時の型変数復元を、明示的な関数 type parameter の束縛に限定しました。`.Pair` のようなシグネチャ内の非 type parameter 推論結果は保持し、field accessor / overload 解決から `Vec<i32>` として参照できるようにしました。関数チェック失敗時は従来どおり関数シグネチャ全体の snapshot を復元し、失敗した部分推論が外へ漏れないようにしています。

### 検証

確認済み:

- `cargo check -p nepl-core`
- `cargo run -p nepl-cli -- -i tmp/rv-core-014-repro.nepl --check --target core`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-rv-core-014.json -j 1` (`total=45`, `passed=45`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-rv-core-014.json -j 1` (`total=37`, `passed=37`, `failed=0`)
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`caseCount=13`, `passedCount=13`, `failedCount=0`)

## RV-CORE-015: 深い HIR を check pipeline が再帰処理して stack overflow する

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: bug
- 対象: `nepl-core/src/compiler.rs`, `nepl-core/src/lib.rs`, `nepl-cli/src/main.rs`

### 根拠

- `RV-CORE-003` の 1105 identity prefix call chain は typecheck / wasm semantics では通る。
- 一方で `cargo run -p nepl-cli -- -i tmp/rv-core-003-large.nepl --check --target core` は、`DEBUG: Calling compile_module` 後に `thread 'main' has overflowed its stack` で異常終了する。

### 問題

`--check` は型検査だけで終わらず compile pipeline を進めています。後段の drop insertion / move check / codegen precheck / wasm codegen などが深い `HirExpr` を再帰的にたどるため、型検査済みの正当な長い式でも native stack overflow で落ちます。

### 影響

深いが正当なプログラムを CLI で確認できず、診断ではなくプロセス異常終了になります。CI や editor integration では入力サイズに依存して compiler process が落ちるため、`RV-CORE-003` の typecheck 改善だけではユーザー体験が安定しません。

### 修正方針

`--check` の責務を型検査成功可否に限定できる path として分離します。artifact 生成が必要ない確認用途で、drop insertion / move check / codegen precheck / wasm codegen へ進まないようにします。

### 対応

`nepl_core::check_module_with_source_map` を追加し、target/profile precheck と typecheck までを実行する check-only path を提供しました。`nepl-cli --check` はこの API を呼ぶようにし、`compile_module_with_source_map` による artifact 生成へ進まないようにしました。

これにより、`RV-CLI-001` で必要だった「未定義シンボルなどの compiler diagnostics を拾う」性質は維持しつつ、1105 identity prefix call chain のような深いが正当な入力で後段の再帰 pipeline に入らなくなりました。

対応中に、`--output` による実際の wasm artifact 生成は同じ深い HIR で引き続き native stack overflow することを確認しました。この残件は `RV-CORE-016` として分離し、後段 HIR pass / codegen の iterative 化で追跡します。

### 検証

確認済み:

- `cargo test -p nepl-core --test check_pipeline`
- `cargo test -p nepl-cli check_`
- `cargo run -p nepl-cli -- -i tmp/rv-core-003-large.nepl --check --target core`

未解決として分離:

- `cargo run -p nepl-cli -- -i tmp/rv-core-003-large.nepl --target core -o tmp/rv-core-003-large-rv-core-015.wasm` は `thread 'main' has overflowed its stack` で失敗する。artifact 生成側の深い HIR traversal は `RV-CORE-016` で追跡します。

## RV-CORE-016: 深い HIR を artifact codegen pipeline が再帰処理して stack overflow する

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: bug
- 対象: `nepl-core/src/monomorphize.rs`, `nepl-core/src/passes/move_check.rs`, `nepl-core/src/passes/codegen_precheck.rs`, `nepl-core/src/wasm_shared.rs`, `nepl-core/src/codegen_wasm.rs`

### 根拠

`RV-CORE-015` の check-only path 分離後、同じ 1105 identity prefix call chain を実際に artifact 生成すると、`compile_module` 後段で native stack overflow します。

再現:

- `cargo run -p nepl-cli -- -i tmp/rv-core-003-large.nepl --target core -o tmp/rv-core-003-large-rv-core-015.wasm`

### 問題

artifact 生成に必要な HIR pass / codegen backend は、深い `HirExpr` を再帰的に走査します。`--check` は分離済みですが、実際の wasm 出力では drop insertion、monomorphize、move check、codegen precheck、wasm codegen のいずれかで native stack を消費し、診断ではなくプロセス異常終了になります。

### 影響

型検査上は正当な深い式でも、wasm artifact を生成できません。CI や配布ビルドで入力サイズ依存の compiler crash が残ります。

### 修正方針

artifact 生成側の HIR traversal を段階別に切り分け、深い call chain で再帰しない iterative visitor へ置き換えます。最低限、1105-call chain の wasm 出力が stack overflow せず成功する regression を追加します。

### 対応

monomorphize では非 generic 関数の元 HIR を clone せず移動し、type parameter mapping が空の具象関数では body 全体の再帰 substitute を避け、callee queue のみを iterative traversal で処理するようにしました。move check は borrow / control-flow に関わらない単純な値式を explicit stack で走査する fast path を追加し、既存の ownership semantics が必要な式は従来の分岐へ残しています。

wasm codegen precheck と signature / reachable collection は shared helper 側で recursive walk をやめ、explicit stack で `HirExpr` を辿るようにしました。さらに wasm lowering では単純な literal / variable / direct call tree を iterative post-order lowering できる path を追加し、1105 個の prefix call chain を native stack に積まずに wasm instruction へ落とせるようにしました。

### 検証

確認済み:

- `cargo fmt --all --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core --test check_pipeline` (`3 passed`)
- `cargo run -p nepl-cli -- -i tmp/rv-core-003-large.nepl --target core -o tmp/rv-core-003-large-rv-core-016.wasm`
- `cargo check --workspace`
- `trunk build`
- `node tests/compiler/tree/run.js` (`total=19`, `passed=19`, `failed=0`)
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`13/13 passed`)

## RV-CORE-017: 関数値として渡した関数と lambda が backend 到達時に未登録になる

- 解決済: true
- 状態: fixed
- 優先度: P0
- 種別: bug
- 対象: `nepl-core/src/typecheck.rs`, `nepl-core/src/monomorphize.rs`, `nepl-core/src/wasm_shared.rs`, `nepl-core/src/codegen_wasm.rs`, `nepl-core/src/codegen_llvm.rs`

### 根拠

- GitHub Actions run `24932659255` の `wasi-test` / `nmd-doctest`: `tests/compiler/functions.n.md::doctest#6` が `error[D4008]: unknown function value 'square__i32__i32__pure' reached wasm codegen` で失敗。
- 同 run: `tests/compiler/functions.n.md::doctest#8` が `error[D4007]: unknown variable 'add_op' reached wasm codegen`、`doctest#12` が generated lambda 名 `__lambda_0_214_218` の unknown variable で失敗。
- 同 run: `tests/compiler/list_dot_map.n.md`、`tests/compiler/move_check.n.md`、`tests/compiler/overload.n.md`、`tests/compiler/prelude_copy.n.md`、`tests/compiler/typeannot.n.md` でも、関数値として渡した `inc` / `token_id` / `calc` / `as_i32` / `f` が `D4008` で失敗。
- 同 run の `tutorials-test`: 競プロ I/O 系 tutorial 6 件が `stdlib/std/streamio.nepl` の `stream_writer_noncopy_marker__i32__i32__pure` を unknown function value として失敗。
- `tests/compiler/tree/08_function_value_call_indirect.js` は function value call の lowering を期待しているが、実際の doctest / Rust integration 経路では関数値が backend へ届く前後の登録が揃っていない。
- 手元の `stdlib/alloc/collections/vec.nepl` 広域 doctest でも、`map` / `fold` / `reduce` / `find` / `take_while` などの高階 API が `error[D4008]: unknown function value ... reached wasm codegen` で失敗した。
- `Vec` 高階 API は `(.T)->.U` や `(.U,.T)->.U` の関数値を引数に取るため、CI の関数値 failure と同じ lowering 契約不備に含めて追跡する。

### 問題

型検査は higher-order function や lambda を通している一方で、monomorphize / reachable collection / backend lowering のどこかで、関数値としてだけ参照される関数や generated lambda が module の callable set に安定して登録されていません。その結果、ユーザーコード上は関数値として正当な式が、backend の内部不変条件エラーである `D4007` / `D4008` に到達します。

これは `RV-CORE-007` で panic を diagnostic 化した後に可視化された根本バグです。diagnostic を返すこと自体は改善ですが、正当な higher-order code を compile できない状態は残っています。

### 影響

標準ライブラリの `List::map` / `filter` / `fold` 相当、`Vec::map` / `filter` / `fold` 相当、`streamio` の writer marker、tutorial の競プロ I/O、関数値を返す基本サンプルが CI 上で失敗します。`wasi-test` / `nmd-doctest` / `tutorials-test` / `stdlib-test` にまたがるため、CI の大半が赤くなり、他の stdlib 不具合も同じ failure set に埋もれます。

### 修正方針

関数値を first-class に扱うための lowering 契約を明確にします。typecheck で関数値・lambda・indirect call の型を確定した後、monomorphize と reachable function collection は「直接 call される関数」だけでなく「値として参照される関数」と generated lambda も収集し、WASM / LLVM backend の function table と name map に登録します。

`@func` と bare `func` の扱い、純粋/非純粋 signature、capturing lambda の未対応診断を整理し、未実装の capture は typecheck diagnostic に留めます。backend に到達してから unknown になる経路をなくします。

### 対応

`monomorphize.rs` の concrete 関数処理で、直接 call だけでなく `Var` / `FnValue` として現れる関数参照も収集し、対応する specialized function を worklist へ積むようにしました。

generic 関数の `substitute_expr` では既に関数値参照を `request_instantiation` へ通していましたが、type parameter mapping が空の concrete 関数では `queue_concrete_callees` が direct `Call` しか見ていませんでした。そのため `square`、`add_op`、generated lambda のように値として渡される関数が backend の `name_map` に入らず、`D4007` / `D4008` に到達していました。

今回の修正では concrete 関数の param / let / match bind を local 名として集めたうえで、local ではない bare `Var` と `FnValue` をユーザー関数として解決し、direct call と同じ monomorphize queue に登録します。

### 検証

確認済み:

- `cargo fmt --all -- --check`
- `cargo test -p nepl-core --test functions function_first_class -- --nocapture` (`2 passed`)
- `cargo test -p nepl-core --test functions function_return -- --nocapture`
- `cargo test -p nepl-core --test functions` (`12 passed`, `1 ignored`)
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/functions.n.md -o tmp/functions-rv-core-017.json -j 1` (`41/41 passed`)
- `node nodesrc/tests.js -i tests/compiler/list_dot_map.n.md -o tmp/list-dot-map-rv-core-017.json -j 1` (`23/23 passed`)
- `node nodesrc/tests.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -o tmp/tutorial-io-rv-core-017.json -j 1` (`22/22 passed`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl -o tmp/vec-rv-core-017.json -j 1` (`57/57 passed`)

未解決として分離:

- `node nodesrc/tests.js -i stdlib/tests/vec.n.md -o tmp/vec-tests-rv-core-017.json -j 1` は `20/21 passed` で、`partition` の right bucket 読み取りが runtime 値不一致になる。関数値登録漏れではなく nested aggregate field access の問題として `RV-CORE-018` へ分離しました。

GitHub Actions での確認は、この修正を push した後の run で行います。

## RV-CORE-018: nested aggregate を tuple から取り出すと 2 番目以降の値が壊れる

- 解決済: false
- 状態: open
- 優先度: P0
- 種別: bug
- 対象: `nepl-core/src/codegen_wasm.rs`, `nepl-core/src/codegen_llvm.rs`, `nepl-core/src/typecheck.rs`

### 根拠

- `node nodesrc/tests.js -i stdlib/tests/vec.n.md -o tmp/vec-tests-rv-core-017.json -j 1`: `stdlib/tests/vec.n.md::doctest#2` が `err assert_eq_i32 failed: expected=1 actual=0` で失敗。
- 失敗箇所は `partition<i32>` の戻り値 `.Pair` から `get parts 1` で取り出した odd 側 `Vec<i32>` の先頭要素。
- 一時再現 `tmp/rv-pair-vec-runtime.n.md` では、`Tuple: left right` の 2 番目の `Vec<i32>` を `get pair 1` で取り出し、`get_ref &got_right 0` すると expected `1` に対して actual `2208` になった。

### 問題

`Vec<i32>` の単体操作と `partition` の predicate / length 計算は動いています。一方で、tuple に nested struct / aggregate を 2 つ入れた後、2 番目以降の aggregate を取り出すと内部 field、特に data pointer か payload layout が壊れます。

`RV-CORE-014` は `.Pair` から取り出した generic collection の型伝播問題でしたが、今回は型検査は通り runtime 値が壊れるため別問題です。aggregate の ABI / layout / field selector lowering のどこかで、tuple field offset と nested struct value の copy 幅が一致していない可能性があります。

### 影響

`Vec::partition` の `(matched, rest)` の rest 側、複数 collection を返す stdlib API、nested struct を tuple / anonymous aggregate 経由で返すユーザーコードが誤った値を読みます。runtime trap ではなく値化けになるため、検出しにくいデータ破壊です。

### 修正方針

WASM / LLVM の aggregate layout を同じ仕様に揃え、tuple field access が nested struct の size / alignment / field offset を正しく使うことを確認します。最小回帰として、2 本の `Vec<i32>` を tuple に入れて 2 番目を取り出し、len と先頭要素を検査する doctest または Rust integration test を追加します。

### 検証

- `stdlib/tests/vec.n.md::doctest#2` が `partition` の even / odd 両側で `Vec` 要素を正しく読めること。
- nested `Tuple(Vec<i32>, Vec<i32>)` の 2 番目以降の `Vec` で `len_ref` / `get_ref` が正しいこと。
